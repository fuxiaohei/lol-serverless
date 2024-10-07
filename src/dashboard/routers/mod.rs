use axum::{http::StatusCode, response::IntoResponse};
use super::templates::Engine;

/// index shows the dashboard page
pub async fn index(engine: Engine) -> Result<impl IntoResponse, ServerError> {
    Ok(HtmlMinified(
        "index.hbs",
        engine,
        &(),
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
