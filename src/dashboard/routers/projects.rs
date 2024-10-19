use super::ServerError;
use crate::dashboard::{
    examples,
    routers::{error_html, hx_redirect, notfound_page, ok_html, redirect, HtmlMinified},
    templates::Engine,
    tplvars::{self, AuthUser, BreadCrumbKey, Empty, Page, Vars},
};
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension, Form, Json};
use land_dao::{deploys, projects, settings};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

/// index shows the project dashboard page
pub async fn index(
    engine: Engine,
    Extension(user): Extension<AuthUser>,
) -> Result<impl IntoResponse, ServerError> {
    Ok(HtmlMinified(
        "projects.hbs",
        engine,
        Vars::new(
            Page::new("Projects", BreadCrumbKey::Projects, Some(user)),
            Empty::default(),
        ),
    ))
}

/// new is handler for projects new page, /new
pub async fn new(
    Extension(user): Extension<AuthUser>,
    engine: Engine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub examples: Vec<examples::Item>,
    }
    let examples = examples::defaults();
    Ok(HtmlMinified(
        "project-new.hbs",
        engine,
        Vars {
            page: Page::new("New Project", BreadCrumbKey::ProjectNew, Some(user)),
            examples,
        },
    ))
}

/// handle_new is handler for projects new page, /new/:name
pub async fn handle_new(
    Extension(user): Extension<AuthUser>,
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
    let (project, playground) = projects::create_with_playground(
        user.id,
        example.lang.parse()?,
        example.description,
        source.unwrap(),
    )
    .await?;
    let dp = deploys::create(
        user.id,
        user.uuid,
        project.id,
        project.uuid,
        project.prod_domain,
        deploys::DeployType::Production,
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
    Extension(user): Extension<AuthUser>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub project_name: String,
        pub project: tplvars::Project,
    }
    let project = projects::get_by_name(&name, Some(user.id)).await?;
    if project.is_none() {
        let msg = format!("Project {} not found", name);
        return Ok(notfound_page(engine, &msg, user).into_response());
    }
    let project = tplvars::Project::new_with_source(&project.unwrap()).await?;
    Ok(HtmlMinified(
        "project-single.hbs",
        engine,
        Vars {
            page: Page::new(&name, BreadCrumbKey::ProjectSingle, Some(user)),
            project_name: name,
            project,
        },
    )
    .into_response())
}

/// settings is handler for projects settings page, /projects/:name/settings
pub async fn settings(
    engine: Engine,
    Extension(user): Extension<AuthUser>,
    Path(name): Path<String>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub project_name: String,
        pub project: tplvars::Project,
        pub domain: String,
        pub env_keys: Vec<String>,
    }
    let project = projects::get_by_name(&name, Some(user.id)).await?;
    if project.is_none() {
        let msg = format!("Project {} not found", name);
        return Ok(notfound_page(engine, &msg, user).into_response());
    }
    let domain_settings = settings::get_domain_settings().await?;
    let project = tplvars::Project::new_with_source(&project.unwrap()).await?;
    let env = land_dao::envs::get(project.id).await?;
    let env_keys = if let Some(env) = env {
        land_dao::envs::get_keys(env).await?
    } else {
        vec![]
    };
    Ok(HtmlMinified(
        "project-settings.hbs",
        engine,
        Vars {
            page: Page::new(&name, BreadCrumbKey::ProjectSettings, Some(user)),
            project_name: name,
            project,
            domain: domain_settings.domain_suffix,
            env_keys,
        },
    )
    .into_response())
}

/// SettingsForm is the form for updating project settings
#[derive(Deserialize, Debug)]
pub struct SettingsForm {
    pub name: String,
    pub description: String,
}

/// handle_settings is handler for projects settings page, /projects/:name/settings
pub async fn handle_settings(
    Extension(user): Extension<AuthUser>,
    Path(name): Path<String>,
    Form(f): Form<SettingsForm>,
) -> Result<impl IntoResponse, ServerError> {
    let project = projects::get_by_name(&name, Some(user.id)).await?;
    if project.is_none() {
        return Ok(error_html("Project not found").into_response());
    }
    if name != f.name && !projects::is_unique_name(&f.name).await? {
        warn!(
            owner_id = user.id,
            project_name = f.name,
            "Project name already exists",
        );
        return Ok(error_html("Project name already exists").into_response());
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

/// handle_envs is route of user envs settings page, /projects/{name}/envs
pub async fn handle_envs(
    Extension(user): Extension<AuthUser>,
    Path(name): Path<String>,
    Json(j): Json<land_dao::envs::EnvsQuery>,
) -> Result<impl IntoResponse, ServerError> {
    let project = projects::get_by_name(&name, Some(user.id)).await?;
    if project.is_none() {
        return Ok(error_html("Project not found").into_response());
    }
    let project = project.unwrap();
    let env = land_dao::envs::get(project.id).await?;
    if let Some(env) = env {
        land_dao::envs::update(env, j).await?;
        debug!(owner_id = user.id, project_name = name, "Update envs");
    } else {
        let _ = land_dao::envs::create(user.id, project.id, j).await?;
        debug!(owner_id = user.id, project_name = name, "Create envs");
    }
    let dp = projects::create_deploy(&project, deploys::DeployType::Envs).await?;
    info!(project_name = name, dp_id = dp.id, "Update envs");
    Ok(ok_html("Envs updated").into_response())
}
