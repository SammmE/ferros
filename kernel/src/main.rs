#![no_std]
#![no_main]

extern crate alloc;

use bootloader_api::{BootInfo, BootloaderConfig, config::Mapping, entry_point};
use font8x8::{BASIC_FONTS, UnicodeFonts};
use futures_util::stream::StreamExt;
use pc_keyboard::{DecodedKey, HandleControl, KeyCode, Keyboard, ScancodeSet1, layouts};
use x86_64::VirtAddr;

use kernel::allocator;
use kernel::framebuffer::{self, WRITER};
use kernel::init_all;
use kernel::memory::{self, BootInfoFrameAllocator};
use kernel::serial_println;
use kernel::task::keyboard::ScancodeStream;
use kernel::task::{Task, executor::Executor};
use kernel::{print, println};

use alloc::{boxed::Box, rc::Rc, vec, vec::Vec};

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

async fn async_number() -> u32 {
    42
}

async fn example_task() {
    let number = async_number().await;
    println!("Async number: {}", number);
}

async fn print_keypresses() {
    let mut scancode_stream = ScancodeStream::new();
    let mut keyboard = Keyboard::new(
        ScancodeSet1::new(),
        layouts::Us104Key,
        HandleControl::Ignore,
    );

    while let Some(scancode) = scancode_stream.next().await {
        if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
            if let Some(key) = keyboard.process_keyevent(key_event) {
                match key {
                    DecodedKey::Unicode(character) => print!("{}", character),
                    DecodedKey::RawKey(KeyCode::Backspace) => print!("\x08"),
                    DecodedKey::RawKey(key) => print!("{:?}", key),
                }
            }
        }
    }
}

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    serial_println!("Kernel initialized successfully!\n");
    init_all();
    serial_println!("IDT initialized.\n");

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset.into_option().unwrap());
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("heap initialization failed");

    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let info = framebuffer.info();
        let buffer = framebuffer.buffer_mut();
        let mut writer = WRITER.lock();
        *writer = Some(framebuffer::FrameBufferWriter::new(buffer, info));
    }

    println!("Hello World from the Framebuffer!");

    if let Some(writer) = WRITER.lock().as_mut() {
        writer.set_color(0, 255, 0); // Green
    }
    println!("This should be green");

    if let Some(writer) = WRITER.lock().as_mut() {
        writer.set_color(255, 255, 255); // Reset to white
    }

    let executor = Executor::new();
    executor.spawn(Task::new(print_keypresses()));
    executor.spawn(Task::new(example_task()));

    executor.run();
}
