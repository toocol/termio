#![allow(dead_code)]
use super::terminal_panel::TerminalPanel;
use crate::pty::pty_receive_pool;
use derivative::Derivative;
use tmui::{prelude::*, tlib::object::ObjectSubclass};

/*
                          |- Session/Emulation |- ScreenWidow/Screens
          - TerminalPanel |
          |               |- Session/Emulation |- ScreenWidow/Screens
 Terminal-|
          |               |- Session/Emulation |- ScreenWidow/Screens
          - TerminalPanel |
                          |- Session/Emulation |- ScreenWidow/Screens
*/
/// The terminal's main widget. Responsible for all layouts management of `TerminalView`,
/// forward the client's input information from the ipc channel.
#[extends(Widget, Layout(VBox))]
#[derive(Childrenable)]
pub struct TerminalEmulator {
    #[children]
    terminal_panel: Box<TerminalPanel>,
}

impl ObjectSubclass for TerminalEmulator {
    const NAME: &'static str = "TerminalEmulator";
}

impl ObjectImpl for TerminalEmulator {
    fn construct(&mut self) {
        self.parent_construct();

        self.set_vexpand(true);
        self.set_hexpand(true);
    }

    fn initialize(&mut self) {
        pty_receive_pool().start();
    }
}

impl WidgetImpl for TerminalEmulator {
    #[inline]
    fn font_changed(&mut self) {
        self.terminal_panel.set_terminal_font(self.font().clone())
    }
}

impl TerminalEmulator {
    #[inline]
    pub fn new() -> Box<Self> {
        Object::new(&[])
    }
}
