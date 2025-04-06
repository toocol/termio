#![allow(dead_code)]
pub mod translator_manager;
pub mod translator_reader;

use tmui::{
    prelude::{StaticType, ToValue},
    tlib::{
        implements_enum_value,
        namespace::{AsNumeric, KeyCode, KeyboardModifier},
        values::{FromBytes, FromValue, ToBytes},
        Type, Value,
    },
};
pub use translator_manager::*;
pub use translator_reader::*;

use lazy_static::lazy_static;
use regex::Regex;
use std::{collections::HashMap, mem::size_of, rc::Rc};

lazy_static! {
    pub static ref TITLE_REGEX: Regex = Regex::new("keyboard\\s+\"(.*)\"").unwrap();
    pub static ref KEY_REGEX: Regex =
        Regex::new("key\\s+([\\w\\+\\s\\-\\*\\.]+)\\s*:\\s*(\"(.*)\"|\\w+)").unwrap();
}

#[cfg(target_os = "windows")]
pub const CTRL_MODIFIER: KeyboardModifier = KeyboardModifier::ControlModifier;
#[cfg(target_os = "linux")]
pub const CTRL_MODIFIER: KeyboardModifier = KeyboardModifier::ControlModifier;
#[cfg(target_os = "macos")]
pub const CTRL_MODIFIER: KeyboardModifier = KeyboardModifier::MetaModifier;

lazy_static::lazy_static! {
    static ref DEFAULT_TRANSLATOR_TEXT: &'static [u8] = {
        "keyboard \"Fallback Key Translator\"\n
key Tab : \"\\t\"".as_bytes()
    };
}

#[inline]
fn one_or_zero(value: bool) -> u8 {
    if value {
        1
    } else {
        0
    }
}

#[inline]
fn is_printable_char(ch: u8) -> bool {
    (32..127).contains(&ch)
}

#[inline]
fn is_letter_or_number(ch: u8) -> bool {
    ch.is_ascii_lowercase() || ch.is_ascii_uppercase() || ch.is_ascii_digit()
}

#[inline]
fn is_xdigit(ch: u8) -> bool {
    ch.is_ascii_digit() || (b'A'..=b'F').contains(&ch) || (b'a'..=b'f').contains(&ch)
}

/// A convertor which maps between key sequences pressed by the user and the
/// character strings which should be sent to the terminal and commands
/// which should be invoked when those character sequences are pressed.
///
/// Supports multiple keyboard translators, allowing the user to
/// specify the character sequences which are sent to the terminal when particular key sequences are pressed.
///
/// A key sequence is defined as a key code, associated keyboard modifiers (Shift,Ctrl,Alt,Meta etc.)
/// and state flags which indicate the state which the terminal must be in for the key sequence to apply.
#[repr(u8)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum State {
    /// Indicates that no special state is active.
    #[default]
    None,
    /// Indicates that terminal is in new line state.
    NewLine,
    /// Indicates that the terminal is in 'Ansi' mode.
    Ansi,
    /// Indicates that the terminal is in cursor key state.
    CursorKeys,
    /// Indicates that the alternate screen ( typically used by interactive
    /// programs such as screen or vim ) is active
    AlternateScreen,
    /// Indicates that any of the modifier keys is active.
    AnyModifier,
    /// Indicates that the numpad is in application mode.
    ApplicationKeypad,
    /// State combinations.
    Combination(u8),
}
impl State {
    #[inline]
    pub fn or(&self, other: Self) -> Self {
        let one = self.as_u8();
        let other = other.as_u8();
        Self::Combination(one | other)
    }

    #[inline]
    pub fn has(&self, has: Self) -> bool {
        match self {
            Self::Combination(state) => state & has.as_u8() != 0,
            _ => *self == has,
        }
    }

