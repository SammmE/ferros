use super::types::{Color, Size};

pub trait Surface {
    /// Returns the dimensions of the surface
    fn size(&self) -> Size;

    /// Get a pixel (Unsafe for performance, implementations may skip bounds checks)
    unsafe fn get_pixel_unchecked(&self, x: u32, y: u32) -> Color;

    /// Set a pixel (Unsafe for performance, implementations may skip bounds checks)
    unsafe fn set_pixel_unchecked(&mut self, x: u32, y: u32, color: Color);

    /// Fill the entire surface with a color
    fn clear(&mut self, color: Color) {
        let size = self.size();
        for y in 0..size.height {
            for x in 0..size.width {
                unsafe { self.set_pixel_unchecked(x, y, color) };
            }
        }
    }
}
