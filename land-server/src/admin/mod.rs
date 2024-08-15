use crate::{
    dash::ServerError,
    templates::{new_handlebar, Engine, RenderHtmlMinified},
};
use anyhow::Result;
use axum::{
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Extension, Router,
};
use land_vars::{AuthUser, BreadCrumbKey, Page, Project};
use serde::Serialize;
use tower_http::services::ServeDir;

mod deploylogs;
mod middle;
mod projects;
mod settings;
mod users;
mod workers;

async fn handler(
    Extension(user): Extension<AuthUser>,
    engine: Engine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub nav_admin: bool,
        pub projects: Vec<Project>,
        pub users: Vec<AuthUser>,
    }
    let (projects_data, _) = land_dao::projects::list(None, None, 1, 5).await?;
    let projects = Project::new_from_models(projects_data, true).await?;
    let (user_models, _) = land_dao::users::list(None, 1, 5).await?;
    let users: Vec<_> = user_models.iter().map(AuthUser::new).collect();
    Ok(RenderHtmlMinified(
        "admin/index.hbs",
        engine,
        Vars {
            nav_admin: true,
            page: Page::new("Admin Dashboard", BreadCrumbKey::Admin, Some(user)),
            projects,
            users,
        },
    ))
}

pub async fn route(assets_dir: &str, tpl_dir: Option<String>) -> Result<Router> {
    // Extract templates
    let hbs = new_handlebar(assets_dir, tpl_dir.clone())?;
    // set static assets directory
    let static_assets_dir = format!("{}/static", tpl_dir.unwrap_or(assets_dir.to_string()));

    let app = Router::new()
        .route("/", get(handler))
        .route("/projects", get(projects::index))
        .route("/projects/traffic", post(projects::traffic))
        .route("/projects/source", get(projects::source))
        .route("/projects/:name", get(projects::details))
        .route("/projects/:name/redeploy", post(projects::redeploy))
        .route("/projects/:name/enable", post(projects::enable))
        .route("/projects/:name/disable", post(projects::disable))
        .route("/settings", get(settings::index))
        .route("/settings/domains", post(settings::update_domains))
        .route("/settings/prometheus", post(settings::update_prometheus))
        .route("/settings/storage", post(settings::update_storage))
        .route("/workers", get(workers::index))
        .route("/workers/tokens/create", post(workers::create_token))
        .route("/workers/tokens/remove", post(workers::remove_token))
        .route("/users", get(users::index))
        .route("/deploy-logs", get(deploylogs::index))
        .nest_service("/static", ServeDir::new(static_assets_dir))
        .route_layer(middleware::from_fn(middle::check_admin))
        .route_layer(middleware::from_fn(crate::dash::middle::logger))
        .route_layer(middleware::from_fn(crate::dash::middle::auth))
        .with_state(Engine::from(hbs));
    Ok(app)
}
