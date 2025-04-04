use derivative::Derivative;
use tmui::{prelude::Point, tlib::namespace::KeyCode};
use wchar::wchar_t;
use widestring::WideString;

use crate::{core::uwchar_t, tools::event::KeyPressedEvent};

#[derive(Debug, Derivative)]
#[derivative(Default)]
pub struct LocalDisplay {
    buffer: Vec<wchar_t>,
    cursor: usize,
    /// (col, row)
    cursor_origin: Point,
    colmuns: i32,
    #[derivative(Default(value = "true"))]
    is_executing: bool,
    is_tabing: bool,

    u_stack: Vec<Vec<wchar_t>>,
    d_stack: Vec<Vec<wchar_t>>,
}

impl LocalDisplay {
    #[inline]
    pub fn set_terminal_info(&mut self, x: i32, y: i32, columns: i32) {
        self.cursor_origin = Point::new(x, y);
        self.colmuns = columns;
    }

    #[inline]
    pub fn executed(&mut self) {
        self.is_executing = false;
    }

    /// @return
    /// string is not empty: the character is control character, and send the control character to emulation
    /// string is empty: do nothing.
    /// string is "\u{200B}": show local display string
    pub fn extend(&mut self, evt: &KeyPressedEvent, mut text: String) -> String {
        match evt.key_code() {
            KeyCode::KeyLeft => {
                if self.cursor != 0 {
                    self.cursor -= 1;
                    text
                } else {
                    String::new()
                }
            }
            KeyCode::KeyRight => {
                if self.cursor < self.buffer.len() {
                    self.cursor += 1;
                    text
                } else {
                    String::new()
                }
            }
            KeyCode::KeyUp => {
                if let Some(u_pop) = self.u_stack.pop() {
                    if !self.buffer.is_empty() {
                        self.d_stack.push(self.buffer.clone());
                    }
                    self.buffer = u_pop;
                    self.cursor = self.buffer.len();

                    self.get_redisplay_text()
                } else {
                    String::new()
                }
            }
            KeyCode::KeyDown => {
                if let Some(d_pop) = self.d_stack.pop() {
                    if !self.buffer.is_empty() {
                        self.u_stack.push(self.buffer.clone());
                    }
                    self.buffer = d_pop;
                    self.cursor = self.buffer.len();

                    self.get_redisplay_text()
                } else {
                    String::new()
                }
            }
            KeyCode::KeyHome => {
                self.cursor = 0;
                let cursor_pos = self.cursor_to_position();
                format!("\x1B[{};{}H", cursor_pos.0, cursor_pos.1)
            }
            KeyCode::KeyEnd => {
                self.cursor = self.buffer.len();
                let cursor_pos = self.cursor_to_position();
                format!("\x1B[{};{}H", cursor_pos.0, cursor_pos.1)
            }
            KeyCode::KeyEnter => {
                self.is_executing = true;
                self.u_stack.push(self.buffer.clone());
                self.buffer.clear();
                self.cursor = 0;
                text
            }
            KeyCode::KeyBackspace => {
                if self.cursor != 0 {
                    self.cursor -= 1;
                    self.buffer.remove(self.cursor);

                    text.push_str("\x1B[0K");
                    text.push_str(&self.get_display_string_from(self.cursor));

                    text
                } else {
                    String::new()
                }
            }
            KeyCode::KeyTab => {
                self.is_tabing = true;
                String::new()
            }
            _ => {
                let str = evt.text();
                if !str.is_empty() {
                    let utf16_text = WideString::from_str(&str);
                    let len = utf16_text.len();
                    #[allow(clippy::useless_transmute)]
                    let slice: Vec<wchar_t> = unsafe { std::mem::transmute(utf16_text.into_vec()) };
                    self.buffer
                        .splice(self.cursor..self.cursor, slice);
                    self.cursor += len;
                    "\u{200B}".to_string()
                } else {
                    text
                }
            }
        }
    }

    #[inline]
    pub fn is_tabing(&self) -> bool {
        self.is_tabing
    }

    #[inline]
    pub fn set_tabing(&mut self, tabing: bool) {
        self.is_tabing = tabing
    }

    #[inline]
    pub fn get_display_string(&self) -> String {
        if self.is_executing {
            return String::new();
        }
        let start_index = (self.cursor.max(1) - 1).min(self.buffer.len());
        self.get_display_string_from(start_index)
    }

    #[inline]
    pub fn get_all_display_string(&self) -> String {
        self.get_display_string_from(0)
    }

    #[inline]
    fn get_display_string_from(&self, from: usize) -> String {
        let slice = &self.buffer[from..];
        #[allow(clippy::useless_transmute)]
        let slice: &[uwchar_t] = unsafe { std::mem::transmute(slice) };
        let mut string = WideString::from_vec(slice.to_vec()).to_string_lossy();
        let cursor_pos = self.cursor_to_position();
        string.push_str(&format!("\x1B[{};{}H", cursor_pos.0, cursor_pos.1));
        string
    }

    #[inline]
    fn get_redisplay_text(&self) -> String {
        let mut text = String::new();
        text.push_str(&format!(
            "\x1B[{};{}H\x1B[K",
            self.cursor_origin.y(),
            self.cursor_origin.x()
        ));
        text.push_str(&self.get_all_display_string());
        text
    }

    #[inline]
    fn cursor_to_position(&self) -> (i32, i32) {
        let row = (self.cursor as i32 + self.cursor_origin.x()) / (self.colmuns + 1);
        let col = (self.cursor as i32 + self.cursor_origin.x()) % (self.colmuns + 1);
        (self.cursor_origin.y() + row, col)
    }
}
