use crate::{
    middle::{WorkerInfo, WorkerMetrics},
    ServerError, ENABLE_WASMTIME_AOT, ENDPOINT_NAME,
};
use anyhow::Result;
use axum::{
    body::{Body, HttpBody},
    extract::ConnectInfo,
    http::Request,
    response::{IntoResponse, Response},
    Extension,
};
use land_core::memenvs;
use land_wasm_host::{hostcall, pool::prepare_worker, Context, Worker};
use std::net::SocketAddr;
use tokio::time::Instant;
use tracing::{debug, info, info_span, warn, Instrument};

pub async fn run(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    Extension(info): Extension<WorkerInfo>,
    Extension(metrics): Extension<WorkerMetrics>,
    req: Request<Body>,
) -> Result<impl IntoResponse, ServerError> {
    let st = Instant::now();
    metrics.req_fn_total.increment(1);

    // get remote ip
    // if cf-connecting-ip,x-real-ip exists, use it
    let remote = if let Some(cf_ip) = req.headers().get("cf-connecting-ip") {
        cf_ip.to_str().unwrap().to_string()
    } else if let Some(real_ip) = req.headers().get("x-real-ip") {
        real_ip.to_str().unwrap().to_string()
    } else {
        addr.to_string()
    };

    // prepare span info
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
        let new_uri = format!("http://{}{}", host, uri.path());
        uri = new_uri.parse().unwrap();
    }
    let method = req.method().clone();
    let env_key = format!("{}-{}", info.user_id, info.project_id);
    let envs = memenvs::get(&env_key).await;
    let mut context = Context::new(envs);
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
    let worker = prepare_worker(wasm_path, *aot_enable)
        .instrument(info_span!("[WASM]", wasm_path = %wasm_path))
        .await?;
    debug!("Wasm worker pool ok: {}", wasm_path);
    Ok(worker)
}
