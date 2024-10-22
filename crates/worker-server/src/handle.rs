use super::{
    middle::{WorkerInfo, WorkerMetrics},
    ENDPOINT_NAME,
};
use crate::ENABLE_WASMTIME_AOT;
use anyhow::Result;
use axum::{
    body::{Body, HttpBody},
    extract::{ConnectInfo, Request},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Extension,
};
use land_kernel::memenvs;
use land_wasm_host::{hostcall, Ctx, Worker};
use std::net::SocketAddr;
use tokio::time::Instant;
use tracing::{debug, info, info_span, warn, Instrument};

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
    let span_clone = span.clone();

    // if wasm_module is empty, return 404
    if info.wasm_module.is_empty() {
        let _enter = span.enter();
        warn!(
            status = 404,
            elapsed = %st.elapsed().as_micros(),
            "Function not found",
        );
        metrics.req_fn_notfound_total.increment(1);
        return Err(ServerError::not_found(info, "Function not found"));
    }

    // collect post body size
    let body_size = req.body().size_hint().exact().unwrap_or(0);
    metrics.req_fn_in_bytes_total.increment(body_size);

    // call wasm async
    async move {
        let result = wasm(req, &info).await;
        if let Err(err) = result {
            let elapsed = st.elapsed().as_micros();
            warn!(
                status = 500,
                elapsed = %elapsed,
                "Internal error: {}",
                err,
            );
            metrics.req_fn_error_total.increment(1);
            let msg = format!("Internal error: {}", err);
            return Err(ServerError::internal_error(info, &msg));
        }
        let resp = result.unwrap();
        let status_code = resp.status().as_u16();
        let elapsed = st.elapsed().as_micros();
        if status_code >= 400 {
            warn!( status=%status_code,elapsed=%elapsed, "Done");
        } else {
            info!( status=%status_code,elapsed=%elapsed, "Done");
        }
        let body_size = resp.body().size_hint().exact().unwrap_or(0);
        metrics.req_fn_out_bytes_total.increment(body_size);
        metrics.req_fn_success_total.increment(1);
        Ok(resp)
    }
    .instrument(span_clone)
    .await
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

async fn wasm(req: Request<Body>, info: &WorkerInfo) -> Result<Response<Body>> {
    let req_id = info.req_id.clone();
    let worker = init_worker(&info.wasm_module).await?;

    // convert request to host-call request
    let mut headers: Vec<(String, String)> = vec![];
    let req_headers = req.headers().clone();
    req_headers.iter().for_each(|(k, v)| {
        // if key start with x-land, ignore
        let key = k.to_string();
        if key.starts_with("x-land") {
            return;
        }
        headers.push((key, v.to_str().unwrap().to_string()));
    });

    let mut uri = req.uri().clone();
    // if no host, use host value to generate new one, must be full uri
    if uri.authority().is_none() {
        let host = req
            .headers()
            .get("host")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown");
        let mut new_uri = format!("http://{}{}", host, uri.path());
        if let Some(query) = uri.query() {
            new_uri.push('?');
            new_uri.push_str(query);
        }
        uri = new_uri.parse().unwrap();
    }
    let method = req.method().clone();
    let env_key = format!("{}-{}", info.user_id, info.project_id);
    let envs = memenvs::get(&env_key).await;
    let mut context = Ctx::new(envs, req_id.clone());
    // if method is GET or DELETE, set body to None
    let body_handle = if method == "GET" || method == "DELETE" {
        0
    } else {
        let body = req.into_body();
        context.set_body(0, body)
    };
    debug!("Set body_handle: {:?}", body_handle);

    let wasm_req = hostcall::Request {
        method: method.to_string(),
        uri: uri.to_string(),
        headers,
        body: Some(body_handle),
    };

    let (wasm_resp, wasm_resp_body) = match worker.handle_request(wasm_req, context).await {
        Ok((wasm_resp, wasm_resp_body)) => (wasm_resp, wasm_resp_body),
        Err(e) => {
            let builder = Response::builder().status(500);
            return Ok(builder.body(Body::from(e.to_string())).unwrap());
        }
    };

    // convert host-call response to response
    let mut builder = Response::builder().status(wasm_resp.status);
    for (k, v) in wasm_resp.headers.clone() {
        builder = builder.header(k, v);
    }
    if builder.headers_ref().unwrap().get("x-request-id").is_none() {
        builder = builder.header("x-request-id", req_id.clone());
    }
    builder = builder.header("x-served-by", ENDPOINT_NAME.get().unwrap());
    Ok(builder.body(wasm_resp_body).unwrap())
}

/// init_worker is a helper function to prepare wasm worker
async fn init_worker(wasm_path: &str) -> Result<Worker> {
    let aot_enable = ENABLE_WASMTIME_AOT.get().unwrap();
    let worker = Worker::new_in_pool(wasm_path, *aot_enable)
        .instrument(info_span!("[WASM]", wasm_path = %wasm_path))
        .await?;
    debug!("Wasm worker pool ok: {}", wasm_path);
    Ok(worker)
}

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
    pub fn internal_error(ctx: super::middle::WorkerInfo, msg: &str) -> Self {
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
