use super::utils::{get_playground, get_project, set_failed};
use crate::confs::WasmConfig;
use anyhow::{anyhow, Result};
use land_dao::{deploy_task, deploys, models, projects, settings, wasm_artifacts, workers};
use std::time::Duration;
use tokio::time::interval;
use tracing::{debug, info, instrument, warn};

/// init_background starts handling waitin deploy tasks
pub async fn init_background() {
    debug!("deployer init_waiting");
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_secs(1));
        ticker.tick().await;
        loop {
            match handle().await {
                Ok(_) => {}
                Err(e) => {
                    warn!("deployer wait handle error: {:?}", e);
                }
            };
            ticker.tick().await;
        }
    });
}

#[instrument("[DEPLOY-WAIT]")]
async fn handle() -> Result<()> {
    // find all waiting deploys
    let deploy_data = deploys::list_by_deploy_status(deploys::Status::WaitDeploy).await?;
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

async fn handle_internal(dp: &models::deploys::Model) -> Result<()> {
    debug!(id = dp.id, "Handle waiting");

    // 1. get dp's project
    let project = get_project(dp.project_id).await?;

    // 2. if project is not created by playground, currently only playground can create project
    if project.created_by != projects::CreatedBy::Playground.to_string() {
        return set_failed(
            dp.id,
            Some(dp.project_id),
            "Project not created by playground",
        )
        .await;
    }

    // 3. get playground
    let playground = get_playground(dp.project_id).await?;

    // 4. all data are ready, try compile playground source
    let agent_wasm_config = compile_playground(dp, &project, &playground).await?;

    // 5. create workers tasks
    create_worker_tasks(dp, &agent_wasm_config).await?;

    // 6. set deploy status to deploying
    deploys::set_deploy_status(dp.id, deploys::Status::Deploying, "Deploying").await?;
    projects::set_deploy_status(dp.project_id, deploys::Status::Deploying, "Deploying").await?;

    info!(id = dp.id, "Handle waiting OK");

    Ok(())
}

async fn compile_playground(
    dp: &models::deploys::Model,
    project: &models::project::Model,
    playground: &models::playground::Model,
) -> Result<WasmConfig> {
    // 4.1. set deploy status to compiling
    deploys::set_deploy_status(dp.id, deploys::Status::Compiling, "Compiling").await?;

    // 4.2. write playground source to temp file
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
    std::fs::write(&source_js, playground.source.clone())?;

    // 4.3. build js to wasm
    let target_wasm = dir
        .path()
        .join(format!("{}_{}.wasm", playground.project_id, playground.id));
    land_wasm_gen::componentize_js(
        source_js.to_str().unwrap(),
        target_wasm.to_str().unwrap(),
        None,
    )?;
    debug!("Compile success: {:?}", target_wasm);

    // 4.4. set uploading
    deploys::set_deploy_status(dp.id, deploys::Status::Uploading, "Uploading").await?;

    // 4.5. create wasm artifact record
    let now_text = chrono::Utc::now().format("%Y%m%d%H%M%S").to_string();
    let file_name = format!("{}/{}_{}.wasm", project.uuid, dp.domain, now_text);
    let file_data = std::fs::read(&target_wasm)?;
    let file_hash = format!("{:x}", md5::compute(&file_data));
    let file_size = file_data.len() as i32;
    let artifact_record = wasm_artifacts::create(
        dp.owner_id,
        dp.project_id,
        dp.id,
        &dp.task_id,
        &file_name,
        &file_hash,
        file_size,
    )
    .await?;
    debug!("Create wasm artifacts record: {:?}", artifact_record);

    // 4.6. save file to storage
    debug!("Save file to storage begin: {:?}", file_name);
    let real_filepath = format!("wasm/{}", file_name);
    crate::storage::save(&real_filepath, file_data).await?;
    debug!("Save file to storage end: {:?}", file_name);
    let target_url = crate::storage::build_url(&file_name).await?;
    debug!("Save file to storage url: {:?}", target_url);
    wasm_artifacts::set_success(artifact_record.id, Some(target_url.clone())).await?;

    // 4.7. generate agent wasm config
    let domain_settings = settings::get_domain_settings().await?;
    Ok(WasmConfig {
        user_id: dp.owner_id,
        project_id: dp.project_id,
        deploy_id: dp.id,
        task_id: dp.task_id.clone(),
        file_name,
        file_hash,
        download_url: target_url,
        domain: format!("{}.{}", dp.domain, domain_settings.domain_suffix),
        content: None,
    })
}

async fn create_worker_tasks(
    dp: &models::deploys::Model,
    agent_wasm_config: &WasmConfig,
) -> Result<()> {
    // 5.1. get online workers
    // if no worker online, set deploy status to failed
    let workers_value = workers::find_all(Some(workers::Status::Online)).await?;
    if workers_value.is_empty() {
        return Err(anyhow!("No worker online"));
    }

    // 5.2. prepare task content from agent wasm config
    let task_content = serde_json::to_string(agent_wasm_config)?;

    // 5.3. create details task for each worker
    let mut rips = vec![];
    for worker in workers_value.iter() {
        let task = deploy_task::create(
            dp,
            deploy_task::TaskType::DeployWasmToWorker,
            &task_content,
            worker.id,
            &worker.ip,
        )
        .await?;
        debug!("Create task: {:?}", task);
        rips.push(worker.ip.clone());
    }
    deploys::set_rips(dp.id, rips.join(","), rips.len() as i32).await?;

    Ok(())
}
