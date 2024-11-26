use super::ServerError;
use crate::{routers::HtmlMinified, templates::Engine};
use axum::{
    body::{Body, HttpBody},
    extract::{ConnectInfo, OriginalUri, Request},
    http::{HeaderValue, Response, StatusCode, Uri},
    middleware::Next,
    response::{Html, IntoResponse},
};
use axum_htmx::HxRedirect;
use land_tplvars::{new_vars, BreadCrumbKey};
use std::{net::SocketAddr, str::FromStr};
use tracing::{info, instrument, warn};

/// redirect returns a 302 redirect response
pub fn redirect(url: &str) -> impl IntoResponse {
    Response::builder()
        .status(StatusCode::FOUND)
        .header("Location", url)
        .body(Body::empty())
        .unwrap()
}

/// hx_redirect returns a htmx redirect response
pub fn hx_redirect(url: &str) -> Result<impl IntoResponse, ServerError> {
    let uri = Uri::from_str(url)?;
    let parts = HxRedirect(uri);
    Ok((parts, ()).into_response())
}

/// notfound_page returns a html response with not found page
pub fn notfound_page(engine: Engine, msg: &str, user: land_tplvars::User) -> impl IntoResponse {
    #[derive(Debug, serde::Serialize)]
    struct Data {
        pub msg: String,
    }
    (
        StatusCode::NOT_FOUND,
        HtmlMinified(
            "not-found.hbs",
            engine,
            new_vars(
                "Page Not Found",
                BreadCrumbKey::NotFound,
                Some(user),
                Data {
                    msg: msg.to_string(),
                },
            ),
        ),
    )
        .into_response()
}

/// error_htmx returns a htmx response with error message
pub fn error_htmx(msg: &str) -> impl IntoResponse {
    Html(format!("<div class=\"htmx-err-message\">{}</div>", msg))
}

/// ok_htmx returns a htmx response with ok message
pub fn ok_htmx(msg: &str) -> impl IntoResponse {
    Html(format!("<div class=\"htmx-ok-message\">{}</div>", msg))
}

/// logger is a middleware that logs the request and response.
#[instrument("[http]", skip_all)]
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
        let content_size = resp.body().size_hint().exact().unwrap_or_default();
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