    #[inline]
    pub fn as_u8(&self) -> u8 {
        match self {
            Self::None => 0,
            Self::NewLine => 1,
            Self::Ansi => 2,
            Self::CursorKeys => 4,
            Self::AlternateScreen => 8,
            Self::AnyModifier => 16,
            Self::ApplicationKeypad => 32,
            Self::Combination(state) => *state,
        }
    }
}
impl From<State> for u8 {
    #[inline]
    fn from(val: State) -> Self {
        val.as_u8()
    }
}
impl From<u8> for State {
    fn from(value: u8) -> Self {
        match value {
            0 => Self::None,
            1 => Self::NewLine,
            2 => Self::Ansi,
            4 => Self::CursorKeys,
            8 => Self::AlternateScreen,
            16 => Self::AnyModifier,
            32 => Self::ApplicationKeypad,
            _ => Self::Combination(value),
        }
    }
}

/// This enum describes commands which are associated with particular key sequences.
#[repr(C)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum Command {
    /// Indicates that no command is associated with this command sequence.
    #[default]
    None,
    /// Send command.
    Send,
    /// Scroll the terminal display up one page.
    ScrollPageUp,
    /// Scroll the terminal display down one page.
    ScrollPageDown,
    /// Scroll the terminal display up one line.
    ScrollLineUp,
    /// Scroll the terminal display down one line.
    ScrollLineDown,
    /// Toggles scroll lock mode.
    ScrollLock,
    /// Scroll the terminal display up to the start of history.
    ScrollUpToTop,
    /// Scroll the terminal display down to the end of history.
    ScrollDownToBottom,
    /// Echos the operating system specific erase character.
    Erase,
    Combination(u16),
}
impl Command {
    #[inline]
    pub fn or(&self, other: Self) -> Self {
        let one = self.as_u16();
        let other = other.as_u16();
        Self::Combination(one | other)
    }

    #[inline]
    pub fn has(&self, has: Self) -> bool {
        match self {
            Self::Combination(cmd) => cmd & has.as_u16() != 0,
            _ => *self == has,
        }
    }

    #[inline]
    pub fn as_u16(&self) -> u16 {
        match self {
            Self::None => 0,
            Self::Send => 1,
            Self::ScrollPageUp => 2,
            Self::ScrollPageDown => 4,
            Self::ScrollLineUp => 8,
            Self::ScrollLineDown => 16,
            Self::ScrollLock => 32,
            Self::ScrollUpToTop => 64,
            Self::ScrollDownToBottom => 128,
            Self::Erase => 256,
            Self::Combination(x) => *x,
        }
    }
}
impl From<Command> for u16 {
    #[inline]
    fn from(val: Command) -> Self {
        val.as_u16()
    }
}
impl From<u16> for Command {
    fn from(x: u16) -> Self {
        match x {
            0 => Self::None,
            1 => Self::Send,
            2 => Self::ScrollPageUp,
            4 => Self::ScrollPageDown,
            8 => Self::ScrollLineUp,
            16 => Self::ScrollLineDown,
            32 => Self::ScrollLock,
            64 => Self::ScrollUpToTop,
            128 => Self::ScrollDownToBottom,
            256 => Self::Erase,
            _ => Self::Combination(x),
        }
    }
}
impl AsNumeric<u16> for Command {
    #[inline]
    fn as_numeric(&self) -> u16 {
        self.as_u16()
    }
}
implements_enum_value!(Command, u16);

/// Represents an association between a key sequence pressed by the user
/// and the character sequence and commands associated with it for a particular KeyboardTranslator.
#[derive(Debug, PartialEq, Eq)]
pub struct Entry {
    key_code: u32,
    modifiers: KeyboardModifier,
    modifier_mask: KeyboardModifier,

    state: State,
    state_mask: State,

    command: Command,
    text: Vec<u8>,

    is_null: bool,
}

impl Default for Entry {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl Entry {
    pub fn new() -> Self {
        Self {
            key_code: 0,
            modifiers: KeyboardModifier::NoModifier,
            modifier_mask: KeyboardModifier::NoModifier,
            state: State::None,
            state_mask: State::None,
            command: Command::None,
            text: vec![],
            is_null: true,
        }
    }

    /// Returns true if this entry is null.
    /// This is true for newly constructed entries which have no properties set.
    #[inline]
    pub fn is_null(&self) -> bool {
        self.is_null
    }

    /// Returns the commands associated with this entry
    #[inline]
    pub fn command(&self) -> Command {
        self.command
    }

