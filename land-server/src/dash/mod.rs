use crate::templates::{new_handlebar, Engine, RenderHtmlMinified};
use anyhow::{anyhow, Result};
use axum::{
    body::Body,
    http::StatusCode,
    middleware,
    response::{Html, IntoResponse, Response},
    routing::{get, post},
    Extension, Router,
};
use land_vars::{AuthUser, BreadCrumbKey, Page, Project};
use serde::Serialize;
use tower_http::services::ServeDir;

mod auth;
pub mod middle;
mod projects;
mod settings;
mod traffic;

/// redirect returns a redirect response
pub fn redirect(url: &str) -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", url)
        .body(Body::empty())
        .unwrap()
}

/// error_html returns a html response with error message
pub fn error_html(msg: &str) -> impl IntoResponse {
    Html(format!("<div class=\"err-message\">{}</div>", msg))
}

/// ok_html returns a html response with ok message
pub fn ok_html(msg: &str) -> impl IntoResponse {
    Html(format!("<div class=\"ok-message\">{}</div>", msg))
}

/// notfound_html returns a html response with not found page
fn notfound_html(engine: Engine, msg: &str, user: AuthUser) -> impl IntoResponse {
    #[derive(Debug, serde::Serialize)]
    struct Vars {
        pub page: Page,
        pub msg: String,
    }
    (
        StatusCode::NOT_FOUND,
        RenderHtmlMinified(
            "not-found.hbs",
            engine,
            Vars {
                page: Page::new("Page Not Found", BreadCrumbKey::NotFound, Some(user)),
                msg: msg.to_string(),
            },
        ),
    )
        .into_response()
}

async fn handler(
    Extension(user): Extension<AuthUser>,
    engine: Engine,
) -> Result<impl IntoResponse, ServerError> {
    #[derive(Serialize)]
    struct Vars {
        pub page: Page,
        pub projects: Vec<Project>,
    }
    let (projects_data, _) = land_dao::projects::list(Some(user.id), None, 1, 5).await?;
    Ok(RenderHtmlMinified(
        "index.hbs",
        engine,
        Vars {
            page: Page::new("Dashboard", BreadCrumbKey::Home, Some(user)),
            projects: Project::new_from_models(projects_data, false).await?,
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
        .route("/sign-in", get(auth::sign_in))
        .route("/sign-callback", get(auth::callback))
        .route("/sign-out", get(auth::sign_out))
        .route("/projects", get(projects::index))
        .route("/projects/:name", get(projects::single))
        .route(
            "/projects/:name/edit",
            get(projects::edit).post(projects::handle_edit),
        )
        .route("/projects/:name/status", post(projects::handle_status))
        .route("/projects/:name/traffic", get(projects::traffic))
        .route(
            "/projects/:name/settings",
            get(projects::settings).post(projects::handle_settings),
        )
        .route("/projects/:name/envs", post(projects::handle_envs))
        .route("/new", get(projects::new))
        .route("/new/:name", get(projects::handle_new))
        .route("/settings", get(settings::index))
        .route("/settings/tokens/create", post(settings::create_token))
        .route("/settings/tokens/remove", post(settings::remove_token))
        .route("/traffic/requests", post(traffic::requests))
        .route("/traffic/flows", post(traffic::flows))
        .route("/traffic/projects", post(traffic::projects))
        .nest_service("/static", ServeDir::new(static_assets_dir))
        .route_layer(middleware::from_fn(middle::auth))
        .route_layer(middleware::from_fn(middle::logger))
        .with_state(Engine::from(hbs));
    Ok(app)
}

// Make our own error that wraps `anyhow::Error`.
pub struct ServerError(pub StatusCode, pub anyhow::Error);

impl Clone for ServerError {
    fn clone(&self) -> Self {
        Self(self.0, anyhow::anyhow!(self.1.to_string()))
    }
}

impl ServerError {
    /// status_code creates a new `ServerError` with the given status code and message.
    pub fn status_code(code: StatusCode, msg: &str) -> Self {
        Self(code, anyhow!(msg.to_string()))
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
