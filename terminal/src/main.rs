// #![windows_subsystem = "windows"]
use asset::Asset;
use cli::{constant::ProtocolType, scheme::color_scheme_mgr::ColorSchemeMgr};
use emulator::core::terminal_emulator::TerminalEmulator;
use tmui::{
    application::Application, application_window::ApplicationWindow, prelude::*, widget::ChildOp,
};

fn main() {
    ColorSchemeMgr::loads::<Asset>("themes/builtin_themes.json");

    log4rs::init_file("terminal/log4rs.yaml", Default::default()).unwrap();

    let app = Application::builder()
        .width(1024)
        .height(720)
        .title("Termio Terminal Emulator")
        .opti_track(true)
        .defer_display(true)
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
            let theme = ColorSchemeMgr::get("Dark").unwrap();
            let emulator = w.downcast_mut::<TerminalEmulator>().unwrap();
            emulator.set_background(theme.background_color());

            emulator.start_session(0, ProtocolType::Cmd);
            emulator.start_session(1, ProtocolType::PowerShell);
            emulator.set_theme(&theme);
        }
    });
}
