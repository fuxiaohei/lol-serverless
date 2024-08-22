use super::export_service::land::http::fetching::{send_request, RequestError, RequestOptions};
use super::export_service::land::http::types::{self, RedirectPolicy};
use super::http::{Body, Request, Response};

impl Default for RequestOptions {
    fn default() -> Self {
        RequestOptions {
            timeout: 30,
            redirect: RedirectPolicy::Follow,
        }
    }
}

/// `fetch` is a helper function to make http request.
/// It will return a `Response` or `RequestError`.
///
/// # Example
///
/// ```no_run
/// use land_sdk::http::{fetch, Body, Error, Request, RequestOptions, Response};
/// use land_sdk::http_main;
///
/// #[http_main]
/// pub fn handle_request(_req: Request) -> Result<Response, Error> {
///     let fetch_request = http::Request::builder()
///         .method("GET")
///         .uri("https://www.rust-lang.org/")
///         .body(Body::from(""))
///         .unwrap();
///     let fetch_response = fetch(fetch_request, RequestOptions::default()).unwrap();
///     Ok(http::Response::builder()
///         .status(fetch_response.status())
///         .body(fetch_response.into_body())
///         .unwrap())
/// }
/// ```
///
pub fn fetch(req: Request, options: RequestOptions) -> Result<Response, RequestError> {
    let mut headers = vec![];
    for (key, value) in req.headers() {
        headers.push((key.to_string(), value.to_str().unwrap().to_string()));
    }
    let uri = req.uri().clone().to_string();
    let method = req.method().clone();

    let fetch_req = types::Request {
        headers,
        uri,
        method: method.to_string(),
        body: Some(req.body().body_handle()),
    };
    let fetch_resp = send_request(&fetch_req, options)?;

    let mut builder = http::Response::builder().status(fetch_resp.status);
    for (key, value) in fetch_resp.headers {
        builder = builder.header(key, value);
    }
    let resp_body = Body::from_handle(fetch_resp.body.unwrap());
    let resp = builder.body(resp_body).unwrap();

    Ok(resp)
}
