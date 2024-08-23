mod asyncio;
mod body;
mod body_impl;
mod client;
mod context;
mod fetch;
mod guest;
mod host;

pub use client::init_clients;
pub use context::HostContext;
pub use guest::exports::land::http::incoming::{Request, Response};
pub use guest::HttpHandlerPre;
pub use host::HttpService;

impl host::land::http::types::Host for HostContext {}
impl host::land::asyncio::types::Host for HostContext {}