    /// Sets the command associated with this entry.
    #[inline]
    pub fn set_command(&mut self, command: Command) {
        self.command = command
    }

    /// Returns the character sequence associated with this entry, optionally
    /// replacing wildcard '*' characters with numbers to indicate the keyboard
    /// modifiers being pressed.
    ///
    ///
    /// @param expandWildCards Specifies whether wild cards (occurrences of the
    /// '*' character) in the entry should be replaced with a number to indicate
    // the modifier keys being pressed.
    ///
    /// @param modifiers The keyboard modifiers being pressed.
    pub fn text(
        &self,
        expand_wild_cards: Option<bool>,
        modifiers: Option<KeyboardModifier>,
    ) -> Vec<u8> {
        let expand_wild_cards = expand_wild_cards.is_some();
        let modifiers = modifiers.unwrap_or(KeyboardModifier::NoModifier);
        let mut expand_text = self.text.clone();

        if expand_wild_cards {
            let mut modifier_value = 1u8;
            modifier_value += one_or_zero(modifiers.has(KeyboardModifier::ShiftModifier));
            modifier_value += one_or_zero(modifiers.has(KeyboardModifier::AltModifier)) << 1;
            modifier_value += one_or_zero(modifiers.has(CTRL_MODIFIER)) << 2;

            // for i in 0..self.text.len() {
            for et in expand_text.iter_mut().take(self.text.len()) {
                if *et == b'*' {
                    *et = b'0' + modifier_value
                }
            }
        }
        expand_text
    }

    /// Sets the character sequence associated with this entry.
    #[inline]
    pub fn set_text(&mut self, text: Vec<u8>) {
        self.text = self.unescape(text)
    }

    ///  Returns the character sequence associated with this entry,
    /// with any non-printable characters replaced with escape sequences.
    ///
    /// eg. \\E for Escape, \\t for tab, \\n for new line.
    ///
    /// @param expandWildCards See text()
    /// @param modifiers See text()
    pub fn escaped_text(
        &self,
        expand_wild_cards: Option<bool>,
        modifiers: Option<KeyboardModifier>,
    ) -> Vec<u8> {
        let mut result = self.text(expand_wild_cards, modifiers);
        let mut i = 0usize;

        loop {
            if i >= result.len() {
                break;
            }

            let ch = result[i];
            let replacement = match ch {
                8 => b'b',
                9 => b't',
                10 => b'n',
                13 => b'r',
                12 => b'f',
                27 => b'E',
                //any character which is not printable is replaced by an equivalent
                // \xhh escape sequence (where 'hh' are the corresponding hex digits)
                _ => {
                    if is_printable_char(ch) {
                        0
                    } else {
                        b'x'
                    }
                }
            };

            if replacement == b'x' {
                let hex_str = hex::encode([ch]);
                let hex = hex_str.as_bytes();
                result[i] = b'\\';
                result.insert(i + 1, b'x');
                result.insert(i + 2, hex[0]);
                result.insert(i + 3, hex[1]);
            } else if replacement != 0 {
                result.remove(i);
                result.insert(i, b'\\');
                result.insert(i + 1, replacement);
            }
            i += 1;
        }

        result
    }

    /// Returns the character code ( from the tlib::[`KeyCode`] enum ) associated with this entry.
    #[inline]
    pub fn key_code(&self) -> u32 {
        self.key_code
    }

    /// Sets the character code associated with this entry.
    #[inline]
    pub fn set_key_code(&mut self, key_code: u32) {
        self.key_code = key_code
    }

    /// Returns a bitwise-OR of the enabled keyboard modifiers associated with
    /// this entry. If a modifier is set in modifierMask() but not in
    /// modifiers(), this means that the entry only matches when that modifier is not pressed.
    ///
    /// If a modifier is not set in modifierMask() then the entry matches whether
    /// the modifier is pressed or not.
    #[inline]
    pub fn modifiers(&self) -> KeyboardModifier {
        self.modifiers
    }

    /// Returns the keyboard modifiers which are valid in this entry. See modifiers().
    #[inline]
    pub fn modifier_mask(&self) -> KeyboardModifier {
        self.modifier_mask
    }

