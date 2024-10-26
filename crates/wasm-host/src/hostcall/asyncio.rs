use super::host::land::asyncio::asyncio::{self, Handle};
use std::{
    collections::HashMap,
    sync::{atomic::AtomicU32, Arc},
};
use tokio::sync::{Mutex, Notify};
use tracing::debug;

/// Status is task status.
#[derive(Clone, Debug, PartialEq)]
pub enum Status {
    Pending, // ready to run
    // Running,
    // Canceled,
    Finished, // run done
}

/// Task is async task.
#[derive(Clone, Debug)]
struct Task {
    timing: Option<Status>,
    status: Status,
}

impl Task {
    /// is_runnable returns if task is runnable.
    /// if task is pending and timing is not pending, it is not runnable.
    pub fn is_runnable(&self) -> bool {
        if let Some(t) = &self.timing {
            if *t == Status::Pending {
                return false;
            }
        }
        self.status == Status::Pending
    }
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
    /// new_task creates new task and returns handle id
    fn new_task(&mut self) -> Result<Handle, ()> {
        let seq_id = self
            .seq_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let task = Task {
            timing: None,
            status: Status::Pending,
        };
        self.tasks.insert(seq_id, task);
        // println!("asyncio->new_task: {}", seq_id);
        debug!("asyncio->new_task: {}", seq_id);
        Ok(seq_id)
    }
    /// new_sleep creates new sleep task and returns handle id
    async fn new_sleep(&mut self) -> Result<Handle, ()> {
        let seq_id = self
            .seq_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let task = Task {
            timing: Some(Status::Pending),
            status: Status::Pending,
        };
        self.tasks.insert(seq_id, task);
        Ok(seq_id)
    }
    /// timeup sets timing task to finished
    async fn timeup(&mut self, handle: Handle) {
        let task = self.tasks.get_mut(&handle);
        if let Some(task) = task {
            // println!("asyncio->timeup: {}", handle);
            debug!("asyncio->timeup: {}", handle);
            task.timing = Some(Status::Finished);
            self.notify.notify_one();
        }
    }
    /// select_one select one task to run
    async fn select_one(&mut self) -> (Option<Handle>, bool) {
        // all tasks are exeucted
        if self.tasks.is_empty() {
            // println!("asyncio->select_one: all tasks are exeucted");
            debug!("asyncio->select_one: all tasks are exeucted");
            return (None, false);
        }
        let mut runnable_seq_id = 0;
        for (seq_id, task) in self.tasks.iter() {
            if task.is_runnable() {
                runnable_seq_id = *seq_id;
                // println!("asyncio->select_one: runnable_seq_id: {}", runnable_seq_id);
                debug!("asyncio->select_one: runnable_seq_id: {}", runnable_seq_id);
                break;
            }
        }
        // no runnable task, but some tasks are exists, need wait
        if runnable_seq_id == 0 && !self.tasks.is_empty() {
            // println!("asyncio->select_one: wait");
            debug!("asyncio->select_one: wait");
            return (None, true);
        }
        self.tasks.remove(&runnable_seq_id);
        (Some(runnable_seq_id), true)
    }
}

/// Context is asyncio context.
#[derive(Clone, Debug)]
pub struct Context {
    notify: Arc<Notify>, // same nofiy in Inner
    inner: Arc<Mutex<Inner>>,
}

impl Context {
    /// new creates new asyncio context.
    pub fn new() -> Self {
        let notify = Arc::new(Notify::new());
        Self {
            inner: Arc::new(Mutex::new(Inner::new(notify.clone()))),
            notify,
        }
    }
}

#[async_trait::async_trait]
impl asyncio::Host for Context {
    async fn new(&mut self) -> Result<Handle, ()> {
        self.inner.lock().await.new_task()
    }
    async fn sleep(&mut self, ms: u32) -> Result<Handle, ()> {
        let self2 = self.clone();
        let seq_id = self
            .inner
            .lock()
            .await
            .new_sleep()
            .await
            .expect("new_sleep error");
        // println!("asyncio->new_sleep: {}, {}ms", seq_id, ms);
        debug!("asyncio->new_sleep: {}, {}ms", seq_id, ms);
        tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(ms as u64)).await;
            self2.inner.lock().await.timeup(seq_id).await;
        });
        Ok(seq_id)
    }
    async fn select(&mut self) -> (Option<Handle>, bool) {
        self.inner.lock().await.select_one().await
    }
    async fn ready(&mut self) {
        self.notify.notified().await
    }
}
