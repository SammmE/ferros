use alloc::boxed::Box;
use core::fmt;
use core::sync::atomic::{AtomicU64, Ordering};
use core::{future::Future, pin::Pin};
use spin::Mutex;

pub mod executor;
pub mod keyboard;

pub struct Task {
    pub id: TaskId,
    pub future: Mutex<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + Send + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Mutex::new(Box::pin(future)),
        }
    }
}

impl fmt::Debug for Task {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Task").field("id", &self.id).finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TaskId(u64);

impl TaskId {
    fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}