    /// Set the modifiers.
    #[inline]
    pub fn set_modifiers(&mut self, modifier: KeyboardModifier) {
        self.modifiers = modifier
    }

    /// Set the modifier mask.
    #[inline]
    pub fn set_modifier_mask(&mut self, modifier_mask: KeyboardModifier) {
        self.modifier_mask = modifier_mask
    }

    /// Returns a bitwise-OR of the enabled state flags associated with this
    /// entry. If flag is set in stateMask() but not in state(), this means that
    /// the entry only matches when the terminal is NOT in that state.
    ///
    /// If a state is not set in stateMask() then the entry matches whether the terminal is in that state or not.
    #[inline]
    pub fn state(&self) -> State {
        self.state
    }

    /// Returns the state flags which are valid in this entry.  See state()
    #[inline]
    pub fn state_mask(&self) -> State {
        self.state_mask
    }

    /// Set the state.
    #[inline]
    pub fn set_state(&mut self, state: State) {
        self.state = state
    }

    /// Set the state mask.
    #[inline]
    pub fn set_state_mask(&mut self, mask: State) {
        self.state_mask = mask
    }

    /// Returns this entry's conditions ( ie. its key code, modifier and state criteria ) as a string.
    pub fn condition_to_string(&mut self) -> String {
        let mut result = KeyCode::from(self.key_code).name().to_string();

        self.insert_modifier(&mut result, KeyboardModifier::ShiftModifier);
        self.insert_modifier(&mut result, KeyboardModifier::ControlModifier);
        self.insert_modifier(&mut result, KeyboardModifier::AltModifier);
        self.insert_modifier(&mut result, KeyboardModifier::MetaModifier);
        self.insert_modifier(&mut result, KeyboardModifier::KeypadModifier);

        self.insert_state(&mut result, State::AlternateScreen);
        self.insert_state(&mut result, State::NewLine);
        self.insert_state(&mut result, State::Ansi);
        self.insert_state(&mut result, State::CursorKeys);
        self.insert_state(&mut result, State::AnyModifier);
        self.insert_state(&mut result, State::ApplicationKeypad);

        result
    }

    /// Returns this entry's result ( ie. its command or character sequence ) as a string.
    ///
    /// @param expandWildCards See text()
    /// @param modifiers See text()
    pub fn result_to_string(
        &self,
        expand_wild_cards: Option<bool>,
        modifiers: Option<KeyboardModifier>,
    ) -> String {
        if !self.text.is_empty() {
            String::from_utf8(self.escaped_text(expand_wild_cards, modifiers))
                .expect("Parse `text` to `String` failed.")
        } else if self.command == Command::Erase {
            "Erase".to_string()
        } else if self.command == Command::ScrollPageUp {
            "ScrollPageUpCommand".to_string()
        } else if self.command == Command::ScrollPageDown {
            "ScrollPageDownCommand".to_string()
        } else if self.command == Command::ScrollLineUp {
            "ScrollLineUp".to_string()
        } else if self.command == Command::ScrollLineDown {
            "ScrollLineDown".to_string()
        } else if self.command == Command::ScrollLock {
            "ScrollLock".to_string()
        } else if self.command == Command::ScrollUpToTop {
            "ScrollUpToTop".to_string()
        } else if self.command == Command::ScrollDownToBottom {
            "ScrollDownToBottom".to_string()
        } else {
            String::new()
        }
    }

