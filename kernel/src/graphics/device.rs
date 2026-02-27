use super::buffer::Bitmap;
use super::renderer::Renderer;
use super::types::Color;
use bootloader_api::info::{FrameBufferInfo, PixelFormat};
use spin::Mutex;

/// Global wrapper for the screen driver.
/// We use an Option because it's initialized at runtime.
pub static DISPLAY: Mutex<Option<DisplayDevice>> = Mutex::new(None);

pub struct DisplayDevice {
    info: FrameBufferInfo,
    framebuffer: &'static mut [u8], // VRAM (Write-only mostly)
    backbuffer: Bitmap,             // RAM (Read-Write)
}

impl DisplayDevice {
    pub fn new(info: FrameBufferInfo, framebuffer: &'static mut [u8]) -> Self {
        let width = info.width as u32;
        let height = info.height as u32;

        Self {
            info,
            framebuffer,
            backbuffer: Bitmap::new(width, height),
        }
    }

    /// Returns a renderer that draws to the backbuffer (RAM).
    /// Drawing here is fast and safe.
    pub fn get_renderer(&mut self) -> Renderer {
        Renderer::new(&mut self.backbuffer)
    }

    /// Flushes the backbuffer to VRAM.
    /// This converts the RGBA RAM buffer to the hardware specific format (BGR/RGB).
    pub fn present(&mut self) {
        let width = self.info.width;
        let height = self.info.height;
        let bytes_per_pixel = self.info.bytes_per_pixel;
        let stride = self.info.stride;
        let format = self.info.pixel_format;

        let ram_buffer = self.backbuffer.buffer_as_slice();

        for y in 0..height {
            let row_start_vram = y * stride;
            let row_start_ram = y * width; // RAM buffer is tightly packed

            for x in 0..width {
                let ram_offset = (row_start_ram + x) * 4; // 4 bytes per pixel in RAM (RGBA)
                let vram_offset = (row_start_vram + x) * bytes_per_pixel;

                let r = ram_buffer[ram_offset];
                let g = ram_buffer[ram_offset + 1];
                let b = ram_buffer[ram_offset + 2];
                // let a = ram_buffer[ram_offset + 3]; // Alpha not used for opaque screen

                // Hardware specific pixel packing
                match format {
                    PixelFormat::Rgb => {
                        self.framebuffer[vram_offset] = r;
                        self.framebuffer[vram_offset + 1] = g;
                        self.framebuffer[vram_offset + 2] = b;
                    }
                    PixelFormat::Bgr => {
                        self.framebuffer[vram_offset] = b;
                        self.framebuffer[vram_offset + 1] = g;
                        self.framebuffer[vram_offset + 2] = r;
                    }
                    PixelFormat::U8 => {
                        // Grayscale fallback
                        let gray = ((r as u16 + g as u16 + b as u16) / 3) as u8;
                        self.framebuffer[vram_offset] = gray;
                    }
                    _ => panic!("Unsupported pixel format"),
                }
            }
        }
    }

    pub fn clear(&mut self, color: Color) {
        // Extract dimensions first to avoid conflict with mutable borrow below
        let width = self.info.width as u32;
        let height = self.info.height as u32;

        self.get_renderer()
            .fill_rect(super::types::Rect::new(0, 0, width, height), color);
    }
}

/// Helper to initialize the global display
pub fn init_display(info: FrameBufferInfo, framebuffer: &'static mut [u8]) {
    let mut display = DISPLAY.lock();
    *display = Some(DisplayDevice::new(info, framebuffer));
}
