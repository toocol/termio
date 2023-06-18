use crate::core::terminal_emulator::TerminalEmulator;
use tmui::{
    application::Application, application_window::ApplicationWindow, widget::{WidgetImplExt, WidgetExt},
};

mod asset;
mod config;
mod core;
mod emulation;
mod pty;
mod tools;

fn main() {
    log4rs::init_file("terminal/log4rs.yaml", Default::default()).unwrap();

    let app = Application::builder()
        .width(1280)
        .height(800)
        .title("Termio Terminal Emulator")
        .build();

    app.connect_activate(build_ui);

    app.run();
}

fn build_ui(window: &mut ApplicationWindow) {
    let size = window.size();

    let mut terminal_emulator = TerminalEmulator::new();
    terminal_emulator.width_request(size.width());
    terminal_emulator.height_request(size.height());
    window.child(terminal_emulator);
}
