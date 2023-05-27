#![allow(dead_code)]
use std::rc::Rc;

use super::{
    terminal_panel::ViewLocation,
    terminal_view::{BellMode, TerminalView, TripleClickMode},
};
use crate::{
    emulation::{Emulation, VT102Emulation},
    pty::ProtocolType,
    tools::history::HistoryType,
};
use derivative::Derivative;
use tmui::{
    graphics::figure::Color,
    prelude::*,
    tlib::{
        connect,
        object::{ObjectImpl, ObjectSubclass},
        signals, Object,
    },
};

/// Session represents an open tab used to bridge emulation and pty process.
#[extends(Object)]
#[derive(Derivative)]
#[derivative(Default)]
pub struct Session {
    enviroment: Vec<String>,

    auto_close: bool,
    wanted_close: bool,

    local_tab_title_format: String,
    remote_tab_title_format: String,

    initial_working_dir: String,

    /// flag if the title/icon was changed by user
    is_title_changed: bool,
    add_to_utmp: bool,
    #[derivative(Default(value = "true"))]
    flow_control: bool,
    full_scripting: bool,

    program: String,
    arguments: Vec<String>,

    protocol_type: ProtocolType,
    session_group_id: i32,
    session_id: u64,
    host: String,
    user: String,
    password: String,

    has_dark_background: bool,
    modified_background: Color,

    // Zmodem
    zmodem_busy: bool,
    // zmodem_proc: Process
    emultaion: Option<Box<dyn Emulation>>,
    view: Option<TerminalView>,
    location: ViewLocation,
}
impl ObjectSubclass for Session {
    const NAME: &'static str = "Session";
}
impl ObjectImpl for Session {}

pub trait SessionSignal: ActionExt {
    signals! {
        /// Emitted when the terminal process starts.
        started();

        /// Emitted when the terminal process exits.
        finished();

        /// Emitted when output is received from the terminal process.
        ///
        /// @param text: [`String`]
        receive_data();

        /// Emitted when the session's title has changed.
        title_changed();

        /// Emitted when the session's profile has changed.
        ///
        /// @param profile: [`String`]
        profile_changed();

        /// Emitted when the activity state of this session changes.
        ///
        /// @param state: [`i32`] The new state of the session.  This may be one
        /// of NOTIFYNORMAL, NOTIFYSILENCE or NOTIFYACTIVITY
        state_changed();

        /// Emitted when a bell event occurs in the session.
        ///
        /// @param message: [`String`]
        bell_request();

        /// Requests that the color the text for any tabs associated with
        /// this session should be changed;
        ///
        /// @param [`i32`]
        change_tab_text_color_request();

        /// Requests that the background color of views on this session
        /// should be changed.
        ///
        /// @param [`Color`]
        change_background_color_request();

        /// User click on url link.
        ///
        /// @param url: [`String`]
        open_url_request();

        /// Detected the zmodem.
        zmodem_detected();

        /// Emitted when the terminal process requests a change
        /// in the size of the terminal window.
        ///
        /// @param size: [`Size`] The requested window size in terms of lines and columns.
        resize_request();

        /// Emitted when a profile change command is received from the terminal.
        ///
        /// @param [`String`] The text of the command.  This is a string of the form
        /// "PropertyName=Value;PropertyName=Value ..."
        profile_change_command_received();

        /// Emitted when the flow control state changes.
        ///
        /// @param [`bool`]
        flow_control_enabled_changed();

        /// Broker for Emulation::cursorChanged() signal.
        ///
        /// @param [`SystemCursorShape`]
        /// @param [`bool`] Enable blinking cursor or not.
        cursor_changed();

        silence();

        activity();
    }
}
impl SessionSignal for Session {}

impl Session {
    pub fn new() -> Box<Self> {
        let mut session: Box<Session> = Box::new(Object::new(&[]));
        let emulation = VT102Emulation::new(None).wrap();
        connect!(emulation, title_changed(), session, set_user_title());
        connect!(emulation, state_set(), session, activate_state_set(i32));
        connect!(
            emulation,
            image_resize_request(),
            session,
            on_emulation_size_change(Size)
        );
        connect!(emulation, image_size_changed(), session, on_view_size_change(i32:0, i32:1));

        session.emultaion = Some(emulation);
        session
    }

    pub fn create_terminal_view(&mut self) {
        let mut view = TerminalView::new(self.id());
        view.set_bell_mode(BellMode::NotifyBell);
        view.set_terminal_size_hint(true);
        view.set_triple_click_mode(TripleClickMode::SelectWholeLine);
        view.set_terminal_size_startup(true);
        view.set_random_seed(view.id() as u32);
        self.view = Some(view);
    }

    pub fn init(&mut self) {}

    #[inline]
    pub fn view(&mut self) -> TerminalView {
        self.view.take().unwrap()
    }

    #[inline]
    pub fn set_view(&mut self, view: TerminalView) {
        self.view = Some(view);
    }

    #[inline]
    pub fn emulation(&self) -> &dyn Emulation {
        self.emultaion.as_ref().unwrap().as_ref()
    }

    #[inline]
    pub fn emultaion_mut(&mut self) -> &mut dyn Emulation {
        self.emultaion.as_mut().unwrap().as_mut()
    }

    #[inline]
    pub fn session_id(&self) -> u64 {
        self.session_id
    }

    #[inline]
    pub fn set_auto_close(&mut self, auto: bool) {
        self.auto_close = auto
    }

    #[inline]
    pub fn set_history_type(&mut self, ty: Rc<dyn HistoryType>) {
        self.emultaion_mut().set_history(ty)
    }

    #[inline]
    pub fn set_key_binding(&mut self, id: &str) {
        self.emultaion_mut().set_key_binding(id)
    }

    ///////////////////////////////////////////////////////////////////////////////////////////
    // Slots
    ///////////////////////////////////////////////////////////////////////////////////////////
    pub fn set_user_title(&mut self) {
        // Notice the main program to update the user title.
    }

    pub fn activate_state_set(&mut self, state: i32) {}

    pub fn on_emulation_size_change(&mut self, size: Size) {}

    pub fn on_view_size_change(&mut self, widht: i32, height: i32) {}
}
