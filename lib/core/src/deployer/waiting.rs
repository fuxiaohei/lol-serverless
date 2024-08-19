use crate::agent::Item;
use anyhow::Result;
use land_dao::{
    deploy_task,
    deploys::{self, Status},
    models::deployment,
    playground, projects, settings, store, workers,
};
use tracing::{debug, info, instrument, warn};

/// init_waiting starts handling waiting deploy tasks
pub async fn init_waiting() {
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

/// set_failed sets the deploy and project status to failed
pub(crate) async fn set_failed(dp_id: i32, project_id: i32, mut message: &str) -> Result<()> {
    // find last \n flag in 255 chars in message
    if message.len() > 255 {
        message = &message[..255];
    }
    deploys::set_deploy_status(dp_id, deploys::Status::Failed, message).await?;
    projects::set_deploy_status(project_id, dp_id, deploys::Status::Failed, message).await?;
    warn!(dp_id = dp_id, "set failed: {}", message);
    Ok(())
}

/// set_success sets the deploy and projectstatus to success
pub(crate) async fn set_success(dp_id: i32, project_id: i32) -> Result<()> {
    deploys::set_deploy_status(dp_id, deploys::Status::Success, "Success").await?;
    projects::set_deploy_status(project_id, dp_id, deploys::Status::Success, "Success").await?;
    Ok(())
}

#[instrument("[DEPLOY-WAITING]")]
async fn handle() -> Result<()> {
    let deploy_data = deploys::list_by_deploy_status(Status::WaitDeploy).await?;
    if deploy_data.is_empty() {
        // debug!("No waiting");
        return Ok(());
    }
    info!("Waitings: {}", deploy_data.len());
    for dp in deploy_data.iter() {
        match handle_internal(dp).await {
            Ok(_) => {}
            Err(e) => {
                set_failed(dp.id, dp.project_id, e.to_string().as_str()).await?;
                warn!(dp_id = dp.id, "deployer waiting handle error: {:?}", e);
            }
        }
    }
    Ok(())
}

async fn handle_internal(dp: &deployment::Model) -> Result<()> {
    debug!("Handle waiting: {}", dp.id);

    match dp.deploy_type.parse::<deploys::DeployType>()? {
        deploys::DeployType::Production | deploys::DeployType::Development => {
            handle_deploy_wasm(dp).await
        }
        deploys::DeployType::Disabled => handle_deploy_disable(dp).await,
        // _ => Err(anyhow!("Deploy type '{}' not supported", dp.deploy_type)),
    }
}

async fn handle_deploy_wasm(dp: &deployment::Model) -> Result<()> {
    // 1. get project
    let project = projects::get_by_id(dp.project_id).await?;
    if project.is_none() {
        return set_failed(dp.id, dp.project_id, "Project not found").await;
    }
    let project = project.unwrap();

    // 2. if project is not created by playground, currently only playground can create project
    if project.created_by != projects::CreatedBy::Playground.to_string() {
        return set_failed(dp.id, dp.project_id, "Project not created by playground").await;
    }

    // 3. get playground
    let playground = playground::get_by_project(dp.project_id).await?;
    if playground.is_none() {
        return set_failed(dp.id, dp.project_id, "Playground not found").await;
    }
    let playground = playground.unwrap();

    // 4. set compiling
    deploys::set_deploy_status(dp.id, deploys::Status::Compiling, "Compiling").await?;

    // 5. write source code to file
    let dir = tempfile::Builder::new().prefix("runtime-land").tempdir()?;
    let source_js = dir
        .path()
        .join(format!("{}_{}.js", playground.project_id, playground.id));
    debug!(
        "Write playground source to: {:?}, size: {}",
        source_js,
        playground.source.len()
    );
    let source_dir = source_js.parent().unwrap().to_path_buf();
    std::fs::create_dir_all(source_dir)?;
    std::fs::write(&source_js, playground.source)?;

    // 6. build js to wasm
    let target_wasm = dir
        .path()
        .join(format!("{}_{}.wasm", playground.project_id, playground.id));
    land_wasm_gen::componentize_js(
        source_js.to_str().unwrap(),
        target_wasm.to_str().unwrap(),
        None,
    )?;
    debug!("Compile success: {:?}", target_wasm);

    // 7. set uploading
    deploys::set_deploy_status(dp.id, deploys::Status::Uploading, "Uploading").await?;

    // 8. create storage record
    let now_text = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
    let file_name = format!("{}/{}_{}.wasm", project.uuid, dp.domain, now_text);
    let file_data = std::fs::read(&target_wasm)?;
    let file_hash = format!("{:x}", md5::compute(&file_data));
    let file_size = file_data.len() as i32;
    let storage_record = store::create(
        dp.owner_id,
        dp.project_id,
        dp.id,
        &dp.task_id,
        &file_name,
        &file_hash,
        file_size,
    )
    .await?;
    debug!("Create storage record: {:?}", storage_record);

    // 9. save file to storage
    debug!("Save file to storage begin: {:?}", file_name);
    crate::storage::save(&file_name, file_data).await?;
    debug!("Save file to storage end: {:?}", file_name);
    let target_url = crate::storage::build_url(&file_name).await?;
    debug!("Save file to storage url: {:?}", target_url);
    store::set_success(storage_record.id, Some(target_url.clone())).await?;

    // 10. create details task for each worker
    let workers_value = workers::find_all(Some(workers::Status::Online)).await?;
    if workers_value.is_empty() {
        warn!(dp_id = dp.id, "No worker online");
        return set_failed(dp.id, dp.project_id, "No worker online").await;
    }

    // 11. create conf values
    let domain_settings = settings::get_domain_settings().await?;
    let item = Item {
        user_id: dp.owner_id,
        project_id: dp.project_id,
        deploy_id: dp.id,
        task_id: dp.task_id.clone(),
        file_name,
        file_hash,
        download_url: target_url,
        domain: format!("{}.{}", dp.domain, domain_settings.domain_suffix),
    };
    let item_content = serde_json::to_string(&item)?;

    // 12. create details task for each worker
    let mut rips = vec![];
    for worker in workers_value.iter() {
        let task = deploy_task::create(
            dp,
            deploy_task::TaskType::DeployWasmToWorker,
            &item_content,
            worker.id,
            &worker.ip,
        )
        .await?;
        debug!("Create task: {:?}", task);
        rips.push(worker.ip.clone());
    }

    // 13. update deployment status, to trigger review logic
    deploys::set_rips(dp.id, rips.join(","), rips.len() as i32).await?;
    deploys::set_deploy_status(dp.id, deploys::Status::Deploying, "Deploying").await?;

    Ok(())
}

async fn handle_deploy_disable(dp: &deployment::Model) -> Result<()> {
    // 1. get online workers
    let workers_value = workers::find_all(Some(workers::Status::Online)).await?;
    if workers_value.is_empty() {
        warn!(dp_id = dp.id, "No worker online");
        return set_failed(dp.id, dp.project_id, "No worker online").await;
    }

    // 2. create conf values
    let domain_settings = settings::get_domain_settings().await?;
    let item = Item {
        user_id: dp.owner_id,
        project_id: dp.project_id,
        deploy_id: dp.id,
        task_id: dp.task_id.clone(),
        file_name: String::new(),
        file_hash: String::new(),
        download_url: String::new(),
        domain: format!("{}.{}", dp.domain, domain_settings.domain_suffix),
    };
    let item_content = serde_json::to_string(&item)?;

    // 3. create details task for each worker
    let mut rips = vec![];
    for worker in workers_value.iter() {
        let task = deploy_task::create(
            dp,
            deploy_task::TaskType::DisableWasm,
            &item_content,
            worker.id,
            &worker.ip,
        )
        .await?;
        debug!("Create task: {:?}", task);
        rips.push(worker.ip.clone());
    }

    // 4. update deployment status, to trigger review logic
    deploys::set_rips(dp.id, rips.join(","), rips.len() as i32).await?;
    deploys::set_deploy_status(dp.id, deploys::Status::Deploying, "Deploying").await?;

    Ok(())
}
