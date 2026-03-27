use crate::process::scheduler::Scheduler;
use crate::serial_println;

pub fn sys_exit(code: usize) -> usize {
    Scheduler::exit_current();
}
