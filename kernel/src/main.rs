#![no_std]
#![no_main]

use bootloader_api::{BootInfo, entry_point};
use core::fmt::Write;
use font8x8::{BASIC_FONTS, UnicodeFonts};
use x86_64::instructions::hlt;

use kernel::serial::{QemuExitCode, exit_qemu, serial};

entry_point!(kernel_main);

fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    if let Some(framebuffer) = boot_info.framebuffer.as_mut() {
        let info = framebuffer.info();
        let bytes_per_pixel = info.bytes_per_pixel;
        let stride = info.stride;
        let buffer = framebuffer.buffer_mut();

        for pixel in buffer.chunks_exact_mut(bytes_per_pixel) {
            pixel[0] = 0; // Blueish
            pixel[1] = 0; // Greenish
            pixel[2] = 0; // Reddish
        }

        let message = "Hello World!";
        let mut x_pos = 100; // Start 100 pixels from the left
        let y_pos = 100; // Start 100 pixels from the top

        for char in message.chars() {
            // Draw the character at the current position
            draw_char(buffer, stride, bytes_per_pixel, x_pos, y_pos, char);

            // Move the cursor 8 pixels to the right for the next letter
            x_pos += 8;
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

/// Draws a single character to the framebuffer
fn draw_char(
    buffer: &mut [u8],
    stride: usize,
    bytes_per_pixel: usize,
    x: usize,
    y: usize,
    char: char,
) {
    if let Some(bitmap) = BASIC_FONTS.get(char) {
        for (row_i, row_byte) in bitmap.iter().enumerate() {
            for col_i in 0..8 {
                if *row_byte & (1 << col_i) != 0 {
                    let pixel_index = ((y + row_i) * stride + (x + col_i)) * bytes_per_pixel;

                    if pixel_index + 2 < buffer.len() {
                        buffer[pixel_index] = 255; // Blue
                        buffer[pixel_index + 1] = 255; // Green
                        buffer[pixel_index + 2] = 255; // Red
                    }
                }
            }
        }
    }
}
