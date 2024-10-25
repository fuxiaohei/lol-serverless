use anyhow::{anyhow, Result};
use serde::Deserialize;
use tracing::instrument;
use super::WasmConfig;

#[derive(Deserialize, Default, Clone, Debug)]
struct PingResponse {
    pub status: String,
    pub message: String,
    pub data: Vec<WasmConfig>,
}

/// request agent ping
#[instrument("[AGT-PING]", skip_all)]
pub async fn request(addr: String, token: String, dir: String) -> Result<()> {
    let ipinfo = super::ip::get_ip().await;
    let client = super::CLIENT.get().unwrap();

    let api = format!("{}/_worker_api/ping", addr);
    let token = format!("Bearer {}", token);
    let res = client
        .post(api)
        .header("Authorization", token)
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

    let resp: PingResponse = res.json().await?;
    let conf_file = format!("{}/confs.json", dir);
    if resp.status != "ok"{
        return Err(anyhow!("sync error: {}", resp.message));
    }
    // debug!("sync data: {}, {}", resp.status, resp.message);
    // write resp to file
    std::fs::write(conf_file, serde_json::to_string(&resp.data).unwrap()).unwrap();
    Ok(())
}
