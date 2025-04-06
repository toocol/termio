#![allow(unused_macros)]
use super::{data_sender::DataSender, BaseEmulation, Emulation};
use crate::{
    core::{
        screen::{
            MODES_SCREEN, MODE_CURSOR, MODE_INSERT, MODE_NEWLINE, MODE_ORIGIN, MODE_SCREEN,
            MODE_WRAP,
        },
        screen_window::ScreenWindow,
        terminal_view::KeyboardCursorShape,
        uwchar_t,
    },
    emulation::{EmulationSignal, EmulationState},
    tools::{
        character::{
            LINE_DOUBLE_HEIGHT, LINE_DOUBLE_WIDTH, RE_BLINK, RE_BOLD, RE_CONCEAL, RE_FAINT,
            RE_ITALIC, RE_OVERLINE, RE_REVERSE, RE_STRIKEOUT, RE_UNDERLINE,
        },
        character_color::{
            COLOR_SPACE_256, COLOR_SPACE_DEFAULT, COLOR_SPACE_RGB, COLOR_SPACE_SYSTEM,
            VT100_GRAPHICS,
        },
        event::KeyPressedEvent,
        history::HistoryType,
        terminal_character_decoder::TerminalCharacterDecoder,
        translators::{Command, KeyboardTranslatorManager, State, CTRL_MODIFIER},
    },
};
use std::{cell::RefCell, collections::HashMap, ptr::NonNull, rc::Rc};
use tmui::{
    clipboard::ClipboardLevel,
    prelude::*,
    system::System,
    tlib::{
        emit,
        figure::Size,
        impl_as_any,
        namespace::{AsNumeric, KeyCode, KeyboardModifier},
        nonnull_ref,
    },
};
use wchar::{wch, wchar_t};
use widestring::WideString;

/// Processing the incoming byte stream.
/// --------------------------------------------------------------------
/// Incoming Bytes Event pipeline
///
/// This section deals with decoding the incoming character stream.
/// Decoding means here, that the stream is first separated into `tokens'
/// which are then mapped to a `meaning' provided as operations by the`Screen' class or by the emulation class itself.
///
/// The pipeline proceeds as follows:
///
/// - Tokenizing the ESC codes (onReceiveChar). <br>
/// - VT100 code page translation of plain characters (applyCharset). <br>
/// - Interpretation of ESC codes (processToken). <br>
///
/// The escape codes and their meaning are described in the technical reference of this program.
/// (Doc comment on struct [`VT102Emulation`])
///
/// Tokens:
/// --------------------------------------------------------------------
///
/// Since the tokens are the central notion if this section, we've put them in front.
/// They provide the syntactical elements used to represent the terminals operations as byte sequences.
///
/// They are encodes here into a single machine word, so that we can later switch over them easily.
/// Depending on the token itself, additional argument variables are filled with parameter values.
///
/// Definitions: <br>
/// - C          - A single (required) character.
/// - Pn         - Numeric parameter
/// - Pt         - A text parameter composed of printable characters.
/// - CSI        - `Esc [` (Control Sequence Introducer, 0x9b)
///
/// The tokens are defined below:
///
/// - CHR        - Printable characters     (32..255 but DEL (=127)) <br>
/// - CTL        - Control characters       (0..31 but ESC (= 27), DEL)
/// - ESC        - Escape codes of the form <ESC><CHR but '[]()+*#'>
/// - ESC_DE     - Escape codes of the form <ESC><any of '()+*#%'> C
/// - CSI_PN     - Escape codes of the form <ESC>'['     {Pn} ';' {Pn} C
/// - CSI_PS     - Escape codes of the form <ESC>'['     {Pn} ';' ...  C
/// - CSI_PS_SP  - Escape codes of the form <ESC>'['     {Pn} ';' ... {Space} C
/// - CSI_PR     - Escape codes of the form <ESC>'[' '?' {Pn} ';' ...  C
/// - CSI_PE     - Escape codes of the form <ESC>'[' '!' {Pn} ';' ...  C
/// - VT52       - VT52 escape codes
///              - <ESC><Chr>
///              - <ESC>'Y'{Pc}{Pc}
/// - XTE_HA     - Xterm window/terminal attribute commands
///                of the form <ESC>']' {Pn} ';' {Pt} <BEL>
///                (Note that these are handled differently to the other formats
///                see also https://invisible-island.net/xterm/ctlseqs/ctlseqs.html#h3-Operating-System-Commands)
///
/// The last two forms allow list of arguments.
/// Since the elements of the lists are treated individually the same way,
/// they are passed as individual tokens to the interpretation. Further,
/// because the meaning of the parameters are names (althought represented as numbers),
/// they are includes within the token ('N').<br>
macro_rules! ty_construct {
    ( $t:expr, $a:expr, $n:expr ) => {
        (((($n as i32) & 0xffff) << 16) | ((($a as i32) & 0xff) << 8) | (($t as i32) & 0xff))
    };
}
macro_rules! ty_chr {
    () => {
        ty_construct!(0, 0, 0)
    };
}
macro_rules! ty_ctl {
    ( $a:expr ) => {
        ty_construct!(1, $a, 0)
    };
}
macro_rules! ty_esc {
    ( $a:expr ) => {
        ty_construct!(2, $a, 0)
    };
}
macro_rules! ty_esc_cs {
    ( $a:expr, $b:expr ) => {
        ty_construct!(3, $a, $b)
    };
}
macro_rules! ty_esc_de {
    ( $a:expr ) => {
        ty_construct!(4, $a, 0)
    };
}
macro_rules! ty_csi_ps {
    ( $a:expr, $n:expr ) => {
        ty_construct!(5, $a, $n)
    };
}
macro_rules! ty_csi_pn {
    ( $a:expr ) => {
        ty_construct!(6, $a, 0)
    };
}
macro_rules! ty_csi_pr {
    ( $a:expr, $n:expr ) => {
        ty_construct!(7, $a, $n)
    };
}
macro_rules! ty_csi_ps_sp {
    ( $a:expr, $n:expr ) => {
        ty_construct!(11, $a, $n)
    };
}

macro_rules! ty_vt52 {
    ( $a:expr ) => {
        ty_construct!(8, $a, 0)
    };
}
macro_rules! ty_csi_pg {
    ( $a:expr ) => {
        ty_construct!(9, $a, 0)
    };
}
macro_rules! ty_csi_pe {
    ( $a:expr ) => {
        ty_construct!(10, $a, 0)
    };
}

const MAX_ARGUMENT: usize = 4096;
const MAX_TOKEN_LENGTH: usize = 256;
const MAXARGS: usize = 15;

/// Mode #1.
const MODE_APP_SCREEN: usize = MODES_SCREEN;
/// Application cursor key (DECCKM).
const MODE_APP_CURSOR_KEY: usize = MODES_SCREEN + 1;
/// Application key pad.
const MODE_APP_KEY_PAD: usize = MODES_SCREEN + 2;
/// Send mouse X,Y position on press and release
const MODE_MOUSE_1000: usize = MODES_SCREEN + 3;
/// Use hilight mouse tracking.
const MODE_MOUSE_1001: usize = MODES_SCREEN + 4;
/// Use cell motion mouse tracking.
const MODE_MOUSE_1002: usize = MODES_SCREEN + 5;
/// Use all motion mouse tracking.
const MODE_MOUSE_1003: usize = MODES_SCREEN + 6;
/// Xterm-style extended coordinates.
const MODE_MOUSE_1005: usize = MODES_SCREEN + 7;
/// 2nd Xterm-style extended coordinates.
const MODE_MOUSE_1006: usize = MODES_SCREEN + 8;
/// Urxvt-style extended coordinates.
const MODE_MOUSE_1015: usize = MODES_SCREEN + 9;
/// Use US Ascii for character sets G0-G3 (DECANM).
const MODE_ANSI: usize = MODES_SCREEN + 10;
/// 80 <-> 132 column mode switch (DECCOLM).
const MODE_132_COLUMNS: usize = MODES_SCREEN + 11;
/// Allow DECCOLM mode.
const MODE_ALLOW_132_COLUMNS: usize = MODES_SCREEN + 12;
/// Xterm-style bracketed paste mode.
const MODE_BRACKETD_PASTE: usize = MODES_SCREEN + 13;
/// The total size of modes.
const MODE_TOTAL: usize = MODES_SCREEN + 14;

#[derive(Default)]
struct CharCodes {
    /// Coding information.
    charset: [u8; 4],
    /// Actual charset.
    current_charset: i32,
    /// Some VT100 tricks.
    graphic: bool,
    /// Some VT100 tricks.
    pound: bool,
    /// Saved graphic.
    saved_graphic: bool,
    /// Saved pound.
    saved_pound: bool,
}

#[derive(Default)]
struct TerminalState {
    mode: [bool; MODE_TOTAL],
}

/// The VT102 Emulation.
///--------------------------------------------------------------------------------------
/// Provides an `xterm` compatible terminal emulation based on the DEC `VT102`
/// terminal. A full description of this terminal can be found at
/// http://vt100.net/docs/vt102-ug/
///
/// In addition, various additional xterm escape sequences are supported to
/// provide features such as mouse input handling. See
/// http://rtfm.etla.org/xterm/ctlseq.html for a description of xterm's escape sequences.
pub struct VT102Emulation {
    emulation: Option<Box<BaseEmulation>>,
    token_buffer: [wchar_t; MAX_TOKEN_LENGTH],
    token_buffer_pos: usize,
    argv: [i32; MAXARGS],
    argc: i32,
    cc: wchar_t,
    prev_cc: wchar_t,
    /// Set of flags for each of the ASCII characters which indicates what category they fall into
    /// (printable character, control, digit etc.) for the purposes of decoding terminal output
    char_class: [i32; 256],
    charset: [CharCodes; 2],
    current_modes: TerminalState,
    saved_modes: TerminalState,
    pending_title_updates: HashMap<i32, String>,
    report_focus_event: bool,
    signal_source: Option<ObjectId>,
    // TODO: Add timer: title_update_timer
}
impl_as_any!(VT102Emulation);
impl ObjectOperation for VT102Emulation {
    #[inline]
    fn id(&self) -> ObjectId {
        self.emulation().id()
    }

    #[inline]
    fn set_property(&mut self, name: &str, value: Value) {
        self.emulation_mut().set_property(name, value)
    }

    #[inline]
    fn get_property(&self, name: &str) -> Option<&Value> {
        self.emulation().get_property(name)
    }

    #[inline]
    fn constructed(&self) -> bool {
        self.emulation().constructed()
    }

    #[inline]
    fn set_name(&mut self, name: &str) {
        self.emulation_mut().set_name(name)
    }

    #[inline]
    fn set_signal_source(&mut self, id: Option<ObjectId>) {
        self.signal_source = id;
    }

