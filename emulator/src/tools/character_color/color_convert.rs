use super::{ColorEntry, FontWeight, TABLE_COLORS};
use cli::scheme::ColorScheme;
use tmui::prelude::Color;

pub trait ColorConvert {
    fn convert_entry(&self) -> [ColorEntry; TABLE_COLORS];
}

impl ColorConvert for ColorScheme {
    fn convert_entry(&self) -> [ColorEntry; TABLE_COLORS] {
        [
            // Normal colors:
            ColorEntry {
                color: Color::hex(self.foreground()),
                transparent: true,
                font_weight: FontWeight::UseCurrentFormat,
            },
            ColorEntry {
                color: Color::hex(self.background()),
                transparent: true,
                font_weight: FontWeight::UseCurrentFormat,
            },
            ColorEntry {
                color: Color::hex(self.black()),
                transparent: false,
                font_weight: FontWeight::UseCurrentFormat,
            },
            ColorEntry {
                color: Color::hex(self.red()),
                transparent: false,
                font_weight: FontWeight::UseCurrentFormat,
            },
            ColorEntry {
                color: Color::hex(self.green()),
                transparent: false,
                font_weight: FontWeight::UseCurrentFormat,
            },
            ColorEntry {
                color: Color::hex(self.yellow()),
                transparent: false,
                font_weight: FontWeight::UseCurrentFormat,
            },
            ColorEntry {
                color: Color::hex(self.blue()),
                transparent: false,
                font_weight: FontWeight::UseCurrentFormat,
            },
            ColorEntry {
                color: Color::hex(self.magenta()),
                transparent: false,
                font_weight: FontWeight::UseCurrentFormat,
            },
            ColorEntry {
                color: Color::hex(self.cyan()),
                transparent: false,
                font_weight: FontWeight::UseCurrentFormat,
            },
            ColorEntry {
                color: Color::hex(self.white()),
                transparent: false,
                font_weight: FontWeight::UseCurrentFormat,
            },
            // Intensive color:
            ColorEntry {
                color: Color::hex(self.bright_foreground()),
                transparent: true,
                font_weight: FontWeight::UseCurrentFormat,
            },
            ColorEntry {
                color: Color::hex(self.bright_background()),
                transparent: true,
                font_weight: FontWeight::UseCurrentFormat,
            },
            ColorEntry {
                color: Color::hex(self.bright_black()),
                transparent: false,
                font_weight: FontWeight::UseCurrentFormat,
            },
            ColorEntry {
                color: Color::hex(self.bright_red()),
                transparent: false,
                font_weight: FontWeight::UseCurrentFormat,
            },
            ColorEntry {
                color: Color::hex(self.bright_green()),
                transparent: false,
                font_weight: FontWeight::UseCurrentFormat,
            },
            ColorEntry {
                color: Color::hex(self.bright_yellow()),
                transparent: false,
                font_weight: FontWeight::UseCurrentFormat,
            },
            ColorEntry {
                color: Color::hex(self.bright_blue()),
                transparent: false,
                font_weight: FontWeight::UseCurrentFormat,
            },
            ColorEntry {
                color: Color::hex(self.bright_magenta()),
                transparent: false,
                font_weight: FontWeight::UseCurrentFormat,
            },
            ColorEntry {
                color: Color::hex(self.bright_cyan()),
                transparent: false,
                font_weight: FontWeight::UseCurrentFormat,
            },
            ColorEntry {
                color: Color::hex(self.bright_white()),
                transparent: false,
                font_weight: FontWeight::UseCurrentFormat,
            },
        ]
    }
}
