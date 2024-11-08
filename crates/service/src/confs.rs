use anyhow::Result;
use land_dao::wasm_artifacts;
use land_utils::crypt;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use tokio::{sync::Mutex, time::Instant};
use tracing::{debug, instrument, warn};

/// WasmConfig is the config for wasm file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmConfig {
    pub user_id: i32,
    pub project_id: i32,
    pub deploy_id: i32,
    pub task_id: String,
    pub file_name: String,
    pub download_url: String,
    pub file_hash: String,
    pub domain: String,
    pub content: Option<String>,
}

lazy_static! {
    static ref CONFS: Mutex<(String, Vec<WasmConfig>)> = Mutex::new(("".to_string(), vec![]));
}

/// init_confs_background is used to generate confs in background
pub async fn init_confs_background() {
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(60));
        loop {
            ticker.tick().await;
            match generate().await {
                Ok(_) => {}
                Err(e) => {
                    warn!("confs::gen error: {}", e);
                }
            }
        }
    });
}

/// gen generate config
#[instrument("[AGENT-CONFS]")]
pub async fn generate() -> anyhow::Result<()> {
    let st = Instant::now();
    let ids = land_dao::deploys::success_ids().await?;
    if ids.is_empty() {
        return Ok(());
    }
    debug!("Generate confs for deploys: {:?}", ids);
    let ids_hash = crypt::obj_hash(ids.clone())?;
    let mut confs = CONFS.lock().await;
    if confs.0 == ids_hash {
        // debug!("No changed");
        return Ok(());
    }
    confs.0.clone_from(&ids_hash);
    confs.1 = gen_confs(ids).await?;
    let elasped = st.elapsed().as_millis();
    debug!("Generated in {}ms, hash: {}", elasped, ids_hash);
    Ok(())
}

async fn gen_confs(ids: Vec<i32>) -> Result<Vec<WasmConfig>> {
    let domain_settings = land_dao::settings::get_domain_settings().await?;

    // get deploys data
    let deploy_data = land_dao::deploys::list_by_ids(ids.clone()).await?;
    let storage_data = wasm_artifacts::list_success_by_deploys(ids).await?;

    // build confs
    let mut items = Vec::new();
    for deploy in deploy_data {
        let storage_item = storage_data.get(&deploy.id);
        if storage_item.is_none() {
            warn!("Storage not found for deploy {}", deploy.id);
            continue;
        }
        let storage_item = storage_item.unwrap();
        let domain = format!("{}.{}", deploy.domain, domain_settings.domain_suffix);
        let item = WasmConfig {
            user_id: deploy.owner_id,
            project_id: deploy.project_id,
            deploy_id: deploy.id,
            task_id: deploy.task_id.clone(),
            file_name: storage_item.path.clone(),
            download_url: storage_item.file_target.clone(),
            file_hash: storage_item.file_hash.clone(),
            domain,
            content: None,
        };
        items.push(item);
    }
    Ok(items)
}

/// get get config
pub async fn get() -> (String, Vec<WasmConfig>) {
    CONFS.lock().await.clone()
}
