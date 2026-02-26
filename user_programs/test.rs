// user_hello.rs
#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let msg = "Hello from a real ELF file!\n";
    let ptr = msg.as_ptr() as u64;
    let len = msg.len() as u64;

    unsafe {
        core::arch::asm!(
            "syscall",
            in("rax") 1,    // Syscall ID 1 (Print)
            in("rdi") ptr,  // Arg1: String Pointer
            in("rsi") len,  // Arg2: String Length
            // We don't care about return values yet
            lateout("rcx") _,
            lateout("r11") _,
        );
    }

    // Loop forever so we don't execute garbage memory after the syscall
    loop {}
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
