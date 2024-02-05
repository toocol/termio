use super::{HistoryScroll, HistoryTypeNone};
use crate::tools::character::Character;
use std::{rc::Rc, cell::RefCell};

///////////////////////////////////////////////////////////////////////
// History Scroll None (No history)
///////////////////////////////////////////////////////////////////////
pub struct HistoryScrollNone {
    history_type: Rc<RefCell<HistoryTypeNone>>,
}
impl HistoryScrollNone {
    pub fn new() -> Self {
        Self {
            history_type: Rc::new(RefCell::new(HistoryTypeNone::new())),
        }
    }
}
impl HistoryScroll for HistoryScrollNone {
    type HistoryType = HistoryTypeNone;

    fn has_scroll(&self) -> bool {
        false
    }

    fn get_lines(&self) -> i32 {
        0
    }

    fn get_line_len(&mut self, _lineno: i32) -> i32 {
        0
    }

    fn get_cells(&mut self, _lineno: i32, _colno: i32, _count: i32, _res: &mut [Character]) {}

    fn is_wrapped_line(&mut self, _lineno: i32) -> bool {
        false
    }

    fn add_cells(&mut self, _character: &[Character], _count: i32) {}

    fn add_line(&mut self, _previous_wrapped: bool) {}

    fn get_type(&self) -> Rc<RefCell<Self::HistoryType>> {
        self.history_type.clone()
    }
}
