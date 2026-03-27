use core::slice;

use crate::{process::scheduler::SCHEDULER, serial_println};

pub fn sys_read(fd: usize, buf: *mut u8, count: usize) -> usize {
    let object = {
        let scheduler = SCHEDULER.lock();
        let current_pid = scheduler
            .current_process
            .expect("sys_read called without active process");
        let process = scheduler.processes.get(&current_pid).unwrap();

        let fd_table = process.fd_table.lock();
        match fd_table.get(&fd) {
            Some(obj) => obj.clone(),
            None => {
                serial_println!("sys_read error: Invalid FD {}", fd);
                return usize::MAX; // -1 error code
            }
        }
    };

    let buffer_slice = unsafe { slice::from_raw_parts_mut(buf, count) };
    object.read(buffer_slice)
}

pub fn sys_write(fd: usize, buf: *const u8, count: usize) -> usize {
    let object = {
        let scheduler = SCHEDULER.lock();
        let current_pid = scheduler
            .current_process
            .expect("sys_write called without active process");
        let process = scheduler.processes.get(&current_pid).unwrap();

        let fd_table = process.fd_table.lock();
        match fd_table.get(&fd) {
            Some(obj) => obj.clone(),
            None => {
                serial_println!("sys_write error: Invalid FD {}", fd);
                return usize::MAX; // -1 error code
            }
        }
    }; // <-- Locks dropped here

    let buffer_slice = unsafe { slice::from_raw_parts(buf, count) };

    object.write(buffer_slice)
}
