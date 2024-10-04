mod body;
mod body_ctx;
mod client;
mod ctx;
mod fetch;
mod guest;
mod host;
mod asyncio;

pub use client::init_clients;
pub use ctx::HostCtx;
pub use guest::exports::land::http::incoming::{Request, Response};
pub use guest::HttpHandlerPre;
pub use host::HttpService;

impl host::land::http::types::Host for HostCtx {}
impl host::land::asyncio::types::Host for HostCtx {}