    #[inline]
    fn get_signal_source(&self) -> Option<ObjectId> {
        self.signal_source
    }
}
impl Default for VT102Emulation {
    #[inline]
    fn default() -> Self {
        Self {
            emulation: Default::default(),
            token_buffer: [0; MAX_TOKEN_LENGTH],
            token_buffer_pos: Default::default(),
            argv: [0; MAXARGS],
            argc: Default::default(),
            cc: Default::default(),
            prev_cc: Default::default(),
            char_class: [0; 256],
            charset: Default::default(),
            current_modes: Default::default(),
            saved_modes: Default::default(),
            pending_title_updates: Default::default(),
            report_focus_event: Default::default(),
            signal_source: None,
        }
    }
}
impl ActionExt for VT102Emulation {}
impl Reflect for VT102Emulation {
    #[inline]
    fn as_reflect(&self) -> &dyn Reflect {
        self
    }
    #[inline]
    fn as_reflect_mut(&mut self) -> &mut dyn Reflect {
        self
    }
    #[inline]
    fn as_reflect_boxed(self: Box<Self>) -> Box<dyn Reflect> {
        self
    }
}

//////////////////////// Character Class flags used while decoding
/// Control character
const CTL: i32 = 1;
/// Printable character
const CHR: i32 = 2;
/// The character used in control sequnce
const CPN: i32 = 4;
/// Numberic digit
const DIG: i32 = 8;
/// Special characters in "()+*%"
const SCS: i32 = 16;
/// Special characters in "()+*#[]%"
const GRP: i32 = 32;
/// Character('t') which indicates end of window resize (escape sequence '\e[8;<row>;<col>t')
const CPS: i32 = 64;

impl VT102Emulation {
    /// Constructer to create a new Emulation.
    #[allow(clippy::field_reassign_with_default)]
    pub fn new(translator_manager: Option<NonNull<KeyboardTranslatorManager>>) -> Self {
        let base_emulation = BaseEmulation::new(translator_manager);
        let mut vt102_emulation: VT102Emulation = Default::default();
        vt102_emulation.emulation = Some(base_emulation);

        vt102_emulation.init_tokenizer();
        vt102_emulation.reset();

        vt102_emulation
    }

    /// Wrap trait `Emulation` to `EmulationWrapper`.
    pub fn wrap(self) -> Box<dyn Emulation> {
        let mut wrapper: Box<dyn Emulation> = Box::new(self);
        wrapper.init();
        wrapper
    }

    #[inline]
    pub fn emulation(&self) -> &BaseEmulation {
        self.emulation.as_deref().unwrap()
    }
    #[inline]
    pub fn emulation_mut(&mut self) -> &mut BaseEmulation {
        self.emulation.as_deref_mut().unwrap()
    }

    //////////////////////////////////////////////////////// Private function
    fn init_tokenizer(&mut self) {
        for i in 0..256 {
            self.char_class[i] = 0;
        }
        for i in 0..32 {
            self.char_class[i] |= CTL;
        }
        for i in 32..256 {
            self.char_class[i] |= CHR;
        }
        for s in "@ABCDEFGHILMPSTXZbcdfry".as_bytes().iter() {
            self.char_class[*s as usize] |= CPN;
        }
        for s in "t".as_bytes().iter() {
            self.char_class[*s as usize] |= CPS;
        }
        for s in "0123456789".as_bytes().iter() {
            self.char_class[*s as usize] |= DIG;
        }
        for s in "()+*%".as_bytes().iter() {
            self.char_class[*s as usize] |= SCS;
        }
        for s in "()+*#[]%".as_bytes().iter() {
            self.char_class[*s as usize] |= GRP;
        }

        self.reset_tokenizer();
    }

    fn reset_tokenizer(&mut self) {
        self.token_buffer_pos = 0;
        self.argc = 0;
        self.argv[0] = 0;
        self.argv[1] = 0;
        self.cc = 0;
        self.prev_cc = 0;
    }

