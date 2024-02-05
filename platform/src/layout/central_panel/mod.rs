use super::left_panel::LeftPanel;
use emulator::core::terminal_emulator::TerminalEmulator;
use tmui::{
    prelude::*,
    tlib::object::{ObjectImpl, ObjectSubclass},
    widget::WidgetImpl,
};

#[extends(Widget, Layout(Pane))]
#[derive(Childrenable)]
pub struct CentralPanel {
    #[children]
    left_panel: Box<LeftPanel>,

    #[children]
    terminal: Box<TerminalEmulator>,
}

impl ObjectSubclass for CentralPanel {
    const NAME: &'static str = "CentralPanel";
}

impl ObjectImpl for CentralPanel {
    fn initialize(&mut self) {
        self.set_hexpand(true);
        self.set_vexpand(true);
        self.set_strict_children_layout(true);
    }
}

impl WidgetImpl for CentralPanel {}
