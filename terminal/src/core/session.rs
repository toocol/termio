#![allow(dead_code)]
use super::terminal_view::{BellMode, TerminalView, TripleClickMode};
#[cfg(target_os = "windows")]
use crate::pty::con_pty::ConPty;
#[cfg(not(target_os = "windows"))]
use crate::pty::posix_pty::PosixPty;

use crate::{
    core::terminal_view::TerminalViewSignals,
    emulation::{Emulation, VT102Emulation},
    pty::{ProtocolType, Pty},
    tools::{history::HistoryType, event::KeyPressedEvent},
};
use derivative::Derivative;
use log::debug;
use std::{ptr::NonNull, rc::Rc, cell::RefCell};
use tmui::{
    prelude::*,
    scroll_area::{ScrollArea, ScrollAreaExt},
    scroll_bar::ScrollBarPosition,
    tlib::{
        connect, emit,
        figure::Color,
        namespace::{ExitStatus, Orientation},
        nonnull_mut, nonnull_ref,
        object::{ObjectImpl, ObjectSubclass},
        signals, Object,
    },
};

/// Session represents an open tab used to bridge emulation and pty process.
#[extends(Object)]
pub struct Session {
    enviroment: Vec<String>,

    #[cfg(target_os = "windows")]
    #[derivative(Default(value = "ConPty::new()"))]
    shell_process: Box<dyn Pty>,

    #[cfg(not(target_os = "windows"))]
    #[derivative(Default(value = "PosixPty::new()"))]
    shell_process: Box<dyn Pty>,

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
    emulation: Option<Box<dyn Emulation>>,
    scrolled_view: Option<NonNull<ScrollArea>>,
    view: Option<NonNull<TerminalView>>,
}
impl ObjectSubclass for Session {
    const NAME: &'static str = "Session";
}
impl ObjectImpl for Session {
    fn construct(&mut self) {}
}

pub trait SessionSignal: ActionExt {
    signals! {
        Session:

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
        let mut session: Box<Session> = Object::new(&[]);
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

        // Bind connections between `session` and it's `shell_process`:
        session.shell_process.set_utf8_mode(true);
        connect!(
            session.shell_process,
            receive_data(),
            session,
            on_receive_block(String)
        );
        connect!(session.shell_process, finished(), session, done(i32:0, ExitStatus:1));

        connect!(
            emulation,
            send_data(),
            session.shell_process,
            send_data(String)
        );
        connect!(
            emulation,
            use_utf8_request(),
            session.shell_process,
            set_utf8_mode(bool)
        );

        session.emulation = Some(emulation);
        session
    }

    pub fn create_terminal_view(&mut self) -> Box<ScrollArea> {
        if self.scrolled_view.is_some() {
            panic!(
                "Session has already create the `TerminalView`, session id {}",
                self.session_id
            )
        }

        let mut view = TerminalView::new(self.id());
        view.set_bell_mode(BellMode::NotifyBell);
        view.set_terminal_size_hint(true);
        view.set_triple_click_mode(TripleClickMode::SelectWholeLine);
        view.set_terminal_size_startup(true);
        view.set_blinking_cursor(true);
        view.set_random_seed(view.id() as u32);

        let mut scroll_area: Box<ScrollArea> = Object::new(&[]);
        scroll_area.set_scroll_bar_position(ScrollBarPosition::End);
        scroll_area.set_orientation(Orientation::Vertical);
        scroll_area.set_hexpand(true);
        scroll_area.set_vexpand(true);

        view.set_scroll_bar(scroll_area.get_scroll_bar_mut());
        scroll_area.set_area(view);

        self.scrolled_view = NonNull::new(scroll_area.as_mut());
        let view = scroll_area.get_area_cast_mut::<TerminalView>().unwrap();
        self.view = NonNull::new(view);

        self.bind_view_to_emulation();
        connect!(view, changed_content_size_signal(), self, on_view_size_change(i32:0, i32:1));

        scroll_area
    }

