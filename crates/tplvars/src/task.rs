use land_dao::models::deploy_task;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: i32,
    pub content: String,
    pub task_id: String,
    pub task_type: String,
    pub created_at: i64,
}

impl Task {
    pub fn new(m: &deploy_task::Model) -> Self {
        Self {
            id: m.id,
            content: m.task_content.clone(),
            task_id: m.task_id.clone(),
            task_type: m.task_type.clone(),
            created_at: m.created_at.and_utc().timestamp(),
        }
    }
}
