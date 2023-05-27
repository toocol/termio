#![allow(dead_code)]
use std::rc::Rc;

use tmui::{prelude::*, tlib::object::ObjectSubclass};

use crate::tools::history::HistoryTypeBuffer;

use super::session::Session;
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
#[derive(Default, Childrenable)]
pub struct TerminalEmulator {}

impl ObjectSubclass for TerminalEmulator {
    const NAME: &'static str = "TerminalEmulator";
}
impl ObjectImpl for TerminalEmulator {
    fn initialize(&mut self) {}
}
impl WidgetImpl for TerminalEmulator {}

impl TerminalEmulator {
    pub fn create_session(&self) -> Box<Session> {
        let mut session = Session::new();
        session.set_auto_close(true);
        session.set_history_type(Rc::new(HistoryTypeBuffer::new(10000)));
        session.set_key_binding("");
        session
    }
}
