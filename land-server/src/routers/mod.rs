use crate::templates::{self, Engine};
use anyhow::Result;
use axum::{
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::{get, post},
    Extension, Router,
};
use land_dao::projects;
use land_tplvars::{new_vars, BreadCrumbKey, Project};
use serde::Serialize;
use tower_http::services::ServeDir;
use tracing::debug;

mod admin;
mod auth;
mod install;
mod project;
mod setting;
mod utils;
mod worker_api;

/// handle_notfound returns a not found response.
async fn handle_notfound() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "Page not found")
}

/// new creates a new router
pub async fn new(assets_dir: &str, tpl_dir: Option<String>) -> Result<Router> {
    // prepare templates
    let hbs = templates::new_handlebar(assets_dir, tpl_dir.clone())?;
    // set static assets directory
    let static_assets_dir = format!("{}/static", tpl_dir.unwrap_or(assets_dir.to_string()));

    let app = Router::new()
        .route("/", get(index))
        .route("/install", get(install::index).post(install::handle))
        .route("/sign-in", get(auth::sign_in).post(auth::handle_sign_in))
        .route("/sign-out", get(auth::sign_out))
        .route("/new", get(project::new))
        .route("/new/:name", get(project::handle_new))
        .route("/projects", get(project::index))
        .route("/projects/:name", get(project::single))
        .route(
            "/projects/:name/settings",
            get(project::settings).post(project::handle_update_settings),
        )
        .route("/projects/:name/envs", post(project::handle_update_envs))
        .route(
            "/projects/:name/edit",
            get(project::edit).post(project::handle_edit),
        )
        .route(
            "/projects/:name/check-status",
            post(project::handle_check_status),
        )
        .route("/projects/:name/deployments", get(project::deployments))
        .route("/settings", get(setting::index))
        .route("/settings/tokens", post(setting::handle_token))
        .route("/admin", get(admin::index))
        .route("/admin/general", get(admin::general))
        .route("/admin/domains", post(admin::handle_update_domains))
        .route("/admin/storage", post(admin::handle_update_storage))
        .route("/admin/workers", get(admin::workers))
        .route("/admin/workers/tokens", post(admin::handle_workers_token))
        .route("/admin/projects", get(admin::projects))
        .route("/admin/deploys", get(admin::deploys))
        .route("/_worker_api/heartbeat", post(worker_api::heartbeat))
        .route("/_worker_api/tasks", post(worker_api::task::tasks))
        .route("/_worker_api/download/*path", get(worker_api::download))
        .nest_service("/static", ServeDir::new(static_assets_dir))
        .fallback(handle_notfound)
        .route_layer(middleware::from_fn(auth::middle))
        .route_layer(middleware::from_fn(install::middle))
        .route_layer(middleware::from_fn(utils::logger))
        .with_state(Engine::from(hbs));
    Ok(app)
}

/// index shows the dashboard page
pub async fn index(
    engine: Engine,
    Extension(user): Extension<land_tplvars::User>,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Data {
        pub projects: Vec<Project>,
    }
    let (projects, _) = projects::list(Some(user.id), None, 1, 5).await?;
    debug!("projects: {:?}", projects.len());
    Ok(HtmlMinified(
        "index.hbs",
        engine,
        new_vars(
            "Dashboard",
            BreadCrumbKey::Dashboard,
            Some(user),
            Data {
                projects: Project::new_from_models(projects, false).await?,
            },
        ),
    ))
}

// ServerError makes our own error that wraps `anyhow::Error`.
pub struct ServerError(pub StatusCode, pub anyhow::Error);

impl ServerError {
    /// status_code creates a new `ServerError` with the given status code and message.
    pub fn status_code(code: StatusCode, msg: &str) -> Self {
        Self(code, anyhow::anyhow!(msg.to_string()))
    }
}

impl Clone for ServerError {
    fn clone(&self) -> Self {
        Self(self.0, anyhow::anyhow!(self.1.to_string()))
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, RespError>`. That way you don't need to do that manually.
impl<E> From<E> for ServerError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, err.into())
    }
}

// Tell axum how to convert `RespError` into a response.
impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        let mut resp = (self.0, self.1.to_string()).into_response();
        let exts = resp.extensions_mut();
        exts.insert(self);
        resp
    }
}

/// HtmlMinified is a wrapper for axum::response::Html that minifies the html.
pub struct HtmlMinified<K, E, S>(pub K, pub E, pub S);

impl<K, E, S> axum::response::IntoResponse for HtmlMinified<K, E, S>
where
    E: axum_template::TemplateEngine,
    S: serde::Serialize,
    K: AsRef<str>,
{
    fn into_response(self) -> axum::response::Response {
        let HtmlMinified(key, engine, data) = self;

        let result = engine.render(key.as_ref(), data);
        match result {
            Ok(x) => {
                let mut cfg = minify_html::Cfg::spec_compliant();
                cfg.minify_js = true;
                cfg.minify_css = true;
                let minified = minify_html::minify(x.as_bytes(), &cfg);
                axum::response::Html(minified).into_response()
            }
            Err(x) => x.into_response(),
        }
    }
}
