//! # Rust SDK for Runtime.land.
//!
//! This SDK is used to develop Runtime.land functions.
//! It provides a set of methods to handle http request and response via `land_sdk::http` module on Runtime.land.
//!
//! # Hello World
//!
//! ```no_run
//! use land_sdk::http::{Body, Error, Request, Response};
//! use land_sdk::http_main;
//!
//! #[http_main]
//! pub fn handle_request(req: Request) -> Result<Response, Error> {
//!     // read uri and method from request
//!     let url = req.uri().clone();
//!     let method = req.method().to_string().to_uppercase();
//!
//!     // build response
//!     Ok(http::Response::builder()
//!         .status(200)
//!         .header("X-Request-Url", url.to_string())
//!         .header("X-Request-Method", method)
//!         .body(Body::from("Hello Runtime.land!!"))
//!         .unwrap())
//! }
//! ```
//!

// Make sure all our public APIs have docs.
#![warn(missing_docs)]

mod body;
mod execution_ctx;
mod fetch;
mod http_service;

/// `http` is a module for http request and response.
pub mod http;
/// `router` is a module for routing http request.
pub mod router;

/// `ExecutionCtx` is a module for async execution context.
pub use execution_ctx::ExecutionCtx;

/// Re-export macro from sdk-macro
pub use land_sdk_macro::http_main;

/// `crypto` is a module for crypto functions.
pub mod crypto;
