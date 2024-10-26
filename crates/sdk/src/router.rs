//! `router` is a simple router for http server
//!
//! # Example
//!
//! ```no_run
//! use land_sdk::http::{Body, Error, Request, Response};
//! use land_sdk::http_main;
//! use land_sdk::router;
//!
//! #[http_main]
//! pub fn handle_http_request(mut req: Request) -> Result<Response, Error> {
//!     router::get("/hello", echo_hello).unwrap();
//!     router::get("/foo/bar", echo_foo_bar).unwrap();
//!     router::get("/params/:value", echo_params).unwrap();
//!     router::route(req)
//! }
//!
//! pub fn echo_hello(_req: Request) -> Result<Response, Error> {
//!     Ok(http::Response::builder()
//!         .status(200)
//!         .body(Body::from("Hello, World"))
//!         .unwrap())
//! }
//!
//! pub fn echo_foo_bar(_req: Request) -> Result<Response, Error> {
//!     Ok(http::Response::builder()
//!         .status(200)
//!         .body(Body::from("Foo Bar"))
//!         .unwrap())
//! }
//!
//! pub fn echo_params(req: Request) -> Result<Response, Error> {
//!     let value = router::params(&req, "value".to_string()).unwrap();
//!     Ok(http::Response::builder()
//!         .status(200)
//!         .body(Body::from(format!("value: {value}")))
//!         .unwrap())
//! }
//!
//! ```
//!

use crate::http::{error_response, Error, Request, Response};
use http::{Method, StatusCode};
use matchit::Router;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::str::FromStr;
use std::sync::{Arc, Mutex};

/// trait Handler for handle http request
pub trait Handler: Send + Sync + 'static {
    /// call is a function for handle http request
    fn call(&self, req: Request) -> Result<Response, Error>;
}

impl Debug for dyn Handler {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("dyn Handler").finish()
    }
}

impl<F> Handler for F
where
    F: Send + Sync + 'static + Fn(Request) -> Result<Response, Error>,
{
    fn call<'a>(&'_ self, req: Request) -> Result<Response, Error> {
        (self)(req)
    }
}

// RouteHandler is router with method
type RouteHandler = HashMap<Method, Router<Arc<dyn Handler>>>;

lazy_static::lazy_static! {
     static ref ROUTER: Mutex<RouteHandler> = Mutex::new(HashMap::new());
}

/// macro for add handler for method
macro_rules! method_route {
    ($method:ident) => {
        /// add handler for method
        pub fn $method(
            path: &str,
            handler: impl Handler,
        ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let method_name = Method::from_str(&stringify!($method).to_uppercase())?;
            ROUTER
                .lock()
                .unwrap()
                .entry(method_name)
                .or_default()
                .insert(path, Arc::new(handler))?;
            Ok(())
        }
    };
}

method_route!(post);
method_route!(get);
method_route!(put);
method_route!(delete);
method_route!(head);
method_route!(options);
method_route!(patch);

/// any add handler for all methods
pub fn any(
    path: &str,
    handler: impl Handler + std::marker::Copy,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    get(path, handler)?;
    post(path, handler)?;
    put(path, handler)?;
    delete(path, handler)?;
    head(path, handler)?;
    options(path, handler)?;
    patch(path, handler)?;
    Ok(())
}

/// route runs handler
pub fn route(mut req: Request) -> Result<Response, Error> {
    // get method and path to match router
    let method = req.method().clone();
    let path = req.uri().clone();
    let path = path.path();

    // get router
    let mut router = ROUTER.lock().unwrap();
    let router = router.entry(method).or_default();

    // match router
    let matched = router.at(path);
    if matched.is_err() {
        return Ok(error_response(
            StatusCode::NOT_FOUND,
            "route mismatch".to_string(),
        ));
    }

    // prepare params
    let mut route_params = HashMap::new();
    matched.as_ref().unwrap().params.iter().for_each(|(k, v)| {
        route_params.insert(k.to_string(), v.to_string());
    });
    req.extensions_mut().insert(route_params);

    // call handler
    let handler = matched.unwrap().value;
    handler.call(req)
}

/// params get value from request
pub fn params(req: &Request, key: String) -> Option<String> {
    let ext = req.extensions();
    // find params map
    let params = ext.get::<HashMap<String, String>>()?;
    // get value
    let value = params.get(&key)?;
    Some(value.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::http::{Body, Error, Request, Response};

    #[test]
    fn test_handler_impl() {
        let handler: Arc<dyn Handler> =
            Arc::new(|_req: Request| Ok(Response::new(Body::from_handle(2))));
        let req = http::Request::builder()
            .uri("/")
            .body(Body::from_handle(1))
            .unwrap();
        let resp = handler.call(req).unwrap();
        assert_eq!(resp.status(), 200);
        assert_eq!(resp.body().body_handle(), 2);
    }

    pub fn test_route_1(req: Request) -> Result<Response, Error> {
        let url = req.uri().clone();
        let method = req.method().to_string().to_uppercase();
        Ok(http::Response::builder()
            .status(200)
            .header("X-Request-Url", url.to_string())
            .header("X-Request-Method", method)
            .body(Body::from_handle(2))
            .unwrap())
    }

    #[test]
    fn test_router() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        {
            get("/abc", test_route_1)?;
        }

        {
            let mut router = ROUTER.lock().unwrap();
            let router = router.entry(Method::GET).or_default();
            let matched = router.at("/abcd");
            if matched.is_err() {
                assert_eq!(matched.err().unwrap(), matchit::MatchError::NotFound);
            }
        }

        {
            let mut router = ROUTER.lock().unwrap();
            let router = router.entry(Method::GET).or_default();
            let matched = router.at("/abc");
            assert!(matched.is_ok());
            let handler = matched.unwrap();
            let req = http::Request::builder()
                .method(Method::GET)
                .uri("/abc")
                .body(Body::from_handle(1))
                .unwrap();
            let resp = handler.value.call(req).unwrap();
            assert_eq!(resp.status(), 200);
        }
        Ok(())
    }

    #[test]
    fn test_wildcard() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        {
            any("/xyz/*path", test_route_1)?;
        }
        {
            let mut router = ROUTER.lock().unwrap();
            let router = router.entry(Method::GET).or_default();
            let matched = router.at("/xyz/abc");
            assert!(matched.is_ok());

            let mut route_params = HashMap::new();
            matched.as_ref().unwrap().params.iter().for_each(|(k, v)| {
                route_params.insert(k.to_string(), v.to_string());
            });

            let path_str = matched.as_ref().unwrap().params.get("path").unwrap();
            assert_eq!(path_str, "abc");

            let handler = matched.unwrap();
            let mut req = http::Request::builder().method(Method::GET).uri("/xyz/abc");
            req.extensions_mut().unwrap().insert(route_params);
            let req = req.body(Body::from_handle(1)).unwrap();

            let p = params(&req, "path".to_string());
            assert_eq!(p, Some("abc".to_string()));

            let resp = handler.value.call(req).unwrap();
            assert_eq!(resp.status(), 200);
        }
        Ok(())
    }
}
