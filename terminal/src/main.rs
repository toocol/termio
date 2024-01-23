use termulator::core::terminal_emulator::TerminalEmulator;
use libs::constant::IPC_NAME;
use tmui::{
    application::Application, application_window::ApplicationWindow, platform::PlatformType,
    widget::WidgetImplExt,
};

fn main() {
    log4rs::init_file("terminal/log4rs.yaml", Default::default()).unwrap();

    let app = if true {
        Application::<(), ()>::shared_builder(IPC_NAME)
            .platform(PlatformType::Ipc)
            .shared_widget_id("terminal")
            .build()
    } else {
        Application::builder()
            .width(200)
            .height(120)
            .title("Termio Terminal Emulator")
            .build()
    };

    app.connect_activate(build_ui);

    app.run();
}

fn build_ui(window: &mut ApplicationWindow) {
    let terminal_emulator = TerminalEmulator::new();
    window.child(terminal_emulator);
}