    pub fn bind_view_to_emulation(&mut self) {
        let terminal_view = nonnull_mut!(self.view);
        connect!(self, finished(), terminal_view, terminate());

        let emulation = self.emulation_mut();

        terminal_view.set_uses_mouse(emulation.program_use_mouse());
        terminal_view.set_bracketed_paste_mode(emulation.program_bracketed_paste_mode());

        // Connect `TerminalView`'s signal to emulation:
        connect!(terminal_view, key_pressed_signal(), emulation, send_key_event(KeyPressedEvent:0, bool:1));
        connect!(terminal_view, mouse_signal(), emulation, send_mouse_event(i32:0, i32:1, i32:2, u8:3));
        connect!(terminal_view, send_string_to_emulation(), emulation, send_string(String:0, i32:1));

        // allow emulation to notify view when the foreground process
        // indicates whether or not it is interested in mouse signals:
        connect!(
            emulation,
            program_uses_mouse_changed(),
            terminal_view,
            set_uses_mouse(bool)
        );
        connect!(
            emulation,
            program_bracketed_paste_mode_changed(),
            terminal_view,
            set_bracketed_paste_mode(bool)
        );

        terminal_view.set_screen_window(nonnull_mut!(emulation.create_window()));

        emit!(emulation.output_changed());
    }

    #[inline]
    pub fn view(&self) -> &TerminalView {
        nonnull_ref!(self.view)
    }

    #[inline]
    pub fn view_mut(&mut self) -> &mut TerminalView {
        nonnull_mut!(self.view)
    }

    #[inline]
    pub fn scrolled_view(&self) -> &ScrollArea {
        nonnull_ref!(self.scrolled_view)
    }

    #[inline]
    pub fn scrolled_view_mut(&mut self) -> &mut ScrollArea {
        nonnull_mut!(self.scrolled_view)
    }

    #[inline]
    pub fn emulation(&self) -> &dyn Emulation {
        self.emulation.as_ref().unwrap().as_ref()
    }

    #[inline]
    pub fn emulation_mut(&mut self) -> &mut dyn Emulation {
        self.emulation.as_mut().unwrap().as_mut()
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
    pub fn set_history_type(&mut self, ty: Rc<RefCell<dyn HistoryType>>) {
        self.emulation_mut().set_history(ty)
    }

    #[inline]
    pub fn set_key_binding(&mut self, id: &str) {
        self.emulation_mut().set_key_binding(id)
    }

    #[inline]
    pub fn set_blinking_cursor(&mut self, blink: bool) {
        self.view_mut().set_blinking_cursor(blink)
    }

    ///////////////////////////////////////////////////////////////////////////////////////////
    // private
    ///////////////////////////////////////////////////////////////////////////////////////////
    fn update_terminal_size(&mut self) {
        debug!("TerminalView's size has changed, update the terminal size.");
        let mut min_lines = -1;
        let mut min_columns = -1;

        // minimum number of lines and columns that views require for
        // their size to be taken into consideration ( to avoid problems
        // with new view widgets which haven't yet been set to their correct size )
        const VIEW_LINES_THRESHOLD: i32 = 2;
        const VIEW_COLUMNS_THRESHOLD: i32 = 2;

        let view = self.view();
        if view.visible()
            && view.lines() >= VIEW_LINES_THRESHOLD
            && view.columns() >= VIEW_COLUMNS_THRESHOLD
        {
            min_lines = view.lines();
            min_columns = view.columns();
        }

        if min_lines > 0 && min_columns > 0 {
            self.emulation_mut().set_image_size(min_lines, min_columns);
            self.shell_process.set_window_size(min_columns, min_lines);
        }
    }

    fn update_view_size(&mut self, size: Size) {
        if size.width() <= 1 || size.height() <= 1 {
            return;
        }

        self.view_mut().set_size(size.width(), size.height());
    }

    ///////////////////////////////////////////////////////////////////////////////////////////
    // Slots
    ///////////////////////////////////////////////////////////////////////////////////////////
    pub fn set_user_title(&mut self) {
        // Notice the main program to update the user title.
    }

    pub fn activate_state_set(&mut self, state: i32) {}

    #[inline]
    pub fn on_emulation_size_change(&mut self, size: Size) {
        self.update_view_size(size)
    }

    #[inline]
    pub fn on_view_size_change(&mut self, widht: i32, height: i32) {
        self.update_terminal_size()
    }

    pub fn on_receive_block(&mut self, block: String) {
        let block_bytes = block.as_bytes();
        self.emulation_mut()
            .receive_data(block_bytes, block_bytes.len() as i32);

        emit!(self.receive_data(), block);
    }

    #[inline]
    pub fn done(&mut self, exit_code: i32, exit_status: ExitStatus) {
        emit!(self.finished())
    }
}
