use super::http_service::land::asyncio::asyncio;
use std::sync::{Arc, Mutex};

type WaitUntilHandler = Box<dyn Fn() + Send + 'static>;

struct Inner {
    pub handlers: Vec<(u32, WaitUntilHandler)>,
}

impl Inner {
    pub fn new() -> Self {
        Self { handlers: vec![] }
    }
    pub fn wait_until(&mut self, f: WaitUntilHandler) {
        let seq_id = asyncio::new().unwrap();
        self.handlers.push((seq_id, f));
    }
    pub fn execute(&mut self) {
        let current = self.handlers.pop();
        if let Some((seq_id, handler)) = current {
            handler();
            asyncio::finish(seq_id);
        } else {
            // if nothing pop, check is-pending to wait sleep timer tasks
            if asyncio::is_pending() {
                asyncio::wait();
            }
        }
    }
    pub fn is_pending(&self) -> bool {
        !self.handlers.is_empty() || asyncio::is_pending()
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
        asyncio::sleep(ms).unwrap()
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
