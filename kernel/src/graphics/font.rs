use font8x8::{BASIC_FONTS, UnicodeFonts};

pub trait Font {
    fn get_glyph(&self, c: char) -> Option<[u8; 8]>;
    fn width(&self) -> u32 {
        8
    }
    fn height(&self) -> u32 {
        8
    }
}

pub struct BasicFont;

impl Font for BasicFont {
    fn get_glyph(&self, c: char) -> Option<[u8; 8]> {
        BASIC_FONTS.get(c)
    }
}
