use super::surface::Surface;
use super::types::{Color, Size};
use alloc::vec;
use alloc::vec::Vec;

pub struct Bitmap {
    pub width: u32,
    pub height: u32,
    pub buffer: Vec<u8>,
}

impl Bitmap {
    pub fn new(width: u32, height: u32) -> Self {
        let size = (width * height * 4) as usize;
        Self {
            width,
            height,
            buffer: vec![0u8; size],
        }
    }

    pub fn buffer_as_slice(&self) -> &[u8] {
        &self.buffer
    }
}

impl Surface for Bitmap {
    fn size(&self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    unsafe fn get_pixel_unchecked(&self, x: u32, y: u32) -> Color {
        let offset = (y * self.width + x) as usize * 4;
        Color::with_alpha(
            self.buffer[offset],
            self.buffer[offset + 1],
            self.buffer[offset + 2],
            self.buffer[offset + 3],
        )
    }

    unsafe fn set_pixel_unchecked(&mut self, x: u32, y: u32, color: Color) {
        let offset = (y * self.width + x) as usize * 4;
        self.buffer[offset] = color.r;
        self.buffer[offset + 1] = color.g;
        self.buffer[offset + 2] = color.b;
        self.buffer[offset + 3] = color.a;
    }
}
