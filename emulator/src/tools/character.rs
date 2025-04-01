#![allow(dead_code)]
use lazy_static::lazy_static;
use libc::wchar_t;
use std::{
    cell::RefCell,
    collections::HashMap,
    ptr::null_mut,
    sync::atomic::{AtomicPtr, Ordering},
};
use wchar::wch;

use super::character_color::{
    CharacterColor, ColorEntry, FontWeight, BASE_COLORS, COLOR_SPACE_DEFAULT, COLOR_SPACE_SYSTEM,
    DEFAULT_BACK_COLOR, DEFAULT_FORE_COLOR,
};
pub type LineProperty = u8;

pub const LINE_DEFAULT: u8 = 0;
pub const LINE_WRAPPED: u8 = 1 << 0;
pub const LINE_DOUBLE_WIDTH: u8 = 1 << 1;
pub const LINE_DOUBLE_HEIGHT: u8 = 1 << 2;

pub const DEFAULT_RENDITION: wchar_t = 0;
pub const RE_BOLD: wchar_t = 1 << 0;
pub const RE_BLINK: wchar_t = 1 << 1;
pub const RE_UNDERLINE: wchar_t = 1 << 2;
pub const RE_REVERSE: wchar_t = 1 << 3; // screen only
pub const RE_INTENSIVE: wchar_t = 1 << 3; // widget only
pub const RE_ITALIC: wchar_t = 1 << 4;
pub const RE_CURSOR: wchar_t = 1 << 5;
pub const RE_EXTEND_CHAR: wchar_t = 1 << 6;
pub const RE_FAINT: wchar_t = 1 << 7;
pub const RE_STRIKEOUT: wchar_t = 1 << 8;
pub const RE_CONCEAL: wchar_t = 1 << 9;
pub const RE_OVERLINE: wchar_t = 1 << 10;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CharacterUnion {
    /// The unicode character value for this character.
    Character(wchar_t),
    /// Experimental addition which allows a single Character instance to contain
    /// more than one unicode character.
    ///
    /// charSequence is a hash code which can be used to look up the unicode
    /// character sequence in the ExtendedCharTable used to create the sequence.
    CharSequence(wchar_t),
}
impl CharacterUnion {
    pub fn equals(&self, data: wchar_t) -> bool {
        match self {
            Self::Character(wch) => *wch == data,
            Self::CharSequence(seq) => *seq == data,
        }
    }

    pub fn data(&self) -> wchar_t {
        match self {
            Self::Character(wch) => *wch,
            Self::CharSequence(seq) => *seq,
        }
    }

