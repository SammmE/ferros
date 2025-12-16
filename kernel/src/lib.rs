#![no_std]
#![feature(abi_x86_interrupt)]

extern crate alloc;

pub mod allocator;
pub mod framebuffer;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod panic;
pub mod serial;

pub fn init_all() {
    gdt::init();
    interrupts::init_idt();

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

    interrupts::init_pit();
    x86_64::instructions::interrupts::enable();

    // Keep the int3 here for now to be safe!
    x86_64::instructions::interrupts::int3();

    serial_println!("All systems initialized.");
}
