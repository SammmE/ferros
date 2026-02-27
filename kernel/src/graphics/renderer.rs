use super::font::Font;
use super::surface::Surface;
use super::types::{Color, Point, Rect};

pub struct Renderer<'a> {
    surface: &'a mut dyn Surface,
    clip_rect: Rect,
}

impl<'a> Renderer<'a> {
    pub fn new(surface: &'a mut dyn Surface) -> Self {
        let size = surface.size();
        Self {
            surface,
            clip_rect: Rect::new(0, 0, size.width, size.height),
        }
    }

    /// Sets the clipping area. Drawing outside this area is ignored.
    pub fn set_clip_rect(&mut self, rect: Rect) {
        let size = self.surface.size();
        let screen_rect = Rect::new(0, 0, size.width, size.height);

        // Intersect requested clip with actual screen bounds for safety
        if let Some(valid_rect) = screen_rect.intersect(&rect) {
            self.clip_rect = valid_rect;
        } else {
            // If no intersection, clip everything (empty rect)
            self.clip_rect = Rect::new(0, 0, 0, 0);
        }
    }

    pub fn draw_pixel(&mut self, p: Point, color: Color) {
        if p.x >= self.clip_rect.x
            && p.y >= self.clip_rect.y
            && p.x < (self.clip_rect.x + self.clip_rect.width as i32)
            && p.y < (self.clip_rect.y + self.clip_rect.height as i32)
        {
            unsafe {
                self.surface
                    .set_pixel_unchecked(p.x as u32, p.y as u32, color);
            }
        }
    }

    pub fn fill_rect(&mut self, rect: Rect, color: Color) {
        // Calculate intersection with clip rect
        if let Some(draw_rect) = self.clip_rect.intersect(&rect) {
            for y in draw_rect.y..(draw_rect.y + draw_rect.height as i32) {
                for x in draw_rect.x..(draw_rect.x + draw_rect.width as i32) {
                    unsafe {
                        self.surface.set_pixel_unchecked(x as u32, y as u32, color);
                    }
                }
            }
        }
    }

    pub fn draw_char(&mut self, pos: Point, c: char, font: &dyn Font, color: Color) {
        if let Some(glyph) = font.get_glyph(c) {
            for (row_i, row_byte) in glyph.iter().enumerate() {
                for col_i in 0..8 {
                    if *row_byte & (1 << col_i) != 0 {
                        self.draw_pixel(Point::new(pos.x + col_i, pos.y + row_i as i32), color);
                    }
                }
            }
        }
    }

    pub fn draw_string(&mut self, mut pos: Point, s: &str, font: &dyn Font, color: Color) {
        let start_x = pos.x;
        for c in s.chars() {
            match c {
                '\n' => {
                    pos.x = start_x;
                    pos.y += font.height() as i32;
                }
                _ => {
                    self.draw_char(pos, c, font, color);
                    pos.x += font.width() as i32;
                }
            }
        }
    }

    /// Copies a source surface onto the destination at `pos`
    pub fn blit(&mut self, source: &dyn Surface, pos: Point) {
        let src_size = source.size();
        let target_rect = Rect::new(pos.x, pos.y, src_size.width, src_size.height);

        // Only iterate over the visible intersection
        if let Some(draw_rect) = self.clip_rect.intersect(&target_rect) {
            for y in draw_rect.y..(draw_rect.y + draw_rect.height as i32) {
                for x in draw_rect.x..(draw_rect.x + draw_rect.width as i32) {
                    // Calculate source coordinates (relative to source origin 0,0)
                    let src_x = (x - pos.x) as u32;
                    let src_y = (y - pos.y) as u32;

                    unsafe {
                        let color = source.get_pixel_unchecked(src_x, src_y);
                        // Simple alpha blending check (0 = transparent)
                        if color.a > 0 {
                            self.surface.set_pixel_unchecked(x as u32, y as u32, color);
                        }
                    }
                }
            }
        }
    }
}
