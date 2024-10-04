use super::{
    body_ctx::BodyCtx,
    host::land::http::body::{self, BodyError, BodyHandle},
};
use crate::hostcall::asyncio::Context as AsyncioContext;
use crate::hostcall::host::land::asyncio::asyncio;
use crate::hostcall::host::land::asyncio::types::Handle as AsyncioHandle;
use axum::body::Body;

/// HostCtx is the context of host calls
pub struct HostCtx {
    // body related
    body_ctx: BodyCtx,
    // elapsed time need
    created_at: tokio::time::Instant,
    // asyncio context
    asyncio_ctx: AsyncioContext,
}

impl HostCtx {
    /// new host context
    pub fn new() -> Self {
        Self {
            body_ctx: BodyCtx::new(),
            created_at: tokio::time::Instant::now(),
            asyncio_ctx: AsyncioContext::new(),
        }
    }

    /// new_body creates new empty body and returns handle id
    pub fn new_empty_body(&mut self) -> u32 {
        self.body_ctx.new_empty()
    }

    /// set_body sets body by id, it will return handle id
    pub fn set_body(&mut self, id: u32, body: Body) -> u32 {
        self.body_ctx.set_body(id, body)
    }

    /// take_body takes body by id, it will remove body from map
    pub fn take_body(&mut self, id: u32) -> Option<Body> {
        self.body_ctx.take_body(id)
    }

    /// read_body reads body by id
    pub async fn read_body(
        &mut self,
        handle: u32,
        size: u32,
    ) -> Result<(Vec<u8>, bool), BodyError> {
        self.body_ctx.read_body(handle, size).await
    }

    /// read_body_all reads all body by id
    pub async fn read_body_all(&mut self, handle: u32) -> Result<Vec<u8>, BodyError> {
        self.body_ctx.read_body_all(handle).await
    }

    /// new_writable_body creates new body stream and returns handle id
    pub fn new_writable_body(&mut self) -> u32 {
        self.body_ctx.new_writable_body()
    }

    /// write_body is used to write data to body
    pub async fn write_body(&mut self, handle: u32, data: Vec<u8>) -> Result<u64, BodyError> {
        self.body_ctx.write_body(handle, data).await
    }

    /// elapsed returns the elapsed time in milliseconds
    pub fn elapsed(&self) -> tokio::time::Duration {
        self.created_at.elapsed()
    }
}

#[async_trait::async_trait]
impl body::Host for HostCtx {
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
impl asyncio::Host for HostCtx {
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
