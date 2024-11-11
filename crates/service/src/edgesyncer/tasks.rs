use crate::{
    confs::{traefik, WasmConfig},
    memenvs,
};
use anyhow::{anyhow, Result};
use land_dao::deploy_task::{self, TaskType};
use land_tplvars::Task;
use land_utils::localip;
use lazy_static::lazy_static;
use reqwest::Url;
use serde::Deserialize;
use std::{collections::HashMap, time::Duration};
use tokio::{sync::Mutex, time::interval};
use tracing::{debug, info, instrument, warn};

#[derive(Deserialize, Default, Clone, Debug)]
struct Response {
    pub status: String,
    pub message: String,
    pub data: Vec<Task>,
}

/// init starts background tasks
pub async fn init(addr: String, token: String, dir: String, service_name: String) {
    debug!("agent init_task, {}, {}, {}", token, dir, service_name);

    // init client
    super::CLIENT_ONCE.call_once(|| {
        let client = reqwest::Client::new();
        super::CLIENT.set(client).unwrap();
    });

    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(1));
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
    let ipinfo = localip::get().await;
    let client = super::CLIENT.get().unwrap();
    let mut tasks = TASK_RES.lock().await;

    // get tasks from server with current agent ip
    let api = format!("{}/_worker_api/tasks?ip={}", addr, ipinfo.ip);
    let token_header = format!("Bearer {}", token);
    let res = client
        .post(api)
        .header("Authorization", token_header)
        .json(&tasks.clone())
        .send()
        .await?;

    let status_code = res.status().as_u16();
    // 204 means no tasks to do
    if status_code == 204 {
        // no tasks, means TASK_RES should be empty, no results for tasks
        tasks.clear();
        // debug!("no change");
        return Ok(());
    }
    // 400+ is error
    if status_code >= 400 {
        let content = res.text().await?;
        return Err(anyhow!("Bad status:{}, Error:{}", status_code, content));
    }

    // parse response to get tasks to do
    let resp: Response = res.json().await?;
    if resp.status != "ok" {
        warn!("Bad response: {}", resp.message);
        return Err(anyhow!("Bad response: {}", resp.message));
    }
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
        match handle_each_task(
            task,
            addr.clone(),
            dir.clone(),
            service_name.clone(),
            token.clone(),
        )
        .await
        {
            Ok(status) => {
                // success or doing
                tasks.insert(task_id, status.to_string());
            }
            Err(e) => {
                warn!(task_id = task_id, "handle task error: {:?}", e);
                tasks.insert(task_id, e.to_string());
            }
        }
    }

    Ok(())
}

async fn handle_each_task(
    t: land_tplvars::Task,
    addr: String,
    dir: String,
    service_name: String,
    token: String,
) -> Result<deploy_task::Status> {
    let item: WasmConfig = serde_json::from_str(&t.content)?;
    match t.task_type.parse()? {
        TaskType::DeployWasmToWorker => {
            handle_each_deploy(
                &item,
                addr.clone(),
                dir.clone(),
                service_name.clone(),
                token.clone(),
            )
            .await
        }
        TaskType::DisableWasm => handle_disable(&item, &dir).await,
        TaskType::DeployEnvs => handle_env(item, &dir).await,
    }
}

async fn handle_each_deploy(
    item: &WasmConfig,
    addr: String,
    dir: String,
    service_name: String,
    token: String,
) -> Result<deploy_task::Status> {
    debug!("handle_each_deploy: {:?}", item);

    // 1. parse download url, download from s3 like or land-server
    let real_url = parse_download_url(&item.download_url, &addr)?;
    println!("real_url: {:?}", real_url);

    // 2. download wasm file
    download_wasm(&real_url, &token, &dir, &item.file_name).await?;

    // 3. load wasm to worker
    land_wasm_host::Worker::new_in_pool(&item.file_name, true).await?;

    // 4. generate traefik config
    build_traefik(&item, &dir, &service_name).await?;

    Ok(deploy_task::Status::Success)
}

fn parse_download_url(url: &str, addr: &str) -> Result<String> {
    let u = Url::parse(url)?;
    if u.scheme() == "file" {
        println!("{:?}", u);
        return Ok(format!(
            "{}/_worker_api/download/{}/{}",
            addr,
            u.host_str().unwrap(),
            u.path().trim_start_matches('/')
        ));
    }
    if u.scheme() == "http" || u.scheme() == "https" {
        return Ok(url.to_string());
    }
    Err(anyhow!("Unsupported download url: {}", url))
}

async fn download_wasm(url: &str, token: &str, dir: &str, filename: &str) -> Result<()> {
    let client = super::CLIENT.get().unwrap();
    let token_header = format!("Bearer {}", token);
    let res = client
        .get(url)
        .header("Authorization", token_header)
        .send()
        .await?;
    // 400+ is error
    let status_code = res.status().as_u16();
    if status_code >= 400 {
        let content = res.text().await?;
        return Err(anyhow!("Bad status:{}, Error:{}", status_code, content));
    }
    let wasm_bytes = res.bytes().await?;
    let local_file = format!("{}/{}", dir, filename);
    let parent_dir = std::path::Path::new(&local_file).parent().unwrap();
    std::fs::create_dir_all(parent_dir)?;
    std::fs::write(&local_file, wasm_bytes)?;
    info!("Download wasm file to: {}", local_file);
    Ok(())
}

async fn build_traefik(item: &WasmConfig, dir: &str, service_name: &str) -> Result<()> {
    let traefik_file = format!("{}/traefik/{}.yaml", dir, item.domain.replace('.', "_"));
    let confs = traefik::build(item, service_name)?;

    let traefik_dir = format!("{}/traefik", dir);
    std::fs::create_dir_all(traefik_dir)?;

    let content = serde_yaml::to_string(&confs)?;
    std::fs::write(&traefik_file, content)?;
    debug!("generate traefik success: {}", traefik_file);
    Ok(())
}

async fn handle_disable(item: &WasmConfig, dir: &str) -> Result<deploy_task::Status> {
    let traefik_file = format!("{}/traefik/{}.yaml", dir, item.domain.replace('.', "_"));
    if std::path::Path::new(&traefik_file).exists() {
        std::fs::remove_file(&traefik_file)?;
        debug!("remove traefik success: {}", traefik_file);
    } else {
        debug!("traefik not exist: {}", traefik_file);
    }
    Ok(deploy_task::Status::Success)
}

async fn handle_env(item: WasmConfig, dir: &str) -> Result<deploy_task::Status> {
    let envs_file = format!(
        "{}/envs/{}-{}.envs.json",
        dir, item.user_id, item.project_id
    );
    let envs_dir = format!("{}/envs", dir);
    std::fs::create_dir_all(envs_dir)?;
    let content = item.content.unwrap_or_default();
    std::fs::write(&envs_file, content)?;
    memenvs::read_file(&envs_file).await?;
    info!("generate envs success: {}", envs_file);
    Ok(deploy_task::Status::Success)
}
