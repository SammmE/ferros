use alloc::boxed::Box;
use alloc::collections::BTreeMap;
use alloc::sync::Arc;
use core::fmt;
use core::sync::atomic::{AtomicU64, Ordering};
use core::{future::Future, pin::Pin};
use spin::Mutex;

pub mod executor;
pub mod keyboard;

pub trait KernelObject: Send + Sync {
    fn read(&self, buf: &mut [u8]) -> usize {
        0
    }
    fn write(&self, buf: &[u8]) -> usize {
        0
    }
}

pub struct Task {
    pub id: TaskId,
    pub future: Mutex<Pin<Box<dyn Future<Output = ()> + Send + 'static>>>,

    pub fd_table: Mutex<BTreeMap<usize, Arc<dyn KernelObject>>>,
}

impl Task {
    pub fn new(future: impl Future<Output = ()> + Send + 'static) -> Task {
        Task {
            id: TaskId::new(),
            future: Mutex::new(Box::pin(future)),
            fd_table: Mutex::new(BTreeMap::new()),
        }
    }

    pub fn allocate_fd(&self, object: Arc<dyn KernelObject>) -> usize {
        let mut table = self.fd_table.lock();

        let mut fd = 0;
        while table.contains_key(&fd) {
            fd += 1;
        }

        table.insert(fd, object);
        fd
    }

    pub fn get_fd(&self, fd: usize) -> Option<Arc<dyn KernelObject>> {
        let table = self.fd_table.lock();
        table.get(&fd).cloned()
    }

    pub fn close_fd(&self, fd: usize) {
        let mut table = self.fd_table.lock();
        table.remove(&fd);
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
