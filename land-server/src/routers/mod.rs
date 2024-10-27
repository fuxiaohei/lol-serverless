use crate::templates::{self, Engine};
use anyhow::Result;
use axum::{
    body::Body,
    response::{Html, IntoResponse},
    routing::get,
    Extension, Router,
};
use axum_htmx::HxRedirect;
use http::{StatusCode, Uri};
use land_tplvars::{BreadCrumbKey, Empty};
use std::str::FromStr;
use tower_http::services::ServeDir;

mod auth;
mod install;

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
        .nest_service("/static", ServeDir::new(static_assets_dir))
        .fallback(handle_notfound)
        .route_layer(axum::middleware::from_fn(auth::middle))
        .route_layer(axum::middleware::from_fn(install::middle))
        .with_state(Engine::from(hbs));
    Ok(app)
}

/// index shows the dashboard page
pub async fn index(
    engine: Engine,
    Extension(user): Extension<land_tplvars::User>,
) -> Result<impl IntoResponse, ServerError> {
    Ok(HtmlMinified(
        "index.hbs",
        engine,
        Empty::new_vars("Dashboard", BreadCrumbKey::Dashboard, Some(user)),
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
