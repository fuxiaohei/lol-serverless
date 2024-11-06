use super::UrlBuilder;
use anyhow::Result;
use land_dao::settings;
use land_utils::crypt;
use opendal::{services::S3, Operator};
use serde::{Deserialize, Serialize};
use tracing::debug;

static S3_SETTINGS: &str = "storage-s3";

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    pub bucket: String,
    pub region: String,
    pub endpoint: String,
    pub access_key: String,
    pub secret_key: String,
    pub directory: Option<String>,
    pub url: Option<String>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            bucket: "".to_string(),
            region: "".to_string(),
            endpoint: "".to_string(),
            access_key: "".to_string(),
            secret_key: "".to_string(),
            directory: None,
            url: None,
        }
    }
}

/// init_defaults init default values for s3 storage settings
pub async fn init_defaults() -> Result<()> {
    let settings: Option<Settings> = settings::get(S3_SETTINGS).await?;
    if settings.is_none() {
        let setting = Settings::default();
        settings::set(S3_SETTINGS, &setting).await?;
        debug!("init s3 storage settings: {:?}", setting);
    }
    Ok(())
}

/// get get s3 storage settings
pub async fn get() -> Result<Settings> {
    let settings: Option<Settings> = settings::get(S3_SETTINGS).await?;
    if settings.is_none() {
        return Err(anyhow::anyhow!("s3 storage settings not found"));
    }
    Ok(settings.unwrap())
}

/// set set s3 storage settings
pub async fn set(s: Settings) -> Result<()> {
    settings::set(S3_SETTINGS, &s).await?;
    Ok(())
}

/// hash hash s3 storage settings
pub async fn hash() -> Result<String> {
    let settings = get().await?;
    crypt::obj_hash(settings)
}

/// new_operator generate a new s3 storage operator
pub async fn new_operator() -> Result<Operator> {
    let settings = get().await?;
    debug!("s3 storage settings: {:?}", settings.endpoint);
    let builder = S3::default()
        .root(&settings.directory.unwrap_or("/".to_string()))
        .bucket(&settings.bucket)
        .region(&settings.region)
        .endpoint(&settings.endpoint)
        .access_key_id(&settings.access_key)
        .secret_access_key(&settings.secret_key)
        .batch_max_operations(100);
    let op = Operator::new(builder)?.finish();
    Ok(op)
}

impl UrlBuilder for Settings {
    fn build_url(&self, name: &str) -> String {
        let mut u = self
            .url
            .clone()
            .unwrap_or_else(|| format!("{}/{}", self.endpoint.trim_end_matches('/'), self.bucket))
            .trim_end_matches('/')
            .to_string();
        if self.directory.is_some() {
            u.push_str(&format!(
                "/{}",
                self.directory.clone().unwrap().trim_end_matches('/')
            ));
        }
        format!("{}/{}", u, name)
    }
}
