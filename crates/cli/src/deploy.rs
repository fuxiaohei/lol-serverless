use anyhow::Result;
use land_core::meta::Meta;
use land_rpc::client::Client;
use land_rpc::{DeploymentResponse, ProjectResponse};
use std::path::Path;
use tracing::{debug, info, warn};

pub async fn deploy(
    meta: &mut Meta,
    mut project_name: String,
    token: String,
    addr: String,
    is_production: bool,
) {
    debug!("deploy: {:?}", meta);

    let output = meta.get_output();
    debug!("output: {:?}", output);

    // if output file is not exist, suggest to run build command
    if !Path::new(&output).exists() {
        warn!("output file not found, \nplease run `land-cli build`");
        return;
    }
    if project_name.is_empty() {
        project_name = meta.get_project_name();
    }
    if project_name.is_empty() {
        project_name = meta.generate_project_name();
    }
    println!("Fetching Project '{project_name}'");

    // fetch project info
    let mut client = Client::new(addr, token).await.unwrap();
    let project = fetch_project(&mut client, project_name, meta.language.clone())
        .await
        .expect("Fetch project '{project_name}' failed")
        .unwrap();

    // upload wasm file to project
    let wasm_binary = std::fs::read(output).unwrap();
    println!(
        "Uploading assets to project '{project_name}', size: {size} KB",
        project_name = project.name,
        size = wasm_binary.len() / 1024,
    );
    let deployment = create_deploy(&mut client, &project, wasm_binary, is_production)
        .await
        .unwrap();

    debug!("deploy: {:?}", deployment);

    println!("Deployed to project '{}' success\n", project.name,);
    if is_production {
        println!("Deploy to Production");
    }
    println!("View at:");
    println!("- {}", deployment.url);
}

async fn fetch_project(
    client: &mut Client,
    project_name: String,
    language: String,
) -> Result<Option<ProjectResponse>> {
    // fetch project
    let mut project = client
        .fetch_project(project_name.clone(), language.clone())
        .await
        .map_err(|e| anyhow::anyhow!("fetch project failed: {:?}", e))?;

    // if project is not exist, create empty project with name
    if project.is_none() {
        info!("Project not found, create '{project_name}' project");
        project = client
            .create_project(project_name.clone(), language.clone())
            .await
            .unwrap_or_else(|e| {
                warn!("create project failed: {:?}", e);
                None
            });
        info!(
            "Project '{project_name}' created",
            project_name = project_name
        );
    }
    Ok(project)
}

async fn create_deploy(
    client: &mut Client,
    project: &ProjectResponse,
    binary: Vec<u8>,
    is_production: bool,
) -> Option<DeploymentResponse> {
    client
        .create_deployment(
            project.name.clone(),
            project.uuid.clone(),
            binary,
            "application/wasm".to_string(),
            is_production,
        )
        .await
        .unwrap_or_else(|e| {
            warn!("create deployment failed: {:?}", e);
            None
        })
}
