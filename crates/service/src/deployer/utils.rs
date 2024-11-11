use anyhow::{anyhow, Result};
use land_dao::{deploys, models, playground, projects};
use tracing::warn;

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

/// get_project gets project by id
pub(crate) async fn get_project(id: i32) -> Result<models::project::Model> {
    let project = projects::get_by_id(id).await?;
    if project.is_none() {
        return Err(anyhow!("Project not found"));
    }
    Ok(project.unwrap())
}

/// get_playground gets playground by project id
pub(crate) async fn get_playground(project_id: i32) -> Result<models::playground::Model> {
    // 3. get playground
    let playground = playground::get_by_project(project_id).await?;
    if playground.is_none() {
        return Err(anyhow!("Playground not found"));
    }
    Ok(playground.unwrap())
}
