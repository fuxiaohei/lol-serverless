use super::ServerError;
use crate::{
    routers::{
        utils::{error_htmx, hx_redirect, notfound_page, ok_htmx, redirect},
        HtmlMinified,
    },
    templates::Engine,
};
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension, Form, Json};
use land_dao::{deploys, envs, projects, settings};
use land_service::examples;
use land_tplvars::{new_vars, BreadCrumbKey, Project};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

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

/// single is handler for projects single page, /projects/:name
pub async fn single(
    engine: Engine,
    Extension(user): Extension<land_tplvars::User>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub project_name: String,
        pub project: Project,
    }
    let project = projects::get_by_name(&name, Some(user.id)).await?;
    if project.is_none() {
        let msg = format!("Project {} not found", name);
        return Ok(notfound_page(engine, &msg, user).into_response());
    }
    let project = Project::new_with_source(&project.unwrap()).await?;
    Ok(HtmlMinified(
        "project.hbs",
        engine,
        new_vars(
            &name,
            BreadCrumbKey::ProjectSingle,
            Some(user),
            Vars {
                project_name: name.clone(),
                project,
            },
        ),
    )
    .into_response())
}

/// settings is handler for projects settings page, /projects/:name/settings
pub async fn settings(
    engine: Engine,
    Extension(user): Extension<land_tplvars::User>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Data {
        pub project_name: String,
        pub project: land_tplvars::Project,
        pub domain: String,
        pub env_keys: Vec<String>,
    }
    let project = projects::get_by_name(&name, Some(user.id)).await?;
    if project.is_none() {
        let msg = format!("Project {} not found", name);
        return Ok(notfound_page(engine, &msg, user).into_response());
    }
    let domain_settings = settings::get_domain_settings().await?;
    let project = land_tplvars::Project::new_with_source(&project.unwrap()).await?;
    let env = envs::get(project.id).await?;
    let env_keys = if let Some(env) = env {
        envs::get_keys(env).await?
    } else {
        vec![]
    };
    Ok(HtmlMinified(
        "project-settings.hbs",
        engine,
        new_vars(
            &name,
            BreadCrumbKey::ProjectSettings,
            Some(user),
            Data {
                project_name: name.clone(),
                project,
                domain: domain_settings.domain_suffix,
                env_keys,
            },
        ),
    )
    .into_response())
}

/// SettingsForm is the form for updating project settings
#[derive(Deserialize, Debug)]
pub struct SettingsForm {
    pub name: String,
    pub description: String,
}

/// handle_update_settings is handler for projects settings page, /projects/:name/settings
pub async fn handle_update_settings(
    Extension(user): Extension<land_tplvars::User>,
    Path(name): Path<String>,
    Form(f): Form<SettingsForm>,
) -> Result<impl IntoResponse, ServerError> {
    let project = projects::get_by_name(&name, Some(user.id)).await?;
    if project.is_none() {
        return Ok(error_htmx("Project not found").into_response());
    }
    if name != f.name && !projects::is_unique_name(&f.name).await? {
        warn!(
            owner_id = user.id,
            project_name = f.name,
            "Project name already exists",
        );
        return Ok(error_htmx("Project name already exists").into_response());
    }
    let project = project.unwrap();
    projects::update_names(project.id, &f.name, &f.description).await?;
    info!(
        owner_id = user.id,
        project_old_name = name,
        project_new_name = f.name,
        "Update project names",
    );
    let resp = hx_redirect(format!("/projects/{}/settings", f.name).as_str())?;
    Ok(resp.into_response())
}

/// handle_update_envs is route of user envs settings page, /projects/{name}/envs
pub async fn handle_update_envs(
    Extension(user): Extension<land_tplvars::User>,
    Path(name): Path<String>,
    Json(j): Json<land_dao::envs::EnvsQuery>,
) -> Result<impl IntoResponse, ServerError> {
    let project = projects::get_by_name(&name, Some(user.id)).await?;
    if project.is_none() {
        return Ok(error_htmx("Project not found").into_response());
    }
    let project = project.unwrap();
    let env = envs::get(project.id).await?;
    if let Some(env) = env {
        envs::update(env, j).await?;
        debug!(owner_id = user.id, project_name = name, "Update envs");
    } else {
        let _ = envs::create(user.id, project.id, j).await?;
        debug!(owner_id = user.id, project_name = name, "Create envs");
    }
    let dp = projects::create_deploy(&project, deploys::DeployType::Envs).await?;
    info!(project_name = name, dp_id = dp.id, "Update envs");
    Ok(ok_htmx("Envs updated").into_response())
}
