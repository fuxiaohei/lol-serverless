use anyhow::{anyhow, Result};
use land_dao::settings;
use opendal::{services::Memory, Operator};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
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

/// set_current set current storage name
async fn set_current(current: &str) -> Result<()> {
    let current = Current {
        current: current.to_string(),
    };
    settings::set(CURRENT_SETTINGS, &current).await?;
    Ok(())
}

/// STORAGE_KEY is the global storage key
static STORAGE_KEY: LazyLock<Mutex<String>> = LazyLock::new(|| Mutex::new("".to_string()));

/// STORAGE is the global storage operator
static STORAGE: LazyLock<Mutex<Operator>> = LazyLock::new(|| {
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

/// Vars is the storage settings template variables
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Vars {
    pub current: String,
    pub fs: fs::Settings,
    pub s3: s3::Settings,
}

impl Vars {
    /// get storage settings and convert to template variables
    pub async fn get() -> Result<Self> {
        let current: Option<Current> = settings::get(CURRENT_SETTINGS).await?;
        let current = current.unwrap();
        Ok(Self {
            current: current.current.clone(),
            fs: fs::get().await?,
            s3: s3::get().await?,
        })
    }
}

/// Form is the storage settings form
#[derive(Debug, Deserialize)]
pub struct Form {
    pub checked: String,
    pub endpoint: Option<String>,
    pub bucket: Option<String>,
    pub region: Option<String>,
    pub access_key: Option<String>,
    pub secret_key: Option<String>,
    pub directory: Option<String>,
    pub access_url: Option<String>,
}

/// update_by_form update storage settings by form
pub async fn update_by_form(f: Form) -> Result<()> {
    if f.checked == "s3" {
        let value = s3::Settings {
            endpoint: f.endpoint.unwrap_or_default(),
            bucket: f.bucket.unwrap_or_default(),
            region: f.region.unwrap_or_default(),
            access_key: f.access_key.unwrap_or_default(),
            secret_key: f.secret_key.unwrap_or_default(),
            directory: f.directory.clone(),
            url: f.access_url.clone(),
        };
        s3::set(value).await?;
        set_current("s3").await?;
    } else if f.checked == "fs" {
        let value = fs::Settings {
            local_path: f.directory.unwrap_or_default(),
            local_url: f.access_url.unwrap_or_default(),
        };
        fs::set(value).await?;
        set_current("fs").await?;
    }

    // reload storage operator
    load_global().await?;
    Ok(())
}

/// save bytes storage
pub async fn save(name: &str, data: Vec<u8>) -> Result<()> {
    let storage = STORAGE.lock().await;
    storage.write(name, data).await?;
    Ok(())
}

trait UrlBuilder {
    fn build_url(&self, name: &str) -> String;
}

/// build_url build url
pub async fn build_url(name: &str) -> Result<String> {
    let current = get_current().await?;
    if current == "fs" {
        let settings = fs::get().await?;
        Ok(settings.build_url(name))
    } else if current == "s3" {
        let settings = s3::get().await?;
        Ok(settings.build_url(name))
    } else {
        Err(anyhow!("storage {} not supported", current))
    }
}
