use super::Task;
use alloc::{sync::Arc, task::Wake};
use core::task::{Context, Poll, Waker};
use crossbeam_queue::ArrayQueue;
use lazy_static::lazy_static;
use x86_64::instructions::interrupts::{self, enable_and_hlt};

const MAX_TASKS: usize = 100;

lazy_static! {
    static ref TASK_QUEUE: ArrayQueue<Arc<Task>> = ArrayQueue::new(MAX_TASKS);
}

pub struct Executor;

impl Executor {
    pub fn new() -> Self {
        Self
    }

    pub fn spawn(&self, task: Task) {
        let task = Arc::new(task);
        TASK_QUEUE.push(task).expect("Task queue full");
    }

    pub fn run(&self) -> ! {
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    fn run_ready_tasks(&self) {
        while let Some(task) = TASK_QUEUE.pop() {
            let mut future_slot = task.future.lock();

            let waker = Waker::from(task.clone());
            let mut context = Context::from_waker(&waker);

            match future_slot.as_mut().poll(&mut context) {
                Poll::Ready(()) => {}
                Poll::Pending => {}
            }
        }
    }

    fn sleep_if_idle(&self) {
        interrupts::disable();

        if TASK_QUEUE.is_empty() {
            enable_and_hlt();
        } else {
            interrupts::enable();
        }
    }
}

impl Wake for Task {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }

    fn wake_by_ref(self: &Arc<Self>) {
        self.clone().wake_task();
    }
}

impl Task {
    fn wake_task(self: Arc<Self>) {
        TASK_QUEUE.push(self).expect("Task queue full");
    }
}
