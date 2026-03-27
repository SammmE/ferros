use core::slice;

use crate::{process::scheduler::SCHEDULER, serial_println};

#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct PollFd {
    pub fd: i32,
    pub events: i16,
    pub revents: i16,
}

// Standard POSIX event flags
pub const POLLIN: i16 = 0x001; // There is data to read
pub const POLLOUT: i16 = 0x004; // Writing is now possible
pub const POLLERR: i16 = 0x008; // Error condition
pub const POLLHUP: i16 = 0x010; // Hung up (pipe closed)

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

pub fn sys_open(path_ptr: *const u8, path_len: usize, flags: usize) -> usize {
    todo!("Implement sys_open");
}

pub fn sys_close(fd: usize) -> usize {
    todo!("Implement sys_close");
}

pub fn sys_pipe(fd_array_ptr: *mut usize) -> usize {
    todo!("implement sys_pipe");
}

pub fn sys_poll(fds_ptr: *mut PollFd, nfds: usize, timeout: usize) -> usize {
    let fds = unsafe { core::slice::from_raw_parts_mut(fds_ptr, nfds) };
    // clear events
    for poll_fd in fds.iter_mut() {
        poll_fd.revents = 0;
    }

    todo!("Implement sys_pol");
}
