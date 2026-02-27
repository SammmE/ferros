use super::types::{Color, Size};

pub trait Surface {
    fn size(&self) -> Size;
    unsafe fn get_pixel_unchecked(&self, x: u32, y: u32) -> Color;
    unsafe fn set_pixel_unchecked(&mut self, x: u32, y: u32, color: Color);

    fn clear(&mut self, color: Color) {
        let size = self.size();
        for y in 0..size.height {
            for x in 0..size.width {
                unsafe { self.set_pixel_unchecked(x, y, color) };
            }
        }
    }
}
