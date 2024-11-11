use crate::deployer::utils::{refresh_state, set_failed, set_success};
use anyhow::{anyhow, Result};
use land_dao::{deploy_task, deploys, models};
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, info, instrument, warn};

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
    let deploy_data = deploys::list_by_deploy_status(deploys::Status::Deploying).await?;
    if deploy_data.is_empty() {
        return Ok(());
    }
    info!("Review: {}", deploy_data.len());
    for dp in deploy_data.iter() {
        match handle_one(dp).await {
            Ok(_) => {}
            Err(e) => {
                set_failed(dp.id, Some(dp.project_id), e.to_string().as_str()).await?;
                warn!(dp_id = dp.id, "deployer waiting handle error: {:?}", e);
            }
        }
    }
    Ok(())
}

async fn handle_one(dp: &models::deploys::Model) -> Result<()> {
    debug!("Handle review: {}", dp.id);
    let tasks = deploy_task::list(None, None, Some(dp.task_id.clone())).await?;
    if tasks.len() as i32 != dp.total_count {
        return Err(anyhow!("Task count not match"));
    }
    let mut success_count = 0;
    let mut failed_message = "".to_string();
    let mut done_count = 0;
    for task in tasks.iter() {
        // 1. task is still doing, skip review this task
        if task.status == deploy_task::Status::Doing.to_string() {
            continue;
        }

        // 1.1 task is not doing, mean task is success or failed, must be done
        done_count += 1;

        // 2. task is success
        if task.status == deploy_task::Status::Success.to_string() {
            debug!(
                dp_id = dp.id,
                ip = task.worker_ip,
                task_id = dp.task_id,
                "task success"
            );
            success_count += 1;
            continue;
        }

        // 3. task is failed
        if task.status == deploy_task::Status::Failed.to_string() {
            debug!(
                dp_id = dp.id,
                ip = task.worker_ip,
                task_id = dp.task_id,
                "task failed: {}",
                task.message,
            );
            failed_message.clone_from(&task.message);
            continue;
        }
    }
    // 4. if all tasks are done, update deploy status
    if done_count == tasks.len() as i32 {
        let project_id = if dp.deploy_type != deploys::DeployType::Envs.to_string() {
            Some(dp.project_id)
        } else {
            None
        };
        if done_count != success_count {
            info!(dp_id = dp.id, "review failed: {:?}", failed_message);
            set_failed(dp.id, project_id, &failed_message).await?;
            return Ok(());
        }
        set_success(dp.id, project_id).await?;
        refresh_state(dp).await?;
        info!(dp_id = dp.id, "review success");
    } else {
        info!(dp_id = dp.id, "review not done");
    }
    Ok(())
}
