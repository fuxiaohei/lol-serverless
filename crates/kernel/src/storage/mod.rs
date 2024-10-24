use anyhow::{anyhow, Result};
use land_dao::settings;
use once_cell::sync::Lazy;
use opendal::{services::Memory, Operator};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tracing::{debug, info, instrument};

mod fs;
mod s3;

static CURRENT_SETTINGS: &str = "storage-current";

#[derive(Clone, Debug, Serialize, Deserialize)]
struct Current {
    pub current: String,
}

/// init_defaults init default values for storage settings
pub async fn init_defaults() -> Result<()> {
    let current: Option<Current> = settings::get(CURRENT_SETTINGS).await?;
    if current.is_none() {
        let current = Current {
            current: "fs".to_string(),
        };
        settings::set(CURRENT_SETTINGS, &current).await?;
        debug!("init storage current: {:?}", current);
    }
    fs::init_defaults().await?;
    s3::init_defaults().await?;
    Ok(())
}

/// get_current get current storage name
async fn get_current() -> Result<String> {
    let current: Option<Current> = settings::get(CURRENT_SETTINGS).await?;
    if current.is_none() {
        return Err(anyhow!("storage current not found"));
    }
    let current = current.unwrap().current;
    Ok(current)
}

/// STORAGE_KEY is the global storage key
static STORAGE_KEY: Lazy<Mutex<String>> = Lazy::new(|| Mutex::new("".to_string()));

/// STORAGE is the global storage operator
static STORAGE: Lazy<Mutex<Operator>> = Lazy::new(|| {
    let builder = Memory::default().root("/tmp");
    let op = Operator::new(builder).unwrap().finish();
    Mutex::new(op)
});

/// load_global load storage
#[instrument("[STORAGE]")]
pub async fn load_global() -> Result<()> {
    let current = get_current().await?;
    let mut storage_key = STORAGE_KEY.lock().await;

    if current == "fs" {
        let key = format!("fs-{}", fs::hash().await?);
        if !storage_key.eq(&key) {
            *storage_key = key;
            let op = fs::new_operator().await?;
            let mut storage = STORAGE.lock().await;
            *storage = op;
        }
        info!("load: fs");
        return Ok(());
    }

    if current == "s3" {
        let key = format!("s3-{}", s3::hash().await?);
        if !storage_key.eq(&key) {
            *storage_key = key;
            let op = s3::new_operator().await?;
            let mut storage = STORAGE.lock().await;
            *storage = op;
        }
        info!("load: s3");
        return Ok(());
    }

    Err(anyhow!("{} not supported", current))
}
