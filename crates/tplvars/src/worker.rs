use land_dao::models::worker_node;
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Worker {
    pub id: i32,
    pub ip: String,
    pub hostname: String,
    pub region: String,
    pub status: String,
    pub created_at: i64,
    pub last_seen_at: i64,
}

impl Worker {
    pub fn new(model: &worker_node::Model) -> Self {
        Worker {
            id: model.id,
            ip: model.ip.clone(),
            hostname: model.hostname.clone(),
            region: model.region.clone(),
            status: model.status.clone(),
            created_at: model.created_at.and_utc().timestamp(),
            last_seen_at: model.updated_at.and_utc().timestamp(),
        }
    }
}
