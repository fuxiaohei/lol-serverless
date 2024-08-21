use super::client::get_client;
use super::host::land::http::fetching::{Host, Request, RequestError, RequestOptions, Response};
use super::host::land::http::types::RedirectPolicy;
use super::HostContext;
use axum::body::Body;
use reqwest::redirect;
use std::str::FromStr;
use tracing::{debug, warn};

impl Default for RequestOptions {
    fn default() -> Self {
        RequestOptions {
            timeout: 30,
            redirect: RedirectPolicy::Follow,
        }
    }
}

impl TryFrom<RedirectPolicy> for redirect::Policy {
    type Error = anyhow::Error;
    fn try_from(value: RedirectPolicy) -> Result<Self, Self::Error> {
        match value {
            RedirectPolicy::Follow => Ok(redirect::Policy::default()),
            RedirectPolicy::Error => Ok(redirect::Policy::custom(|attempt| {
                attempt.error(anyhow::anyhow!("redirect policy is error"))
            })),
            RedirectPolicy::Manual => Ok(redirect::Policy::none()),
        }
    }
}

#[async_trait::async_trait]
impl Host for HostContext {
    async fn send_request(
        &mut self,
        request: Request,
        options: RequestOptions,
    ) -> Result<Response, RequestError> {
        let st = tokio::time::Instant::now();
        debug!(method = request.method, uri = request.uri, "Fetch start");

        // read body
        let body = match self.take_body(request.body.unwrap_or(0)) {
            Some(b) => b,
            None => Body::empty(),
        };

        // FIXME: read body bytes is not correct, use stream instead
        let body_bytes = axum::body::to_bytes(body, usize::MAX).await.map_err(|e| {
            warn!(
                method = request.method,
                uri = request.uri,
                "Fetch failed: {e}"
            );
            RequestError::InvalidRequest(e.to_string())
        })?;

        let client = get_client(options.redirect);
        // call fetch
        let mut fetch_builder = client
            .request(
                reqwest::Method::from_str(request.method.as_str()).unwrap(),
                request.uri.clone(),
            )
            .timeout(std::time::Duration::from_secs(options.timeout as u64))
            .body(body_bytes);
        if !request.headers.is_empty() {
            for (key, value) in request.headers {
                fetch_builder = fetch_builder.header(key, value);
            }
        }
        let fetch_result = fetch_builder.send().await;

        // handle fetch result failed
        if fetch_result.is_err() {
            let e = fetch_result.err().unwrap();
            let content = e.to_string();
            warn!(
                method = request.method,
                uri = request.uri,
                "Fetch failed: {content}"
            );
            // check if network error
            if content.contains("connect") {
                return Err(RequestError::NetworkError(content));
            }
            return Err(RequestError::InvalidRequest(format!(
                "Fetch failed: {content}"
            )));
        }

        let fetch_response = fetch_result.unwrap();
        let mut resp_headers = vec![];
        // if body is stream, header should not contain content-length, use Transfer-Encoding:chunk
        let mut is_stream = true;
        let mut content_length: usize = 0;
        for (key, value) in fetch_response.headers() {
            if key == "content-length" {
                is_stream = false;
                content_length = value.to_str().unwrap().parse().unwrap();
            }
            let header_value = String::from_utf8_lossy(value.as_bytes()).to_string();
            resp_headers.push((key.to_string(), header_value));
        }

        let status = fetch_response.status().as_u16();
        let body_handle = if is_stream {
            let body_stream = fetch_response.bytes_stream();
            let body = Body::from_stream(body_stream);
            self.set_body(0, body)
        } else {
            let body = fetch_response.bytes().await.unwrap();
            let body = Body::from(body);
            self.set_body(0, body)
        };
        debug!(
            method = request.method,
            uri = request.uri,
            "Fetch set body: {}, is_stream:{}, content_length:{}",
            body_handle,
            is_stream,
            content_length,
        );
        let resp = Response {
            status,
            headers: resp_headers,
            body: Some(body_handle),
        };
        let elasped = st.elapsed().as_millis();
        debug!(
            method = request.method,
            uri = request.uri,
            status = resp.status,
            handle = body_handle,
            cost = elasped,
            "Fetch done",
        );
        Ok(resp)
    }
}
