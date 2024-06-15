#![allow(dead_code)]
use std::ptr::addr_of_mut;

use once_cell::sync::Lazy;
use tmui::prelude::Font;

pub struct Config {
    font: Font,
}

#[inline]
fn instance() -> &'static mut Config {
    static mut CONFIG: Lazy<Config> = Lazy::new(Config::new);
    unsafe { addr_of_mut!(CONFIG).as_mut().unwrap() }
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