    ///Returns true if this entry matches the given key sequence, specified
    /// as a combination of @p keyCode , @p modifiers and @p state.
    #[allow(unused_mut)]
    pub fn matches(&self, key_code: u32, modifiers: KeyboardModifier, flags: State) -> bool {
        let mut modifiers = modifiers;
        let mut flags = flags;
        #[cfg(target_os = "macos")]
        {
            // On Mac, arrow keys are considered part of keypad. Ignore that.
            modifiers = KeyboardModifier::from(
                modifiers.as_u32() & !KeyboardModifier::KeypadModifier.as_u32(),
            )
        }

        if self.key_code != key_code {
            return false;
        }
        if modifiers.as_u32() & self.modifier_mask.as_u32()
            != self.modifiers.as_u32() & self.modifier_mask.as_u32()
        {
            return false;
        }

        // if modifiers is non-zero, the 'any modifier' state is implicit
        if modifiers.as_u32() & !KeyboardModifier::KeypadModifier.as_u32() != 0 {
            flags = State::from(flags.as_u8() | State::AnyModifier.as_u8());
        }

        if flags.as_u8() & self.state_mask.as_u8() != self.state.as_u8() & self.state_mask.as_u8() {
            return false;
        }

        let any_modifiers_set = modifiers != KeyboardModifier::NoModifier
            && modifiers != KeyboardModifier::KeypadModifier;
        let want_any_modifier = self.state.as_u8() & State::AnyModifier.as_u8() != 0;
        if self.state_mask.as_u8() & State::AnyModifier.as_u8() != 0
            && want_any_modifier != any_modifiers_set
        {
            return false;
        }
        true
    }

    fn insert_modifier(&mut self, item: &mut String, modifier: KeyboardModifier) {
        if modifier.as_u32() & self.modifier_mask.as_u32() == 0 {
            return;
        }

        if modifier.as_u32() & self.modifiers.as_u32() != 0 {
            item.push('+');
        } else {
            item.push('-');
        }

        if modifier == KeyboardModifier::ShiftModifier {
            item.push_str("Shift")
        } else if modifier == KeyboardModifier::ControlModifier {
            item.push_str("Ctrl")
        } else if modifier == KeyboardModifier::AltModifier {
            item.push_str("Alt")
        } else if modifier == KeyboardModifier::MetaModifier {
            item.push_str("Meta")
        } else if modifier == KeyboardModifier::KeypadModifier {
            item.push_str("KeyPad");
        }
    }

    fn insert_state(&mut self, item: &mut String, state: State) {
        if state.as_u8() & self.state_mask.as_u8() == 0 {
            return;
        }

        if state.as_u8() & self.state.as_u8() != 0 {
            item.push('+');
        } else {
            item.push('-');
        }

        if state == State::AlternateScreen {
            item.push_str("AppScreen")
        } else if state == State::NewLine {
            item.push_str("NewLine")
        } else if state == State::Ansi {
            item.push_str("Ansi")
        } else if state == State::CursorKeys {
            item.push_str("AppCursorKeys")
        } else if state == State::AnyModifier {
            item.push_str("AnyModifier")
        } else if state == State::ApplicationKeypad {
            item.push_str("AppKaypad")
        }
    }

    fn unescape(&self, text: Vec<u8>) -> Vec<u8> {
        let mut result = text;
        let mut i = 9usize;
        loop {
            if i >= result.len() - 1 {
                break;
            }

            let ch = result[i];
            if ch == b'\\' {
                let mut replacement = 0u8;
                let mut erase_char = 1;
                let mut escaped_char = true;
                match result[i + 1] {
                    b'b' => replacement = 8,
                    b't' => replacement = 9,
                    b'n' => replacement = 10,
                    b'f' => replacement = 12,
                    b'r' => replacement = 13,
                    b'E' => replacement = 27,
                    b'x' => {
                        let mut hex_digits = [0u8; 2];
                        if i < result.len() - 2 && is_xdigit(result[i + 2]) {
                            hex_digits[0] = result[i + 2];
                        }
                        if i < result.len() - 3 && is_xdigit(result[i + 3]) {
                            hex_digits[1] = result[i + 3];
                        }
                        let char_val =
                            format!("{}{}", hex_digits[0] as char, hex_digits[1] as char);
                        let hex_byte = hex::decode(char_val).unwrap()[0];
                        replacement = hex_byte;
                        erase_char = 3;
                    }
                    _ => escaped_char = false,
                }

                if escaped_char {
                    result[i] = replacement;
                    for _ in 0..erase_char {
                        result.remove(i + 1);
                    }
                }
            }
            i += 1;
        }

        result
    }
}

//////////////////////////////////////////////////////////////////////////////////////
/// A convertor which maps between key sequences pressed by the user and the
/// character strings which should be sent to the terminal and commands
/// which should be invoked when those character sequences are pressed.
///
/// Konsole supports multiple keyboard translators, allowing the user to
/// specify the character sequences which are sent to the terminal
/// when particular key sequences are pressed.
///
/// A key sequence is defined as a key code, associated keyboard modifiers
/// (Shift,Ctrl,Alt,Meta etc.) and state flags which indicate the state
/// which the terminal must be in for the key sequence to apply.
//////////////////////////////////////////////////////////////////////////////////////
#[derive(Debug)]
pub struct KeyboardTranslator {
    entries: HashMap<u32, Vec<Rc<Entry>>>,
    name: String,
    description: String,
}

impl KeyboardTranslator {
    /// Constructs a new keyboard translator with the given @p name.
    pub fn new<T: ToString>(name: T) -> Self {
        Self {
            entries: HashMap::new(),
            name: name.to_string(),
            description: String::new(),
        }
    }

