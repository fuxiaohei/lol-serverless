#![allow(clippy::redundant_clone)]

//! # Rust SDK Macro for Runtime.land.
//!
//! This macro is used to develop Runtime.land functions in `land-sdk`.
//! It should not be used directly.
//!
//! # Hello World
//!
//! ```no_run
//! use land_sdk::http::{Body, Request, Response};
//! use land_sdk::http_main;
//!
//! #[http_main]
//! pub fn handle_request(req: Request) -> Response {
//!     // read uri and method from request
//!     let url = req.uri().clone();
//!     let method = req.method().to_string().to_uppercase();
//!
//!     // build response
//!     http::Response::builder()
//!         .status(200)
//!         .header("X-Request-Url", url.to_string())
//!         .header("X-Request-Method", method)
//!         .body(Body::from("Hello Runtime.land!!"))
//!         .unwrap()
//! }
//! ```

use proc_macro::TokenStream;
use quote::quote;
use std::sync::atomic::AtomicBool;

static HTTP_SRC_INCLUDE: AtomicBool = AtomicBool::new(false);

/// http_main is a macro to generate a http handler function.
#[proc_macro_attribute]
pub fn http_main(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = syn::parse_macro_input!(item as syn::ItemFn);
    let func_name = func.sig.ident.clone();
    let func_args_len = func.sig.inputs.len();

    let src_http_handler = if HTTP_SRC_INCLUDE.load(std::sync::atomic::Ordering::Relaxed) {
        String::new()
    } else {
        HTTP_SRC_INCLUDE.store(true, std::sync::atomic::Ordering::Relaxed);
        include_str!("./http_handler.rs").to_string()
    };
    let iface: TokenStream = src_http_handler
        .parse()
        .expect("cannot parse http_handler.rs");

    let iface_impl = quote!(

        use exports::land::http::incoming;
        use exports::land::asyncio::context;

        struct WorkerHttpImpl;

        impl TryFrom<incoming::Request> for Request {
            type Error = anyhow::Error;

            fn try_from(wasm_req: incoming::Request) -> Result<Self, Self::Error> {
                use std::str::FromStr;

                let mut http_req = http::Request::builder()
                    .method(http::Method::from_str(wasm_req.method.as_str())?)
                    .uri(&wasm_req.uri);

                for (key, value) in wasm_req.headers {
                    http_req = http_req.header(key, value);
                }
                // 1 is the request body handle, which is defined in wasi host functions
                let body = Body::from_handle(wasm_req.body.unwrap_or(1));
                Ok(http_req.body(body)?)
            }
        }

        impl TryFrom<Response> for incoming::Response {
            type Error = anyhow::Error;

            fn try_from(http_res: Response) -> Result<Self, Self::Error> {
                let status = http_res.status().as_u16();
                let mut headers: Vec<(String, String)> = vec![];
                for (key, value) in http_res.headers() {
                    headers.push((key.to_string(), value.to_str()?.to_string()));
                }
                let body = http_res.body();
                Ok(incoming::Response {
                    status,
                    headers,
                    body: Some(body.body_handle()),
                })
            }
        }

    );

    // if func args len is 1, it means that the function has one argument, no ExecutionCtx
    // so context::Guest should not be used
    let mut async_impl = quote!(
        impl context::Guest for WorkerHttpImpl {
            fn is_pending() -> bool{
               return false
            }

            fn select() -> bool {
               return false
            }
        }
    );
    if func_args_len == 2 {
        async_impl = quote!(
            impl context::Guest for WorkerHttpImpl {
                fn is_pending() -> bool{
                    let ctx = ExecutionCtx::get();
                    ctx.is_pending()
                }

                fn select() -> bool {
                    let mut ctx = ExecutionCtx::get();
                    ctx.execute();
                    ctx.is_pending()
                }
            }
        );
    }

    let mut iface_impl2 = quote!(
        impl incoming::Guest for WorkerHttpImpl {
            fn handle_request(req: incoming::Request) -> incoming::Response {
                #func

                // convert wasm_request to sdk_request
                let sdk_request: Request = req.try_into().unwrap();
                let sdk_response = match #func_name(sdk_request){
                    Ok(r) => r,
                    Err(e) => {
                        land_sdk::http::error_response(
                            http::StatusCode::INTERNAL_SERVER_ERROR,
                            e.to_string(),
                        )
                    }
                };

                let sdk_response_body_handle = sdk_response.body().body_handle();
                // convert sdk_response to wasm_response
                match sdk_response.try_into() {
                    Ok(r) => r,
                    Err(_e) => incoming::Response {
                        status: 500,
                        headers: vec![],
                        body: Some(sdk_response_body_handle),
                    },
                }
            }
        }
    );

    // if func args len is 2, it means that the function has two arguments,
    // the first one is the request, the second one is the context
    if func_args_len == 2 {
        iface_impl2 = quote!(
            impl incoming::Guest for WorkerHttpImpl {
                fn handle_request(req: incoming::Request) -> incoming::Response {
                    #func

                    // get execution context
                    let mut ctx = ExecutionCtx::get();
                    // convert wasm_request to sdk_request
                    let sdk_request: Request = req.try_into().unwrap();
                    let sdk_response = match #func_name(sdk_request, ctx){
                        Ok(r) => r,
                        Err(e) => {
                            land_sdk::http::error_response(
                                http::StatusCode::INTERNAL_SERVER_ERROR,
                                e.to_string(),
                            )
                        }
                    };

                    let sdk_response_body_handle = sdk_response.body().body_handle();
                    // convert sdk_response to wasm_response
                    match sdk_response.try_into() {
                        Ok(r) => r,
                        Err(_e) => incoming::Response {
                            status: 500,
                            headers: vec![],
                            body: Some(sdk_response_body_handle),
                        },
                    }
                }
            }
        );
    }

    let iface_impl3 = quote!(
        export!(WorkerHttpImpl);
    );
    let user_code_comment = "// User code start";
    let value =
        format!("{iface}\n\n{user_code_comment}\n\n{iface_impl}\n\n{async_impl}\n\n{iface_impl2}\n\n{iface_impl3}");
    value.parse().unwrap()
}
