#![no_std]
#![no_main]

extern crate alloc;

use bootloader_api::{BootInfo, BootloaderConfig, config::Mapping, entry_point};
use x86_64::VirtAddr;
use x86_64::structures::paging::{FrameAllocator, Mapper, Page, PageTableFlags, Size4KiB};

use kernel::allocator;
use kernel::framebuffer::{self, WRITER};
use kernel::init_all;
use kernel::memory::{self, BootInfoFrameAllocator};
use kernel::println;
use kernel::serial_println;
use kernel::shell;
use kernel::task::{Task, executor::Executor};

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    serial_println!("Kernel initialized successfully!\n");
    init_all();
    serial_println!("IDT initialized.\n");

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    // Initialize Framebuffer
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let info = framebuffer.info();
        let buffer = framebuffer.buffer_mut();
        let mut writer = WRITER.lock();
        *writer = Some(framebuffer::FrameBufferWriter::new(buffer, info));
    }

    println!("Hello World from the Framebuffer!");

    // --- RUN SHELL FIRST ---
    let executor = Executor::new();
    executor.spawn(Task::new(shell::runshell()));

    // Note: The executor.run() below will loop forever.
    executor.run();
}
