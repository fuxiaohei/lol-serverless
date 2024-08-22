use crate::export_service::land::asyncio::asyncio;
use std::sync::{Arc, Mutex};

lazy_static::lazy_static! {
    static ref CTX: Mutex<ExecutionCtx> = Mutex::new(ExecutionCtx::new());
}

/// `ExecutionCtx` is context to handle asyncio tasks
/// It used to add functions after http request done
#[derive(Clone)]
pub struct ExecutionCtx {
    inner: Arc<Mutex<ExecutionCtxInner>>,
}

impl Default for ExecutionCtx {
    fn default() -> Self {
        Self::new()
    }
}

impl ExecutionCtx {
    /// `new` create new exection ctx instance
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(ExecutionCtxInner::new())),
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

    /// `get_ctx` gets global execution ctx instance
    pub fn get_ctx() -> ExecutionCtx {
        CTX.lock().unwrap().clone()
    }

    /// `is_pending`` check if there is any pending task
    pub fn is_pending(&self) -> bool {
        self.inner.lock().unwrap().is_pending()
    }

    /// `execute` execute a pending task, order by adding sequential
    pub fn execute(&mut self) {
        self.inner.lock().unwrap().execute();
    }

    /// `cancel` cancels all tasks running. It will ignore all rest pending tasks
    pub fn cancel(&mut self) {
        self.inner.lock().unwrap().cancel();
    }
}

struct ExecutionCtxInner {
    pub handlers: Vec<(u32, WaitUntilHandler)>,
    pub cancel_flag: bool,
}

type WaitUntilHandler = Box<dyn Fn() + Send + 'static>;

impl ExecutionCtxInner {
    pub fn new() -> Self {
        Self {
            handlers: vec![],
            cancel_flag: false,
        }
    }
    pub fn wait_until(&mut self, f: WaitUntilHandler) {
        let seq_id = asyncio::new_task().unwrap();
        self.handlers.push((seq_id, f));
    }

    pub fn is_pending(&self) -> bool {
        if self.cancel_flag {
            return false;
        }
        !self.handlers.is_empty()
    }

    pub fn execute(&mut self) {
        let current = self.handlers.pop();
        if let Some((seq_id, handler)) = current {
            handler();
            asyncio::finish(seq_id);
        }
    }

    pub fn cancel(&mut self) {
        self.cancel_flag = true;
    }
}
