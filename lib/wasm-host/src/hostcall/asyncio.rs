use super::{
    host::land::asyncio::{asyncio, types::Handle},
    HostContext,
};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicU32, Arc},
};
use tokio::sync::{Mutex, Notify};
use tracing::debug;

#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    Pending,
    // Running,
    // Canceled,
    Finished,
}

#[derive(Clone, Debug)]
struct Task {
    status: Status,
}

#[derive(Debug)]
struct Inner {
    pub seq_id: AtomicU32,
    pub tasks: HashMap<u32, Task>,
    pub notify: Arc<Notify>,
}

impl Inner {
    pub fn new(notify: Arc<Notify>) -> Self {
        Self {
            seq_id: AtomicU32::new(1),
            tasks: HashMap::new(),
            notify,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Context {
    notify: Arc<Notify>,
    inner: Arc<Mutex<Inner>>,
}

impl Context {
    pub fn new() -> Self {
        let notify = Arc::new(Notify::new());
        Self {
            inner: Arc::new(Mutex::new(Inner::new(notify.clone()))),
            notify,
        }
    }
    pub async fn set_finish(&mut self, seq_id: u32) {
        let mut inner = self.inner.lock().await;
        let task = inner.tasks.get_mut(&seq_id);
        if let Some(task) = task {
            if task.status == Status::Pending {
                // println!("asyncio->set_finish: {}", seq_id);
                debug!("asyncio->set_finish: {}", seq_id);
                task.status = Status::Finished;
            }
            // notify to wake up other function to check is_pending
            inner.notify.notify_one();
        }
    }
    /// wait one task done
    /// it a task is done, it wakes up to check is_pending
    pub async fn wait(&self) {
        self.notify.notified().await
    }
    /// is_pending check if there is any task pending
    pub async fn is_pending(&self) -> bool {
        let inner = self.inner.lock().await;
        return inner
            .tasks
            .values()
            .any(|task| task.status == Status::Pending);
    }
}

#[async_trait::async_trait]
impl asyncio::Host for Context {
    async fn new(&mut self) -> Result<Handle, ()> {
        let mut inner = self.inner.lock().await;
        let seq_id = inner
            .seq_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let task = Task {
            status: Status::Pending,
        };
        // println!("asyncio->new: {}", seq_id);
        debug!("asyncio->new: {}", seq_id);
        inner.tasks.insert(seq_id, task);
        Ok(seq_id)
    }
    async fn sleep(&mut self, ms: u32) -> Result<Handle, ()> {
        let seq_id = self.new().await?;
        // println!("asyncio->sleep: {}, {}ms", seq_id, ms);
        debug!("asyncio->sleep: {}, {}ms", seq_id, ms);

        let mut ctx2 = self.clone();
        tokio::task::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await;
            ctx2.set_finish(seq_id).await;
            // println!("asyncio->sleep->done: {}, {}ms", seq_id, ms);
            debug!("asyncio->sleep->done: {}, {}ms", seq_id, ms);
        });
        Ok(seq_id)
    }
    async fn finish(&mut self, handle: u32) {
        // println!("asyncio->finish: {}", handle);
        debug!("asyncio->finish: {}", handle);
        self.set_finish(handle).await;
    }
    async fn is_pending(&mut self) -> bool {
        self.is_pending().await
    }
    async fn wait(&mut self) {
        self.wait().await;
    }
}

#[async_trait::async_trait]
impl asyncio::Host for HostContext {
    async fn new(&mut self) -> Result<Handle, ()> {
        self.asyncio_ctx.new().await
    }
    async fn sleep(&mut self, ms: u32) -> Result<Handle, ()> {
        self.asyncio_ctx.sleep(ms).await
    }
    async fn finish(&mut self, handle: u32) {
        self.asyncio_ctx.finish(handle).await;
    }
    async fn is_pending(&mut self) -> bool {
        self.asyncio_ctx.is_pending().await
    }
    async fn wait(&mut self) {
        self.asyncio_ctx.wait().await;
    }
}

#[cfg(test)]
mod asyncio_test {
    use crate::hostcall::{asyncio::Context, host::land::asyncio::asyncio::Host};

    #[tokio::test]
    async fn test_sleep() {
        let mut ctx = Context::new();
        let _ = ctx.sleep(1500).await;
        let _ = ctx.sleep(1000).await;
        let mut index = 0;
        loop {
            ctx.wait().await;
            index += 1;
            let is_pending = ctx.is_pending().await;
            println!("is_pending: {}, index:{}", is_pending, index);
            if index == 2 {
                assert!(!is_pending)
            } else {
                assert!(is_pending)
            }
            if !is_pending {
                break;
            }
        }
    }
}
