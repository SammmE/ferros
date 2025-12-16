#![no_std]
#![no_main]

extern crate alloc;

use bootloader_api::{BootInfo, BootloaderConfig, config::Mapping, entry_point};
use font8x8::{BASIC_FONTS, UnicodeFonts};
use x86_64::VirtAddr;
use x86_64::instructions::hlt;

use kernel::allocator;
use kernel::framebuffer::{self, WRITER};
use kernel::init_all;
use kernel::memory::{self, BootInfoFrameAllocator};
use kernel::serial_println;
use kernel::{print, println};

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};

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

    // --- HEAP TEST ---
    let heap_value = Box::new(41);
    serial_println!("Heap value at {:p}", heap_value);

    let mut vec = Vec::new();
    for i in 0..500 {
        vec.push(i);
    }
    serial_println!("Vec at {:p}", vec.as_slice());
    // ------------------------------------

    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let info = framebuffer.info();
        let buffer = framebuffer.buffer_mut();

        // Lock the global writer and initialize it
        let mut writer = WRITER.lock();
        *writer = Some(framebuffer::FrameBufferWriter::new(buffer, info));
    }

    println!("Hello World from the Framebuffer!");
    println!("The heap value is: {:?}", Box::new(42));

    loop {
        hlt();
    }
}
