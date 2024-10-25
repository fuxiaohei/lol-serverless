use anyhow::Result;
use land_dao::workers;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use tokio::sync::Mutex;
use tracing::{debug, info, instrument, warn};

#[derive(Debug, Clone)]
pub struct LivingAgent {
    pub ip: super::IP,
    pub last_seen: i64,
}

/// LIVINGS is a map of ipinfo::Info
static LIVINGS: Lazy<Mutex<HashMap<String, LivingAgent>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

///  set_living
pub async fn set_living(ip: super::IP) {
    let mut livings = LIVINGS.lock().await;
    let live_info = LivingAgent {
        ip: ip.clone(),
        last_seen: chrono::Utc::now().timestamp(),
    };
    livings.insert(ip.ip.clone(), live_info);
}

/// init_refreshing starts livings agent refreshing to workers table to update heartbeats
pub async fn init_refreshing() {
    debug!("agent init_refreshing");

    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(10));
        ticker.tick().await;
        loop {
            match refresh().await {
                Ok(_) => {}
                Err(e) => {
                    warn!("agent livings refresh error: {:?}", e);
                }
            };
            ticker.tick().await;
        }
    });
}

#[instrument("[AGT-LIVINGS]")]
async fn refresh() -> Result<()> {
    let workers = workers::find_all(None).await?;
    let livings = LIVINGS.lock().await;
    let now = chrono::Utc::now().timestamp();

    let mut onlines = vec![];
    let mut all_ips = vec![];
    for worker in workers.iter() {
        all_ips.push(worker.ip.clone());

        let living = livings.get(&worker.ip);
        // if not found in livings, check if worker last seen is older than 60 seconds
        if living.is_none() {
            if now - worker.updated_at.and_utc().timestamp() > 60
                && worker.status != workers::Status::Offline.to_string()
            {
                workers::set_offline(&worker.ip).await?;
                info!(ip = &worker.ip, "Set offline by not living");
            }
            continue;
        }
        // if found in livings, check if worker last seen is older than 60 seconds
        if now - living.unwrap().last_seen > 60 {
            if worker.status != workers::Status::Offline.to_string() {
                workers::set_offline(&worker.ip).await?;
                info!(ip = &worker.ip, "Set offline by living expired");
            }
            continue;
        }

        // worker found in livings, check if worker status is offline
        if worker.status != workers::Status::Online.to_string() {
            info!(ip = &worker.ip, "Set online by living");
        }

        onlines.push(worker.ip.clone());
    }

    // set onlines
    workers::set_onlines(onlines).await?;

    for (ip, v) in livings.iter() {
        if all_ips.contains(ip) {
            continue;
        }
        // worker not found in livings, create new worker record
        let ip_info = serde_json::to_string(&v.ip)?;
        let hostname = v.ip.hostname.clone().unwrap_or("".to_string());
        let region = format!("{}, {}, {}", v.ip.city, v.ip.region, v.ip.country);
        let wk = workers::create(&v.ip.ip, "", &hostname, &region, &ip_info).await?;
        info!(ip = ip, "Create new worker: {:?}", wk);
    }

    Ok(())
}
