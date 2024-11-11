use anyhow::Result;
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, instrument, warn};

/// init_background starts handling review deploy tasks
pub async fn init_background() {
    debug!("deployer init_review");
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(1));
        ticker.tick().await;
        loop {
            match handle().await {
                Ok(_) => {}
                Err(e) => {
                    warn!("deployer review handle error: {:?}", e);
                }
            };
            ticker.tick().await;
        }
    });
}

#[instrument("[DEPLOY-REVIEW]")]
async fn handle() -> Result<()> {
    Ok(())
}
