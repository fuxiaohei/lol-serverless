use axum::{
    body::{Body, HttpBody},
    extract::{Path, Request},
    http::{HeaderValue, StatusCode},
    response::IntoResponse,
    Json,
};
use land_service::{confs, storage, workerlivings};
use land_utils::localip;
use serde::{Deserialize, Serialize};
use tracing::debug;

pub mod task;

/// heartbeat handles the worker heartbeat request
pub async fn heartbeat(req: Request<Body>) -> Result<impl IntoResponse, JsonError> {
    let (parts, body) = req.into_parts();
    // if body is empty, means just heartbeat
    if body.size_hint().lower() > 0 {
        // refresh living worker agent
        let body_bytes = axum::body::to_bytes(body, usize::MAX).await?;
        let ipinfo = serde_json::from_slice::<localip::IP>(&body_bytes)?;
        workerlivings::update(ipinfo).await;
    }

    // check confs md5
    let confs = confs::get().await;
    let req_md5 = parts.headers.get("X-Md5");
    if let Some(req_md5) = req_md5 {
        if req_md5.to_str().unwrap() == confs.0 && !confs.0.is_empty() {
            return Ok((StatusCode::NOT_MODIFIED, ()).into_response());
        }
    }

    // if not match, return new confs
    let mut resp = resp_json_ok(confs.1, None).into_response();
    resp.headers_mut()
        .insert("X-Md5", HeaderValue::from_str(confs.0.as_str())?);
    Ok(resp)
}

/// download handles the worker download wasm request
pub async fn download(Path(path): Path<String>) -> Result<impl IntoResponse, JsonError> {
    let real_path = format!("wasm/{}", path);
    let wasm_bytes = storage::read(&real_path).await?;
    debug!("download: {:?}, path:{}", wasm_bytes.len(), path);
    Ok(wasm_bytes.into_response())
}

#[derive(Debug, Serialize, Deserialize)]
struct JsonResponse<T> {
    pub status: String,
    pub message: String,
    pub data: T,
}

/// resp_json_ok returns a response with status ok
pub fn resp_json_ok(data: impl Serialize, msg: Option<String>) -> impl IntoResponse {
    let msg = msg.unwrap_or("ok".to_string());
    Json(JsonResponse {
        status: "ok".to_string(),
        message: msg,
        data,
    })
}

/// resp_json_error returns a response with status error
pub fn resp_json_error(msg: String) -> impl IntoResponse {
    Json(JsonResponse {
        status: "error".to_string(),
        message: msg,
        data: (),
    })
}

// Make our own error that wraps `anyhow::Error`.
pub struct JsonError(pub StatusCode, pub anyhow::Error);

impl Clone for JsonError {
    fn clone(&self) -> Self {
        Self(self.0, anyhow::anyhow!(self.1.to_string()))
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, RespError>`. That way you don't need to do that manually.
impl<E> From<E> for JsonError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(StatusCode::INTERNAL_SERVER_ERROR, err.into())
    }
}

// Tell axum how to convert `RespError` into a response.
impl IntoResponse for JsonError {
    fn into_response(self) -> axum::response::Response {
        let mut resp = resp_json_error(self.1.to_string()).into_response();
        *resp.status_mut() = self.0;
        let exts = resp.extensions_mut();
        exts.insert(self);
        resp
    }
}
