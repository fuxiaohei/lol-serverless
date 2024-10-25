use anyhow::Result;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use tracing::info;

/// IP is the struct for ipinfo data
#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct IP {
    pub ip: String,
    pub city: String,
    pub region: String,
    pub country: String,
    pub loc: String,
    pub org: String,
    pub timezone: String,
    pub hostname: Option<String>,
}

const IPINFO_LINK: &str = "https://ipinfo.io/json";

/// IPDATA is global once cell for ipinfo data
static IPDATA: OnceCell<IP> = OnceCell::new();

/// init_ip gets ip info from ipinfo.io
pub async fn init_ip(ip: Option<String>) -> Result<()> {
    if let Some(ip) = ip {
        IPDATA
            .set(IP {
                ip,
                ..Default::default()
            })
            .unwrap();
        return Ok(());
    }
    let resp = reqwest::get(IPINFO_LINK).await?;
    let mut ip_info: IP = resp.json().await?;
    ip_info.hostname = Some(land_utils::get_hostname()?);
    info!("IP info: {:?}", ip_info);
    IPDATA.set(ip_info).unwrap();
    Ok(())
}
/// get gets ip info from global variable
pub async fn get_ip() -> IP {
    let ip_data = IPDATA.get().unwrap();
    ip_data.clone()
}
