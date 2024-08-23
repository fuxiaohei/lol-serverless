use super::http_service::land::asyncio::asyncio;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

type WaitUntilHandler = Box<dyn Fn() + Send + 'static>;

struct Inner {
    pub handlers: HashMap<u32, WaitUntilHandler>,
}

impl Inner {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
        }
    }
    pub fn wait_until(&mut self, f: WaitUntilHandler) {
        let seq_id = asyncio::new().unwrap();
        self.handlers.insert(seq_id, f);
    }

    /// sleep add empty sleep task to asyncio task with seq_id
    pub fn sleep(&mut self, ms: u32) -> u32 {
        asyncio::sleep(ms).unwrap()
    }

    /// sleep_callback add callback function to asyncio task with seq_id
    pub fn sleep_callback(&mut self, seq_id: u32, f: WaitUntilHandler) {
        self.handlers.insert(seq_id, f);
    }

    /*
    fn execute_runnable(&mut self) -> bool {
        let (handle, is_wait) = asyncio::select();
        if self.handlers.is_empty() {
            return false;
        }

        while let Some(idx) = self
            .handlers
            .iter()
            .position(|(handle, _)| asyncio::is_runnable(*handle))
        {
            let (seq_id, handler) = self.handlers.remove(idx).unwrap();
            println!("asyncio->execute_runnable: {:?}", seq_id);
            handler();
            asyncio::finish(seq_id);
            return true;
        }

        return false;
    }*/

    pub fn execute(&mut self) {
        let (handle, is_wait) = asyncio::select();
        if !is_wait {
            return;
        }
        // no handle to run, but is-wait=true, do wait
        if handle.is_none() {
            asyncio::ready();
            // after ready, select runnable when next time
            return;
        }
        let handle = handle.unwrap();
        let handler = self.handlers.remove(&handle);
        if let Some(handler) = handler {
            // call callback function
            handler();
        }
    }
    pub fn is_pending(&self) -> bool {
        !self.handlers.is_empty()
    }
}

/// `ExecutionCtx` is context to handle asyncio tasks
/// It used to add functions after http request done
#[derive(Clone)]
pub struct ExecutionCtx {
    inner: Arc<Mutex<Inner>>,
}

impl Default for ExecutionCtx {
    fn default() -> Self {
        Self::new()
    }
}

lazy_static::lazy_static! {
    static ref CTX: Mutex<ExecutionCtx> = Mutex::new(ExecutionCtx::new());
}

impl ExecutionCtx {
    /// `get_ctx` gets global execution ctx instance
    pub fn get() -> ExecutionCtx {
        CTX.lock().unwrap().clone()
    }
    /// `new` create new exection ctx instance
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner::new())),
        }
    }
    /// `wait_until` add function to asyncio task
    /// after http request done, it will be executed
    ///
    /// # Example
    ///
    /// ```no_run
    /// use land_sdk::http::{fetch, Body, Error, Request, RequestOptions, Response};
    /// use land_sdk::{http_main, ExecutionCtx};

    /// #[http_main]
    /// pub fn handle_request(req: Request, mut ctx: ExecutionCtx) -> Result<Response, Error> {
    ///     // read uri and method from request
    ///     let url = req.uri().clone();
    ///     let method = req.method().to_string().to_uppercase();
    ///
    ///     ctx.wait_until(|| {
    ///         // this fetch behavior will execute after http request done
    ///         let fetch_request = http::Request::builder()
    ///             .method("GET")
    ///             .uri("https://www.rust-lang.org/")
    ///             .body(Body::from(""))
    ///             .unwrap();
    ///         let fetch_response = fetch(fetch_request, RequestOptions::default()).unwrap();
    ///         println!("wait until fetch: {:?}", fetch_response);
    ///     });
    ///
    ///     // build response
    ///     Ok(http::Response::builder()
    ///         .status(200)
    ///         .header("X-Request-Url", url.to_string())
    ///         .header("X-Request-Method", method)
    ///         .body(Body::from("Hello Runtime.land!!"))
    ///         .unwrap())
    /// }
    /// ```
    ///
    pub fn wait_until<F>(&mut self, f: F)
    where
        F: Fn() + 'static + Send,
    {
        self.inner.lock().unwrap().wait_until(Box::new(f));
    }
    /// `execute` calls one asyncio task
    /// after execute, it will be removed from asyncio task list
    /// then it should check is_pending to check if there is any asyncio task pending
    pub fn execute(&mut self) {
        self.inner.lock().unwrap().execute();
    }
    /// `is_pending` check if there is any asyncio task pending
    pub fn is_pending(&self) -> bool {
        self.inner.lock().unwrap().is_pending()
    }
    /// `sleep` sleep for `ms` milliseconds in hostcall tokio spawn task
    pub fn sleep(&self, ms: u32) -> u32 {
        self.inner.lock().unwrap().sleep(ms)
    }
    /// `sleep_callback` add callback function to asyncio task with seq_id
    pub fn sleep_callback<F>(&self, id: u32, f: F)
    where
        F: Fn() + 'static + Send,
    {
        self.inner.lock().unwrap().sleep_callback(id, Box::new(f));
    }
}

/*
#[cfg(test)]
mod execution_ctx_test {
    use super::ExecutionCtx;

    fn test() {
        let mut ctx = ExecutionCtx::new();
        ctx.wait_until(|| {
            println!("sleep 1s...");
            std::thread::sleep(std::time::Duration::from_secs(1));
            println!("sleep 1s done!");
        });

        ctx.wait_until(|| {
            println!("sleep 2s...");
            std::thread::sleep(std::time::Duration::from_secs(2));
            println!("sleep 2s done!");
        });

        let ctx2 = ctx.clone();
        ctx.wait_until(move || {
            ctx2.sleep(1500);
            println!("sleep 1.5s done!");
        });
    }
}
*/
