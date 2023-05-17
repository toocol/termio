pub mod screen;
pub mod screen_window;
pub mod session;
pub mod session_group;
pub mod terminal_emulator;
pub mod terminal_view;

#[cfg(target_os = "macos")]
#[allow(non_camel_case_types)]
pub type u_wchar_t = u32;
#[cfg(target_os = "windows")]
#[allow(non_camel_case_types)]
pub type u_wchar_t = u16;
