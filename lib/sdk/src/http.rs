//! `http` is a module for http request and response.

use crate::body::Body as SdkBody;

/// `Body` is a wrapper around the wasi http_body API
pub type Body = SdkBody;
/// `Request` is a wrapper around the wasi http_request API
pub type Request = http::Request<Body>;
/// `Response` is a wrapper around the wasi http_response API
pub type Response = http::Response<Body>;

/// `RequestError` is error type when fetching request failed.
pub type RequestError = super::export_service::land::http::fetching::RequestError;
/// `RequestOptions` is options for fetching request, including timeout and redirect policy.
pub type RequestOptions = super::export_service::land::http::fetching::RequestOptions;
/// `RedirectPolicy` is redirect policy for fetching request.
pub type RedirectPolicy = super::export_service::land::http::types::RedirectPolicy;

/// `error_response` is a helper function to build a response with status code and message.
pub fn error_response(status: http::StatusCode, message: String) -> Response {
    let mut response = Response::new(message.into());
    *response.status_mut() = status;
    response
}

/// Error type for SDK, alias to `anyhow::Error`
pub type Error = anyhow::Error;

// re-export http_outgoing into http crate
pub use super::fetch::fetch;