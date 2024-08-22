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
mod export_service;
mod fetch;
pub mod http;
pub mod router;

/// export type from asyncio execution context.
pub use execution_ctx::ExecutionCtx;

/// Re-export macro from sdk-macro
pub use land_sdk_macro::http_main;
