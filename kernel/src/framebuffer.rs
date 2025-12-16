use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use core::fmt;
use font8x8::{BASIC_FONTS, UnicodeFonts};
use spin::Mutex;

// We need a global lock so we can print from interrupts/threads safely
pub static WRITER: Mutex<Option<FrameBufferWriter>> = Mutex::new(None);

pub struct FrameBufferWriter {
    buffer: &'static mut [u8],
    info: FrameBufferInfo,
    x_pos: usize,
    y_pos: usize,
}

impl FrameBufferWriter {
    pub fn new(buffer: &'static mut [u8], info: FrameBufferInfo) -> Self {
        let mut writer = Self {
            buffer,
            info,
            x_pos: 0,
            y_pos: 0,
        };
        writer.clear();
        writer
    }

    pub fn clear(&mut self) {
        self.x_pos = 0;
        self.y_pos = 0;
        self.buffer.fill(0);
    }

    fn newline(&mut self) {
        self.x_pos = 0;
        self.y_pos += 8; // Move down 8 pixels (font height)
    }

    fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            _ => {
                // If we are at the edge of the screen, move to next line
                if self.x_pos >= self.info.width {
                    self.newline();
                }

                // If we hit the bottom, for now just clear and restart (simple scrolling)
                if self.y_pos >= self.info.height {
                    self.clear();
                }

                let x = self.x_pos;
                let y = self.y_pos;

                // Draw the character using font8x8
                if let Some(bitmap) = BASIC_FONTS.get(c) {
                    for (row_i, row_byte) in bitmap.iter().enumerate() {
                        for col_i in 0..8 {
                            if *row_byte & (1 << col_i) != 0 {
                                self.write_pixel(x + col_i, y + row_i, 255, 255, 255); // White
                            }
                        }
                    }
                }
                self.x_pos += 8;
            }
        }
    }

    fn write_pixel(&mut self, x: usize, y: usize, r: u8, g: u8, b: u8) {
        let pixel_offset = y * self.info.stride + x;
        let color = match self.info.pixel_format {
            PixelFormat::Rgb => [r, g, b, 0],
            PixelFormat::Bgr => [b, g, r, 0],
            PixelFormat::U8 => [if r > 128 { 0xff } else { 0 }, 0, 0, 0], // Greyscale fallback
            other => panic!("pixel format {:?} not supported in logger", other),
        };

        let bytes_per_pixel = self.info.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_per_pixel;

        // Bounds check
        if byte_offset + (bytes_per_pixel - 1) < self.buffer.len() {
            self.buffer[byte_offset..(byte_offset + bytes_per_pixel)]
                .copy_from_slice(&color[..bytes_per_pixel]);
        }
    }
}

// Implement fmt::Write so we can use the `write!` macro
impl fmt::Write for FrameBufferWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}

// Global Macros
#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::framebuffer::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    use x86_64::instructions::interrupts;

    // Disable interrupts to avoid deadlock if an interrupt tries to print
    interrupts::without_interrupts(|| {
        if let Some(writer) = WRITER.lock().as_mut() {
            writer.write_fmt(args).unwrap();
        }
    });
}
