#![allow(dead_code)]
use std::cell::RefCell;

use crate::pty::Pty;

use super::terminal_panel::TerminalPanel;
use cli::{constant::ProtocolType, session::SessionPropsId, theme::Theme};
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
        if protocol_type == ProtocolType::Custom {
            panic!("Use `create_custom_session` instead")
        }
        self.terminal_panel.create_session(id, protocol_type);
    }

    #[inline]
    pub fn start_custom_session(&mut self, id: SessionPropsId, custom_pty: Box<dyn Pty>) {
        self.terminal_panel.create_custom_session(id, custom_pty);
    }

    #[inline]
    pub fn set_blinking_cursor(&mut self, id: SessionPropsId, blink: bool) {
        self.terminal_panel.set_blinking_cursor(id, blink);
    }

    #[inline]
    pub fn set_use_local_display(&mut self, id: SessionPropsId, use_local_display: bool) {
        self.terminal_panel
            .set_use_local_display(id, use_local_display);
    }

    #[inline]
    pub fn set_theme(&mut self, id: SessionPropsId, theme: &Theme) {
        self.set_background(theme.background_color());
        self.terminal_panel.set_theme(id, theme);
    }

    #[inline]
    pub fn set_font(&mut self, font: Font) {
        self.terminal_panel.set_terminal_font(font);
    }
}
