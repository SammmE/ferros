// kernel/src/main.rs
#![no_std]
#![no_main]

extern crate alloc;

use bootloader_api::{BootInfo, BootloaderConfig, config::Mapping, entry_point};
use kernel::graphics::device::DISPLAY;
use kernel::graphics::types::{Color, Point, Rect};
use kernel::init_all;

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    init_all(boot_info);

    {
        let mut display_guard = DISPLAY.lock();
        if let Some(display) = display_guard.as_mut() {
            display.clear(Color::new(0, 50, 80));

            let mut renderer = display.get_renderer();

            let window_rect = Rect::new(100, 100, 400, 300);
            renderer.fill_rect(window_rect, Color::WHITE);

            renderer.fill_rect(Rect::new(100, 100, 400, 30), Color::BLUE);

            renderer.fill_rect(Rect::new(150, 180, 50, 50), Color::RED);

            renderer.set_clip_rect(window_rect);
            renderer.fill_rect(Rect::new(0, 0, 50, 50), Color::GREEN);

            display.present();
        }
    }

    loop {
        x86_64::instructions::hlt();
    }
}
