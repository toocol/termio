pub mod screen;
pub mod screen_window;
pub mod session;
pub mod terminal_emulator;
pub mod terminal_panel;
pub mod terminal_view;

#[cfg(not(windows))]
#[allow(non_camel_case_types)]
pub type uwchar_t = u32;
#[cfg(windows)]
#[allow(non_camel_case_types)]
pub type uwchar_t = u16;
