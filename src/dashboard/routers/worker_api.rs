use super::{resp_json_ok, JsonError};
use axum::{
    body::{Body, HttpBody},
    extract::Request,
    http::{HeaderValue, StatusCode},
    response::IntoResponse,
};

/// ping handles the worker ping request
pub async fn ping(req: Request<Body>) -> Result<impl IntoResponse, JsonError> {
    let (parts, body) = req.into_parts();
    // if body is empty, means just heartbeat
    if body.size_hint().lower() > 0 {
        // refresh living worker agent
        let body_bytes = axum::body::to_bytes(body, usize::MAX).await?;
        let ipinfo = serde_json::from_slice::<land_kernel::agent::IP>(&body_bytes)?;
        land_kernel::agent::set_living(ipinfo).await;
    }

    // check confs md5
    let confs = land_kernel::agent::get_confs().await;
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
