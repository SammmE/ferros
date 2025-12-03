#![no_std]
#![no_main]

use bootloader_api::{BootInfo, entry_point};
use core::fmt::Write;
use x86_64::instructions::{port::Port, hlt};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) -> ! {
    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }

    loop {
        hlt();
    }
}

pub fn serial() -> uart_16550::SerialPort {
    let mut serial_port = unsafe { uart_16550::SerialPort::new(0x3F8) };
    serial_port.init();
    serial_port
}

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    let mut port = serial();
    writeln!(port, "Boot info: {boot_info:?}").ok();
    writeln!(port, "hello, world").ok();

    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let info = framebuffer.info();

        for pixel in framebuffer.buffer_mut().chunks_exact_mut(info.bytes_per_pixel) {
            pixel[0] = 255;
            pixel[1] = 0;
            pixel[2] = 0;
        }
    }

    loop {
        hlt();
    }
}

#[panic_handler]
#[cfg(not(test))]
fn panic(info: &core::panic::PanicInfo) -> ! {
    let _ = writeln!(serial(), "PANIC: {info}");
    exit_qemu(QemuExitCode::Failed);
}
