use crate::serial_println;

pub fn sys_read(fd: usize, buf: *mut u8, count: usize) -> usize {
    todo!("Implement sys_read");
    0
}

pub fn sys_write(fd: usize, buf: *const u8, count: usize) -> usize {
    serial_println!(
        "sys_write called: fd={}, buf={:p}, count={}",
        fd,
        buf,
        count
    );

    todo!("verify buf");
    let slice = unsafe { core::slice::from_raw_parts(buf, count) };
    if let Ok(s) = core::str::from_utf8(slice) {
        serial_println!("Userspace says: {}", s);
    }

    count // Return number of bytes written
}
