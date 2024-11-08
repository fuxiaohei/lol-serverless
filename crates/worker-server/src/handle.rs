use crate::{
    middle::{WorkerInfo, WorkerMetrics},
    ENDPOINT_NAME,
};
use axum::{
    body::Body,
    extract::{ConnectInfo, Request},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    Extension,
};
use std::net::SocketAddr;
use tokio::time::Instant;
use tracing::{info_span, warn};

/// run is the main handler for all requests to run wasm components.
pub async fn run(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(info): Extension<WorkerInfo>,
    Extension(metrics): Extension<WorkerMetrics>,
    req: Request<Body>,
) -> Result<impl IntoResponse, ServerError> {
    let st = Instant::now();
    metrics.req_fn_total.increment(1);

    let headers = req.headers();

    // prepare span info
    let remote = get_remote_addr(headers, addr);
    let method = req.method().clone();
    let uri = req.uri().to_string();
    let span = info_span!("[HTTP]",rt = %remote, rid = %info.req_id.clone(), m = %method, u = %uri, h = %info.host);

    let _enter = span.enter();
    warn!(
        status = 404,
        elapsed = %st.elapsed().as_micros(),
        "Function not found",
    );
    metrics.req_fn_notfound_total.increment(1);
    Ok(ServerError::not_found(info, "Function not found").into_response())
}

fn get_remote_addr(headers: &HeaderMap, addr: SocketAddr) -> String {
    // get remote ip
    // if cf-connecting-ip,x-real-ip exists, use it
    let remote = if let Some(cf_ip) = headers.get("cf-connecting-ip") {
        cf_ip.to_str().unwrap().to_string()
    } else if let Some(real_ip) = headers.get("x-real-ip") {
        real_ip.to_str().unwrap().to_string()
    } else {
        addr.to_string()
    };
    remote
}

/// ServerError is a custom error type that we will use to represent errors in our application.
pub struct ServerError(super::middle::WorkerInfo, StatusCode, anyhow::Error);

impl Clone for ServerError {
    fn clone(&self) -> Self {
        Self(self.0.clone(), self.1, anyhow::anyhow!(self.2.to_string()))
    }
}

impl ServerError {
    pub fn not_found(ctx: super::middle::WorkerInfo, msg: &str) -> Self {
        Self(ctx, StatusCode::NOT_FOUND, anyhow::anyhow!(msg.to_string()))
    }
    /*
    pub fn bad_request(ctx: super::middle::WorkerInfo, msg: &str) -> Self {
        Self(
            ctx,
            StatusCode::BAD_REQUEST,
            anyhow::anyhow!(msg.to_string()),
        )
    }
    pub fn unauthorized(ctx: super::middle::WorkerInfo, msg: &str) -> Self {
        Self(
            ctx,
            StatusCode::UNAUTHORIZED,
            anyhow::anyhow!(msg.to_string()),
        )
    }*/
    pub fn _internal_error(ctx: super::middle::WorkerInfo, msg: &str) -> Self {
        Self(
            ctx,
            StatusCode::INTERNAL_SERVER_ERROR,
            anyhow::anyhow!(msg.to_string()),
        )
    }
}

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for ServerError {
    fn into_response(self) -> axum::response::Response {
        let mut resp = (self.1, self.2.to_string()).into_response();
        resp.headers_mut()
            .insert("x-request-id", self.0.req_id.parse().unwrap());
        resp.headers_mut()
            .insert("x-server-by", self.0.endpoint.parse().unwrap());
        let exts = resp.extensions_mut();
        exts.insert(self);
        resp
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for ServerError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(
            super::middle::WorkerInfo {
                endpoint: ENDPOINT_NAME.get().unwrap().to_string(),
                ..Default::default()
            },
            StatusCode::INTERNAL_SERVER_ERROR,
            err.into(),
        )
    }
}
