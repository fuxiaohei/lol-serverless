use crate::templates::{self, Engine};
use anyhow::Result;
use axum::{
    body::Body,
    extract::{ConnectInfo, OriginalUri, Request},
    middleware::Next,
    response::{Html, IntoResponse},
    routing::{get, post},
    Extension, Router,
};
use axum_htmx::HxRedirect;
use http::{HeaderValue, StatusCode, Uri};
use land_tplvars::BreadCrumbKey;
use serde::Serialize;
use std::{net::SocketAddr, str::FromStr};
use tower_http::services::ServeDir;
use tracing::{debug, info, instrument, warn};

mod admin;
mod auth;
mod install;
mod projects;
mod settings;

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
        .route("/install", get(install::page).post(install::handle))
        .route("/sign-in", get(auth::sign_in).post(auth::handle_sign_in))
        .route("/sign-out", get(auth::sign_out))
        .route("/settings", get(settings::index))
        .route("/settings/tokens", post(settings::handle_token))
        .route("/projects", get(projects::index))
        .route("/new", get(projects::new))
        .route("/new/:name", get(projects::handle_new))
        .route("/admin", get(admin::index))
        .route("/admin/general", get(admin::general))
        .route("/admin/domains", post(admin::update_domains))
        .route("/admin/storage", post(admin::update_storage))
        .nest_service("/static", ServeDir::new(static_assets_dir))
        .fallback(handle_notfound)
        .route_layer(axum::middleware::from_fn(auth::middle))
        .route_layer(axum::middleware::from_fn(install::middle))
        .route_layer(axum::middleware::from_fn(logger))
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
        pub projects: Vec<land_tplvars::Project>,
    }
    let (projects, _) = land_dao::projects::list(Some(user.id), None, 1, 5).await?;
    debug!("projects: {:?}", projects.len());
    Ok(HtmlMinified(
        "index.hbs",
        engine,
        land_tplvars::Vars::new(
            land_tplvars::Page::new("Dashboard", BreadCrumbKey::Dashboard, Some(user)),
            Data {
                projects: land_tplvars::Project::new_from_models(projects, false).await?,
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

/// logger is a middleware that logs the request and response.
#[instrument("[HTTP]", skip_all)]
pub async fn logger(request: Request, next: Next) -> Result<axum::response::Response, StatusCode> {
    let path = if let Some(path) = request.extensions().get::<OriginalUri>() {
        // This will include nested routes
        path.0.path().to_owned()
    } else {
        request.uri().path().to_owned()
    };
    if path.starts_with("/static") || path.starts_with("/_worker_api") {
        // ignore static assets log and worker api
        return Ok(next.run(request).await);
    }
    let mut remote = "0.0.0.0".to_string();
    // if x-real-ip exists, use it
    if let Some(real_ip) = request.headers().get("x-real-ip") {
        remote = real_ip.to_str().unwrap().to_string();
    } else if let Some(connect_info) = request.extensions().get::<ConnectInfo<SocketAddr>>() {
        remote = connect_info.to_string();
    }

    /*
    if path.starts_with("/api/v1/worker-api/alive") {
        // high sequence url
        return Ok(next.run(request).await);
    }*/

    let method = request.method().clone().to_string();
    let st = tokio::time::Instant::now();
    let resp = next.run(request).await;
    let server_err = resp.extensions().get::<ServerError>();
    let status = resp.status().as_u16();
    let elasped = st.elapsed().as_millis();
    if let Some(err) = server_err {
        warn!(
            remote = remote,
            method = method,
            path = path,
            status = status,
            elasped = elasped,
            "Failed: {}",
            err.1
        );
    } else {
        let empty_header = HeaderValue::from_str("").unwrap();
        let content_type = resp
            .headers()
            .get("content-type")
            .unwrap_or(&empty_header)
            .to_str()
            .unwrap();
        let content_size = resp
            .headers()
            .get("content-length")
            .unwrap_or(&empty_header)
            .to_str()
            .unwrap();
        if status >= 400 {
            warn!(
                rmt = remote,
                m = method,
                p = path,
                s = status,
                cost = elasped,
                typ = content_type,
                size = content_size,
                "Ok",
            );
        } else {
            info!(
                rmt = remote,
                m = method,
                p = path,
                s = status,
                cost = elasped,
                typ = content_type,
                size = content_size,
                "Ok",
            );
        }
    }
    Ok(resp)
}

/// hx_redirect returns a htmx redirect response
pub fn hx_redirect(url: &str) -> Result<impl IntoResponse, ServerError> {
    let uri = Uri::from_str(url)?;
    let parts = HxRedirect(uri);
    Ok((parts, ()).into_response())
}

/// redirect returns a 302 redirect response
pub fn redirect(url: &str) -> impl IntoResponse {
    http::Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", url)
        .body(Body::empty())
        .unwrap()
}

/// error_htmx returns a htmx response with error message
pub fn error_htmx(msg: &str) -> impl IntoResponse {
    Html(format!("<div class=\"htmx-err-message\">{}</div>", msg))
}

/// ok_htmx returns a htmx response with ok message
pub fn ok_htmx(msg: &str) -> impl IntoResponse {
    Html(format!("<div class=\"htmx-ok-message\">{}</div>", msg))
}
