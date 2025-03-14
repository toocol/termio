use super::session::Session;
use crate::{
    config::Config,
    tools::{event::ToKeyPressedEvent, history::HistoryTypeBuffer},
};
use cli::{constant::ProtocolType, session::SessionPropsId};
use derivative::Derivative;
use std::{cell::RefCell, rc::Rc};
use tmui::{
    prelude::*,
    tlib::{events::KeyEvent, object::ObjectSubclass},
};

/// TerminalPanel was built to manage the terminal view, it holds all the terminal session,
/// and each session has a binded TerminalView.
///
/// Every TerminalPanel has an tab page, it drawed in the main program, not in the terminal program.
#[extends(Widget, Layout(SplitPane))]
#[allow(clippy::vec_box)]
pub struct TerminalPanel {
    /// All the terminal sessions.
    sessions: Vec<Box<Session>>,
}

impl ObjectSubclass for TerminalPanel {
    const NAME: &'static str = "TerminalPanel";
}

impl ObjectImpl for TerminalPanel {
    fn construct(&mut self) {
        self.parent_construct();

        self.set_hexpand(true);
        self.set_vexpand(true);
    }
}

impl WidgetImpl for TerminalPanel {}

impl TerminalPanel {
    pub fn create_session(
        &mut self,
        id: SessionPropsId,
        protocol_type: ProtocolType,
    ) -> &mut Box<Session> {
        let mut session = Session::new(id, protocol_type);
        session.set_auto_close(true);
        session.set_history_type(Rc::new(RefCell::new(HistoryTypeBuffer::new(10000))));
        session.set_key_binding("");

        let scrolled_view = session.create_terminal_view();
        session.view_mut().set_font(Config::font());
        self.add_child(scrolled_view);
        self.window().layout_change(self);

        session.start_shell_process();

        self.sessions.push(session);
        self.sessions.last_mut().unwrap()
    }

    pub fn set_terminal_font(&mut self, font: Font) {
        self.sessions
            .iter_mut()
            .for_each(|session| session.view_mut().set_font(font.clone()))
    }

    pub fn send_key_event(&mut self, event: KeyEvent) {
        self.sessions
            .first_mut()
            .unwrap()
            .emulation_mut()
            .send_key_event(event.to_key_pressed_event(), false);
    }

    pub fn send_text(&mut self, text: String) {
        self.sessions
            .first_mut()
            .unwrap()
            .emulation_mut()
            .send_text(text);
    }

    pub fn set_blinking_cursor(&mut self, _id: SessionPropsId, blink: bool) {
        self.sessions
            .first_mut()
            .unwrap()
            .set_blinking_cursor(blink);
    }
}

impl TerminalPanel {
    #[inline]
    pub fn new() -> Box<Self> {
        Object::new(&[])
    }
}
