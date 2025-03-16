// #![windows_subsystem = "windows"]
use cli::constant::ProtocolType;
use emulator::core::terminal_emulator::TerminalEmulator;
use tmui::{
    application::Application, application_window::ApplicationWindow, prelude::*, widget::ChildOp,
};

fn main() {
    log4rs::init_file("terminal/log4rs.yaml", Default::default()).unwrap();

    let app = Application::builder()
        .width(1024)
        .height(720)
        .title("Termio Terminal Emulator")
        .opti_track(true)
        .build();

    app.connect_activate(build_ui);

    app.run();
}

fn build_ui(window: &mut ApplicationWindow) {
    let terminal_emulator = TerminalEmulator::new();
    let id = terminal_emulator.id();
    window.child(terminal_emulator);

    window.register_run_after(move |win| {
        if let Some(w) = win.find_id_mut(id) {
            let emulator = w.downcast_mut::<TerminalEmulator>().unwrap();
            emulator.start_session(0, ProtocolType::LocalShell);
        }
    });
}
