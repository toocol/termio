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
    terminal_panel: TerminalPanel,
}

impl ObjectSubclass for TerminalEmulator {
    const NAME: &'static str = "TerminalEmulator";
}

impl ObjectImpl for TerminalEmulator {
    fn initialize(&mut self) {
        pty_receive_pool().start();
    }
}

impl WidgetImpl for TerminalEmulator {}

impl TerminalEmulator {
    #[inline]
    pub fn new() -> Self {
        Object::new(&[])
    }
}
