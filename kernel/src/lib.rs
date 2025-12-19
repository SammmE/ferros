#![no_std]
#![feature(abi_x86_interrupt)]

extern crate alloc;

pub mod allocator;
pub mod drivers;
pub mod framebuffer;
pub mod fs;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod panic;
pub mod serial;
pub mod shell;
pub mod task;

pub fn init_all() {
    gdt::init();
    println!("[INIT] GDT initialized.");

    interrupts::init_idt();
    println!("[INIT] IDT initialized.");

    unsafe {
        let mut pics = interrupts::PICS.lock();
        pics.initialize();

        // 0xFC = 1111 1100 (Binary)
        // Zeros mean "Enabled". Ones mean "Disabled".
        // Bit 0 (Timer) = 0
        // Bit 1 (Keyboard) = 0
        // All others = 1
        pics.write_masks(0xFC, 0xFF);
        // ---------------------
    }
    println!("[INIT] PICs initialized.");

    interrupts::init_pit();
    x86_64::instructions::interrupts::enable();
    println!("[INIT] PIT initialized and interrupts enabled.");

    // Keep the int3 here for now to be safe!
    x86_64::instructions::interrupts::int3();

    fs::init_fs();
    println!("[INIT] Filesystem initialized.");

    serial_println!("All systems initialized.");
}
