use axum::{extract::Query, response::IntoResponse, Json};
use land_dao::deploy_task;
use serde::Deserialize;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use super::{resp_json_ok, JsonError};

/// IPQuery is query string for /worker-api/tasks
#[derive(Deserialize, Debug)]
pub struct IPQuery {
    ip: String,
}

type TaskResponse = HashMap<String, String>;

/// tasks is /worker-api/tasks
pub async fn tasks(
    Query(q): Query<IPQuery>,
    Json(j): Json<TaskResponse>,
) -> Result<impl IntoResponse, JsonError> {
    // if tasks is not empty, update task status
    if !j.is_empty() {
        for (task_id, res) in j.iter() {
            if res == &deploy_task::Status::Success.to_string() {
                deploy_task::set_success(q.ip.clone(), task_id.clone()).await?;
                info!(ip = q.ip, "Task {} success", task_id);
            } else if res == &deploy_task::Status::Doing.to_string() {
                debug!(ip = q.ip, "Task {} doing", task_id);
            } else {
                deploy_task::set_failed(q.ip.clone(), task_id.clone(), res.to_string()).await?;
                warn!(ip = q.ip, "Task {} failed: {}", task_id, res);
            }
        }
    }

    let models = deploy_task::list(Some(q.ip), Some(deploy_task::Status::Doing), None).await?;
    if models.is_empty() {
        return Ok(resp_json_ok(vec![], None));
    }
    let tasks: Vec<land_tplvars::Task> = models.iter().map(land_tplvars::Task::new).collect();
    Ok(resp_json_ok(tasks, None))
}
