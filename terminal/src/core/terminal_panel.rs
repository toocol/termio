use super::session::Session;
use crate::tools::history::HistoryTypeBuffer;
use derivative::Derivative;
use std::rc::Rc;
use tmui::{prelude::*, tlib::object::ObjectSubclass};

/// TerminalPanel was built to manage the terminal view, it holds all the terminal session,
/// and each session has a binded TerminalView.
///
/// Every TerminalPanel has an tab page, it drawed in the main program, not in the terminal program.
#[extends(Widget, Layout(SplitPane))]
pub struct TerminalPanel {
    /// All the terminal sessions.
    sessions: Vec<Box<Session>>,
}

impl ObjectSubclass for TerminalPanel {
    const NAME: &'static str = "TerminalPanel";
}

impl ObjectImpl for TerminalPanel {
    fn initialize(&mut self) {
        let session = self.create_session();
        let scrolled_view = session.create_terminal_view();

        self.add_child(scrolled_view);
    }
}

impl WidgetImpl for TerminalPanel {}

impl TerminalPanel {
    pub fn create_session(&mut self) -> &mut Box<Session> {
        let mut session = Session::new();
        session.set_auto_close(true);
        session.set_history_type(Rc::new(HistoryTypeBuffer::new(10000)));
        session.set_key_binding("");

        self.sessions.push(session);
        self.sessions.last_mut().unwrap()
    }
}
