use axum::{
    body::Body,
    extract::{ConnectInfo, OriginalUri, Request},
    http::{HeaderValue, Response, StatusCode, Uri},
    middleware::Next,
    response::{Html, IntoResponse},
};
use axum_htmx::HxRedirect;
use std::{net::SocketAddr, str::FromStr};
use tracing::{info, instrument, warn};

pub mod auth;
pub mod index;
pub mod install;
pub mod projects;

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

/// hx_redirect returns a htmlx redirect response
pub fn hx_redirect(url: &str) -> Result<impl IntoResponse, ServerError> {
    let uri = Uri::from_str(url)?;
    let parts = HxRedirect(uri);
    Ok((parts, ()).into_response())
}

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
    Html(format!("<div class=\"htmx-err-message\">{}</div>", msg))
}

#[instrument("[HTTP]", skip_all)]
pub async fn logger(request: Request, next: Next) -> Result<axum::response::Response, StatusCode> {
    let path = if let Some(path) = request.extensions().get::<OriginalUri>() {
        // This will include nested routes
        path.0.path().to_owned()
    } else {
        request.uri().path().to_owned()
    };
    if path.starts_with("/static") {
        // ignore static assets log
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
