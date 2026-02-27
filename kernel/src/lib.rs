#![no_std]
#![feature(abi_x86_interrupt)]

extern crate alloc;

use bootloader_api::BootInfo;
use x86_64::VirtAddr;

pub mod allocator;
pub mod drivers;
pub mod fs;
pub mod gdt;
pub mod graphics;
pub mod interrupts;
pub mod memory;
pub mod panic;
pub mod process;
pub mod serial;
pub mod syscall;
pub mod task;

pub fn init_all(boot_info: &'static mut BootInfo) {
    gdt::init();
    serial_println!("[INIT] GDT initialized.");

    interrupts::init_idt();
    serial_println!("[INIT] IDT initialized.");

    interrupts::init_pics();
    serial_println!("[INIT] PICs initialized.");

    interrupts::init_pit();
    serial_println!("[INIT] PIT initialized.");

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    memory::init(phys_mem_offset, &boot_info.memory_regions);

    let mut mapper = memory::get_mapper().expect("VMM not ready");
    allocator::init_heap(&mut mapper).expect("heap initialization failed");
    serial_println!("[INIT] Memory & Heap initialized.");

    memory::unmap_null_page().expect("Failed to unmap null page");
    serial_println!("[INIT] Null page unmapped for safety.");

    x86_64::instructions::interrupts::enable();
    serial_println!("[INIT] Interrupts enabled.");

    syscall::init_syscall();
    serial_println!("[INIT] Syscalls initialized.");

    fs::init_fs();
    serial_println!("[INIT] Filesystem initialized.");

    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let info = framebuffer.info();
        let buffer = framebuffer.buffer_mut();
        graphics::device::init_display(info, buffer);
        serial_println!("[INIT] Graphics Subsystem initialized.");
    }
    serial_println!("[INIT] Framebuffer initialized.");

    serial_println!("All systems initialized.");
}
