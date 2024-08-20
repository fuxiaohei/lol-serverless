use super::Item;
use crate::memenvs;
use anyhow::{anyhow, Result};
use land_dao::deploy_task::TaskType;
use land_vars::Task;
use lazy_static::lazy_static;
use reqwest::Client;
use serde::Deserialize;
use std::{collections::HashMap, path::Path};
use tokio::sync::Mutex;
use tracing::{debug, info, instrument, warn};

#[derive(Deserialize, Default, Clone, Debug)]
struct SyncResponse {
    pub status: String,
    pub message: String,
    pub data: Vec<Task>,
}

/// init_task starts background tasks
pub async fn init_task(addr: String, token: String, dir: String, service_name: String) {
    debug!("agent init_task, {}, {}, {}", token, dir, service_name);

    // init client
    super::CLIENT_ONCE.call_once(|| {
        let client = Client::new();
        super::CLIENT.set(client).unwrap();
    });

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(1));
        ticker.tick().await;
        loop {
            match request(
                addr.clone(),
                token.clone(),
                dir.clone(),
                service_name.clone(),
            )
            .await
            {
                Ok(_) => {}
                Err(e) => {
                    warn!("agent task error: {:?}", e);
                }
            };
            ticker.tick().await;
        }
    });
}

lazy_static! {
    static ref TASK_RES: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

#[instrument("[AGT-TASK]", skip_all)]
async fn request(addr: String, token: String, dir: String, service_name: String) -> Result<()> {
    let ipinfo = super::get_ip().await;
    let client = super::CLIENT.get().unwrap();
    let mut tasks = TASK_RES.lock().await;

    let api = format!("{}/worker-api/task?ip={}", addr, ipinfo.ip);
    let token = format!("Bearer {}", token);
    let res = client
        .post(api)
        .header("Authorization", token)
        .header("X-Md5", "".to_string())
        .json(&tasks.clone())
        .send()
        .await?;

    let status_code = res.status().as_u16();
    if status_code == 204 {
        tasks.clear();
        // debug!("no change");
        return Ok(());
    }
    // 400+ is error
    if status_code >= 400 {
        let content = res.text().await?;
        return Err(anyhow!("Bad status:{}, Error:{}", status_code, content));
    }
    let resp: SyncResponse = res.json().await?;
    if resp.status != "ok" {
        warn!("Bad response: {}", resp.message);
        return Err(anyhow!("Bad response: {}", resp.message));
    }
    // debug!("sync response: {}, {}", resp.status, resp.message);
    if resp.data.is_empty() {
        tasks.clear();
        // debug!("no task");
        return Ok(());
    }
    debug!("sync task: {:?}", resp.data);

    // remove not exist task in task-res from current task response
    let current_task_keys = resp
        .data
        .iter()
        .map(|t| t.task_id.clone())
        .collect::<Vec<String>>();
    tasks.retain(|k, _| current_task_keys.contains(k));

    // handle each task
    for task in resp.data {
        let task_id = task.task_id.clone();
        match handle_each_task(task, dir.clone(), service_name.clone()).await {
            Ok(_) => {
                tasks.insert(task_id, "success".to_string());
            }
            Err(e) => {
                warn!(task_id = task_id, "handle task error: {:?}", e);
                tasks.insert(task_id, e.to_string());
            }
        }
    }

    Ok(())
}

async fn handle_each_task(t: Task, dir: String, service_name: String) -> Result<()> {
    let item: Item = serde_json::from_str(&t.content)?;
    match t.task_type.parse()? {
        TaskType::DeployWasmToWorker => {
            handle_each_deploy(item, dir.clone(), service_name.clone()).await
        }
        TaskType::DisableWasm => handle_each_disable(item, dir).await,
        TaskType::DeployEnvs => handle_env(item, dir).await,
    }
}

async fn handle_each_deploy(item: Item, dir: String, service_name: String) -> Result<()> {
    let wasm_target_file = format!("{}/{}", dir, item.file_name);

    // 1. download wasm file
    if !Path::new(&wasm_target_file).exists() {
        let resp = reqwest::get(&item.download_url).await?;
        if resp.status().as_u16() != 200 {
            return Err(anyhow!(
                "download error: {}, url: {}",
                resp.status(),
                item.download_url
            ));
        }
        let content = resp.bytes().await?;
        let content_md5 = format!("{:x}", md5::compute(&content));
        if content_md5 != item.file_hash {
            return Err(anyhow!(
                "download hash dismatch: real: {}, expect: {}, url: {}",
                content_md5,
                item.file_hash,
                item.download_url,
            ));
        }
        let dir = Path::new(&wasm_target_file).parent().unwrap();
        std::fs::create_dir_all(dir)?;
        std::fs::write(&wasm_target_file, content)?;
        debug!("download success: {}", wasm_target_file);
    }

    // 2. generate traefic file
    let traefik_file = format!("{}/traefik/{}.yaml", dir, item.domain.replace('.', "_"));
    let traefik_dir = format!("{}/traefik", dir);
    std::fs::create_dir_all(traefik_dir)?;
    let confs = super::traefik::build(&item, &service_name)?;
    let content = serde_yaml::to_string(&confs)?;
    std::fs::write(&traefik_file, content)?;
    debug!("generate traefik success: {}", traefik_file);

    // 3. prepare worker
    land_wasm_host::pool::prepare_worker(&item.file_name, true).await?;
    debug!("prepare worker success: {}", item.file_name);

    Ok(())
}

async fn handle_each_disable(item: Item, dir: String) -> Result<()> {
    let traefik_file = format!("{}/traefik/{}.yaml", dir, item.domain.replace('.', "_"));
    if Path::new(&traefik_file).exists() {
        std::fs::remove_file(&traefik_file)?;
        debug!("remove traefik success: {}", traefik_file);
    } else {
        debug!("traefik not exist: {}", traefik_file);
    }
    Ok(())
}

async fn handle_env(item: Item, dir: String) -> Result<()> {
    let envs_file = format!(
        "{}/envs/{}-{}.envs.json",
        dir, item.user_id, item.project_id
    );
    let envs_dir = format!("{}/envs", dir);
    std::fs::create_dir_all(envs_dir)?;
    let content = item.content.unwrap_or_default();
    /*
    let env_content = serde_json::from_str::<EnvContent>(&content)?;
    let envs = land_dao::envs::decode_env(&env_content.secret, &env_content.values)?;
    println!("envs: {:?}", envs);
    */
    std::fs::write(&envs_file, content)?;
    memenvs::read_file(&envs_file).await?;
    info!("generate envs success: {}", envs_file);
    Ok(())
}
