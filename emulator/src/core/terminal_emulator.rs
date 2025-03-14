#![allow(dead_code)]
use std::cell::RefCell;

use super::terminal_panel::TerminalPanel;
use crate::pty::pty_receive_pool;
use cli::{constant::ProtocolType, session::SessionPropsId};
use derivative::Derivative;
use tmui::{prelude::*, tlib::object::ObjectSubclass};

thread_local! {
    static EMULATOR_ID: RefCell<ObjectId> = RefCell::new(0);
}

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
#[extends(Widget, Layout(Stack))]
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

        EMULATOR_ID.with(|e| *e.borrow_mut() = self.id())
    }

    #[inline]
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

    #[inline]
    pub fn id() -> ObjectId {
        EMULATOR_ID.with(|e| *e.borrow())
    }

    #[inline]
    pub fn start_session(&mut self, id: SessionPropsId, protocol_type: ProtocolType) {
        self.terminal_panel.create_session(id, protocol_type);
    }

    #[inline]
    pub fn set_blinking_cursor(&mut self, id: SessionPropsId, blink: bool) {
        self.terminal_panel.set_blinking_cursor(id, blink);
    }
}