    #[allow(clippy::if_same_then_else)]
    #[inline]
    fn process_token(&mut self, token: i32, p: wchar_t, q: i32) {
        let current_screen = unsafe {
            self.emulation
                .as_mut()
                .unwrap()
                .current_screen
                .as_mut()
                .unwrap()
                .as_mut()
        };
        if token == ty_chr!() {
            // UTF-16
            unsafe {
                self.emulation
                    .as_mut()
                    .unwrap()
                    .current_screen
                    .as_mut()
                    .unwrap()
                    .as_mut()
                    .display_character(p)
            };
        //////////////////////////////////////////////////////////////
        } else if token == ty_ctl!('@') {
            // NUL: ignored
        } else if token == ty_ctl!('A') {
            // SOH: ignored
        } else if token == ty_ctl!('B') {
            // STX: ignored
        } else if token == ty_ctl!('C') {
            // ETX: ignored
        } else if token == ty_ctl!('D') {
            // EOT: ignored
        } else if token == ty_ctl!('E') {
            // VT100
            self.report_answer_back();
        } else if token == ty_ctl!('F') {
            // ACK: ignored
        } else if token == ty_ctl!('G') {
            // VT100
            emit!(self, state_set(EmulationState::NotifyBell as i32));
        } else if token == ty_ctl!('H') {
            // VT100
            current_screen.backspace();
        } else if token == ty_ctl!('I') {
            // VT100
            current_screen.tab(1);
        } else if token == ty_ctl!('J') {
            // VT100
            current_screen.new_line();
        } else if token == ty_ctl!('K') {
            // VT100
            current_screen.new_line();
        } else if token == ty_ctl!('L') {
            // VT100
            current_screen.new_line();
        } else if token == ty_ctl!('M') {
            // VT100
            current_screen.to_start_of_line();
        //////////////////////////////////////////////////////////////
        } else if token == ty_ctl!('N') {
            // VT100
            self.use_charset(1);
        } else if token == ty_ctl!('O') {
            // VT100
            self.use_charset(0);
        //////////////////////////////////////////////////////////////
        } else if token == ty_ctl!('P') {
            // DLE: ignored
        } else if token == ty_ctl!('Q') {
            // VT100
            // DC1: XON continue
        } else if token == ty_ctl!('R') {
            // DC2: ignored
        } else if token == ty_ctl!('S') {
            // VT100
            // DC3: XOFF halt
        } else if token == ty_ctl!('T') {
            // DC4: ignored
        } else if token == ty_ctl!('U') {
            // NAK: ignored
        } else if token == ty_ctl!('V') {
            // SYN: ignored
        } else if token == ty_ctl!('W') {
            // ETB: ignored
        } else if token == ty_ctl!('X') {
            // VT100
            current_screen.display_character(0x2592);
        } else if token == ty_ctl!('Y') {
            // EM: ignored
        } else if token == ty_ctl!('Z') {
            // VT100
            current_screen.display_character(0x2592);
        } else if token == ty_ctl!('[') {
            // ESC: cannot be seen here.
        } else if token == ty_ctl!('\\') {
            // FS : ignored
        } else if token == ty_ctl!(']') {
            // GS : ignored
        } else if token == ty_ctl!('^') {
            // RS : ignored
        } else if token == ty_ctl!('_') {
            // US : ignored
            //////////////////////////////////////////////////////////////
        } else if token == ty_esc!('D') {
            // VT100
            current_screen.index();
        } else if token == ty_esc!('E') {
            // VT100
            current_screen.next_line();
        } else if token == ty_esc!('H') {
            // VT100
            current_screen.change_tab_stop(true);
        } else if token == ty_esc!('M') {
            // VT100
            current_screen.reverse_index();
        } else if token == ty_esc!('Z') {
            self.report_terminal_type();
        } else if token == ty_esc!('c') {
            self.reset();
        //////////////////////////////////////////////////////////////
        } else if token == ty_esc!('n') {
            self.use_charset(2);
        } else if token == ty_esc!('o') {
            self.use_charset(3);
        } else if token == ty_esc!('7') {
            self.save_cursor();
        } else if token == ty_esc!('8') {
            self.restore_cursor();
        //////////////////////////////////////////////////////////////
        } else if token == ty_esc!('=') {
            self.set_mode(MODE_APP_KEY_PAD);
        } else if token == ty_esc!('>') {
            self.reset_mode(MODE_APP_KEY_PAD);
        } else if token == ty_esc!('<') {
            // VT100
            self.set_mode(MODE_ANSI);
        //////////////////////////////////////////////////////////////
        } else if token == ty_esc_cs!('(', '0') {
            // VT100
            self.set_charset(0, b'0');
        } else if token == ty_esc_cs!('(', 'A') {
            // VT100
            self.set_charset(0, b'A');
        } else if token == ty_esc_cs!('(', 'B') {
            // VT100
            self.set_charset(0, b'B');
        //////////////////////////////////////////////////////////////
        } else if token == ty_esc_cs!(')', '0') {
            // VT100
            self.set_charset(1, b'0');
        } else if token == ty_esc_cs!(')', 'A') {
            // VT100
            self.set_charset(1, b'A');
        } else if token == ty_esc_cs!(')', 'B') {
            // VT100
            self.set_charset(1, b'B');
        //////////////////////////////////////////////////////////////
        } else if token == ty_esc_cs!('*', '0') {
            // VT100
            self.set_charset(2, b'0');
        } else if token == ty_esc_cs!('*', 'A') {
            // VT100
            self.set_charset(2, b'A');
        } else if token == ty_esc_cs!('*', 'B') {
            // VT100
            self.set_charset(2, b'B');
        //////////////////////////////////////////////////////////////
        } else if token == ty_esc_cs!('+', '0') {
            // VT100
            self.set_charset(3, b'0');
        } else if token == ty_esc_cs!('+', 'A') {
            // VT100
            self.set_charset(3, b'A');
        } else if token == ty_esc_cs!('+', 'B') {
            // VT100
            self.set_charset(3, b'B');
        //////////////////////////////////////////////////////////////
        } else if token == ty_esc_cs!('%', 'G') {
            // Linux
            // TODO: setCodec(UTF8Codec)
        } else if token == ty_esc_cs!('%', '@') {
            // Linux
            // TODO: setCodec(LocaleCodec)
            //////////////////////////////////////////////////////////////
        } else if token == ty_esc_de!('3') {
            // Double height line, top half.
            current_screen.set_line_property(LINE_DOUBLE_WIDTH, true);
            current_screen.set_line_property(LINE_DOUBLE_HEIGHT, true);
        } else if token == ty_esc_de!('4') {
            // Double height line, bottom half.
            current_screen.set_line_property(LINE_DOUBLE_WIDTH, true);
            current_screen.set_line_property(LINE_DOUBLE_HEIGHT, true);
        } else if token == ty_esc_de!('5') {
            // Single width, single height line.
            current_screen.set_line_property(LINE_DOUBLE_WIDTH, false);
            current_screen.set_line_property(LINE_DOUBLE_HEIGHT, false);
        } else if token == ty_esc_de!('6') {
            // Double width, single height line.
            current_screen.set_line_property(LINE_DOUBLE_WIDTH, true);
            current_screen.set_line_property(LINE_DOUBLE_HEIGHT, false);
        } else if token == ty_esc_de!('8') {
            current_screen.help_align();
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_ps!('t', 8) {
            // resize = \e[8;<row>;<col>t
            self.set_image_size(p as i32, q);
            emit!(self, image_resize_request(Size::new(q, p as i32)));
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_ps!('t', 28) {
            // change tab text color : \e[28;<color>t  color: 0-16,777,215
            emit!(self, change_tab_text_color_request(p as i32));
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_ps!('K', 0) {
            current_screen.clear_to_end_of_line();
        } else if token == ty_csi_ps!('K', 1) {
            current_screen.clear_to_begin_of_line();
        } else if token == ty_csi_ps!('K', 2) {
            current_screen.clear_entire_line();
        } else if token == ty_csi_ps!('J', 0) {
            current_screen.clear_to_end_of_screen();
        } else if token == ty_csi_ps!('J', 1) {
            current_screen.clear_to_begin_of_screen();
        } else if token == ty_csi_ps!('J', 2) {
            current_screen.clear_entire_screen();
        } else if token == ty_csi_ps!('J', 3) {
            self.clear_history();
        } else if token == ty_csi_ps!('g', 0) {
            // VT100
            current_screen.change_tab_stop(false);
        } else if token == ty_csi_ps!('g', 3) {
            // VT100
            current_screen.change_tab_stop(false);
        } else if token == ty_csi_ps!('h', 4) {
            current_screen.set_mode(MODE_INSERT);
        } else if token == ty_csi_ps!('h', 20) {
            self.set_mode(MODE_NEWLINE);
        } else if token == ty_csi_ps!('i', 0) {
            // IGNORE: attached printer
            // VT100
        } else if token == ty_csi_ps!('l', 4) {
            current_screen.reset_mode(MODE_INSERT);
        } else if token == ty_csi_ps!('l', 20) {
            self.reset_mode(MODE_NEWLINE);
        } else if token == ty_csi_ps!('s', 0) {
            self.save_cursor();
        } else if token == ty_csi_ps!('u', 0) {
            self.restore_cursor();
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_ps!('m', 0) {
            current_screen.set_default_rendition();
        } else if token == ty_csi_ps!('m', 1) {
            // VT100
            current_screen.set_rendition(RE_BOLD);
        } else if token == ty_csi_ps!('m', 2) {
            current_screen.set_rendition(RE_FAINT);
        } else if token == ty_csi_ps!('m', 3) {
            // VT100
            current_screen.set_rendition(RE_ITALIC);
        } else if token == ty_csi_ps!('m', 4) {
            // VT100
            current_screen.set_rendition(RE_UNDERLINE);
        } else if token == ty_csi_ps!('m', 5) {
            // VT100
            current_screen.set_rendition(RE_BLINK);
        } else if token == ty_csi_ps!('m', 7) {
            current_screen.set_rendition(RE_REVERSE);
        } else if token == ty_csi_ps!('m', 8) {
            current_screen.set_rendition(RE_CONCEAL);
        } else if token == ty_csi_ps!('m', 9) {
            current_screen.set_rendition(RE_STRIKEOUT);
        } else if token == ty_csi_ps!('m', 53) {
            current_screen.set_rendition(RE_OVERLINE);
        } else if token == ty_csi_ps!('m', 10) {
            // IGNORED: mapping related
            // Linux
        } else if token == ty_csi_ps!('m', 11) {
            // IGNORED: mapping related
            // Linux
        } else if token == ty_csi_ps!('m', 12) {
            // IGNORED: mapping related
            // Linux
        } else if token == ty_csi_ps!('m', 21) {
            current_screen.reset_rendition(RE_BOLD);
        } else if token == ty_csi_ps!('m', 22) {
            current_screen.reset_rendition(RE_BOLD);
            current_screen.reset_rendition(RE_FAINT);
        } else if token == ty_csi_ps!('m', 23) {
            // VT100
            current_screen.reset_rendition(RE_ITALIC);
        } else if token == ty_csi_ps!('m', 24) {
            current_screen.reset_rendition(RE_UNDERLINE);
        } else if token == ty_csi_ps!('m', 25) {
            current_screen.reset_rendition(RE_BLINK);
        } else if token == ty_csi_ps!('m', 27) {
            current_screen.reset_rendition(RE_REVERSE);
        } else if token == ty_csi_ps!('m', 28) {
            current_screen.reset_rendition(RE_CONCEAL);
        } else if token == ty_csi_ps!('m', 29) {
            current_screen.reset_rendition(RE_STRIKEOUT);
        } else if token == ty_csi_ps!('m', 55) {
            current_screen.reset_rendition(RE_OVERLINE);
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_ps!('m', 30) {
            current_screen.set_foreground_color(COLOR_SPACE_SYSTEM, 0);
        } else if token == ty_csi_ps!('m', 31) {
            current_screen.set_foreground_color(COLOR_SPACE_SYSTEM, 1);
        } else if token == ty_csi_ps!('m', 32) {
            current_screen.set_foreground_color(COLOR_SPACE_SYSTEM, 2);
        } else if token == ty_csi_ps!('m', 33) {
            current_screen.set_foreground_color(COLOR_SPACE_SYSTEM, 3);
        } else if token == ty_csi_ps!('m', 34) {
            current_screen.set_foreground_color(COLOR_SPACE_SYSTEM, 4);
        } else if token == ty_csi_ps!('m', 35) {
            current_screen.set_foreground_color(COLOR_SPACE_SYSTEM, 5);
        } else if token == ty_csi_ps!('m', 36) {
            current_screen.set_foreground_color(COLOR_SPACE_SYSTEM, 6);
        } else if token == ty_csi_ps!('m', 37) {
            current_screen.set_foreground_color(COLOR_SPACE_SYSTEM, 7);
        } else if token == ty_csi_ps!('m', 38) {
            current_screen.set_foreground_color(p as u8, q as u32);
        } else if token == ty_csi_ps!('m', 39) {
            current_screen.set_foreground_color(COLOR_SPACE_DEFAULT, 0);
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_ps!('m', 40) {
            current_screen.set_background_color(COLOR_SPACE_SYSTEM, 0);
        } else if token == ty_csi_ps!('m', 41) {
            current_screen.set_background_color(COLOR_SPACE_SYSTEM, 1);
        } else if token == ty_csi_ps!('m', 42) {
            current_screen.set_background_color(COLOR_SPACE_SYSTEM, 2);
        } else if token == ty_csi_ps!('m', 43) {
            current_screen.set_background_color(COLOR_SPACE_SYSTEM, 3);
        } else if token == ty_csi_ps!('m', 44) {
            current_screen.set_background_color(COLOR_SPACE_SYSTEM, 4);
        } else if token == ty_csi_ps!('m', 45) {
            current_screen.set_background_color(COLOR_SPACE_SYSTEM, 5);
        } else if token == ty_csi_ps!('m', 46) {
            current_screen.set_background_color(COLOR_SPACE_SYSTEM, 6);
        } else if token == ty_csi_ps!('m', 47) {
            current_screen.set_background_color(COLOR_SPACE_SYSTEM, 7);
        } else if token == ty_csi_ps!('m', 48) {
            current_screen.set_background_color(p as u8, q as u32);
        } else if token == ty_csi_ps!('m', 49) {
            current_screen.set_background_color(COLOR_SPACE_DEFAULT, 1);
        //////////////////////////////////////////////////////////////
        // Itensive color
        } else if token == ty_csi_ps!('m', 90) {
            current_screen.set_foreground_color(COLOR_SPACE_SYSTEM, 8);
        } else if token == ty_csi_ps!('m', 91) {
            current_screen.set_foreground_color(COLOR_SPACE_SYSTEM, 9);
        } else if token == ty_csi_ps!('m', 92) {
            current_screen.set_foreground_color(COLOR_SPACE_SYSTEM, 10);
        } else if token == ty_csi_ps!('m', 93) {
            current_screen.set_foreground_color(COLOR_SPACE_SYSTEM, 11);
        } else if token == ty_csi_ps!('m', 94) {
            current_screen.set_foreground_color(COLOR_SPACE_SYSTEM, 12);
        } else if token == ty_csi_ps!('m', 95) {
            current_screen.set_foreground_color(COLOR_SPACE_SYSTEM, 13);
        } else if token == ty_csi_ps!('m', 96) {
            current_screen.set_foreground_color(COLOR_SPACE_SYSTEM, 14);
        } else if token == ty_csi_ps!('m', 97) {
            current_screen.set_foreground_color(COLOR_SPACE_SYSTEM, 15);
        //////////////////////////////////////////////////////////////
        // Itensive color
        } else if token == ty_csi_ps!('m', 100) {
            current_screen.set_background_color(COLOR_SPACE_SYSTEM, 8);
        } else if token == ty_csi_ps!('m', 101) {
            current_screen.set_background_color(COLOR_SPACE_SYSTEM, 9);
        } else if token == ty_csi_ps!('m', 102) {
            current_screen.set_background_color(COLOR_SPACE_SYSTEM, 10);
        } else if token == ty_csi_ps!('m', 103) {
            current_screen.set_background_color(COLOR_SPACE_SYSTEM, 11);
        } else if token == ty_csi_ps!('m', 104) {
            current_screen.set_background_color(COLOR_SPACE_SYSTEM, 12);
        } else if token == ty_csi_ps!('m', 105) {
            current_screen.set_background_color(COLOR_SPACE_SYSTEM, 13);
        } else if token == ty_csi_ps!('m', 106) {
            current_screen.set_background_color(COLOR_SPACE_SYSTEM, 14);
        } else if token == ty_csi_ps!('m', 107) {
            current_screen.set_background_color(COLOR_SPACE_SYSTEM, 15);
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_ps!('n', 5) {
            self.report_status();
        } else if token == ty_csi_ps!('n', 6) {
            self.report_cursor_position();
        } else if token == ty_csi_ps!('q', 0) {
            // IGNORED: LEDs off
            // VT100
        } else if token == ty_csi_ps!('q', 1) {
            // IGNORED: LED1 on
            // VT100
        } else if token == ty_csi_ps!('q', 2) {
            // IGNORED: LED2 on
            // VT100
        } else if token == ty_csi_ps!('q', 3) {
            // IGNORED: LED3 on
            // VT100
        } else if token == ty_csi_ps!('q', 4) {
            // IGNORED: LED4 on
            // VT100
        } else if token == ty_csi_ps!('x', 0) {
            // VT100
            self.report_terminal_params(2);
        } else if token == ty_csi_ps!('x', 1) {
            // VT100
            self.report_terminal_params(3);
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_ps_sp!('q', 0) {
            emit!(
                self,
                cursor_changed(KeyboardCursorShape::BlockCursor as u8, true)
            );
        } else if token == ty_csi_ps_sp!('q', 1) {
            emit!(
                self,
                cursor_changed(KeyboardCursorShape::BlockCursor as u8, true)
            );
        } else if token == ty_csi_ps_sp!('q', 2) {
            emit!(
                self,
                cursor_changed(KeyboardCursorShape::BlockCursor as u8, false)
            );
        } else if token == ty_csi_ps_sp!('q', 3) {
            emit!(
                self,
                cursor_changed(KeyboardCursorShape::UnderlineCursor as u8, true)
            );
        } else if token == ty_csi_ps_sp!('q', 4) {
            emit!(
                self,
                cursor_changed(KeyboardCursorShape::UnderlineCursor as u8, false)
            );
        } else if token == ty_csi_ps_sp!('q', 5) {
            emit!(
                self,
                cursor_changed(KeyboardCursorShape::IBeamCursor as u8, true)
            );
        } else if token == ty_csi_ps_sp!('q', 6) {
            emit!(
                self,
                cursor_changed(KeyboardCursorShape::IBeamCursor as u8, false)
            );
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pn!('@') {
            current_screen.insert_chars(p as i32);
        } else if token == ty_csi_pn!('A') {
            // VT100
            current_screen.cursor_up(p as i32);
        } else if token == ty_csi_pn!('B') {
            // VT100
            current_screen.cursor_down(p as i32);
        } else if token == ty_csi_pn!('C') {
            // VT100
            current_screen.cursor_right(p as i32);
        } else if token == ty_csi_pn!('D') {
            // VT100
            current_screen.cursor_left(p as i32);
        } else if token == ty_csi_pn!('E') {
            // VT100
            current_screen.cursor_next_line(p as i32);
        } else if token == ty_csi_pn!('F') {
            // VT100
            current_screen.cursor_previous_line(p as i32);
        } else if token == ty_csi_pn!('G') {
            // Linux
            current_screen.set_cursor_x(p as i32);
        } else if token == ty_csi_pn!('H') {
            // VT100
            current_screen.set_cursor_yx(p as i32, q);
        } else if token == ty_csi_pn!('I') {
            current_screen.tab(p as i32);
        } else if token == ty_csi_pn!('L') {
            current_screen.insert_lines(p as i32);
        } else if token == ty_csi_pn!('M') {
            current_screen.delete_lines(p as i32);
        } else if token == ty_csi_pn!('P') {
            current_screen.delete_chars(p as i32);
        } else if token == ty_csi_pn!('S') {
            current_screen.scroll_up(p as i32);
        } else if token == ty_csi_pn!('T') {
            current_screen.scroll_down(p as i32);
        } else if token == ty_csi_pn!('X') {
            current_screen.erase_chars(p as i32);
        } else if token == ty_csi_pn!('Z') {
            current_screen.back_tab(p as i32);
        } else if token == ty_csi_pn!('b') {
            current_screen.repeat_chars(p as i32);
        } else if token == ty_csi_pn!('c') {
            // VT100
            self.report_terminal_type();
        } else if token == ty_csi_pn!('d') {
            // Linux
            current_screen.set_cursor_y(p as i32);
        } else if token == ty_csi_pn!('f') {
            // VT100
            current_screen.set_cursor_yx(p as i32, q);
        } else if token == ty_csi_pn!('r') {
            // VT100
            self.set_margins(p as i32, q);
        } else if token == ty_csi_pn!('y') {
            // IGNORED: Confidence test.
            // VT100
            //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 1) {
            self.set_mode(MODE_APP_CURSOR_KEY);
        } else if token == ty_csi_pr!('l', 1) {
            self.reset_mode(MODE_APP_CURSOR_KEY);
        } else if token == ty_csi_pr!('s', 1) {
            self.save_mode(MODE_APP_CURSOR_KEY);
        } else if token == ty_csi_pr!('r', 1) {
            self.restore_mode(MODE_APP_CURSOR_KEY);
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('l', 2) {
            // Vt100
            self.reset_mode(MODE_ANSI);
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 3) {
            self.set_mode(MODE_132_COLUMNS);
        } else if token == ty_csi_pr!('l', 3) {
            self.reset_mode(MODE_132_COLUMNS);
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 4) {
            // IGNORED: soft scrolling.
            // VT100
        } else if token == ty_csi_pr!('l', 4) {
            // IGNORED: soft scrolling.
            // VT100
            //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 5) {
            current_screen.set_mode(MODE_SCREEN);
        } else if token == ty_csi_pr!('l', 5) {
            current_screen.reset_mode(MODE_SCREEN);
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 6) {
            current_screen.set_mode(MODE_ORIGIN);
        } else if token == ty_csi_pr!('l', 6) {
            current_screen.reset_mode(MODE_ORIGIN);
        } else if token == ty_csi_pr!('s', 6) {
            current_screen.save_mode(MODE_ORIGIN);
        } else if token == ty_csi_pr!('r', 6) {
            current_screen.restore_mode(MODE_ORIGIN);
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 7) {
            current_screen.set_mode(MODE_WRAP);
        } else if token == ty_csi_pr!('l', 7) {
            current_screen.reset_mode(MODE_WRAP);
        } else if token == ty_csi_pr!('s', 7) {
            current_screen.save_mode(MODE_WRAP);
        } else if token == ty_csi_pr!('r', 7) {
            current_screen.restore_mode(MODE_WRAP);
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 8) {
            // IGNORED: autorepeat on
            // VT100
        } else if token == ty_csi_pr!('l', 8) {
            // IGNORED: autorepeat off
            // VT100
        } else if token == ty_csi_pr!('s', 8) {
            // IGNORED: autorepeat on
            // VT100
        } else if token == ty_csi_pr!('r', 8) {
            // IGNORED: autorepeat off
            // VT100
            //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 9) {
            // IGNORED: interlace
            // VT100
        } else if token == ty_csi_pr!('l', 9) {
            // IGNORED: interlace
            // VT100
        } else if token == ty_csi_pr!('s', 9) {
            // IGNORED: interlace
            // VT100
        } else if token == ty_csi_pr!('r', 9) {
            // IGNORED: interlace
            // VT100
            //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 12) {
            // IGNORED: Cursor blink
            // att610
        } else if token == ty_csi_pr!('l', 12) {
            // IGNORED: Cursor blink
            // att610
        } else if token == ty_csi_pr!('s', 12) {
            // IGNORED: Cursor blink
            // att610
        } else if token == ty_csi_pr!('r', 12) {
            // IGNORED: Cursor blink
            // att610
            //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 25) {
            // VT100
            self.set_mode(MODE_CURSOR);
        } else if token == ty_csi_pr!('l', 25) {
            // VT100
            self.reset_mode(MODE_CURSOR);
        } else if token == ty_csi_pr!('s', 25) {
            // VT100
            self.save_mode(MODE_CURSOR);
        } else if token == ty_csi_pr!('r', 25) {
            // VT100
            self.restore_mode(MODE_CURSOR);
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 40) {
            // XTerm
            self.set_mode(MODE_ALLOW_132_COLUMNS);
        } else if token == ty_csi_pr!('l', 40) {
            // XTerm
            self.reset_mode(MODE_ALLOW_132_COLUMNS);
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 41) {
            // IGNORED: obsolete more(1) fix
            // XTerm
        } else if token == ty_csi_pr!('l', 41) {
            // IGNORED: obsolete more(1) fix
            // XTerm
        } else if token == ty_csi_pr!('s', 41) {
            // IGNORED: obsolete more(1) fix
            // XTerm
        } else if token == ty_csi_pr!('r', 41) {
            // IGNORED: obsolete more(1) fix
            // XTerm
            //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 47) {
            // VT100
            self.set_mode(MODE_APP_SCREEN);
        } else if token == ty_csi_pr!('l', 47) {
            // VT100
            self.reset_mode(MODE_APP_SCREEN);
        } else if token == ty_csi_pr!('s', 47) {
            // XTerm
            self.save_mode(MODE_APP_SCREEN);
        } else if token == ty_csi_pr!('r', 47) {
            // XTerm
            self.restore_mode(MODE_APP_SCREEN);
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 67) {
            // IGNORED: DECBKM
            // XTerm
        } else if token == ty_csi_pr!('l', 67) {
            // IGNORED: DECBKM
            // XTerm
        } else if token == ty_csi_pr!('s', 67) {
            // IGNORED: DECBKM
            // XTerm
        } else if token == ty_csi_pr!('r', 67) {
            // IGNORED: DECBKM
            // XTerm
            //////////////////////////////////////////////////////////////
            // XTerm defines the following modes:
            // SET_VT200_MOUSE             1000
            // SET_VT200_HIGHLIGHT_MOUSE   1001
            // SET_BTN_EVENT_MOUSE         1002
            // SET_ANY_EVENT_MOUSE         1003
            //
            // Note about mouse modes:
            // There are four mouse modes which xterm-compatible terminals can support
            // - 1000,1001,1002,1003 Konsole currently supports mode 1000 (basic mouse
            // press and release) and mode 1002 (dragging the mouse).
            // TODO:  Implementation of mouse modes 1001 (something called hilight
            // tracking) and 1003 (a slight variation on dragging the mouse)
        } else if token == ty_csi_pr!('h', 1000) {
            // XTerm
            self.set_mode(MODE_MOUSE_1000);
        } else if token == ty_csi_pr!('l', 1000) {
            // XTerm
            self.reset_mode(MODE_MOUSE_1000);
        } else if token == ty_csi_pr!('s', 1000) {
            // XTerm
            self.save_mode(MODE_MOUSE_1000);
        } else if token == ty_csi_pr!('r', 1000) {
            // XTerm
            self.restore_mode(MODE_MOUSE_1000);
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 1001) {
            // IGNORED: hilight mouse tracking
            // XTerm
        } else if token == ty_csi_pr!('l', 1001) {
            // XTerm
            self.reset_mode(MODE_MOUSE_1001);
        } else if token == ty_csi_pr!('s', 1001) {
            // IGNORED: hilight mouse tracking
            // XTerm
        } else if token == ty_csi_pr!('r', 1001) {
            // IGNORED: hilight mouse tracking
            // XTerm
            //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 1002) {
            // XTerm
            self.set_mode(MODE_MOUSE_1002);
        } else if token == ty_csi_pr!('l', 1002) {
            // XTerm
            self.reset_mode(MODE_MOUSE_1002);
        } else if token == ty_csi_pr!('s', 1002) {
            // XTerm
            self.save_mode(MODE_MOUSE_1002);
        } else if token == ty_csi_pr!('r', 1002) {
            // XTerm
            self.restore_mode(MODE_MOUSE_1002);
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 1003) {
            // XTerm
            self.set_mode(MODE_MOUSE_1003);
        } else if token == ty_csi_pr!('l', 1003) {
            // XTerm
            self.reset_mode(MODE_MOUSE_1003);
        } else if token == ty_csi_pr!('s', 1003) {
            // XTerm
            self.save_mode(MODE_MOUSE_1003);
        } else if token == ty_csi_pr!('r', 1003) {
            // XTerm
            self.restore_mode(MODE_MOUSE_1003);
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 1004) {
            self.report_focus_event = true;
        } else if token == ty_csi_pr!('l', 1004) {
            self.report_focus_event = false;
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 1005) {
            // XTerm
            self.set_mode(MODE_MOUSE_1005);
        } else if token == ty_csi_pr!('l', 1005) {
            // XTerm
            self.reset_mode(MODE_MOUSE_1005);
        } else if token == ty_csi_pr!('s', 1005) {
            // XTerm
            self.save_mode(MODE_MOUSE_1005);
        } else if token == ty_csi_pr!('r', 1005) {
            // XTerm
            self.restore_mode(MODE_MOUSE_1005);
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 1006) {
            // XTerm
            self.set_mode(MODE_MOUSE_1006);
        } else if token == ty_csi_pr!('l', 1006) {
            // XTerm
            self.reset_mode(MODE_MOUSE_1006);
        } else if token == ty_csi_pr!('s', 1006) {
            // XTerm
            self.save_mode(MODE_MOUSE_1006);
        } else if token == ty_csi_pr!('r', 1006) {
            // XTerm
            self.restore_mode(MODE_MOUSE_1006);
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 1015) {
            // XTerm
            self.set_mode(MODE_MOUSE_1015);
        } else if token == ty_csi_pr!('l', 1015) {
            // XTerm
            self.reset_mode(MODE_MOUSE_1015);
        } else if token == ty_csi_pr!('s', 1015) {
            // XTerm
            self.save_mode(MODE_MOUSE_1015);
        } else if token == ty_csi_pr!('r', 1015) {
            // XTerm
            self.restore_mode(MODE_MOUSE_1015);
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('r', 1034) {
            // IGNORED: 8bitinput activation
            // XTerm
            //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 1047) {
            // XTerm
            self.set_mode(MODE_APP_SCREEN);
        } else if token == ty_csi_pr!('l', 1047) {
            // XTerm
            self.emulation_mut().screen[1].clear_entire_screen();
            self.reset_mode(MODE_APP_SCREEN);
        } else if token == ty_csi_pr!('s', 1047) {
            // XTerm
            self.save_mode(MODE_APP_SCREEN);
        } else if token == ty_csi_pr!('r', 1047) {
            // XTerm
            self.restore_mode(MODE_APP_SCREEN);
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 1048) {
            // XTerm
            self.save_cursor();
        } else if token == ty_csi_pr!('l', 1048) {
            // XTerm
            self.restore_cursor();
        } else if token == ty_csi_pr!('s', 1048) {
            // XTerm
            self.save_cursor();
        } else if token == ty_csi_pr!('r', 1048) {
            // XTerm
            self.restore_cursor();
        //////////////////////////////////////////////////////////////
        // FIXME: every once new sequences like this pop up in xterm.
        //       Here's a guess of what they could mean.
        } else if token == ty_csi_pr!('h', 1049) {
            // XTerm
            self.save_cursor();
            self.emulation_mut().screen[1].clear_entire_screen();
            self.set_mode(MODE_APP_SCREEN);
        } else if token == ty_csi_pr!('l', 1049) {
            // XTerm
            self.reset_mode(MODE_APP_SCREEN);
            self.restore_cursor();
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pr!('h', 2004) {
            // XTerm
            self.set_mode(MODE_BRACKETD_PASTE);
        } else if token == ty_csi_pr!('l', 2004) {
            // XTerm
            self.reset_mode(MODE_BRACKETD_PASTE);
        } else if token == ty_csi_pr!('s', 2004) {
            // XTerm
            self.save_mode(MODE_BRACKETD_PASTE);
        } else if token == ty_csi_pr!('r', 2004) {
            // XTerm
            self.restore_mode(MODE_BRACKETD_PASTE);
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pe!('p') {
            // IGNORED: reset
            //////////////////////////////////////////////////////////////
            // FIXME: when changing between vt52 and ansi mode evtl do some resetting.
        } else if token == ty_vt52!('A') {
            // VT52
            current_screen.cursor_up(1);
        } else if token == ty_vt52!('B') {
            // VT52
            current_screen.cursor_down(1);
        } else if token == ty_vt52!('C') {
            // VT52
            current_screen.cursor_right(1);
        } else if token == ty_vt52!('D') {
            // VT52
            current_screen.cursor_left(1);
        //////////////////////////////////////////////////////////////
        } else if token == ty_vt52!('F') {
            // VT52
            self.set_and_use_charset(0, '0' as i32);
        } else if token == ty_vt52!('G') {
            // VT52
            self.set_and_use_charset(0, 'B' as i32);
        //////////////////////////////////////////////////////////////
        } else if token == ty_vt52!('H') {
            // VT52
            current_screen.set_cursor_yx(1, 1);
        } else if token == ty_vt52!('I') {
            // VT52
            current_screen.reverse_index();
        } else if token == ty_vt52!('J') {
            // VT52
            current_screen.clear_to_end_of_screen();
        } else if token == ty_vt52!('K') {
            // VT52
            current_screen.clear_to_end_of_line();
        } else if token == ty_vt52!('Y') {
            // VT52
            current_screen.set_cursor_yx(p as i32 - 31, q - 31);
        } else if token == ty_vt52!('Z') {
            // VT52
            self.report_terminal_type();
        } else if token == ty_vt52!('<') {
            // VT52
            self.set_mode(MODE_ANSI);
        } else if token == ty_vt52!('=') {
            // VT52
            self.set_mode(MODE_APP_KEY_PAD);
        } else if token == ty_vt52!('>') {
            // VT52
            self.reset_mode(MODE_APP_KEY_PAD);
        //////////////////////////////////////////////////////////////
        } else if token == ty_csi_pg!('c') {
            self.report_secondary_attributes();
        //////////////////////////////////////////////////////////////
        } else {
            self.report_decoding_error();
        }
    }

    fn process_window_attribute_change(&mut self) {
        // Describes the window or terminal session attribute to change
        // See [`Session::user_title_change`] for possible values
        let mut attribute_to_change = 0;
        let mut i = 2;
        loop {
            if !(i < self.token_buffer_pos
                && self.token_buffer[i] >= wch!('0')
                && self.token_buffer[i] <= wch!('9'))
            {
                break;
            }

            attribute_to_change =
                10 * attribute_to_change + (self.token_buffer[i] as i32 - wch!('0') as i32);
            i += 1;
        }

        if self.token_buffer[i] != wch!(';') {
            self.report_decoding_error();
            return;
        }

        // copy from the first char after ';', and skipping the ending delimiter
        // 0x07 or 0x92. Note that as control characters in OSC text parts are
        // ignored, only the second char in ST ("\e\\") is appended to tokenBuffer.
        let mut new_value = WideString::new();
        let slice: &[uwchar_t] = unsafe {
            std::mem::transmute(&self.token_buffer[i + 1..(self.token_buffer_pos - i - 2)])
        };
        new_value.push_slice(slice);

        self.pending_title_updates.insert(
            attribute_to_change,
            new_value
                .to_string()
                .expect("`wchar_tString` transmit to String failed."),
        );
        // TODO: Update title update timer
    }

    fn request_window_attribute(&self, _p: i32) {
        // No implementation
    }

    fn add_to_current_token(&mut self, cc: wchar_t) {
        let pos = self.token_buffer_pos;
        self.token_buffer[pos] = cc;
        self.token_buffer_pos = (pos + 1).min(MAX_TOKEN_LENGTH - 1);
    }

    fn add_digit(&mut self, digit: i32) {
        if self.argv[self.argc as usize] < MAX_ARGUMENT as i32 {
            self.argv[self.argc as usize] = 10 * self.argv[self.argc as usize] + digit;
        }
    }

    fn add_argument(&mut self) {
        self.argc = (self.argc + 1).min(MAXARGS as i32 - 1);
        self.argv[self.argc as usize] = 0;
    }

    //////////////////////////////////////// VT100 Charset ////////////////////////////////////////
    /*
       The processing contains a VT100 specific code translation layer.
       It's still in use and mainly responsible for the line drawing graphics.

       These and some other glyphs are assigned to codes (0x5f-0xfe)
       normally occupied by the latin letters. Since this codes also
       appear within control sequences, the extra code conversion
       does not permute with the tokenizer and is placed behind it
       in the pipeline. It only applies to tokens, which represent
       plain characters.

       This conversion it eventually continued in TerminalDisplay.C, since
       it might involve VT100 enhanced fonts, which have these
       particular glyphs allocated in (0x00-0x1f) in their code page.
    */
    fn charset(&mut self) -> &mut CharCodes {
        unsafe {
            self.charset
                .get_mut(
                    if self
                        .emulation
                        .as_ref()
                        .unwrap()
                        .current_screen
                        .as_ref()
                        .unwrap()
                        .as_ref()
                        .id()
                        == self.emulation.as_ref().unwrap().screen[1].id()
                    {
                        1
                    } else {
                        0
                    },
                )
                .unwrap()
        }
    }

    fn apply_charset(&mut self, c: wchar_t) -> wchar_t {
        if self.charset().graphic && (0x5f..=0x7e).contains(&c) {
            return VT100_GRAPHICS[(c - 0x5f) as usize];
        }
        if self.charset().pound && c == wch!('#') {
            return 0xa3;
        }
        c
    }

    /// "Charset" related part of the emulation state. This configures the VT100 charset filter.
    ///
    /// While most operation work on the current _screen, the following two are different.
    fn reset_charset(&mut self, scrno: usize) {
        self.charset[scrno].current_charset = 0;
        self.charset[scrno]
            .charset
            .copy_from_slice("BBBB".as_bytes());
        self.charset[scrno].saved_graphic = false;
        self.charset[scrno].saved_pound = false;
        self.charset[scrno].graphic = false;
        self.charset[scrno].pound = false;
    }

    fn set_charset(&mut self, n: i32, cs: u8) {
        self.charset[0].charset[(n & 3) as usize] = cs;
        self.use_charset(self.charset[0].current_charset);
        self.charset[1].charset[(n & 3) as usize] = cs;
        self.use_charset(self.charset[1].current_charset);
    }

    fn use_charset(&mut self, n: i32) {
        let charset = self.charset();
        charset.current_charset = n & 3;
        charset.graphic = charset.charset[(n & 3) as usize] == b'0';
        // This mode is obsolete.
        charset.pound = charset.charset[(n & 3) as usize] == b'A';
    }

    fn set_and_use_charset(&mut self, n: i32, cs: i32) {
        self.charset().charset[(n & 3) as usize] = cs as u8;
        self.use_charset(n & 3);
    }

    fn save_cursor(&mut self) {
        let charset = self.charset();
        charset.saved_graphic = charset.graphic;
        charset.saved_pound = charset.pound;
        // we are not clear about these
        // sa_charset = charsets[cScreen->_charset];
        // sa_charset_num = cScreen->_charset;
        unsafe {
            self.emulation
                .as_mut()
                .unwrap()
                .current_screen
                .as_mut()
                .unwrap()
                .as_mut()
                .save_cursor();
        }
    }

    fn restore_cursor(&mut self) {
        let charset = self.charset();
        charset.graphic = charset.saved_graphic;
        charset.pound = charset.saved_pound;
        unsafe {
            self.emulation
                .as_mut()
                .unwrap()
                .current_screen
                .as_mut()
                .unwrap()
                .as_mut()
                .resotre_cursor();
        }
    }

    fn set_margins(&mut self, top: i32, bottom: i32) {
        self.emulation_mut().screen[0].set_margins(top, bottom);
        self.emulation_mut().screen[1].set_margins(top, bottom);
    }

    /// Set margins for all screens back to their defaults.
    fn set_default_margins(&mut self) {
        self.emulation_mut().screen[0].set_default_margins();
        self.emulation_mut().screen[1].set_default_margins();
    }

    //////////////////////////////////////// Modes operations ////////////////////////////////////////
    /*
       Some of the emulations state is either added to the state of the screens.

       This causes some scoping problems, since different emulations choose to
       located the mode either to the current _screen or to both.

       For strange reasons, the extend of the rendition attributes ranges over
       all screens and not over the actual _screen.

       We decided on the precise precise extend, somehow.
    */
    /// Returns true if 'mode' is set or false otherwise.
    fn get_mode(&self, mode: usize) -> bool {
        self.current_modes.mode[mode]
    }

    /// Saves the current boolean value of 'mode'.
    fn save_mode(&mut self, mode: usize) {
        self.saved_modes.mode[mode] = self.current_modes.mode[mode];
    }

    /// Restores the boolean value of 'mode'.
    fn restore_mode(&mut self, mode: usize) {
        if self.saved_modes.mode[mode] {
            self.set_mode(mode)
        } else {
            self.reset_mode(mode)
        }
    }

    /// Resets all modes (except MODE_Allow132Columns).
    fn reset_modes(&mut self) {
        // MODE_Allow132Columns is not reset here
        // to match Xterm's behaviour (see Xterm's VTReset() function)
        self.reset_mode(MODE_132_COLUMNS);
        self.save_mode(MODE_132_COLUMNS);
        self.reset_mode(MODE_MOUSE_1000);
        self.save_mode(MODE_MOUSE_1000);
        self.reset_mode(MODE_MOUSE_1001);
        self.save_mode(MODE_MOUSE_1001);
        self.reset_mode(MODE_MOUSE_1002);
        self.save_mode(MODE_MOUSE_1002);
        self.reset_mode(MODE_MOUSE_1003);
        self.save_mode(MODE_MOUSE_1003);
        self.reset_mode(MODE_MOUSE_1005);
        self.save_mode(MODE_MOUSE_1005);
        self.reset_mode(MODE_MOUSE_1006);
        self.save_mode(MODE_MOUSE_1006);
        self.reset_mode(MODE_MOUSE_1015);
        self.save_mode(MODE_MOUSE_1015);
        self.reset_mode(MODE_BRACKETD_PASTE);
        self.save_mode(MODE_BRACKETD_PASTE);

        self.reset_mode(MODE_APP_SCREEN);
        self.save_mode(MODE_APP_SCREEN);
        self.reset_mode(MODE_APP_CURSOR_KEY);
        self.save_mode(MODE_APP_CURSOR_KEY);
        self.reset_mode(MODE_APP_KEY_PAD);
        self.save_mode(MODE_APP_KEY_PAD);
        self.reset_mode(MODE_NEWLINE);
        self.set_mode(MODE_ANSI);
    }

    fn report_decoding_error(&self) {
        if self.token_buffer_pos == 0
            || self.token_buffer_pos == 1 && self.token_buffer[0] & 0xff >= 32
        {}
    }

    fn report_terminal_type(&self) {
        // Primary device attribute response (Request was: ^[[0c or ^[[c (from TT321
        // Users Guide)) VT220:  ^[[?63;1;2;3;6;7;8c   (list deps on emul.
        // capabilities) VT100:  ^[[?1;2c VT101:  ^[[?1;0c VT102:  ^[[?6v
        if self.get_mode(MODE_ANSI) {
            // VT100
            self.send_string("\u{001b}[?1;2c".to_string(), -1)
        } else {
            // VT52
            self.send_string("\u{001b}/Z".to_string(), -1)
        }
    }

    fn report_secondary_attributes(&self) {
        // Seconday device attribute response (Request was: ^[[>0c or ^[[>c)
        if self.get_mode(MODE_ANSI) {
            self.send_string("\u{001b}[>0;115;0c".to_string(), -1)
        } else {
            self.send_string("\u{001b}/Z".to_string(), -1)
        }
    }

    fn report_status(&self) {
        self.send_string("\u{001b}[0n".to_string(), -1)
    }

    fn report_answer_back(&self) {
        // This is really obsolete VT100 stuff.
        self.send_string("".to_string(), -1)
    }

    fn report_cursor_position(&self) {
        let current_screen = unsafe {
            self.emulation
                .as_ref()
                .unwrap()
                .current_screen
                .as_ref()
                .unwrap()
                .as_ref()
        };

        let str = format!(
            "\u{001b}[{};{}R",
            current_screen.get_cursor_y() + 1,
            current_screen.get_cursor_x() + 1
        );
        self.send_string(str, -1)
    }

    fn report_terminal_params(&self, p: i32) {
        let str = format!("\u{001b}[{};1;1;112;112;1;0;x", p);
        self.send_string(str, -1)
    }

    fn on_scroll_lock(&self) {}

    fn scroll_lock(&self, _lock: bool) {}

    fn clear_screen_and_set_columns(&mut self, column_count: i32) {
        unsafe {
            self.set_image_size(
                self.emulation
                    .as_ref()
                    .unwrap()
                    .current_screen
                    .as_ref()
                    .unwrap()
                    .as_ref()
                    .get_lines(),
                column_count,
            );

            self.clear_entire_screen();
            self.set_default_margins();
            self.emulation
                .as_mut()
                .unwrap()
                .current_screen
                .as_mut()
                .unwrap()
                .as_mut()
                .set_cursor_yx(0, 0);
        }
    }

    //////////////////////////////////////////////////////// Slots
    /// The focus lost event can be used by Vim (or other terminal applications)
    /// to recognize that the konsole window has lost focus.
    /// The escape sequence is also used by iTerm2.
    /// Vim needs the following plugin to be installed to convert the escape
    /// sequence into the FocusLost autocmd: https://github.com/sjl/vitality.vim
    pub fn focus_lost(&self) {
        if self.report_focus_event {
            self.send_string("\u{001b}[O".to_string(), -1);
        }
    }

    /// The focus gained event can be used by Vim (or other terminal applications)
    /// to recognize that the konsole window has gained focus again.
    /// The escape sequence is also used by iTerm2.
    /// Vim needs the following plugin to be installed to convert the escape
    /// sequence into the FocusGained autocmd: https://github.com/sjl/vitality.vim
    pub fn focus_gained(&self) {
        if self.report_focus_event {
            self.send_string("\u{001b}[I".to_string(), -1);
        }
    }

    /// causes changeTitle() to be emitted for each (int,QString) pair in
    /// pendingTitleUpdates used to buffer multiple title updates
    fn update_title(&mut self) {
        for arg in self.pending_title_updates.keys() {
            let borrowed = &self.pending_title_updates;
            let title = match borrowed.get(arg) {
                Some(val) => val,
                None => "",
            };
            emit!(self, title_changed(*arg, title.to_string()));
        }
        self.pending_title_updates.clear();
    }
}
const ESC: wchar_t = 27;
const DEL: wchar_t = 127;
impl VT102Emulation {
    /**
       Ok, here comes the nasty part of the decoder.

       Instead of keeping an explicit state, we deduce it from the
       token scanned so far. It is then immediately combined with
       the current character to form a scanning decision.

       This is done by the following defines.

       - p is the length of the token scanned so far.
       - l (often p-1) is the position on which contents we base a decision.
       - c is a character or a group of characters (taken from 'charClass').

       Note that they need to applied in proper order
       and thoes macros used in [`VT102Emulation::receive_char(&self, cc: wchar_t)`]]
    */
    #[inline]
    fn lec(&self, p: usize, l: usize, c: wchar_t) -> bool {
        self.token_buffer_pos == p && self.token_buffer[l] == c
    }

    #[inline]
    fn lun(&self) -> bool {
        self.token_buffer_pos == 1 && self.cc >= 32
    }

    #[inline]
    fn les(&self, p: usize, l: usize, c: wchar_t) -> bool {
        self.token_buffer_pos == p
            && self.token_buffer[l] < 256
            && self.char_class[self.token_buffer[l] as usize] & c as i32 == c as i32
    }

    #[inline]
    fn eec(&self, c: wchar_t) -> bool {
        self.token_buffer_pos >= 3 && self.cc == c
    }

    #[inline]
    fn ees(&self, c: wchar_t) -> bool {
        self.token_buffer_pos >= 3
            && self.cc < 256
            && self.char_class[self.cc as usize] & c as i32 == c as i32
    }

    #[inline]
    fn eps(&self, c: wchar_t) -> bool {
        self.token_buffer_pos >= 3
            && self.token_buffer[2] != wch!('?')
            && self.token_buffer[2] != wch!('!')
            && self.token_buffer[2] != wch!('>')
            && self.cc < 256
            && self.char_class[self.cc as usize] & c as i32 == c as i32
    }

    #[inline]
    fn epp(&self) -> bool {
        self.token_buffer_pos >= 3 && self.token_buffer[2] == wch!('?')
    }

    #[inline]
    fn epe(&self) -> bool {
        self.token_buffer_pos >= 3 && self.token_buffer[2] == wch!('!')
    }

    #[inline]
    fn egt(&self) -> bool {
        self.token_buffer_pos >= 3 && self.token_buffer[2] == wch!('>')
    }

    #[inline]
    fn esp(&self) -> bool {
        self.token_buffer_pos >= 3 && self.token_buffer[2] == wch!(' ')
    }

    #[inline]
    fn xpe(&self) -> bool {
        self.token_buffer_pos >= 2 && self.token_buffer[1] == wch!(']')
    }

    #[inline]
    fn xte(&self) -> bool {
        self.xpe() && (self.cc == 7 || (self.prev_cc == 27 && self.cc == 92))
    }

    #[inline]
    fn ces(&self, c: wchar_t) -> bool {
        self.cc < 256 && self.char_class[self.cc as usize] & c as i32 == c as i32 && !self.xte()
    }

    #[inline]
    fn cntl(&self, c: wchar_t) -> wchar_t {
        c - wch!('@')
    }
}
impl Emulation for VT102Emulation {
    fn init(&mut self) {
        self.emulation_mut().init()
    }

    fn create_window(&mut self) -> Option<NonNull<ScreenWindow>> {
        self.emulation_mut().create_window()
    }

    fn image_size(&self) -> Size {
        self.emulation().image_size()
    }

    fn line_count(&self) -> i32 {
        self.emulation().line_count()
    }

    fn set_history(&mut self, history_type: Rc<RefCell<dyn HistoryType>>) {
        self.emulation_mut().set_history(history_type)
    }

    fn history(&self) -> Rc<RefCell<dyn HistoryType>> {
        self.emulation().history()
    }

    fn clear_history(&mut self) {
        self.emulation_mut().clear_history()
    }

    fn write_to_stream(
        &mut self,
        decoder: &mut dyn TerminalCharacterDecoder,
        start_line: i32,
        end_line: i32,
    ) {
        self.emulation_mut()
            .write_to_stream(decoder, start_line, end_line)
    }

    fn erase_char(&self) -> char {
        let entry = unsafe {
            self.emulation()
                .key_translator
                .as_ref()
                .unwrap()
                .as_ref()
                .find_entry(
                    KeyCode::KeyBackspace as u32,
                    KeyboardModifier::NoModifier,
                    Some(State::None),
                )
        };

        let text = entry.text(None, None);
        if !text.is_empty() {
            text[0] as char
        } else {
            '\u{b}'
        }
    }

    fn set_keyboard_layout(&mut self, name: &str) {
        self.emulation_mut().set_keyboard_layout(name)
    }

    fn keyboard_layout(&self) -> String {
        self.emulation().keyboard_layout()
    }

    fn clear_entire_screen(&mut self) {
        self.emulation_mut().clear_entire_screen()
    }

    fn reset(&mut self) {
        self.reset_tokenizer();
        self.reset_modes();
        self.reset_charset(0);
        self.emulation_mut().screen[0].reset(None);
        self.reset_charset(1);

        self.direct_update();
    }

    fn program_use_mouse(&self) -> bool {
        self.emulation().program_use_mouse()
    }

    fn set_use_mouse(&mut self, on: bool) {
        self.emulation_mut().set_use_mouse(on)
    }

    fn program_bracketed_paste_mode(&self) -> bool {
        self.emulation().program_bracketed_paste_mode()
    }

    fn set_bracketed_paste_mode(&mut self, on: bool) {
        self.emulation_mut().set_bracketed_paste_mode(on)
    }

    fn set_mode(&mut self, mode: usize) {
        self.current_modes.mode[mode] = true;

        match mode {
            MODE_132_COLUMNS => {
                if self.get_mode(MODE_ALLOW_132_COLUMNS) {
                    self.clear_screen_and_set_columns(132);
                }
            }
            MODE_MOUSE_1000 => emit!(self, program_uses_mouse_changed(false)),
            MODE_MOUSE_1001 => emit!(self, program_uses_mouse_changed(false)),
            MODE_MOUSE_1002 => emit!(self, program_uses_mouse_changed(false)),
            MODE_MOUSE_1003 => emit!(self, program_uses_mouse_changed(false)),
            MODE_BRACKETD_PASTE => emit!(self, program_bracketed_paste_mode_changed(true)),
            MODE_APP_SCREEN => {
                self.emulation_mut().screen[1].clear_selection();
                self.set_screen(1);
            }
            _ => {}
        }

        if mode < MODES_SCREEN || mode == MODE_NEWLINE {
            self.emulation_mut().screen[0].set_mode(mode);
            self.emulation_mut().screen[1].set_mode(mode);
        }
    }

    fn reset_mode(&mut self, mode: usize) {
        self.current_modes.mode[mode] = false;

        match mode {
            MODE_132_COLUMNS => {
                if self.get_mode(MODE_ALLOW_132_COLUMNS) {
                    self.clear_screen_and_set_columns(80);
                }
            }
            MODE_MOUSE_1000 => emit!(self, program_uses_mouse_changed(true)),
            MODE_MOUSE_1001 => emit!(self, program_uses_mouse_changed(true)),
            MODE_MOUSE_1002 => emit!(self, program_uses_mouse_changed(true)),
            MODE_MOUSE_1003 => emit!(self, program_uses_mouse_changed(true)),
            MODE_BRACKETD_PASTE => emit!(self, program_bracketed_paste_mode_changed(false)),
            MODE_APP_SCREEN => {
                self.emulation_mut().screen[0].clear_selection();
                self.set_screen(0);
            }
            _ => {}
        }

        if mode < MODES_SCREEN || mode == MODE_NEWLINE {
            self.emulation_mut().screen[0].reset_mode(mode);
            self.emulation_mut().screen[1].reset_mode(mode);
        }
    }

    #[inline]
    fn receive_char(&mut self, cc: wchar_t) {
        if cc == DEL {
            return;
        }
        self.cc = cc;

        if self.ces(CTL as wchar_t) {
            // ignore control characters in the text part of Xpe (aka OSC) "ESC]"
            // escape sequences; this matches what XTERM docs say
            if self.xpe() {
                self.prev_cc = cc;
                return;
            }

            // DEC HACK ALERT! Control Characters are allowed *within* esc sequences in
            // VT100 This means, they do neither a reset_tokenizer() nor a push_to_token().
            // Some of them, do of course. Guess this originates from a weakly layered
            // handling of the X-on X-off protocol, which comes really below this level.
            if cc == self.cntl(wch!('X')) || cc == self.cntl(wch!('Z')) || cc == ESC {
                self.reset_tokenizer();
            }
            if cc != ESC {
                self.process_token(ty_ctl!(cc + wch!('@')), 0, 0);
                return;
            }
        }

        // advance the state
        self.add_to_current_token(cc);

        if self.get_mode(MODE_ANSI) {
            if self.lec(1, 0, ESC) {
                return;
            }
            if self.lec(1, 0, ESC + 128) {
                self.token_buffer[0] = ESC;
                self.receive_char(wch!('['));
                return;
            }
            if self.les(2, 1, GRP as wchar_t) {
                return;
            }
            if self.xte() {
                self.process_window_attribute_change();
                self.reset_tokenizer();
                return;
            }
            if self.xpe() {
                self.prev_cc = cc;
                return;
            }
            if self.lec(3, 2, wch!('?')) {
                return;
            }
            if self.lec(3, 2, wch!('>')) {
                return;
            }
            if self.lec(3, 2, wch!('!')) {
                return;
            }
            if self.lun() {
                let apply_charset = self.apply_charset(cc);
                self.process_token(ty_chr!(), apply_charset, 0);
                self.reset_tokenizer();
                return;
            }
            if self.lec(2, 0, ESC as wchar_t) {
                self.process_token(ty_esc!(self.token_buffer[1]), 0, 0);
                self.reset_tokenizer();
                return;
            }
            if self.lec(3, 1, SCS as wchar_t) {
                self.process_token(ty_esc_cs!(self.token_buffer[1], self.token_buffer[2]), 0, 0);
                self.reset_tokenizer();
                return;
            }
            if self.lec(3, 1, wch!('#')) {
                self.process_token(ty_esc_de!(self.token_buffer[2]), 0, 0);
                self.reset_tokenizer();
                return;
            }
            if self.eps(CPN as wchar_t) {
                self.process_token(ty_csi_pn!(cc), self.argv[0] as wchar_t, self.argv[1]);
                self.reset_tokenizer();
                return;
            }
            if self.esp() {
                return;
            }
            if self.lec(5, 4, wch!('q')) && self.token_buffer[3] == wch!(' ') {
                self.process_token(ty_csi_ps_sp!(cc, self.argv[0]), self.argv[0] as wchar_t, 0);
                self.reset_tokenizer();
                return;
            }

            // resize = \e[8;<row>;<col>t
            if self.eps(CPS as wchar_t) {
                self.process_token(
                    ty_csi_ps!(cc, self.argv[0]),
                    self.argv[1] as wchar_t,
                    self.argv[2],
                );
                self.reset_tokenizer();
                return;
            }

            if self.epe() {
                self.process_token(ty_csi_pe!(cc), 0, 0);
                self.reset_tokenizer();
                return;
            }
            if self.ees(DIG as wchar_t) {
                self.add_digit((cc - wch!('0')) as i32);
                return;
            }
            if self.eec(wch!(';')) || self.eec(wch!(':')) {
                self.add_argument();
                return;
            }

            let mut i = 0usize;
            loop {
                if i > self.argc as usize {
                    break;
                }
                if self.epp() {
                    self.process_token(ty_csi_pr!(cc, self.argv[i]), 0, 0);
                } else if self.egt() {
                    // spec. case for ESC]>0c or ESC]>c
                    self.process_token(ty_csi_pg!(cc), 0, 0);
                } else if cc == wch!('m')
                    && self.argc - i as i32 >= 4
                    && (self.argv[i] == 38 || self.argv[i] == 48)
                    && self.argv[i + 1] == 2
                {
                    // ESC[ ... 48;2;<red>;<green>;<blue> ... m -or- ESC[ ...
                    // 38;2;<red>;<green>;<blue> ... m
                    i += 2;
                    let q = (self.argv[i] << 16) | (self.argv[i + 1] << 8) | self.argv[i + 2];
                    self.process_token(
                        ty_csi_ps!(cc, self.argv[i - 2]),
                        COLOR_SPACE_RGB as wchar_t,
                        q,
                    );
                    i += 2;
                } else if cc == wch!('m')
                    && self.argc - i as i32 >= 2
                    && (self.argv[i] == 38 || self.argv[i] == 48)
                    && self.argv[i + 1] == 5
                {
                    // ESC[ ... 48;5;<index> ... m -or- ESC[ ... 38;5;<index> ... m
                    i += 2;
                    self.process_token(
                        ty_csi_ps!(cc, self.argv[i - 2]),
                        COLOR_SPACE_256 as wchar_t,
                        self.argv[i],
                    );
                } else {
                    self.process_token(ty_csi_ps!(cc, self.argv[i]), 0, 0);
                }

                i += 1;
            }

            self.reset_tokenizer();
        } else {
            // VT52 Mode
            if self.lec(1, 0, ESC as wchar_t) {
                return;
            }
            if self.les(1, 0, CHR as wchar_t) {
                self.process_token(ty_chr!(), self.token_buffer[0], 0);
                self.reset_tokenizer();
                return;
            }
            if self.lec(2, 1, wch!('Y')) {
                return;
            }
            if self.lec(3, 1, wch!('Y')) {
                return;
            }
            if self.token_buffer_pos < 4 {
                self.process_token(ty_vt52!(self.token_buffer[1]), 0, 0);
                self.reset_tokenizer();
                return;
            }

            self.process_token(
                ty_vt52!(self.token_buffer[1]),
                self.token_buffer[2],
                self.token_buffer[3] as i32,
            );
            self.reset_tokenizer();
        }
    }

    #[inline]
    fn set_screen(&mut self, index: i32) {
        self.emulation_mut().set_screen(index)
    }

    ////////////////////////////////////////////////// Slots //////////////////////////////////////////////////
    #[inline]
    fn set_image_size(&mut self, lines: i32, columns: i32) {
        self.emulation
            .as_mut()
            .unwrap()
            .set_image_size(lines, columns)
    }

    #[inline]
    fn send_text(&self, text: String) {
        self.emulation().send_text(text)
    }

    #[inline]
    fn send_key_event(&mut self, event: KeyPressedEvent, from_paste: bool) {
        let modifiers = event.modifier();
        let mut states = State::None;

        if self.get_mode(MODE_NEWLINE) {
            states = states.or(State::NewLine)
        }
        if self.get_mode(MODE_ANSI) {
            states = states.or(State::Ansi)
        }
        if self.get_mode(MODE_APP_CURSOR_KEY) {
            states = states.or(State::CursorKeys)
        }
        if self.get_mode(MODE_APP_SCREEN) {
            states = states.or(State::AlternateScreen)
        }
        if self.get_mode(MODE_APP_KEY_PAD) && modifiers.has(KeyboardModifier::KeypadModifier) {
            states = states.or(State::ApplicationKeypad)
        }

        if modifiers.has(KeyboardModifier::ControlModifier) {
            match event.key_code() {
                KeyCode::KeyS => emit!(self, flow_control_key_pressed(true)),
                KeyCode::KeyQ | KeyCode::KeyC => emit!(self, flow_control_key_pressed(false)),
                _ => {}
            }
        }

        if self.emulation().key_translator.is_some() {
            let entry = nonnull_ref!(self.emulation().key_translator).find_entry(
                event.key_code().as_numeric(),
                modifiers,
                Some(states),
            );

            let mut text_to_send: Vec<u8> = vec![];

            // special handling for the Alt (aka. Meta) modifier.  pressing
            // Alt+[Character] results in Esc+[Character] being sent
            // (unless there is an entry defined for this particular combination
            // in the keyboard modifier)
            let wants_alt_modifier = entry
                .modifiers()
                .and(entry.modifier_mask())
                .has(KeyboardModifier::AltModifier);
            let wants_meta_modifer = entry
                .modifiers()
                .and(entry.modifier_mask())
                .has(KeyboardModifier::MetaModifier);
            let wants_any_modifier = (entry.state().as_u8() as u32
                & entry.modifier_mask().as_u32()
                & State::AnyModifier.as_u8() as u32)
                != 0;

            if modifiers.has(KeyboardModifier::AltModifier)
                && !(wants_alt_modifier || wants_any_modifier)
                && !event.text().is_empty()
            {
                text_to_send.splice(0..0, b"\x1B".to_owned());
            }
            if modifiers.has(KeyboardModifier::MetaModifier)
                && !(wants_meta_modifer || wants_any_modifier)
                && !event.text().is_empty()
            {
                text_to_send.splice(0..0, b"\x1B@s".to_owned());
            }

            if entry.command() != Command::None {
                if entry.command().has(Command::Erase) {
                    text_to_send.push(self.erase_char() as u8);
                } else {
                    emit!(self, handle_command_from_keyboard(entry.command()));
                }
            } else if !entry.text(None, None).is_empty() {
                text_to_send.extend_from_slice(
                    String::from_utf8(entry.text(Some(true), Some(modifiers)))
                        .unwrap()
                        .as_bytes(),
                );
            } else if modifiers.has(CTRL_MODIFIER)
                && event.key_code() >= KeyCode::KeyAt
                && event.key_code() <= KeyCode::KeyUnderscore
            {
                let key = (event.key_code().as_numeric() - KeyCode::KeyAt.as_numeric()) as u8 + 64;
                text_to_send.push(key & 0x1f);
            } else if event.key_code() == KeyCode::KeyTab {
                text_to_send.push(0x09);
            } else if event.key_code() == KeyCode::KeyPageUp {
                text_to_send.extend_from_slice(b"\x1B[5~".as_ref());
            } else if event.key_code() == KeyCode::KeyPageDown {
                text_to_send.extend_from_slice(b"\x1B[6~".as_ref());
            } else {
                text_to_send.extend_from_slice(event.text().as_bytes());
            }

            if !from_paste && !text_to_send.is_empty() {
                emit!(self, output_from_keypress_event());
            }

            if text_to_send.is_empty() {
                return;
            }

            let text_to_send = String::from_utf8(text_to_send).unwrap();
            emit!(self, send_data(text_to_send.as_str()));

            if self.emulation().use_local_display {
                let text_to_send = self
                    .emulation_mut()
                    .local_display
                    .extend(&event, text_to_send);
                if text_to_send.as_str() == "\u{200B}" {
                    let display_text = self.emulation().local_display.get_display_string();
                    if !display_text.is_empty() {
                        self.receive_data(
                            display_text.as_bytes(),
                            display_text.len() as i32,
                            DataSender::LocalDisplay,
                        );
                    }
                } else if !text_to_send.is_empty() {
                    self.receive_data(
                        text_to_send.as_bytes(),
                        text_to_send.len() as i32,
                        DataSender::LocalDisplay,
                    );
                }
            }
        } else {
            let translator_error = r#"No keyboard translator available.  
The information needed to convert key presses 
into characters to send to the terminal 
is missing."#;

            self.reset();
            let buffer = translator_error.as_bytes();
            self.receive_data(buffer, buffer.len() as i32, DataSender::Pty)
        }
    }

    #[inline]
    fn send_mouse_event(&self, mut cb: i32, cx: i32, cy: i32, event_type: u8) {
        if cx < 1 || cy < 1 {
            return;
        }

        // With the exception of the 1006 mode, button release is encoded in cb.
        // Note that if multiple extensions are enabled, the 1006 is used, so it's
        // okay to check for only that.
        if event_type == 2 && !self.get_mode(MODE_MOUSE_1006) {
            cb = 3;
        }

        // normal buttons are passed as 0x20 + button,
        // mouse wheel (buttons 4,5) as 0x5c + button
        if cb >= 4 {
            cb += 0x3c;
        }

        // Handle mouse motion:
        // add 32 to signify motion event
        if (self.get_mode(MODE_MOUSE_1002) || self.get_mode(MODE_MOUSE_1003)) && event_type == 1 {
            cb += 0x20;
        }

        let mut command = String::new();
        // Check the extensions in decreasing order of preference. Encoding the
        // release event above assumes that 1006 comes first.
        if self.get_mode(MODE_MOUSE_1006) {
            command = format!(
                "\x1B[<{};{};{}{}",
                cb,
                cx,
                cy,
                if event_type == 2 { 'm' } else { 'M' }
            )
        } else if self.get_mode(MODE_MOUSE_1015) {
            command = format!("\x1B[{};{};{}M", cb + 0x20, cx, cy)
        } else if self.get_mode(MODE_MOUSE_1005) {
            if cx <= 2015 && cy <= 2015 {
                // The xterm extension uses UTF-8 (up to 2 bytes) to encode
                // coordinate+32, no matter what the locale is.
                let coords: String = [
                    char::from_u32(cx as u32 + 0x20).unwrap(),
                    char::from_u32(cy as u32 + 0x20).unwrap(),
                ]
                .iter()
                .collect();
                command = format!("\x1B[M{}{}", (cb as u8 + 0x20) as char, coords)
            }
        } else if cx <= 223 && cy <= 223 {
            command = format!(
                "\x1B[M{}{}{}",
                (cb as u8 + 0x20) as char,
                (cx as u8 + 0x20) as char,
                (cy as u8 + 0x20) as char
            )
        }

        let len = command.len() as i32;
        self.send_string(command, len)
    }

    #[inline]
    fn send_string(&self, string: String, length: i32) {
        if length >= 0 {
            emit!(self, send_data(string.as_str()));
        } else {
            let len = string.len() as i32;
            if len == 0 {
                return;
            }
            emit!(self, send_data(string.as_str()));
        }
    }

    fn receive_data(&mut self, buffer: &[u8], len: i32, data_sender: DataSender) {
        if len == 0 {
            return;
        }
        emit!(self, state_set(EmulationState::NotifyActivity as i32));

        let utf8_text = String::from_utf8(buffer.to_vec())
            .expect("`Emulation` receive_data() parse utf-8 string failed.");
        let utf16_text = WideString::from_str(&utf8_text);

        // Send characters to terminal emulator
        let text_slice = utf16_text.as_slice();
        let mut execution_finish = false;
        for &ts in text_slice.iter() {
            #[allow(clippy::useless_transmute)]
            let end: uwchar_t = unsafe { std::mem::transmute(wch!('\u{200B}')) };

            if ts == end {
                execution_finish = true;
                continue;
            }

            self.receive_char(ts as wchar_t);
        }

        // Look for z-modem indicator
        for i in 0..len as usize {
            #[allow(clippy::char_lit_as_u8)]
            if buffer[i] == '\u{0030}' as u8
                && len as usize - i - 1 > 3
                && String::from_utf8(buffer[i + 1..i + 4].to_vec()).unwrap() == "B00"
            {
                emit!(self, zmodem_detected());
            }
        }

        self.direct_update();

        if data_sender == DataSender::Pty && self.emulation().use_local_display {
            if self.emulation().local_display.is_tabing() {
                let ld = &mut self.emulation_mut().local_display;
                ld.set_tabing(false);
            }

            if execution_finish {
                let screen = nonnull_ref!(self.emulation_mut().current_screen);

                self.emulation_mut().local_display.set_terminal_info(
                    screen.get_cursor_x() + 1,
                    screen.get_cursor_y() + 1,
                    screen.get_columns(),
                );

                self.emulation_mut().local_display.executed();
                let display_text = self.emulation().local_display.get_all_display_string();
                if !display_text.is_empty() {
                    self.receive_data(
                        display_text.as_bytes(),
                        display_text.len() as i32,
                        DataSender::LocalDisplay,
                    );
                }
            }
        }
    }

    #[inline]
    fn show_bulk(&mut self) {
        self.emulation_mut().show_bulk()
    }

    #[inline]
    fn direct_update(&mut self) {
        self.emulation_mut().direct_update()
    }

    #[inline]
    fn buffered_update(&mut self) {
        self.emulation_mut().buffered_update();
    }

    #[inline]
    fn uses_mouse_changed(&mut self, uses_mouse: bool) {
        self.emulation_mut().uses_mouse_changed(uses_mouse)
    }

    #[inline]
    fn bracketed_paste_mode_changed(&mut self, bracketed_paste_mode: bool) {
        self.emulation_mut()
            .bracketed_paste_mode_changed(bracketed_paste_mode)
    }

    #[inline]
    fn emit_cursor_change(&mut self, cursor_shape: u8, enable_blinking_cursor: bool) {
        self.emulation_mut()
            .emit_cursor_change(cursor_shape, enable_blinking_cursor)
    }

    #[inline]
    fn set_key_binding(&mut self, id: &str) {
        self.emulation_mut().set_key_binding(id)
    }

    #[inline]
    fn set_use_local_display(&mut self, use_local_display: bool) {
        self.emulation_mut().use_local_display = use_local_display;
    }

    #[inline]
    fn handle_control_insert(&mut self) {
        if self.emulation().current_screen().has_selected() {
            let origin_selection = self.emulation().current_screen().selected_text(true);
            let selection = origin_selection
                .trim_end_matches('\n')
                .trim_start_matches('\n');

            if !selection.is_empty() {
                System::clipboard().set_text(selection, ClipboardLevel::Os);
            }
        }
    }

    #[inline]
    fn handle_shift_insert(&mut self) {
        if let Some(text) = System::clipboard().text(ClipboardLevel::Os) {
            text.split('\n').for_each(|line| {
                if line.trim().is_empty() {
                    return;
                }

                let data = format!("{}\n", line);
                let evt =
                    KeyPressedEvent::new(KeyCode::Unknown, data, KeyboardModifier::NoModifier);
                self.send_key_event(evt, true);
            });
        }
    }
}
