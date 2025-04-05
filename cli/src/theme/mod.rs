pub mod theme_mgr;

use getset::Getters;
use serde::{Deserialize, Serialize};
use tmui::prelude::Color;

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Getters)]
pub struct Theme {
    #[getset(get = "pub")]
    name: String,
    #[getset(get = "pub")]
    foreground: String,
    #[getset(get = "pub")]
    background: String,
    #[getset(get = "pub")]
    black: String,
    #[getset(get = "pub")]
    red: String,
    #[getset(get = "pub")]
    green: String,
    #[getset(get = "pub")]
    yellow: String,
    #[getset(get = "pub")]
    blue: String,
    #[getset(get = "pub")]
    magenta: String,
    #[getset(get = "pub")]
    cyan: String,
    #[getset(get = "pub")]
    white: String,

    #[getset(get = "pub")]
    bright_foreground: String,
    #[getset(get = "pub")]
    bright_background: String,
    #[getset(get = "pub")]
    bright_black: String,
    #[getset(get = "pub")]
    bright_red: String,
    #[getset(get = "pub")]
    bright_green: String,
    #[getset(get = "pub")]
    bright_yellow: String,
    #[getset(get = "pub")]
    bright_blue: String,
    #[getset(get = "pub")]
    bright_magenta: String,
    #[getset(get = "pub")]
    bright_cyan: String,
    #[getset(get = "pub")]
    bright_white: String,
}

impl Theme {
    #[inline]
    pub fn background_color(&self) -> Color {
        Color::hex(&self.background)
    }

    #[inline]
    pub fn foreground_color(&self) -> Color {
        Color::hex(&self.foreground)
    }
}
