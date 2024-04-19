#![allow(dead_code)]
use once_cell::sync::Lazy;
use tmui::prelude::Font;

pub struct Config {
    font: Font,
}

#[inline]
fn instance() -> &'static mut Config {
    static mut CONFIG: Lazy<Config> = Lazy::new(|| Config::new());
    unsafe { &mut CONFIG }
}

impl Config {
    #[inline]
    fn new() -> Self {
        Config { 
            font: Font::with_families(&["Courier New"]),
        }
    }

    #[inline]
    pub fn font() -> Font {
        instance().font.clone()
    }
    #[inline]
    pub fn set_font(font: Font) {
        instance().font = font
    }
}
