use alloc::collections::{btree_map::BTreeMap, vec_deque::VecDeque};
use lazy_static::lazy_static;
use spin::Mutex;

use crate::process::{Process, ProcessId, State};

lazy_static! {
    pub static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
}

pub struct Scheduler {
    pub processes: BTreeMap<ProcessId, Process>,
    pub ready_queue: VecDeque<ProcessId>,
    pub current_process: Option<ProcessId>,
}

impl Scheduler {
    pub fn new() -> Self {
        Scheduler {
            processes: BTreeMap::new(),
            ready_queue: VecDeque::new(),
            current_process: None,
        }
    }

    pub fn spawn(process: Process) {
        let mut scheduler = SCHEDULER.lock();
        let pid = process.id;
        scheduler.processes.insert(pid, process);
        scheduler.ready_queue.push_back(pid);
    }

    pub fn schedule_next(old_stack_ptr: u64) -> u64 {
        let mut scheduler = SCHEDULER.lock();

        if let Some(current_id) = scheduler.current_process {
            let current_process = scheduler.processes.get_mut(&current_id).unwrap();

            current_process.saved_rsp = old_stack_ptr; // SAVED TO PCB

            if matches!(current_process.state, State::Running) {
                current_process.state = State::Ready;
                scheduler.ready_queue.push_back(current_id);
            }
        }

        let next_id = scheduler
            .ready_queue
            .pop_front()
            .expect("No processes to run! System idle.");

        scheduler.current_process = Some(next_id);

        let next_process = scheduler.processes.get_mut(&next_id).unwrap();
        next_process.state = State::Running;

        unsafe {
            crate::memory::switch_address_space(next_process.page_table);
        }

        next_process.saved_rsp // RETURN TO ASSEMBLY
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn process_schedule_next(old_stack_ptr: u64) -> u64 {
    unsafe {
        crate::interrupts::PICS
            .lock()
            .notify_end_of_interrupt(crate::interrupts::InterruptIndex::Timer.as_u8());
    }

    Scheduler::schedule_next(old_stack_ptr)
}
