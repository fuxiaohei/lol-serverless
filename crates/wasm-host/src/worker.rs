use crate::{hostcall::HttpHandlerPre, Context};
use anyhow::Result;
use axum::body::Body;
use tracing::debug;
use wasmtime::{
    component::{Component, Linker},
    Engine, Store, UpdateDeadline,
};

/// Worker is used to run wasm component
#[derive(Clone)]
pub struct Worker {
    path: String,
    engine: Engine,
    instance_pre: HttpHandlerPre<Context>,
}

impl std::fmt::Debug for Worker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Worker").field("path", &self.path).finish()
    }
}

impl Worker {
    // from_binary is used to create worker from bytes
    pub async fn from_binary(bytes: &[u8], path: Option<String>) -> Result<Self> {
        let engine = crate::engine::get("default")?;
        let component = Component::from_binary(&engine, bytes)?;
        debug!("Load wasm component from binary, size:{}", bytes.len());

        // create linker
        let mut linker: Linker<crate::context::Context> = Linker::new(&engine);

        // init wasi context
        wasmtime_wasi::add_to_linker_async(&mut linker)?;

        // init http_service
        crate::hostcall::HttpService::add_to_linker(&mut linker, Context::host_ctx)
            .expect("add http_service failed");

        // create instance-pre
        let instance_pre = linker.instantiate_pre(&component)?;
        Ok(Self {
            path: path.unwrap_or("binary".to_string()),
            engine,
            instance_pre: HttpHandlerPre::new(instance_pre)?,
        })
    }

    async fn from_aot(path: String) -> Result<Self> {
        let engine = crate::engine::get("default")?;
        let bytes = std::fs::read(&path)?;
        debug!(
            "Load wasm component from AOT file: {}, size: {}",
            path,
            bytes.len()
        );

        let component = unsafe { Component::deserialize(&engine, bytes)? };

        // create linker
        let mut linker: Linker<crate::context::Context> = Linker::new(&engine);
        // init wasi context
        wasmtime_wasi::add_to_linker_async(&mut linker)?;
        // init http_service
        crate::hostcall::HttpService::add_to_linker(&mut linker, crate::context::Context::host_ctx)
            .expect("add http_service failed");

        let instance_pre = linker.instantiate_pre(&component)?;
        Ok(Self {
            path,
            engine,
            instance_pre: HttpHandlerPre::new(instance_pre)?,
        })
    }

    pub fn compile_aot(src: &str, dst: &str) -> Result<()> {
        let engine = super::engine::get("default")?;
        let component = Component::from_file(&engine, src)?;
        let bytes = Component::serialize(&component)?;
        debug!("Write AOT from {} to {}, size: {}", src, dst, bytes.len());
        std::fs::write(dst, bytes)?;
        Ok(())
    }

    /// new a worker from path
    pub async fn new(path: &str, is_aot: bool) -> Result<Self> {
        let binary = std::fs::read(path)?;

        // compile aot wasm
        if is_aot {
            let suffix = format!(".wasm.{}.aot", crate::engine::MODULE_VERSION);
            let aot_path = path.replace(".wasm", &suffix);
            if std::path::Path::new(&aot_path).exists() {
                return Self::from_aot(aot_path).await;
            }
            let path2 = path.to_string();
            std::thread::spawn(move || {
                match Self::compile_aot(&path2, &aot_path) {
                    Ok(_) => debug!("Compile AOT success: {}", &aot_path),
                    Err(e) => debug!("Compile AOT failed: {}", e),
                };
            });
        }
        Self::from_binary(&binary, Some(path.to_string())).await
    }

    /// handle_request is used to handle http request
    pub async fn handle_request(
        &self,
        req: crate::hostcall::Request,
        context: crate::context::Context,
    ) -> Result<(crate::hostcall::Response, Body)> {
        // create store
        let mut store = Store::new(&self.engine, context);
        store.set_epoch_deadline(1);
        store.epoch_deadline_callback(move |store| {
            debug!(
                "epoch_deadline_callback, cost:{:.2?}",
                store.data().elapsed()
            );
            Ok(UpdateDeadline::Yield(1))
        });
        store.limiter(|ctx| &mut ctx.limiter);

        // get exports and call handle_request
        let exports = self.instance_pre.instantiate_async(&mut store).await?;
        let resp = exports
            .land_http_incoming()
            .call_handle_request(&mut store, &req)
            .await?;
        let body_handle = resp.body.unwrap();
        let body = store.data_mut().take_body(body_handle).unwrap();
        debug!("response is ready, body:{}", body_handle);
        // check async task is pending
        let is_pending = exports
            .land_asyncio_context()
            .call_is_pending(&mut store)
            .await?;
        debug!("async task is pending: {}", is_pending);
        if is_pending {
            // let req_id = store.data().req_id();
            // let span = debug_span!("[ASYNC]", req_id);
            tokio::task::spawn(async move {
                let now = tokio::time::Instant::now();
                loop {
                    let res = match exports.land_asyncio_context().call_select(&mut store).await {
                        Ok(res) => res,
                        Err(e) => {
                            debug!("async task pending failed, error: {}", e);
                            return;
                        }
                    };
                    debug!("async task is done, res: {:?}", res);
                    if !res {
                        break;
                    }
                }
                debug!("async tasks all done, cost:{:.2?}", now.elapsed());
                // println!("async task is done, cost:{:.2?}", now.elapsed());
            });
        }
        Ok((resp, body))
    }

    /// new_in_pool is used to create worker in pool
    pub async fn new_in_pool(key: &str, is_aot: bool) -> Result<Worker> {
        crate::pool::prepare_worker(key, is_aot).await
    }
}
