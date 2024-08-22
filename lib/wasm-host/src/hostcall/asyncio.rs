use super::{
    host::land::asyncio::{asyncio, types::AsyncHandle},
    HostContext,
};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicU32, Arc},
};
use tokio::sync::Mutex;

#[derive(Clone, Debug, PartialEq)]
pub enum AsyncioTaskStatus {
    Pending,
    // Running,
    Canceled,
    Finished,
}

#[derive(Clone, Debug)]
struct AsyncioTask {
    _handle: u32,
    status: AsyncioTaskStatus,
}

struct AsyncioContextInner {
    pub seq_id: AtomicU32,
    pub tasks: HashMap<u32, AsyncioTask>,
}

impl AsyncioContextInner {
    pub fn new() -> Self {
        Self {
            seq_id: AtomicU32::new(1),
            tasks: HashMap::new(),
        }
    }
    pub fn set_finished(&mut self, handle: u32) {
        let task = self.tasks.get_mut(&handle);
        if let Some(task) = task {
            if task.status == AsyncioTaskStatus::Pending {
                task.status = AsyncioTaskStatus::Finished;
            }
        }
    }
    pub fn set_canceled(&mut self, handle: u32) {
        let task = self.tasks.get_mut(&handle);
        if let Some(task) = task {
            if task.status == AsyncioTaskStatus::Pending {
                task.status = AsyncioTaskStatus::Canceled;
            }
        }
    }
    pub fn is_job_pending(&self) -> bool {
        self.tasks
            .iter()
            .any(|(_, task)| task.status == AsyncioTaskStatus::Pending)
    }
    fn _get(&self, handle: u32) -> Option<AsyncioTask> {
        self.tasks.get(&handle).cloned()
    }
}

pub struct AsyncioContext {
    inner: Arc<Mutex<AsyncioContextInner>>,
}

impl AsyncioContext {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(Mutex::new(AsyncioContextInner::new())),
        }
    }
    async fn _get(&self, handle: u32) -> Option<AsyncioTask> {
        self.inner.lock().await._get(handle)
    }
}

#[async_trait::async_trait]
impl asyncio::Host for AsyncioContext {
    async fn new_task(&mut self) -> Result<AsyncHandle, ()> {
        let mut inner = self.inner.lock().await;
        let seq_id = inner
            .seq_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let task = AsyncioTask {
            _handle: seq_id,
            status: AsyncioTaskStatus::Pending,
        };
        println!("new_task: {}", seq_id);
        inner.tasks.insert(seq_id, task);
        Ok(seq_id)
    }

    async fn finish(&mut self, handle: AsyncHandle) {
        self.inner.lock().await.set_finished(handle);
    }
    
    async fn sleep(&mut self, ms: u32) -> Result<AsyncHandle, ()> {
        let mut inner = self.inner.lock().await;
        let seq_id = inner
            .seq_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let task = AsyncioTask {
            _handle: seq_id,
            status: AsyncioTaskStatus::Pending,
        };
        println!("sleep: {}, handle: {}", ms, seq_id);
        inner.tasks.insert(seq_id, task);

        let inner2 = self.inner.clone();
        tokio::task::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(ms as u64)).await;
            inner2.lock().await.set_finished(seq_id);
            println!("sleep finished: {}, handle: {}", ms, seq_id);
        });
        Ok(seq_id)
    }

    async fn cancel(&mut self, handle: AsyncHandle) {
        let mut inner = self.inner.lock().await;
        inner.set_canceled(handle);
    }

    async fn is_job_pending(&mut self) -> bool {
        let inner = self.inner.lock().await;
        inner.is_job_pending()
    }

    async fn execute_job(&mut self) -> bool {
        true
    }
}

#[async_trait::async_trait]
impl asyncio::Host for HostContext {
    async fn new_task(&mut self) -> Result<AsyncHandle, ()> {
        self.asyncio_ctx.new_task().await
    }
    async fn finish(&mut self, handle: AsyncHandle) {
        self.asyncio_ctx.finish(handle).await;
    }
    async fn sleep(&mut self, ms: u32) -> Result<AsyncHandle, ()> {
        self.asyncio_ctx.sleep(ms).await
    }

    async fn cancel(&mut self, handle: AsyncHandle) {
        self.asyncio_ctx.cancel(handle).await;
    }

    async fn is_job_pending(&mut self) -> bool {
        self.asyncio_ctx.is_job_pending().await
    }

    async fn execute_job(&mut self) -> bool {
        self.asyncio_ctx.execute_job().await
    }
}

#[cfg(test)]
mod asyncio_test {
    use crate::hostcall::{
        asyncio::{AsyncioContext, AsyncioTaskStatus},
        host::land::asyncio::asyncio::Host,
    };

    #[tokio::test]
    async fn test_sleep() {
        let mut ctx = AsyncioContext::new();
        let handle = ctx.sleep(1000).await.unwrap();
        assert_eq!(handle, 1);

        // wait for 1.2s
        assert!(ctx.is_job_pending().await);

        tokio::time::sleep(tokio::time::Duration::from_millis(1200)).await;
        let task = ctx._get(handle).await.unwrap();
        assert_eq!(task.status, AsyncioTaskStatus::Finished);

        // nothing is pending
        assert!(!ctx.is_job_pending().await);
    }

    #[tokio::test]
    async fn test_cancel() {
        let mut ctx = AsyncioContext::new();
        let handle = ctx.sleep(1000).await.unwrap();
        assert_eq!(handle, 1);
        // call cancel to set task status to canceled
        ctx.cancel(handle).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(1200)).await;
        let task = ctx._get(handle).await.unwrap();
        assert_eq!(task.status, AsyncioTaskStatus::Canceled);
    }
}
