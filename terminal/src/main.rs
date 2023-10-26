use crate::core::terminal_emulator::TerminalEmulator;
use tmui::{
    application::Application,
    application_window::ApplicationWindow,
    widget::WidgetImplExt,
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
        .width(200)
        .height(120)
        .title("Termio Terminal Emulator")
        .build();

    app.connect_activate(build_ui);

    app.run();
}

fn build_ui(window: &mut ApplicationWindow) {
    let terminal_emulator = TerminalEmulator::new();
    window.child(terminal_emulator);
}
