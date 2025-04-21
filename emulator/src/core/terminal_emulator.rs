#![allow(dead_code)]
use std::cell::RefCell;

use crate::{core::terminal_panel::TerminalPanelSignals, pty::Pty};

use super::terminal_panel::TerminalPanel;
use cli::{constant::ProtocolType, scheme::ColorScheme, session::SessionPropsId};
use derivative::Derivative;
use log::warn;
use nohash_hasher::IntMap;
use tmui::{prelude::*, tlib::object::ObjectSubclass};

thread_local! {
    static EMULATOR_ID: RefCell<ObjectId> = const { RefCell::new(0) };
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
pub struct TerminalEmulator {
    index_map: IntMap<SessionPropsId, usize>,
    session_id_map: IntMap<ObjectId, Vec<SessionPropsId>>,
}

impl ObjectSubclass for TerminalEmulator {
    const NAME: &'static str = "TerminalEmulator";
}

impl ObjectImpl for TerminalEmulator {
    fn construct(&mut self) {
        self.parent_construct();

        self.set_vexpand(true);
        self.set_hexpand(true);

        EMULATOR_ID.with(|e| *e.borrow_mut() = self.id());
    }
}

impl WidgetImpl for TerminalEmulator {
    #[inline]
    fn font_changed(&mut self) {
        let font = self.font().clone();
        if let Some(terminal_panel) = self.cur_terminal_panel_mut() {
            terminal_panel.set_terminal_font(font)
        }
    }
}

impl TerminalEmulator {
    #[inline]
    pub fn new() -> Tr<Self> {
        Self::new_alloc()
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
        let terminal_panel = TerminalPanel::new();
        connect!(
            terminal_panel,
            session_finished(),
            self,
            handle_session_finished(ObjectId, SessionPropsId)
        );
        connect!(
            terminal_panel,
            finished(),
            self,
            handle_session_panel_finished(ObjectId)
        );
        self.session_id_map
            .entry(terminal_panel.id())
            .or_default()
            .push(id);
        self.add_child(terminal_panel);

        self.switch();

        let index = self.current_index;
        self.index_map.insert(id, index);

        if let Some(terminal_panel) = self.cur_terminal_panel_mut() {
            terminal_panel.create_session(id, protocol_type);
        }
    }

    #[inline]
    pub fn start_custom_session(&mut self, id: SessionPropsId, custom_pty: Box<dyn Pty>) {
        let terminal_panel = TerminalPanel::new();
        connect!(
            terminal_panel,
            session_finished(),
            self,
            handle_session_finished(ObjectId, SessionPropsId)
        );
        connect!(
            terminal_panel,
            finished(),
            self,
            handle_session_panel_finished(ObjectId)
        );
        self.session_id_map
            .entry(terminal_panel.id())
            .or_default()
            .push(id);
        self.add_child(terminal_panel);

        self.switch();

        let index = self.current_index;
        self.index_map.insert(id, index);

        if let Some(terminal_panel) = self.cur_terminal_panel_mut() {
            terminal_panel.create_custom_session(id, custom_pty);
        }
    }

    #[inline]
    pub fn switch_session(&mut self, id: SessionPropsId) {
        if let Some(idx) = self.index_map.get(&id).copied() {
            self.switch_index(idx);
        } else {
            warn!(
                "[TerminalEmulator::switch_session] Get index with session id {} is None.",
                id
            );
        }
    }

    #[inline]
    pub fn remove_session(&mut self, id: SessionPropsId) {
        if let Some(idx) = self.index_map.get(&id).copied() {
            self.remove_index(idx);
        } else {
            warn!(
                "[TerminalEmulator::switch_session] Get index with session id {} is None.",
                id
            );
        }
    }

    #[inline]
    pub fn set_blinking_cursor(&mut self, id: SessionPropsId, blink: bool) {
        if let Some(terminal_panel) = self.cur_terminal_panel_mut() {
            terminal_panel.set_blinking_cursor(id, blink);
        }
    }

    #[inline]
    pub fn set_use_local_display(&mut self, id: SessionPropsId, use_local_display: bool) {
        if let Some(terminal_panel) = self.cur_terminal_panel_mut() {
            terminal_panel.set_use_local_display(id, use_local_display);
        }
    }

    #[inline]
    pub fn set_color_scheme(&mut self, theme: &ColorScheme) {
        self.set_background(theme.background_color());
        if let Some(terminal_panel) = self.cur_terminal_panel_mut() {
            terminal_panel.set_color_scheme(theme);
        }
    }

    #[inline]
    pub fn set_terminal_font(&mut self, font: Font) {
        if let Some(terminal_panel) = self.cur_terminal_panel_mut() {
            terminal_panel.set_terminal_font(font);
        }
    }

    #[inline]
    pub fn cur_terminal_panel_mut(&mut self) -> Option<&mut TerminalPanel> {
        self.current_child_mut()
            .map(|c| c.downcast_mut::<TerminalPanel>().unwrap())
    }

    #[inline]
    pub fn cur_terminal_panel(&self) -> Option<&TerminalPanel> {
        self.current_child()
            .map(|c| c.downcast_ref::<TerminalPanel>().unwrap())
    }
}

impl TerminalEmulator {
    #[inline]
    fn handle_session_finished(&mut self, panel_id: ObjectId, id: SessionPropsId) {
        self.index_map.remove(&id);
        if let Some(ids) = self.session_id_map.get_mut(&panel_id) {
            ids.retain(|i| *i != id);
        }
    }

    #[inline]
    fn handle_session_panel_finished(&mut self, id: ObjectId) {
        self.remove_children(id);

        let panel_id = match self.cur_terminal_panel() {
            Some(cur) => cur.id(),
            None => return,
        };

        let session_id = match self.session_id_map.get(&panel_id) {
            Some(ids) => match ids.first().copied() {
                Some(id) => id,
                None => return,
            },
            None => return,
        };

        self.cur_terminal_panel_mut()
            .unwrap()
            .set_session_focus(session_id);
    }
}
