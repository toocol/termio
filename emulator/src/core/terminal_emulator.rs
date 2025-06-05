#![allow(dead_code)]
use std::cell::RefCell;

use crate::{core::terminal_panel::TerminalPanelSignals, pty::Pty};

use super::terminal_panel::TerminalPanel;
use cli::{constant::ProtocolType, scheme::ColorScheme, session::SessionPropsId};
use derivative::Derivative;
use log::warn;
use nohash_hasher::IntMap;
use tlib::signals;
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
    index_map: IntMap<ObjectId, usize>,
    session_id_map: IntMap<ObjectId, Vec<SessionPropsId>>,
}

pub trait TerminalEmulatorTrait: ActionExt {
    signals!(
        TerminalEmulator:

        session_finished(SessionPropsId);

        session_panel_finished(ObjectId);
    );
}
impl TerminalEmulatorTrait for TerminalEmulator {}

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

        let panel_id = terminal_panel.id();
        self.add_child(terminal_panel);

        self.switch_index(self.children().len() - 1);

        let index = self.current_index;
        self.index_map.insert(panel_id, index);

        if let Some(terminal_panel) = self.cur_terminal_panel_mut() {
            terminal_panel.create_session(id, protocol_type);
            terminal_panel.set_session_focus(id);
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

        let panel_id = terminal_panel.id();
        self.add_child(terminal_panel);

        self.switch();

        let index = self.current_index;
        self.index_map.insert(panel_id, index);

        if let Some(terminal_panel) = self.cur_terminal_panel_mut() {
            terminal_panel.create_custom_session(id, custom_pty);
            terminal_panel.set_session_focus(id);
        }
    }

    #[inline]
    pub fn switch_session(&mut self, id: SessionPropsId) {
        if let Some(idx) = self.find_session_index(id) {
            self.switch_index(idx);

            if let Some(cur_terminal_panel) = self.cur_terminal_panel_mut() {
                cur_terminal_panel.set_session_focus(id);
            } else {
                warn!("[TerminalEmulator::switch_session] Current terminal panel is None.");
            }
        } else {
            warn!(
                "[TerminalEmulator::switch_session] Get index with session id {} is None.",
                id
            );
        }
    }

    #[inline]
    pub fn remove_session(&mut self, id: SessionPropsId) {
        if let Some(terminal_panel) = self.find_session_panel(id) {
            terminal_panel.close_session(id);
        } else {
            warn!(
                "[TerminalEmulator::remove_session] find session panel with session id {} is None.",
                id
            )
        }
    }

    #[inline]
    pub fn set_blinking_cursor(&mut self, id: SessionPropsId, blink: bool) {
        if let Some(terminal_panel) = self.cur_terminal_panel_mut() {
            terminal_panel.set_blinking_cursor(id, blink);
        } else {
            warn!("[TerminalEmulator::set_blinking_cursor] get current terminal panel is None.")
        }
    }

    #[inline]
    pub fn set_use_local_display(&mut self, id: SessionPropsId, use_local_display: bool) {
        if let Some(terminal_panel) = self.cur_terminal_panel_mut() {
            terminal_panel.set_use_local_display(id, use_local_display);
        } else {
            warn!("[TerminalEmulator::set_use_local_display] get current terminal panel is None.")
        }
    }

    #[inline]
    pub fn set_color_scheme(&mut self, theme: &ColorScheme) {
        self.set_background(theme.background_color());
        if let Some(terminal_panel) = self.cur_terminal_panel_mut() {
            terminal_panel.set_color_scheme(theme);
        } else {
            warn!("[TerminalEmulator::set_color_scheme] get current terminal panel is None.")
        }
    }

    #[inline]
    pub fn set_terminal_font(&mut self, font: Font) {
        if let Some(terminal_panel) = self.cur_terminal_panel_mut() {
            terminal_panel.set_terminal_font(font);
        } else {
            warn!("[TerminalEmulator::set_terminal_font] get current terminal panel is None.")
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
        if let Some(ids) = self.session_id_map.get_mut(&panel_id) {
            ids.retain(|i| *i != id);
        }

        emit!(self, session_finished(id));
    }

    #[inline]
    fn handle_session_panel_finished(&mut self, id: ObjectId) {
        let idx = self.index_map.remove(&id).unwrap_or_else(|| {
            panic!(
                "[TerminalEmulator::handle_session_finished] remove with session id {} is None",
                id
            )
        });

        self.remove_index(idx);

        let mut new_indexs = vec![];
        for (id, index) in self.index_map.iter() {
            if *index > idx {
                new_indexs.push((*id, *index - 1));
            }
        }
        for (id, index) in new_indexs {
            self.index_map.insert(id, index);
        }

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

        emit!(self, session_panel_finished(id));
    }

    fn find_session_index(&self, session_id: SessionPropsId) -> Option<usize> {
        let mut session_panel_id = None;
        for (panel_id, ids) in self.session_id_map.iter() {
            for id in ids {
                if *id == session_id {
                    session_panel_id = Some(*panel_id)
                }
            }
        }

        if let Some(id) = session_panel_id {
            self.index_map.get(&id).copied()
        } else {
            None
        }
    }

    fn find_session_panel(&self, session_id: SessionPropsId) -> Option<&mut TerminalPanel> {
        let mut session_panel_id = None;
        for (panel_id, ids) in self.session_id_map.iter() {
            for id in ids {
                if *id == session_id {
                    session_panel_id = Some(*panel_id)
                }
            }
        }

        if let Some(id) = session_panel_id {
            Some(
                ApplicationWindow::window()
                    .find_id_mut(id)
                    .unwrap()
                    .downcast_mut::<TerminalPanel>()
                    .unwrap(),
            )
        } else {
            None
        }
    }
}
