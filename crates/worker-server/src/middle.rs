use super::{DEFAULT_WASM, ENABLE_METRICS, ENDPOINT_NAME};
use axum::{
    extract::Request,
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use metrics::{counter, Counter};
use serde::Serialize;

/// WorkerInfo is current wasm worker info from http header
#[derive(Default, Clone, Serialize, Debug)]
pub struct WorkerInfo {
    pub req_id: String,
    pub wasm_module: String,
    pub user_id: String,
    pub project_id: String,
    pub deploy_id: String,
    pub host: String,
    pub endpoint: String,
}

impl WorkerInfo {
    pub fn new(headers: &HeaderMap, req_id: String) -> Self {
        // get module path
        let default_wasm_path = DEFAULT_WASM.get().unwrap();
        let wasm_module = headers
            .get("x-land-m")
            .and_then(|v| v.to_str().ok())
            .unwrap_or(default_wasm_path.as_str())
            .to_string();

        // get user-id, project-id, deploy-id, host
        let user_id = headers
            .get("x-land-uid")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("0")
            .to_string();
        let project_id = headers
            .get("x-land-pid")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("0")
            .to_string();
        let deploy_id = headers
            .get("x-land-did")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("0")
            .to_string();
        let host = headers
            .get("host")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("unknown")
            .to_string();

        let endpoint = ENDPOINT_NAME.get().unwrap().to_string();

        WorkerInfo {
            req_id,
            wasm_module,
            user_id,
            project_id,
            deploy_id,
            host,
            endpoint,
        }
    }
}

#[derive(Clone)]
pub struct WorkerMetrics {
    pub req_fn_total: Counter,
    pub req_fn_notfound_total: Counter,
    pub req_fn_success_total: Counter,
    pub req_fn_error_total: Counter,
    pub req_fn_in_bytes_total: Counter,
    pub req_fn_out_bytes_total: Counter,
}

impl WorkerMetrics {
    pub fn new(pid: String, uid: String, did: String, ep: String) -> Self {
        if !ENABLE_METRICS.get().unwrap() {
            let noop = Counter::noop();
            return WorkerMetrics {
                req_fn_total: noop.clone(),
                req_fn_notfound_total: noop.clone(),
                req_fn_success_total: noop.clone(),
                req_fn_error_total: noop.clone(),
                req_fn_in_bytes_total: noop.clone(),
                req_fn_out_bytes_total: noop,
            };
        }
        let labels = vec![("pid", pid), ("uid", uid), ("did", did), ("ep", ep)];
        let mut req_fn_total_labels = labels.clone();
        req_fn_total_labels.push(("typ", "all".to_string()));
        let mut req_fn_notfound_total_labels = labels.clone();
        req_fn_notfound_total_labels.push(("typ", "notfound".to_string()));
        let mut req_fn_success_total_labels = labels.clone();
        req_fn_success_total_labels.push(("typ", "success".to_string()));
        let mut req_fn_error_total_labels = labels.clone();
        req_fn_error_total_labels.push(("typ", "error".to_string()));
        let mut req_fn_in_bytes_total_labels = labels.clone();
        req_fn_in_bytes_total_labels.push(("typ", "main_in_bytes".to_string()));
        let mut req_fn_out_bytes_total_labels = labels.clone();
        req_fn_out_bytes_total_labels.push(("typ", "main_out_bytes".to_string()));
        WorkerMetrics {
            req_fn_total: counter!("req_fn_total", &req_fn_total_labels),
            req_fn_notfound_total: counter!("req_fn_total", &req_fn_notfound_total_labels),
            req_fn_success_total: counter!("req_fn_total", &req_fn_success_total_labels),
            req_fn_error_total: counter!("req_fn_total", &req_fn_error_total_labels),
            req_fn_in_bytes_total: counter!("req_fn_bytes", &req_fn_in_bytes_total_labels),
            req_fn_out_bytes_total: counter!("req_fn_bytes", &req_fn_out_bytes_total_labels),
        }
    }
}

/// worker_info to get worker info
pub async fn worker_info(mut request: Request, next: Next) -> Result<Response, StatusCode> {
    let req_id = xid::new().to_string();
    let headers = request.headers();
    let info = WorkerInfo::new(headers, req_id);
    let metrics = WorkerMetrics::new(
        info.project_id.clone(),
        info.user_id.clone(),
        info.deploy_id.clone(),
        info.endpoint.clone(),
    );
    request.extensions_mut().insert(info);
    request.extensions_mut().insert(metrics);
    Ok(next.run(request).await)
}