    /// Returns the name of this keyboard translator.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Sets the name of this keyboard translator.
    pub fn set_name(&mut self, name: String) {
        self.name = name
    }

    /// Returns the descriptive name of this keyboard translator.
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Sets the descriptive name of this keyboard translator.
    pub fn set_description(&mut self, description: String) {
        self.description = description
    }

    /// Looks for an entry in this keyboard translator which matches the given key code, keyboard modifiers and state flags.
    ///
    /// Returns the matching entry if found or a null Entry otherwise ( ie. entry.isNull() will return true )
    ///
    /// @param keyCode A key code from the Qt::Key enum
    /// @param modifiers A combination of modifiers
    /// @param state Optional flags which specify the current state of the terminal
    pub fn find_entry(
        &self,
        key_code: u32,
        modifiers: KeyboardModifier,
        state: Option<State>,
    ) -> Rc<Entry> {
        let state = state.unwrap_or(State::None);
        for it in self.entries.iter() {
            if *it.0 == key_code {
                for en in it.1.iter() {
                    if en.matches(key_code, modifiers, state) {
                        return en.clone();
                    }
                }
            }
        }
        Rc::new(Entry::new())
    }

    /// Adds an entry to this keyboard translator's table.  Entries can be looked
    /// up according to their key sequence using findEntry()
    pub fn add_entry(&mut self, entry: Entry) {
        let key_code = entry.key_code();
        self.entries
            .entry(key_code)
            .or_default()
            .push(Rc::new(entry));
    }

    /// Replaces an entry in the translator.  If the @p existing entry is null,
    /// then this is equivalent to calling addEntry(@p replacement)
    pub fn replace_entry(&mut self, existing: Entry, replacement: Entry) {
        if !existing.is_null {
            if let Some(es) = self.entries.get_mut(&existing.key_code) {
                es.retain(|e| !(existing == **e));
                es.push(Rc::new(replacement));
            }
        }
    }

    /// Removes an entry from the table.
    pub fn remove_entry(&mut self, entry: Rc<Entry>) {
        if !entry.is_null {
            if let Some(es) = self.entries.get_mut(&entry.key_code) {
                es.retain(|e| !(*entry == **e));
            }
        }
    }

    /// Returns a list of all entries in the translator.
    pub fn entries(&self) -> Vec<Rc<Entry>> {
        let mut entries = vec![];
        for es in self.entries.iter() {
            for e in es.1.iter() {
                entries.push(e.clone())
            }
        }
        entries
    }
}

#[cfg(test)]
mod tests {
    use super::{KEY_REGEX, TITLE_REGEX};

    #[test]
    fn test_regex() {
        let line = "keyboard \"Default (XFree 4)\"";
        let s = TITLE_REGEX.captures(line).unwrap().get(1).unwrap().as_str();
        println!("{}", s);

        let line = "key Right-Shift-Ansi : \"\\EC\"";
        let caps = KEY_REGEX.captures(line).unwrap();
        let s = caps.get(0).unwrap().as_str();
        println!("0: {}", s);
        let s = caps.get(1).unwrap().as_str();
        println!("1: {}", s);
        let s = caps.get(2).unwrap().as_str();
        println!("2: {}", s);
        let s = caps.get(3).unwrap().as_str();
        println!("3: {}", s);
    }
}
