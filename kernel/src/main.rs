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

    // Animation State
    let mut x_pos = 100;
    let mut y_pos = 100;
    let mut x_vel = 5;
    let mut y_vel = 5;
    let rect_size = 50;

    loop {
        {
            let mut display_guard = DISPLAY.lock();
            if let Some(display) = display_guard.as_mut() {
                let screen_width = display.width() as i32;
                let screen_height = display.height() as i32;

                display.clear(Color::new(0, 50, 80));

                x_pos += x_vel;
                y_pos += y_vel;

                if x_pos + rect_size >= screen_width || x_pos <= 0 {
                    x_vel = -x_vel;
                }
                if y_pos + rect_size >= screen_height || y_pos <= 0 {
                    y_vel = -y_vel;
                }

                let mut renderer = display.get_renderer();

                renderer.fill_rect(
                    Rect::new(x_pos, y_pos, rect_size as u32, rect_size as u32),
                    Color::RED,
                );

                display.present();
            }
        }

        for _ in 0..5_000_000 {
            core::hint::spin_loop();
        }
    }
}
