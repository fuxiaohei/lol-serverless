use anyhow::{anyhow, Result};
use land_utils::localip;
use serde::Deserialize;
use std::sync::{Once, OnceLock};
use tracing::{debug, instrument, warn};

static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
static CLIENT_ONCE: Once = Once::new();

/// init start a loop to ping server every 10 seconds
pub async fn init(addr: String, token: String, dir: String) {
    debug!("agent init_heartbeat_ping");

    // init client
    CLIENT_ONCE.call_once(|| {
        let client = reqwest::Client::new();
        CLIENT.set(client).unwrap();
    });

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(10));
        ticker.tick().await;
        loop {
            match heartbeat_ping(addr.clone(), token.clone(), dir.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    warn!("agent heartbeat_ping error: {:?}", e);
                }
            };
            ticker.tick().await;
        }
    });
}

#[derive(Deserialize, Default, Clone, Debug)]
struct SyncResponse {
    pub status: String,
    pub message: String,
    pub data: Vec<crate::confs::WasmConfig>,
}

#[instrument("[AGT-SYNC]", skip_all)]
async fn heartbeat_ping(addr: String, token: String, dir: String) -> Result<()> {
    let ipinfo = localip::get().await;
    let client = CLIENT.get().unwrap();

    let api = format!("{}/_worker_api/heartbeat", addr);
    let token = format!("Bearer {}", token);
    let res = client
        .post(api)
        .header("Authorization", token)
        .header("X-Md5", "".to_string())
        .json(&ipinfo)
        .send()
        .await?;

    let status_code = res.status().as_u16();
    if status_code == 304 {
        // debug!("no change");
        return Ok(());
    }
    // 400+ is error
    if status_code >= 400 {
        let content = res.text().await?;
        return Err(anyhow!("Bad status:{}, Error:{}", status_code, content));
    }
    let resp: SyncResponse = res.json().await?;
    let conf_file = format!("{}/confs.json", dir);
    if resp.status != "ok" {
        return Err(anyhow!("sync error: {}", resp.message));
    }
    // debug!("sync data: {}, {}", resp.status, resp.message);
    // write resp to file
    std::fs::write(conf_file, serde_json::to_string(&resp.data).unwrap()).unwrap();
    Ok(())
}
