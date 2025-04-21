#![allow(dead_code)]
use super::terminal_view::{BellMode, TerminalView, TripleClickMode};
#[cfg(target_os = "windows")]
use crate::pty::con_pty::ConPty;
#[cfg(not(target_os = "windows"))]
use crate::pty::posix_pty::PosixPty;

use crate::{
    core::terminal_view::TerminalViewSignals,
    emulation::{Emulation, VT102Emulation},
    pty::Pty,
    tools::{event::KeyPressedEvent, history::HistoryType},
};
use cli::{constant::ProtocolType, session::SessionPropsId};
use derivative::Derivative;
use log::{debug, warn};
use std::{cell::RefCell, ptr::NonNull, rc::Rc};
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

    shell_process: Option<Box<dyn Pty>>,
    protocol_type: ProtocolType,

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

    session_group_id: i32,
    session_id: u64,

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
        finished(SessionPropsId);

        /// Emitted when output is received from the terminal process.
        ///
        /// @param text: [`String`]
        receive_data(String);

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
    pub fn new(
        id: SessionPropsId,
        protocol_type: ProtocolType,
        pty: Option<Box<dyn Pty>>,
    ) -> Box<Self> {
        let mut session: Box<Session> = Object::new(&[]);
        session.session_id = id;
        session.protocol_type = protocol_type;
        let emulation = VT102Emulation::new(None).wrap();
        connect!(
            emulation,
            title_changed(),
            session,
            set_user_title(i32, String)
        );
        connect!(emulation, state_set(), session, activate_state_set(i32));
        connect!(
            emulation,
            image_resize_request(),
            session,
            on_emulation_size_change(Size)
        );

        let mut shell_process: Option<Box<dyn Pty>> = match protocol_type {
            #[cfg(target_os = "windows")]
            ProtocolType::Cmd => {
                let shell_process = ConPty::new();
                Some(shell_process)
            }
            #[cfg(target_os = "windows")]
            ProtocolType::PowerShell => {
                let shell_process = ConPty::new();
                Some(shell_process)
            }
            ProtocolType::Custom => pty,
            _ => None,
        };

        if let Some(ref mut shell_process) = shell_process {
            // Bind connections between `session` and it's `shell_process`:
            shell_process.set_utf8_mode(true);
            connect!(
                shell_process,
                finished(),
                session,
                done(SessionPropsId, ExitStatus)
            );

            connect!(emulation, send_data(), shell_process, send_data(String));
            connect!(
                emulation,
                use_utf8_request(),
                shell_process,
                set_utf8_mode(bool)
            );
        }

        session.shell_process = shell_process;
        session.emulation = Some(emulation);
        session
    }

    pub fn create_terminal_view(&mut self) -> Tr<ScrollArea> {
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
        let id = view.id();
        view.set_random_seed(id);

        let mut scroll_area = ScrollArea::new_alloc();
        scroll_area.set_scroll_bar_position(ScrollBarPosition::End);
        scroll_area.set_orientation(Orientation::Vertical);
        scroll_area.set_hexpand(true);
        scroll_area.set_vexpand(true);

        let scroll_bar = scroll_area.scroll_bar_mut();
        scroll_bar.set_background(Color::TRANSPARENT);
        scroll_bar.set_color(Color::GREY_MEDIUM.with_a(150));
        scroll_bar.set_active_color(Some(Color::GREY_LIGHT));
        scroll_bar.set_slider_radius(5.);
        scroll_bar.set_auto_hide(false);

        view.set_scroll_bar(scroll_bar);
        scroll_area.set_area(view);
        scroll_area.set_layout_mode(LayoutMode::Normal);

        self.scrolled_view = NonNull::new(scroll_area.as_mut());
        let view = scroll_area.get_area_cast_mut::<TerminalView>().unwrap();
        self.view = NonNull::new(view);

        self.bind_view_to_emulation();
        connect!(
            view,
            changed_content_size_signal(),
            self,
            on_view_size_change(i32, i32)
        );

        scroll_area
    }

    pub fn bind_view_to_emulation(&mut self) {
        let terminal_view = nonnull_mut!(self.view);
        connect!(self, finished(), terminal_view, terminate(SessionPropsId));

        let emulation = self.emulation_mut();

        terminal_view.set_uses_mouse(emulation.program_use_mouse());
        terminal_view.set_bracketed_paste_mode(emulation.program_bracketed_paste_mode());

        // Connect `TerminalView`'s signal to emulation:
        connect!(
            terminal_view,
            key_pressed_signal(),
            emulation,
            send_key_event(KeyPressedEvent, bool)
        );
        connect!(
            terminal_view,
            control_insert_detected(),
            emulation,
            handle_control_insert()
        );
        connect!(
            terminal_view,
            shift_insert_detected(),
            emulation,
            handle_shift_insert()
        );
        connect!(
            terminal_view,
            mouse_signal(),
            emulation,
            send_mouse_event(i32, i32, i32, u8)
        );
        connect!(
            terminal_view,
            send_string_to_emulation(),
            emulation,
            send_string(String, i32)
        );

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

    #[inline]
    pub fn get_protocol_type(&self) -> ProtocolType {
        self.protocol_type
    }

    #[inline]
    pub fn get_pty(&mut self) -> Option<&mut dyn Pty> {
        self.shell_process.as_mut().map(|p| p.as_mut())
    }

    #[inline]
    pub fn start_shell_process(&mut self) {
        match self.protocol_type {
            ProtocolType::Cmd => {
                #[cfg(target_os = "windows")]
                self.shell_process.as_mut().unwrap().start(
                    self.session_id,
                    "cmd.exe",
                    vec!["/K"],
                    vec![],
                );
            }
            ProtocolType::PowerShell => {
                #[cfg(target_os = "windows")]
                self.shell_process.as_mut().unwrap().start(
                    self.session_id,
                    "PowerShell.exe",
                    vec![""],
                    vec![],
                );
            }
            ProtocolType::Custom => {
                if let Some(shell_process) = self.shell_process.as_mut() {
                    shell_process.start(self.session_id, "", vec![], vec![]);
                } else {
                    warn!("Custom pty is not assigned.")
                }
            }
            _ => {}
        }
    }

    ///////////////////////////////////////////////////////////////////////////////////////////
    // private
    ///////////////////////////////////////////////////////////////////////////////////////////
    fn update_terminal_size(&mut self) {
        if self.shell_process.is_none() {
            return;
        }

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

        debug!("TerminalView's size has changed, update the terminal size. min_lines {}, min_columns {}", min_lines, min_columns);
        if min_lines > 0 && min_columns > 0 {
            self.emulation_mut().set_image_size(min_lines, min_columns);
            self.shell_process
                .as_mut()
                .unwrap()
                .set_window_size(min_columns, min_lines);
        }
    }

    fn update_view_size(&mut self, size: Size) {
        if size.width() <= 1 || size.height() <= 1 {
            return;
        }

        self.view_mut().set_size();
    }

    ///////////////////////////////////////////////////////////////////////////////////////////
    // Slots
    ///////////////////////////////////////////////////////////////////////////////////////////
    pub fn set_user_title(&mut self, state: i32, title: String) {
        // Notice the main program to update the user title.
    }

    pub fn activate_state_set(&mut self, _state: i32) {}

    #[inline]
    pub fn on_emulation_size_change(&mut self, size: Size) {
        self.update_view_size(size)
    }

    #[inline]
    pub fn on_view_size_change(&mut self, _width: i32, _height: i32) {
        self.update_terminal_size()
    }

    #[inline]
    pub fn done(&mut self, id: SessionPropsId, _exit_status: ExitStatus) {
        emit!(self, finished(id));
    }
}
