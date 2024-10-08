use super::ServerError;
use crate::dashboard::{
    examples,
    routers::{redirect, HtmlMinified},
    templates::Engine,
    tplvars::{AuthUser, BreadCrumbKey, Empty, Page, Vars},
};
use axum::{extract::Path, http::StatusCode, response::IntoResponse, Extension};
use land_dao::{deploys, projects};
use serde::Serialize;
use tracing::info;

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
