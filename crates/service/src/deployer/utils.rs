use anyhow::{anyhow, Result};
use land_dao::{deploys, models, playground, projects};
use tracing::{debug, warn};

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

/// set_success sets the deploy and projectstatus to success
pub(crate) async fn set_success(dp_id: i32, project_id: Option<i32>) -> Result<()> {
    deploys::set_deploy_status(dp_id, deploys::Status::Success, "Success").await?;
    if let Some(project_id) = project_id {
        projects::set_deploy_status(project_id, deploys::Status::Success, "Success").await?;
    }
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

/// refresh_state refreshes deploy state record
pub(crate) async fn refresh_state(dp: &models::deploys::Model) -> Result<()> {
    // if deploy type is envs, refresh state as env type
    if dp.deploy_type == deploys::DeployType::Envs.to_string() {
        deploys::refresh_state(
            dp.owner_id,
            dp.project_id,
            dp.id,
            dp.task_id.clone(),
            deploys::StateType::Envs,
        )
        .await?;
        debug!(dp_id = dp.id, "refresh env state");
        return Ok(());
    }

    // if deploy type is disabled, drop record in deploy-state table
    if dp.deploy_type == deploys::DeployType::Disabled.to_string() {
        deploys::drop_state(
            dp.owner_id,
            dp.project_id,
            dp.id,
            deploys::StateType::WasmDeploy,
        )
        .await?;
        debug!(
            dp_id = dp.id,
            "deploy type is disabled, drop record in deploy-state table"
        );
        return Ok(());
    }

    if dp.deploy_type == deploys::DeployType::Production.to_string()
        || dp.deploy_type == deploys::DeployType::Development.to_string()
    {
        // deploy is wasm case
        deploys::refresh_state(
            dp.owner_id,
            dp.project_id,
            dp.id,
            dp.task_id.clone(),
            deploys::StateType::WasmDeploy,
        )
        .await?;
        debug!(dp_id = dp.id, "refresh wasm state");
        return Ok(());
    }
    Err(anyhow!(
        "Unknown refresh state deploy type: {}",
        dp.deploy_type
    ))
}
