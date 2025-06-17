use super::session::Session;
use crate::{
    config::Config,
    core::session::SessionSignal,
    emulation::data_sender::DataSender,
    pty::Pty,
    tools::{
        character_color::color_convert::ColorConvert, event::ToKeyPressedEvent,
        history::HistoryTypeBuffer,
    },
};
use cli::{constant::ProtocolType, scheme::ColorScheme, session::SessionPropsId};
use derivative::Derivative;
use log::warn;
use nohash_hasher::IntMap;
use std::{cell::RefCell, rc::Rc};
use tlib::{close_handler, iter_executor, ptr_mut, signals};
use tmui::{
    prelude::*,
    tlib::{events::KeyEvent, object::ObjectSubclass},
    widget::IterExecutor,
};

/// TerminalPanel was built to manage the terminal view, it holds all the terminal session,
/// and each session has a binded TerminalView.
///
/// Every TerminalPanel has an tab page, it drawed in the main program, not in the terminal program.
#[extends(Widget, Layout(SplitPane))]
#[allow(clippy::vec_box)]
#[iter_executor]
#[close_handler]
pub struct TerminalPanel {
    /// All the terminal sessions.
    sessions: IntMap<SessionPropsId, Box<Session>>,
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

pub trait TerminalPanelSignals: ActionExt {
    signals! {
        TerminalPanel:

        /// Emit when session finished
        session_finished(ObjectId, SessionPropsId);

        /// Emit when all session closed.
        finished(ObjectId);
    }
}
impl TerminalPanelSignals for TerminalPanel {}

impl TerminalPanel {
    #[inline]
    pub fn new() -> Tr<Self> {
        Self::new_alloc()
    }

    pub fn create_session(
        &mut self,
        id: SessionPropsId,
        protocol_type: ProtocolType,
        custom_pty: Option<Box<dyn Pty>>,
    ) -> &mut Box<Session> {
        let mut session = Session::new(id, protocol_type, custom_pty);
        session.set_auto_close(true);
        session.set_history_type(Rc::new(RefCell::new(HistoryTypeBuffer::new(10000))));
        session.set_key_binding("");

        let scrolled_view = session.create_terminal_view();
        session.view_mut().set_font(Config::font());
        self.add_child(scrolled_view);
        ApplicationWindow::window().layout_change(self);

        session.start_shell_process();

        connect!(
            session,
            finished(),
            self,
            handle_session_finished(SessionPropsId)
        );

        self.sessions.insert(id, session);
        self.sessions.get_mut(&id).unwrap()
    }

    pub fn set_terminal_font(&mut self, font: Font) {
        self.sessions
            .iter_mut()
            .for_each(|(_, session)| session.view_mut().set_font(font.clone()))
    }

    pub fn send_key_event(&mut self, id: SessionPropsId, event: KeyEvent) {
        if let Some(session) = self.sessions.get_mut(&id) {
            session
                .emulation_mut()
                .send_key_event(event.to_key_pressed_event(), false);
        }
    }

    pub fn send_text(&mut self, id: SessionPropsId, text: String) {
        if let Some(session) = self.sessions.get_mut(&id) {
            session.emulation_mut().send_text(text);
        }
    }

    pub fn set_blinking_cursor(&mut self, id: SessionPropsId, blink: bool) {
        if let Some(session) = self.sessions.get_mut(&id) {
            session.set_blinking_cursor(blink);
        }
    }

    #[inline]
    pub fn set_use_local_display(&mut self, id: SessionPropsId, use_local_display: bool) {
        if let Some(session) = self.sessions.get_mut(&id) {
            session
                .emulation_mut()
                .set_use_local_display(use_local_display);
        }
    }

    #[inline]
    pub fn set_color_scheme(&mut self, color_scheme: &ColorScheme) {
        self.set_background(color_scheme.background_color());

        self.sessions.iter_mut().for_each(|(_, session)| {
            session
                .scrolled_view_mut()
                .set_background(color_scheme.background_color());
            session
                .view_mut()
                .set_color_table(&color_scheme.convert_entry());
        });
    }

    #[inline]
    pub fn set_session_focus(&mut self, session_id: SessionPropsId) {
        if let Some(session) = self.sessions.get_mut(&session_id) {
            session.view_mut().set_focus(true);
        }
    }

    #[inline]
    pub fn close_session(&mut self, session_id: SessionPropsId) {
        if let Some(session) = self.sessions.get_mut(&session_id) {
            session.close();
        } else {
            warn!("Find the session by session id {} is None.", session_id);
        }
    }
}

/// Private functions:
impl TerminalPanel {
    #[inline]
    fn handle_session_finished(&mut self, id: SessionPropsId) {
        let panel_id = self.id();
        if let Some(session) = self.sessions.remove(&id) {
            self.remove_children(session.scrolled_view().id());
            emit!(self, session_finished(panel_id, id));
        }

        if self.sessions.is_empty() {
            emit!(self, finished(panel_id));
        }
    }
}

impl IterExecutor for TerminalPanel {
    fn iter_execute(&mut self) {
        let mut closed = vec![];

        for session in self.sessions.values_mut() {
            if let Some(shell_process) = session.get_pty() {
                if shell_process.is_closed() {
                    closed.push(shell_process.as_mut_ptr());
                    continue;
                }

                let data = shell_process.read_data();
                if !data.is_empty() {
                    session
                        .emulation_mut()
                        .receive_data(&data, data.len() as i32, DataSender::Pty);
                }
            } else {
                warn!("The custom pty is not assigned.");
            }
        }

        for pty in closed {
            ptr_mut!(pty).emit_finished();
        }
    }
}

impl CloseHandler for TerminalPanel {
    #[inline]
    fn handle(&mut self) {
        for session in self.sessions.values_mut() {
            if let Some(shell_process) = session.get_pty() {
                shell_process.on_window_closed();
            }
        }
    }
}
