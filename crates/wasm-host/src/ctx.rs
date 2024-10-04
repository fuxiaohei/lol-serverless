use std::collections::HashMap;
use crate::hostcall::HostCtx;
use axum::body::Body;
use bytesize::ByteSize;
use tracing::debug;
use wasmtime::{component::ResourceTable, ResourceLimiter};
use wasmtime_wasi::{WasiCtx, WasiCtxBuilder, WasiView};

#[derive(Default)]
pub struct Limiter {
    /// Total memory allocated so far.
    pub memory_allocated: usize,
}

impl ResourceLimiter for Limiter {
    fn memory_growing(
        &mut self,
        current: usize,
        desired: usize,
        _maximum: Option<usize>,
    ) -> anyhow::Result<bool> {
        // Track the diff in memory allocated over time. As each instance will start with 0 and
        // gradually resize, this will track the total allocations throughout the lifetime of the
        // instance.
        self.memory_allocated += desired - current;
        debug!("Memory: {}", ByteSize(self.memory_allocated as u64),);
        Ok(true)
    }

    fn table_growing(
        &mut self,
        _current: u32,
        _desired: u32,
        _maximum: Option<u32>,
    ) -> anyhow::Result<bool> {
        Ok(true)
    }
}

/// Ctx for the Wasm host.
pub struct Ctx {
    wasi_ctx: WasiCtx,
    host_ctx: HostCtx,

    table: ResourceTable,
    pub limiter: Limiter,
    req_id: String,
}

impl WasiView for Ctx {
    fn table(&mut self) -> &mut ResourceTable {
        &mut self.table
    }
    fn ctx(&mut self) -> &mut WasiCtx {
        &mut self.wasi_ctx
    }
}


impl Default for Ctx {
    fn default() -> Self {
        Self::new(None, String::new())
    }
}

impl Ctx {
    pub fn new(envs: Option<HashMap<String, String>>, req_id: String) -> Self {
        let table = ResourceTable::new();
        let mut wasi_ctx_builder = WasiCtxBuilder::new();
        wasi_ctx_builder.inherit_stdio();
        if let Some(envs) = envs {
            for (k, v) in envs {
                // set env key as upper case
                wasi_ctx_builder.env(k.to_uppercase(), v);
            }
        }
        Ctx {
            wasi_ctx: wasi_ctx_builder.build(),
            host_ctx: HostCtx::new(),
            limiter: Limiter::default(),
            table,
            req_id,
        }
    }
    /// get host_ctx
    pub fn host_ctx(&mut self) -> &mut HostCtx {
        &mut self.host_ctx
    }
    /// take body
    pub fn take_body(&mut self, handle: u32) -> Option<Body> {
        self.host_ctx.take_body(handle)
    }
    /// set body
    pub fn set_body(&mut self, handle: u32, body: Body) -> u32 {
        self.host_ctx.set_body(handle, body)
    }
    /// elapsed returns the duration since the request started
    pub fn elapsed(&self) -> tokio::time::Duration {
        self.host_ctx.elapsed()
    }
    /// req_id returns the request id
    pub fn req_id(&self) -> &str {
        &self.req_id
    }
}
