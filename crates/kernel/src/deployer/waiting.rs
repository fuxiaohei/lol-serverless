use anyhow::Result;
use land_dao::{
    deploys::{self, Status},
    models::deployment,
    projects,
};
use tracing::{debug, info, instrument, warn};

/// init_background starts handling waiting deploy tasks
pub async fn init_background() {
    debug!("deployer init_waiting");
    tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(1));
        ticker.tick().await;
        loop {
            match handle().await {
                Ok(_) => {}
                Err(e) => {
                    warn!("deployer waiting handle error: {:?}", e);
                }
            };
            ticker.tick().await;
        }
    });
}

#[instrument("[DEPLOY-WAITING]", skip_all)]
async fn handle() -> Result<()> {
    let deploy_data = deploys::list_by_deploy_status(Status::WaitDeploy).await?;
    if deploy_data.is_empty() {
        // debug!("No waiting");
        return Ok(());
    }
    info!("Found: {}", deploy_data.len());
    for dp in deploy_data.iter() {
        match handle_internal(dp).await {
            Ok(_) => {}
            Err(e) => {
                set_failed(dp.id, Some(dp.project_id), e.to_string().as_str()).await?;
                warn!(dp_id = dp.id, "deployer waiting handle error: {:?}", e);
            }
        }
    }
    Ok(())
}

async fn handle_internal(dp: &deployment::Model) -> Result<()> {
    debug!("Handle waiting: {}", dp.id);
    Ok(())
}

/// set_failed sets the deploy and project status to failed
pub(crate) async fn set_failed(
    dp_id: i32,
    project_id: Option<i32>,
    mut message: &str,
) -> Result<()> {
    // find last \n flag in 255 chars in message
    if message.len() > 255 {
        message = &message[..255];
    }
    deploys::set_deploy_status(dp_id, deploys::Status::Failed, message).await?;
    if let Some(project_id) = project_id {
        projects::set_deploy_status(project_id, deploys::Status::Failed, message).await?;
    }
    warn!(dp_id = dp_id, "set failed: {}", message);
    Ok(())
}