    pub fn set_data(&mut self, data: wchar_t) {
        match self {
            Self::Character(ch) => *ch = data,
            Self::CharSequence(seq) => *seq = data,
        }
    }
}
impl Default for CharacterUnion {
    fn default() -> Self {
        Self::Character(wch!(' '))
    }
}
impl From<CharacterUnion> for wchar_t {
    fn from(val: CharacterUnion) -> Self {
        match val {
            CharacterUnion::Character(ch) => ch,
            CharacterUnion::CharSequence(seq) => seq,
        }
    }
}
impl From<wchar_t> for CharacterUnion {
    fn from(x: wchar_t) -> Self {
        Self::Character(x)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct Character {
    // The union of character, is one of `Character` or `CharSequence`
    pub character_union: CharacterUnion,
    /// A combination of `rendition` flags which specify options for drawing the character.
    pub rendition: wchar_t,
    /// The foreground color used to draw this character. */
    pub foreground_color: CharacterColor,
    /// The color used to draw this character's background. */
    pub background_color: CharacterColor,
}

impl Default for Character {
    fn default() -> Self {
        Self {
            character_union: Default::default(),
            rendition: DEFAULT_RENDITION,
            foreground_color: CharacterColor::new(COLOR_SPACE_DEFAULT, DEFAULT_FORE_COLOR),
            background_color: CharacterColor::new(COLOR_SPACE_DEFAULT, DEFAULT_BACK_COLOR),
        }
    }
}

impl Character {
    pub fn new(c: wchar_t, f: CharacterColor, b: CharacterColor, r: wchar_t) -> Self {
        Self {
            character_union: CharacterUnion::Character(c),
            rendition: r,
            foreground_color: f,
            background_color: b,
        }
    }

    /// Returns true if this character has a transparent background when
    /// it is drawn with the specified @p palette.
    #[inline]
    pub fn is_transparent(&self, palette: &[ColorEntry]) -> bool {
        ((self.background_color.color_space == COLOR_SPACE_DEFAULT)
            && palette[self.background_color.u as usize
                + (if self.background_color.v > 0 {
                    BASE_COLORS
                } else {
                    0
                })]
            .transparent)
            || ((self.background_color.color_space == COLOR_SPACE_SYSTEM)
                && palette[self.background_color.u as usize
                    + 2
                    + (if self.background_color.v > 0 {
                        BASE_COLORS
                    } else {
                        0
                    })]
                .transparent)
    }

    /// Returns true if this character should always be drawn in bold when
    /// it is drawn with the specified @p palette, independent of whether
    /// or not the character has the RE_BOLD rendition flag.
    #[inline]
    pub fn font_weight(&self, palette: &[ColorEntry]) -> FontWeight {
        if self.background_color.color_space == COLOR_SPACE_DEFAULT {
            palette[self.background_color.u as usize
                + (if self.background_color.v > 0 {
                    BASE_COLORS
                } else {
                    0
                })]
            .font_weight
        } else if self.background_color.color_space == COLOR_SPACE_SYSTEM {
            palette[self.background_color.u as usize
                + 2
                + (if self.background_color.v > 0 {
                    BASE_COLORS
                } else {
                    0
                })]
            .font_weight
        } else {
            FontWeight::UseCurrentFormat
        }
    }

    /// returns true if the format (color, rendition flag) of the compared
    /// characters is equal
    #[inline]
    pub fn equals_format(&self, other: &Character) -> bool {
        self.background_color == other.background_color
            && self.foreground_color == other.foreground_color
            && self.rendition == other.rendition
    }
}

/// A table which stores sequences of unicode characters, referenced
/// by hash keys.  The hash key itself is the same size as a unicode
/// character ( ushort ) so that it can occupy the same space in a structure.
#[derive(Debug, Default)]
pub struct ExtendedCharTable(
    /// internal, maps hash keys to character sequence buffers.  The first wchar_t
    /// in each value is the length of the buffer, followed by the ushorts in the buffer themselves.
    RefCell<HashMap<wchar_t, Vec<wchar_t>>>,
);

impl ExtendedCharTable {
    pub fn instance<'a>() -> &'a Self {
        unsafe { INSTANCE.load(Ordering::SeqCst).as_ref().unwrap() }
    }

    pub fn initialize(&mut self) {
        INSTANCE.store(self, Ordering::SeqCst);
    }

    /// Adds a sequences of unicode characters to the table and returns
    /// a hash code which can be used later to look up the sequence using lookupExtendedChar()
    ///
    /// If the same sequence already exists in the table, the hash of the existing sequence will be returned.
    ///
    /// @param unicodePoints An array of unicode character points
    /// @param length Length of @p unicodePoints
    pub fn create_extended_char(&self, unicode_points: &[wchar_t], length: wchar_t) -> wchar_t {
        // look for the sequence of points in the table
        let mut hash = self.extended_char_hash(unicode_points, length);

        // check existing entry of match
        while self.0.borrow().contains_key(&hash) {
            if self.extended_char_match(hash, unicode_points, length) {
                return hash;
            } else {
                hash += 1;
            }
        }

        // add the new sequence to the table and return that index.
        let mut buffer = vec![0 as wchar_t; (length + 1) as usize];
        buffer[0] = length;
        buffer[1..].copy_from_slice(&unicode_points[0..length as usize]);

        self.0.borrow_mut().insert(hash, buffer);

        hash
    }

    /// Looks up and returns a pointer to a sequence of unicode characters which was added to the table using createExtendedChar().
    ///
    /// @param hash The hash key returned by createExtendedChar()
    /// @param length This variable is set to the length of the character sequence.
    ///
    /// @return A unicode character sequence of size @p length.
    pub fn lookup_extended_char(
        &self,
        hash: wchar_t,
        length: &mut wchar_t,
    ) -> Option<Vec<wchar_t>> {
        // lookup index in table and if found, set the length
        // argument and return a reference to the character sequence
        let map = self.0.borrow();
        let buffer = map.get(&hash);
        if let Some(buffer) = buffer {
            *length = buffer[0];
            let mut ret = vec![wchar_t::default(); buffer.len() - 1];
            ret.copy_from_slice(&buffer[1..]);
            Some(ret)
        } else {
            *length = 0;
            None
        }
    }

    /// calculates the hash key of a sequence of unicode points of size 'length'
    fn extended_char_hash(&self, unicode_points: &[wchar_t], length: wchar_t) -> wchar_t {
        let mut hash = 0 as wchar_t;
        for &up in unicode_points.iter().take(length as usize) {
            hash = 31 * hash + up;
        }
        hash
    }

    /// tests whether the entry in the table specified by 'hash' matches the
    /// character sequence 'unicodePoints' of size 'length'
    fn extended_char_match(
        &self,
        hash: wchar_t,
        unicode_points: &[wchar_t],
        length: wchar_t,
    ) -> bool {
        let map = self.0.borrow();
        let entry = map.get(&hash);
        if let Some(entry) = entry {
            // compare given length with stored sequence length ( given as the first
            // ushort in the stored buffer )
            if entry[0] != length {
                return false;
            }
            // if the lengths match, each character must be checked.  the stored buffer
            // starts at entry[1]
            for i in 0..length as usize {
                if entry[i + 1] != unicode_points[i] {
                    return false;
                }
            }
            true
        } else {
            false
        }
    }
}

lazy_static! {
    static ref INSTANCE: AtomicPtr<ExtendedCharTable> = AtomicPtr::new(null_mut());
}
