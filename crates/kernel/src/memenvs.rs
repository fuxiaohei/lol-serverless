use anyhow::Result;
use land_utils::crypt;
use lazy_static::lazy_static;
use serde::Deserialize;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tracing::{debug, instrument, warn};

/// EnvsMap env map
pub type EnvsMap = HashMap<String, String>;

lazy_static! {
    static ref ENVS: Mutex<HashMap<String, EnvsMap>> = Mutex::new(HashMap::new());
}

#[derive(Deserialize, Debug)]
struct EnvContent {
    pub secret: String,
    pub values: String,
}

#[instrument("[ENVS]", skip_all)]
pub async fn init_envs(dir: String) -> Result<()> {
    debug!("Init envs: {}", dir);
    std::fs::create_dir_all(&dir)?;
    let mut global_envs = ENVS.lock().await;
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry?;
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if !path.to_str().unwrap().ends_with(".envs.json") {
            continue;
        }
        let basepath = path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .trim_end_matches(".envs.json");
        let content = std::fs::read_to_string(path)?;
        let env_content = match serde_json::from_str::<EnvContent>(&content) {
            Ok(v) => v,
            Err(e) => {
                warn!("Error parse envs file: {}, {}", path.display(), e);
                continue;
            }
        };
        let envs = match crypt::decode(&env_content.secret, &env_content.values) {
            Ok(v) => v,
            Err(e) => {
                warn!("Error decode envs file: {}, {}", path.display(), e);
                continue;
            }
        };
        global_envs.insert(basepath.to_string(), envs);
        debug!("Load envs: {}", path.display());
    }
    Ok(())
}

/// get envs
pub async fn get(key: &str) -> Option<EnvsMap> {
    let global_envs = ENVS.lock().await;
    global_envs.get(key).cloned()
}

/// read_file read file and load into ENVsMap
pub async fn read_file(file: &str) -> Result<()> {
    let fpath = std::path::Path::new(file);
    let basepath = fpath
        .file_name()
        .unwrap()
        .to_str()
        .unwrap()
        .trim_end_matches(".envs.json");
    let content = std::fs::read_to_string(fpath)?;
    let env_content = serde_json::from_str::<EnvContent>(&content)?;
    let envs = crypt::decode(&env_content.secret, &env_content.values)?;
    let mut global_envs = ENVS.lock().await;
    global_envs.insert(basepath.to_string(), envs);
    debug!("Load envs: {}", fpath.display());
    Ok(())
}
