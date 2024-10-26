use super::{
    asyncio,
    body::BodyContext,
    host::land::http::{
        body::{self, BodyError},
        types::BodyHandle,
    },
};
use crate::hostcall::host::land::asyncio::asyncio::Host as AsyncioHost;
use crate::hostcall::host::land::asyncio::types::Handle as AsyncioHandle;
use axum::body::Body;

/// HostContext is a struct that holds the context of the host.
/// It is used to store the host's state and provide a way to interact with the host.
pub struct HostContext {
    // body related
    body_context: BodyContext,
    // asyncio context
    asyncio_ctx: asyncio::Context,
    // elapsed time need
    created_at: tokio::time::Instant,
}

impl Default for HostContext {
    fn default() -> Self {
        Self::new()
    }
}

impl HostContext {
    /// new host context
    pub fn new() -> Self {
        Self {
            body_context: BodyContext::new(),
            created_at: tokio::time::Instant::now(),
            asyncio_ctx: asyncio::Context::new(),
        }
    }

    /// new_body creates new empty body and returns handle id
    pub fn new_empty_body(&mut self) -> u32 {
        self.body_context.new_empty()
    }

    /// set_body sets body by id, it will return handle id
    pub fn set_body(&mut self, id: u32, body: Body) -> u32 {
        self.body_context.set_body(id, body)
    }

    /// take_body takes body by id, it will remove body from map
    pub fn take_body(&mut self, id: u32) -> Option<Body> {
        self.body_context.take_body(id)
    }

    /// read_body reads body by id
    pub async fn read_body(
        &mut self,
        handle: u32,
        size: u32,
    ) -> Result<(Vec<u8>, bool), BodyError> {
        self.body_context.read_body(handle, size).await
    }

    /// read_body_all reads all body by id
    pub async fn read_body_all(&mut self, handle: u32) -> Result<Vec<u8>, BodyError> {
        self.body_context.read_body_all(handle).await
    }

    /// new_writable_body creates new body stream and returns handle id
    pub fn new_writable_body(&mut self) -> u32 {
        self.body_context.new_writable_body()
    }

    /// write_body is used to write data to body
    pub async fn write_body(&mut self, handle: u32, data: Vec<u8>) -> Result<u64, BodyError> {
        self.body_context.write_body(handle, data).await
    }

    /// elapsed returns the elapsed time in milliseconds
    pub fn elapsed(&self) -> tokio::time::Duration {
        self.created_at.elapsed()
    }
}

#[async_trait::async_trait]
impl body::Host for HostContext {
    async fn read(&mut self, handle: BodyHandle, size: u32) -> Result<(Vec<u8>, bool), BodyError> {
        self.read_body(handle, size).await
    }

    async fn read_all(&mut self, handle: BodyHandle) -> Result<Vec<u8>, BodyError> {
        self.read_body_all(handle).await
    }

    async fn write(&mut self, handle: BodyHandle, data: Vec<u8>) -> Result<u64, BodyError> {
        self.write_body(handle, data).await
    }

    async fn new(&mut self) -> Result<BodyHandle, BodyError> {
        Ok(self.new_empty_body())
    }

    async fn new_stream(&mut self) -> Result<BodyHandle, BodyError> {
        Ok(self.new_writable_body())
    }
}

#[async_trait::async_trait]
impl AsyncioHost for HostContext {
    async fn new(&mut self) -> Result<AsyncioHandle, ()> {
        self.asyncio_ctx.new().await
    }
    async fn sleep(&mut self, ms: u32) -> Result<AsyncioHandle, ()> {
        self.asyncio_ctx.sleep(ms).await
    }
    async fn select(&mut self) -> (Option<AsyncioHandle>, bool) {
        self.asyncio_ctx.select().await
    }
    async fn ready(&mut self) {
        self.asyncio_ctx.ready().await
    }
}
