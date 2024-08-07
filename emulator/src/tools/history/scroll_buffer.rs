use super::{HistoryScroll, HistoryTypeBuffer};
use crate::tools::character::Character;
use bitvec::vec::BitVec;
use libc::{c_void, memcpy, memset};
use std::{cell::RefCell, mem::size_of, rc::Rc};

////////////////////////////////////////////////////////////////////////
// Buffer-based history (limited to a fixed nb of lines)
////////////////////////////////////////////////////////////////////////
type HistoryLine = Vec<Character>;
pub struct HistoryScrollBuffer {
    history_type: Rc<RefCell<HistoryTypeBuffer>>,

    history_buffer: Vec<HistoryLine>,
    wrapped_line: BitVec,
    max_line_count: i32,
    used_lines: i32,
    head: i32,
}
impl HistoryScrollBuffer {
    pub fn new(max_nb_lines: Option<usize>) -> Self {
        let max_nb_lines = max_nb_lines.unwrap_or(1000);

        let mut scroll = Self {
            history_type: Rc::new(RefCell::new(HistoryTypeBuffer::new(max_nb_lines))),
            history_buffer: vec![],
            wrapped_line: BitVec::new(),
            max_line_count: 0,
            used_lines: 0,
            head: 0,
        };
        scroll.set_max_nb_lines(max_nb_lines);
        scroll
    }

    fn buffer_index(&self, line_number: i32) -> usize {
        assert!(line_number < self.max_line_count);
        assert!(self.used_lines == self.max_line_count || line_number <= self.head);

        if self.used_lines == self.max_line_count {
            ((self.head + line_number + 1) % self.max_line_count) as usize
        } else {
            line_number as usize
        }
    }

    pub fn max_nb_lines(&self) -> i32 {
        self.max_line_count
    }
}
impl HistoryScroll for HistoryScrollBuffer {
    type HistoryType = HistoryTypeBuffer;

    fn has_scroll(&self) -> bool {
        true
    }

    fn get_lines(&self) -> i32 {
        self.used_lines
    }

    fn get_line_len(&mut self, lineno: i32) -> i32 {
        assert!(lineno >= 0 && lineno < self.max_line_count);

        if lineno < self.used_lines {
            let buffer_index = self.buffer_index(lineno);
            self.history_buffer[buffer_index].len() as i32
        } else {
            0
        }
    }

    fn get_cells(&mut self, lineno: i32, colno: i32, count: i32, res: &mut [Character]) {
        if count == 0 {
            return;
        }

        assert!(lineno < self.max_line_count);

        if lineno >= self.used_lines {
            unsafe {
                memset(
                    res as *mut [Character] as *mut c_void,
                    0,
                    count as usize * size_of::<Character>(),
                )
            };
            return;
        }

        let buffer_index = self.buffer_index(lineno);
        let line = &self.history_buffer[buffer_index];

        assert!(colno <= line.len() as i32 - count);

        unsafe {
            memcpy(
                res as *mut [Character] as *mut c_void,
                &line[colno as usize..] as *const [Character] as *const c_void,
                count as usize * size_of::<Character>(),
            )
        };
    }

    fn is_wrapped_line(&mut self, lineno: i32) -> bool {
        assert!(lineno >= 0 && lineno < self.max_line_count);

        if lineno < self.used_lines {
            let buffer_index = self.buffer_index(lineno);
            let opt = self.wrapped_line.get(buffer_index);
            if let Some(bit) = opt {
                bit == true
            } else {
                false
            }
        } else {
            false
        }
    }

    fn add_cells(&mut self, character: &[Character], _count: i32) {
        let new_line = character.to_vec();
        self.add_cells_list(new_line);
    }

    fn add_cells_list(&mut self, list: Vec<Character>) {
        self.head += 1;
        if self.used_lines < self.max_line_count {
            self.used_lines += 1;
        }
        if self.head >= self.max_line_count {
            self.head = 0;
        }

        let buffer_index = self.buffer_index(self.used_lines - 1);
        self.history_buffer[buffer_index] = list;
        self.wrapped_line.set(buffer_index, false);
    }

    fn add_line(&mut self, previous_wrapped: bool) {
        let buffer_index = self.buffer_index(self.used_lines - 1);
        self.wrapped_line.set(buffer_index, previous_wrapped);
    }

    fn get_type(&self) -> Rc<RefCell<Self::HistoryType>> {
        self.history_type.clone()
    }

    fn set_max_nb_lines(&mut self, nb_lines: usize) {
        let old_buffer = &self.history_buffer;
        let mut new_buffer = vec![vec![]; nb_lines];

        for (i, nb) in new_buffer
            .iter_mut()
            .enumerate()
            .take((self.used_lines as usize).min(nb_lines))
        {
            *nb = old_buffer
                .get(self.buffer_index(i as i32))
                .unwrap()
                .to_owned()
        }

        self.used_lines = self.used_lines.min(nb_lines as i32);
        self.max_line_count = nb_lines as i32;
        self.head = if self.used_lines == self.max_line_count {
            0
        } else {
            self.used_lines - 1
        };

        self.history_buffer = new_buffer;
        self.wrapped_line.resize(nb_lines, false);
        self.get_type().borrow_mut().nb_lines = nb_lines;
    }
}
