use super::ServerError;
use crate::{
    routers::{utils::redirect, HtmlMinified},
    templates::Engine,
};
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension};
use land_dao::projects;
use land_service::examples;
use land_tplvars::{new_vars, BreadCrumbKey, Project};
use serde::Serialize;
use tracing::{debug, info};

/// index shows the projects page
pub async fn index(
    engine: Engine,
    Extension(user): Extension<land_tplvars::User>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Data {
        pub projects: Vec<Project>,
    }
    let (projects, _) = projects::list(Some(user.id), None, 1, 999).await?;
    debug!(
        owner_id = user.id,
        "List projects, count: {}",
        projects.len()
    );
    Ok(HtmlMinified(
        "projects.hbs",
        engine,
        new_vars(
            "Projects",
            BreadCrumbKey::Projects,
            Some(user),
            Data {
                projects: Project::new_from_models(projects, false).await?,
            },
        ),
    ))
}

/// new is handler for projects new page, /new
pub async fn new(
    Extension(user): Extension<land_tplvars::User>,
    engine: Engine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Data {
        pub examples: Vec<examples::Item>,
    }
    let examples = examples::defaults();
    Ok(HtmlMinified(
        "new.hbs",
        engine,
        new_vars(
            "New Project",
            BreadCrumbKey::Projects,
            Some(user),
            Data { examples },
        ),
    ))
}

/// handle_new is handler for projects new page, /new/:name
pub async fn handle_new(
    Extension(user): Extension<land_tplvars::User>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    let example = examples::get(&name);
    if example.is_none() {
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "Template not found",
        ));
    }
    let example = example.unwrap();
    let source = example.get_source()?;
    if source.is_none() {
        return Err(ServerError::status_code(
            StatusCode::NOT_FOUND,
            "Template source not found",
        ));
    }
    let (project, playground) = land_dao::projects::create_with_playground(
        user.id,
        example.lang.parse()?,
        example.description,
        source.unwrap(),
    )
    .await?;
    let dp = land_dao::deploys::create(
        user.id,
        user.uuid,
        project.id,
        project.uuid,
        project.prod_domain,
        land_dao::deploys::DeployType::Production,
    )
    .await?;
    info!(
        owner_id = user.id,
        project_name = project.name,
        playground_id = playground.id,
        dp_id = dp.id,
        tpl_name = name,
        "Create new project",
    );
    Ok(redirect(format!("/projects/{}", project.name).as_str()))
}
