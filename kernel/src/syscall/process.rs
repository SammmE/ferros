use crate::process::elf::load_elf;
use crate::process::scheduler::{SCHEDULER, Scheduler};
use crate::serial_println;
use core::str;

pub fn sys_exit(code: usize) -> usize {
    Scheduler::exit_current();
}

pub fn sys_spawn(path_ptr: *const u8, path_len: usize) -> usize {
    let path_slice = unsafe { core::slice::from_raw_parts(path_ptr, path_len) };
    let filename = match str::from_utf8(path_slice) {
        Ok(s) => s,
        Err(_) => {
            serial_println!("sys_spawn error: Invalid UTF-8 in path");
            return usize::MAX; // Return -1 for error
        }
    };

    let inherited_fds = {
        let scheduler = SCHEDULER.lock();
        let current_pid = scheduler
            .current_process
            .expect("sys_spawn without active process");
        let process = scheduler.processes.get(&current_pid).unwrap();

        let fd_table = process.fd_table.lock();
        fd_table.clone()
    }; // Lock dropped here

    match load_elf(filename, inherited_fds) {
        Ok(new_process) => {
            let pid = new_process.id.as_u64() as usize;
            serial_println!("Spawned new process '{}' with PID {}", filename, pid);

            Scheduler::spawn(new_process);

            pid // Return the new Process ID
        }
        Err(e) => {
            serial_println!("sys_spawn error: Failed to load '{}': {}", filename, e);
            usize::MAX
        }
    }
}

pub fn sys_wait(pid: usize, status_ptr: *mut usize) -> usize {
    todo!("implement sys_wait")
}

pub fn sys_yield() -> usize {
    todo!("implement sys_yield")
}
