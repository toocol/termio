use super::session::Session;
use crate::{
    config::Config,
    emulation::data_sender::DataSender,
    pty::{pty_receive_pool, Pty},
    tools::{
        character_color::color_convert::ColorConvert, event::ToKeyPressedEvent,
        history::HistoryTypeBuffer,
    },
};
use cli::{constant::ProtocolType, session::SessionPropsId, theme::Theme};
use derivative::Derivative;
use log::{error, warn};
use nohash_hasher::IntMap;
use std::{
    cell::RefCell,
    rc::Rc,
    sync::mpsc::{channel, Receiver},
};
use tlib::{close_handler, iter_executor};
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

    receiver: Option<Receiver<(SessionPropsId, Vec<u8>)>>,
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

    #[inline]
    fn initialize(&mut self) {
        let (sender, receiver) = channel();
        self.receiver = Some(receiver);

        pty_receive_pool().start(sender);
    }
}

impl WidgetImpl for TerminalPanel {}

impl TerminalPanel {
    pub fn create_session(
        &mut self,
        id: SessionPropsId,
        protocol_type: ProtocolType,
    ) -> &mut Box<Session> {
        if protocol_type == ProtocolType::Custom {
            panic!("Use `create_custom_session` instead")
        }
        let mut session = Session::new(id, protocol_type, None);
        session.set_auto_close(true);
        session.set_history_type(Rc::new(RefCell::new(HistoryTypeBuffer::new(10000))));
        session.set_key_binding("");

        let scrolled_view = session.create_terminal_view();
        session.view_mut().set_font(Config::font());
        self.add_child(scrolled_view);
        ApplicationWindow::window().layout_change(self);

        session.start_shell_process();

        self.sessions.insert(id, session);
        self.sessions.get_mut(&id).unwrap()
    }

    pub fn create_custom_session(
        &mut self,
        id: SessionPropsId,
        custom_pty: Box<dyn Pty>,
    ) -> &mut Box<Session> {
        let mut session = Session::new(id, ProtocolType::Custom, Some(custom_pty));
        session.set_auto_close(true);
        session.set_history_type(Rc::new(RefCell::new(HistoryTypeBuffer::new(10000))));
        session.set_key_binding("");

        let scrolled_view = session.create_terminal_view();
        session.view_mut().set_font(Config::font());
        self.add_child(scrolled_view);
        ApplicationWindow::window().layout_change(self);

        session.start_shell_process();

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
    pub fn set_theme(&mut self, id: SessionPropsId, theme: Theme) {
        if let Some(session) = self.sessions.get_mut(&id) {
            session.view_mut().set_color_table(&theme.convert_entry());
        }
    }
}

impl IterExecutor for TerminalPanel {
    fn iter_execute(&mut self) {
        if let Some(receiver) = self.receiver.as_ref() {
            while let Ok((id, data)) = receiver.try_recv() {
                if let Some(session) = self.sessions.get_mut(&id) {
                    session
                        .emulation_mut()
                        .receive_data(&data, data.len() as i32, DataSender::Pty);
                } else {
                    error!("Get session with id `{}` is None.", id);
                }
            }
        }

        for session in self.sessions.values_mut() {
            if session.get_protocol_type() == ProtocolType::Custom {
                if let Some(shell_process) = session.get_pty() {
                    let data = shell_process.read_data();
                    session
                        .emulation_mut()
                        .receive_data(&data, data.len() as i32, DataSender::Pty);
                } else {
                    warn!("The custom pty is not assigned.");
                }
            }
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
