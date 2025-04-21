use super::left_panel::LeftPanel;
use emulator::core::terminal_emulator::TerminalEmulator;
use tmui::{
    prelude::*,
    tlib::namespace::Orientation,
    tlib::object::{ObjectImpl, ObjectSubclass},
    widget::WidgetImpl,
};

#[extends(Widget, Layout(Pane))]
#[derive(Childrenable)]
pub struct CentralPanel {
    #[children]
    left_panel: Tr<LeftPanel>,

    #[children]
    terminal: Tr<TerminalEmulator>,
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
