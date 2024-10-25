use once_cell::sync::OnceCell;
use reqwest::Client;
use std::sync::Once;
use tracing::{debug, warn};

mod confs;
mod ip;
mod livings;
mod ping;

pub use confs::{get_confs, init_confs};
pub use ip::{init_ip, IP};
pub use livings::{init_refreshing, set_living};

static CLIENT: OnceCell<Client> = OnceCell::new();
static CLIENT_ONCE: Once = Once::new();

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
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

/// init_background starts background tasks
pub async fn init_background(addr: String, token: String, dir: String) {
    debug!("agent init_background");

    // init client
    CLIENT_ONCE.call_once(|| {
        let client = Client::new();
        CLIENT.set(client).unwrap();
    });

    // start ping task
    // register this worker and refresh every 10 seconds as a heartbeat
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(10));
        ticker.tick().await;
        loop {
            match ping::request(addr.clone(), token.clone(), dir.clone()).await {
                Ok(_) => {}
                Err(e) => {
                    warn!("agent ping error: {:?}", e);
                }
            };
            ticker.tick().await;
        }
    });
}
