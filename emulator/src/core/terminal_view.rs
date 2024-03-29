#![allow(dead_code)]
use super::{
    screen_window::{ScreenWindow, ScreenWindowSignals},
    uwchar_t,
};
use crate::tools::{
    character::{
        Character, ExtendedCharTable, LineProperty, DEFAULT_RENDITION, LINE_DOUBLE_HEIGHT,
        LINE_DOUBLE_WIDTH, LINE_WRAPPED, RE_BLINK, RE_BOLD, RE_CONCEAL, RE_CURSOR, RE_EXTEND_CHAR,
        RE_ITALIC, RE_OVERLINE, RE_STRIKEOUT, RE_UNDERLINE,
    },
    character_color::{
        CharacterColor, ColorEntry, BASE_COLOR_TABLE, DEFAULT_BACK_COLOR, DEFAULT_FORE_COLOR,
        TABLE_COLORS,
    },
    event::{KeyPressedEvent, ToKeyPressedEvent},
    filter::{FilterChainImpl, HotSpotImpl, HotSpotType, TerminalImageFilterChain},
    system_ffi::string_width,
};
use derivative::Derivative;
use lazy_static::lazy_static;
use libc::{c_void, memmove};
use log::warn;
use regex::Regex;
use std::{
    mem::size_of,
    ptr::NonNull,
    rc::Rc,
    sync::atomic::{AtomicBool, AtomicU64, Ordering},
    time::Duration,
};
use tmui::{
    application::{self, cursor_blinking_time},
    clipboard::ClipboardLevel,
    cursor::Cursor,
    graphics::painter::Painter,
    label::Label,
    prelude::*,
    scroll_bar::{ScrollBar, ScrollBarPosition},
    skia_safe::{
        self,
        textlayout::{
            FontCollection, ParagraphBuilder, ParagraphStyle, TextStyle, TypefaceFontProvider,
        },
        Matrix,
    },
    system::System,
    tlib::{
        connect, disconnect, emit,
        events::{KeyEvent, MouseEvent},
        figure::{Color, FPoint, FRect, FRegion, Size},
        global::bound64,
        namespace::{KeyCode, KeyboardModifier, MouseButton},
        nonnull_mut, nonnull_ref,
        object::{ObjectImpl, ObjectSubclass},
        ptr_mut, signals,
        timer::Timer,
    },
    widget::WidgetImpl,
};
use wchar::{wch, wchar_t};
#[cfg(not(windows))]
use widestring::U32String;
use widestring::{U16String, WideString};
use LineEncode::*;

lazy_static! {
    pub static ref REGULAR_EXPRESSION: Regex = Regex::new("\\r+$").unwrap();
}

#[extends(Widget, Layout(Stack))]
#[derive(Childrenable)]
pub struct TerminalView {
    extended_char_table: ExtendedCharTable,

    screen_window: Option<NonNull<ScreenWindow>>,

    #[derivative(Default(value = "true"))]
    allow_bell: bool,
    // Whether intense colors should be bold.
    #[derivative(Default(value = "true"))]
    bold_intense: bool,
    // Whether is test mode.
    draw_text_test_flag: bool,

    // whether has fixed pitch.
    #[derivative(Default(value = "true"))]
    fixed_font: bool,
    #[derivative(Default(value = "1."))]
    font_height: f32,
    #[derivative(Default(value = "1."))]
    font_width: f32,
    draw_text_addition_height: f32,

    #[derivative(Default(value = "5."))]
    left_margin: f32,
    #[derivative(Default(value = "5."))]
    top_margin: f32,
    #[derivative(Default(value = "5."))]
    left_base_margin: f32,
    #[derivative(Default(value = "5."))]
    top_base_margin: f32,

    // The total number of lines that can be displayed in the view.
    #[derivative(Default(value = "1"))]
    lines: i32,
    // The total number of columns that can be displayed in the view.
    #[derivative(Default(value = "1"))]
    columns: i32,

    #[derivative(Default(value = "1"))]
    used_lines: i32,
    #[derivative(Default(value = "1"))]
    used_columns: i32,

    #[derivative(Default(value = "1"))]
    content_height: i32,
    #[derivative(Default(value = "1"))]
    content_width: i32,

    image: Option<Vec<Character>>,
    image_size: i32,

    line_properties: Vec<LineProperty>,

    color_table: [ColorEntry; TABLE_COLORS],
    random_seed: u32,

    resizing: bool,
    terminal_size_hint: bool,
    #[derivative(Default(value = "true"))]
    terminal_size_start_up: bool,
    bidi_enable: bool,
    #[derivative(Default(value = "true"))]
    mouse_marks: bool,
    bracketed_paste_mode: bool,
    disable_bracketed_paste_mode: bool,

    drag_info: DragInfo,
    // initial selection point.
    i_pnt_sel: Point,
    // current selection point.
    pnt_sel: Point,
    //  help avoid flicker.
    triple_sel_begin: Point,
    // selection state
    act_sel: i32,
    word_selection_mode: bool,
    line_selection_mode: bool,
    preserve_line_breaks: bool,
    column_selection_mode: bool,

    scroll_bar: Option<NonNull<ScrollBar>>,
    scroll_bar_location: ScrollBarState,
    #[derivative(Default(value = "\":@-./_~\".to_string()"))]
    word_characters: String,
    bell_mode: BellMode,

    // hide text in paint event.
    blinking: bool,
    // has character to blink.
    has_blinker: bool,
    // hide cursor in paint event.
    cursor_blinking: bool,
    // has bliking cursor enable.
    has_blinking_cursor: bool,
    // allow text to blink.
    #[derivative(Default(value = "true"))]
    allow_blinking_text: bool,
    // require Ctrl key for drag.
    ctrl_drag: bool,
    // columns/lines are locked.
    is_fixed_size: bool,
    triple_click_mode: TripleClickMode,
    blink_timer: Timer,
    blink_cursor_timer: Timer,

    // true during visual bell.
    colors_inverted: bool,

    #[children]
    output_suspend_label: Box<Label>,

    #[children]
    resize_widget: Box<Label>,
    resize_timer: Timer,

    line_spacing: u32,
    #[derivative(Default(value = "1."))]
    opacity: f64,
    size: Size,

    // Add background_image Pixmap
    background_mode: BackgroundMode,

    #[derivative(Default(value = "TerminalImageFilterChain::new()"))]
    filter_chain: Box<TerminalImageFilterChain>,
    #[derivative(Default(value = "Some(CoordRegion::new())"))]
    mouse_over_hotspot_area: Option<CoordRegion>,

    cursor_shape: KeyboardCursorShape,
    cursor_color: Color,

    motion_after_pasting: MotionAfterPasting,
    confirm_multiline_paste: bool,
    trim_pasted_trailing_new_lines: bool,

    input_method_data: InputMethodData,

    #[derivative(Default(value = "true"))]
    draw_line_chars: bool,

    bind_session: ObjectId,
    /// `true` when the session this terminal view binded was finised.
    terminate: bool,
}

#[derive(Default)]
struct InputMethodData {
    preedit_string: WideString,
    previous_preedit_rect: FRect,
}

#[derive(Default)]
struct DragInfo {
    state: DragState,
    start: Point,
    // TODO: add `drag object`
}

//////////////////////////////////////////////////////////////////////////////////////////////////////////
/// Widget Implements
//////////////////////////////////////////////////////////////////////////////////////////////////////////
impl ObjectSubclass for TerminalView {
    const NAME: &'static str = "TerminalView";
}

impl ObjectImpl for TerminalView {
    fn construct(&mut self) {
        self.parent_construct();

        self.set_hexpand(true);
        self.set_vexpand(true);
        self.set_focus(true);
        self.set_rerender_difference(true);
    }

    fn initialize(&mut self) {
        self.extended_char_table.initialize();

        self.set_color_table(&BASE_COLOR_TABLE);
        self.set_mouse_tracking(true);

        self.resize_widget.set_halign(Align::Center);
        self.resize_widget.set_valign(Align::Center);
        self.resize_widget.set_content_halign(Align::Center);
        self.resize_widget.set_content_valign(Align::Center);
        self.resize_widget.width_request(10);
        self.resize_widget.height_request(10);

        self.output_suspend_label.set_halign(Align::Center);
        self.output_suspend_label.set_valign(Align::Center);
        self.output_suspend_label.set_content_halign(Align::Center);
        self.output_suspend_label.set_content_valign(Align::Center);
        self.output_suspend_label.width_request(10);
        self.output_suspend_label.height_request(10);

        self.resize_widget.hide();
        self.output_suspend_label.hide();

        connect!(self, size_changed(), self, when_resized(Size));
        connect!(
            self.blink_cursor_timer,
            timeout(),
            self,
            blink_cursor_event()
        );
    }
}

impl WidgetImpl for TerminalView {
    fn paint(&mut self, mut painter: &mut Painter) {
        painter.set_antialiasing(true);

        // TODO: Process the background image.

        if self.draw_text_test_flag {
            self.cal_draw_text_addition_height(&mut painter);
        }

        let region = self.redraw_region().clone();
        if region.is_empty() {
            let rect = self.contents_rect_f(Some(Coordinate::Widget));
            self.draw_background(&mut painter, rect, self.background(), true);
            self.draw_contents(&mut painter, rect);
        } else {
            for rect in region.into_iter() {
                self.draw_background(&mut painter, rect.rect(), self.background(), true);
                self.draw_contents(&mut painter, rect.rect());
            }
        }

        // self.draw_input_method_preedit_string(&mut painter, &self.preddit_rect());
        self.paint_filters(&mut painter);
    }

    fn on_key_pressed(&mut self, event: &KeyEvent) {
        self.act_sel = 0;

        if self.has_blinking_cursor {
            self.blink_cursor_timer
                .start(Duration::from_millis(cursor_blinking_time() as u64));
            if self.cursor_blinking {
                self.blink_cursor_event()
            } else {
                self.cursor_blinking = false
            }
        }

        self.screen_window_mut().unwrap().clear_selection();

        emit!(
            self.key_pressed_signal(),
            (event.to_key_pressed_event(), false)
        );
    }

    fn on_mouse_wheel(&mut self, event: &MouseEvent) {
        if event.delta().y() == 0 {
            return;
        }
        if self.screen_window.is_none() {
            return;
        }
        if self.screen_window().unwrap().screen().get_history_lines() == 0 {
            return;
        }

        // if the terminal program is not interested mouse events
        // then send the event to the ScrollBar.
        if self.mouse_marks {
            self.scroll_bar_mut().unwrap().on_mouse_wheel(event)
        }
    }

    fn on_mouse_pressed(&mut self, event: &MouseEvent) {
        if event.n_press() == 2 {
            self.handle_mouse_double_click(event)
        } else if event.n_press() == 3 {
            self.handle_mouse_triple_click(event)
        } else {
            self.handle_mouse_pressed(event)
        }
    }

    fn on_mouse_released(&mut self, event: &MouseEvent) {
        if self.screen_window().is_none() {
            return;
        }

        let (char_line, char_column) = self.get_character_position(event.position().into());
        if event.mouse_button() == MouseButton::LeftButton {
            if self.drag_info.state == DragState::DiPending {
                self.screen_window_mut().unwrap().clear_selection();
            } else if self.drag_info.state == DragState::DiDragging {
                if self.act_sel > 1 {
                    self.set_selection(
                        self.screen_window()
                            .unwrap()
                            .selected_text(self.preserve_line_breaks),
                    );
                }

                self.act_sel = 0;

                if !self.mouse_marks && !event.modifier().has(KeyboardModifier::ShiftModifier) {
                    let scroll_bar = self.scroll_bar().unwrap();
                    emit!(
                        self.mouse_signal(),
                        0,
                        char_column + 1,
                        char_line + 1 + scroll_bar.value() - scroll_bar.maximum(),
                        2u8
                    )
                }
            }

            self.drag_info.state = DragState::DiNone;
        } // end: event.mouse_button() == MouseButton::LeftButton

        if !self.mouse_marks
            && !event.modifier().has(KeyboardModifier::ShiftModifier)
            && (event.mouse_button() == MouseButton::RightButton
                || event.mouse_button() == MouseButton::MiddleButton)
        {
            let scroll_bar = self.scroll_bar().unwrap();
            let button = if event.mouse_button() == MouseButton::MiddleButton {
                1
            } else {
                2
            };

            emit!(
                self.mouse_signal(),
                button,
                char_column + 1,
                char_line + 1 + scroll_bar.value() - scroll_bar.maximum(),
                2u8
            );
        }
    }

    fn on_mouse_move(&mut self, event: &MouseEvent) {
        let (char_line, char_column) = self.get_character_position(event.position().into());

        let spot = self.filter_chain().hotspot_at(char_line, char_column);
        if let Some(spot) = spot {
            if spot.type_() == HotSpotType::Link {
                let mut previous_hotspot_area = self
                    .mouse_over_hotspot_area
                    .replace(CoordRegion::new())
                    .unwrap();
                let mouse_over_hotspot_area = self.mouse_over_hotspot_area.as_mut().unwrap();

                let mut r = FRect::default();

                if spot.start_line() == spot.end_line() {
                    r.set_coords(
                        spot.start_column() as f32 * self.font_width + self.left_base_margin,
                        spot.start_line() as f32 * self.font_height + self.top_base_margin,
                        spot.end_column() as f32 * self.font_width + self.left_base_margin,
                        (spot.end_line() + 1) as f32 * self.font_height - 1. + self.top_base_margin,
                    );
                    mouse_over_hotspot_area.add_rect(CoordRect::new(r, Coordinate::Widget));
                } else {
                    r.set_coords(
                        spot.start_column() as f32 * self.font_width + self.left_base_margin,
                        spot.start_line() as f32 * self.font_height + self.top_base_margin,
                        self.columns as f32 * self.font_width - 1. + self.left_base_margin,
                        (spot.start_line() + 1) as f32 * self.font_height + self.top_base_margin,
                    );
                    mouse_over_hotspot_area.add_rect(CoordRect::new(r, Coordinate::Widget));

                    for line in spot.start_line() + 1..spot.end_line() {
                        r.set_coords(
                            self.left_base_margin,
                            line as f32 * self.font_height + self.top_base_margin,
                            self.columns as f32 * self.font_width + self.left_base_margin,
                            (line + 1) as f32 * self.font_height + self.top_base_margin,
                        );
                        mouse_over_hotspot_area.add_rect(CoordRect::new(r, Coordinate::Widget));
                    }

                    r.set_coords(
                        self.left_base_margin,
                        spot.end_line() as f32 * self.font_height + self.top_base_margin,
                        spot.end_column() as f32 * self.font_width + self.left_base_margin,
                        (spot.end_line() + 1) as f32 * self.font_height + self.top_base_margin,
                    );
                    mouse_over_hotspot_area.add_rect(CoordRect::new(r, Coordinate::Widget));
                }

                // update
                previous_hotspot_area.add_region(mouse_over_hotspot_area);
                self.update_region(&previous_hotspot_area);
            }
        } else if !self.mouse_over_hotspot_area.as_ref().unwrap().is_empty() {
            let mouse_over_hotspot_area = self
                .mouse_over_hotspot_area
                .replace(CoordRegion::new())
                .unwrap();
            self.update_region(&mouse_over_hotspot_area);
        }

        if !event.mouse_button().has(MouseButton::LeftButton) {
            return;
        }

        if !self.mouse_marks && !event.modifier().has(KeyboardModifier::ShiftModifier) {
            let mut button = 3;
            let mouse_button = event.mouse_button();
            if mouse_button.has(MouseButton::LeftButton) {
                button = 0;
            }
            if mouse_button.has(MouseButton::MiddleButton) {
                button = 1;
            }
            if mouse_button.has(MouseButton::RightButton) {
                button = 2;
            }

            let scroll_bar = self.scroll_bar().unwrap();
            emit!(
                self.mouse_signal(),
                button,
                char_column + 1,
                char_line + 1 + scroll_bar.value() - scroll_bar.maximum(),
                1u8
            );
            return;
        }

        if self.drag_info.state == DragState::DiPending {
            let distance = 10;
            let pos = event.position();
            let drag_start = self.drag_info.start;
            if pos.0 > drag_start.x() + distance
                || pos.0 < drag_start.x() - distance
                || pos.1 > drag_start.y() + distance
                || pos.1 < drag_start.y() - distance
            {
                self.screen_window_mut().unwrap().clear_selection();

                self.do_drag();
            }
        } else if self.drag_info.state == DragState::DiDragging {
            return;
        }

        if self.act_sel == 0 {
            return;
        }

        if event.mouse_button().has(MouseButton::MiddleButton) {
            return;
        }

        self.extend_selection(event.position().into());
    }
}

impl TerminalView {
    pub fn new(session_id: ObjectId) -> Box<Self> {
        let mut view: Box<Self> = Object::new(&[]);
        view.bind_session = session_id;
        view
    }
}
//////////////////////////////////////////////////////////////////////////////////////////////////////////
/// TerminalView Singals
//////////////////////////////////////////////////////////////////////////////////////////////////////////
pub trait TerminalViewSignals: ActionExt {
    signals!(
        TerminalViewSignals:

        /// Emitted when the user presses a key whilst the terminal widget has focus.
        ///
        /// @param [`KeyEvent`] key event.
        /// @param [`bool`] from paste.
        key_pressed_signal();

        /// A mouse event occurred.
        /// @param [`i32`] button: The mouse button (0 for left button, 1 for middle button, 2
        /// for right button, 3 for release) <br>
        /// @param [`i32`] column: The character column where the event occurred <br>
        /// @param [`i32`] row: The character row where the event occurred <br>
        /// @param [`u8`] type: The type of event.  0 for a mouse press / release or 1 for
        /// mouse motion
        mouse_signal();

        changed_font_metrics_signal();
        changed_content_size_signal();

        /// Emitted when the user right clicks on the display, or right-clicks with the
        /// Shift key held down if [`uses_mouse()`] is true.
        ///
        /// This can be used to display a context menu.
        configure_request();

        /// When a shortcut which is also a valid terminal key sequence is pressed
        /// while the terminal widget  has focus, this signal is emitted to allow the
        /// host to decide whether the shortcut should be overridden. When the shortcut
        /// is overridden, the key sequence will be sent to the terminal emulation
        /// instead and the action associated with the shortcut will not be triggered.
        ///
        /// @p [`override`] is set to false by default and the shortcut will be triggered
        /// as normal.
        override_shortcut_check();

        is_busy_selecting();

        /// @param [`String`]
        /// @param [`i32`] length of the string, if there was a empty string, the value was -1/0
        send_string_to_emulation();

        copy_avaliable();
        term_get_focus();
        term_lost_focus();

        notify_bell();
        uses_mouse_changed();
    );
}
impl TerminalViewSignals for TerminalView {}

//////////////////////////////////////////////////////////////////////////////////////////////////////////
/// TerminalView Implements
//////////////////////////////////////////////////////////////////////////////////////////////////////////
impl TerminalView {
    /// Specified whether anti-aliasing of text in the terminal view
    /// is enabled or not.  Defaults to enabled.
    pub fn set_antialiasing(antialias: bool) {
        ANTIALIAS_TEXT.store(antialias, Ordering::SeqCst)
    }
    /// Returns true if anti-aliasing of text in the terminal is enabled.
    pub fn antialias() -> bool {
        ANTIALIAS_TEXT.load(Ordering::SeqCst)
    }

    #[inline]
    pub fn loc(&self, x: i32, y: i32) -> i32 {
        y * self.columns + x
    }

    #[inline]
    pub fn terminate(&mut self) {
        self.terminate = true
    }

    #[inline]
    pub fn is_terminate(&self) -> bool {
        self.terminate
    }

    //////////////////////////////////////////////// Drawing functions start.  ////////////////////////////////////////////////
    /// divides the part of the display specified by 'rect' into
    /// fragments according to their colors and styles and calls
    /// drawTextFragment() to draw the fragments
    fn draw_contents(&mut self, painter: &mut Painter, rect: FRect) {
        let _image = self.image();

        let tl = self.contents_rect(Some(Coordinate::Widget)).top_left();
        let tlx = tl.x();
        let tly = tl.y();

        let lux = (self.used_columns - 1).min(
            0.max(((rect.left() as f32 - tlx as f32 - self.left_margin) / self.font_width) as i32),
        );
        let luy = (self.used_lines - 1).min(
            0.max(((rect.top() as f32 - tly as f32 - self.top_margin) / self.font_height) as i32),
        );
        let rlx = (self.used_columns - 1).min(
            0.max(((rect.right() as f32 - tlx as f32 - self.left_margin) / self.font_width) as i32),
        );
        let rly = (self.used_lines - 1).min(0.max(
            ((rect.bottom() as f32 - tly as f32 - self.top_margin) / self.font_height) as i32,
        ));

        let buffer_size = self.used_columns as usize;
        let mut unistr = vec![0 as wchar_t; buffer_size];

        let mut y = luy;
        while y <= rly {
            let mut c = self.image()[self.loc(lux, y) as usize]
                .character_union
                .data();
            let mut x = lux;
            if c == 0 && x != 0 {
                // Search for start of multi-column character
                x -= 1;
            }
            while x <= rlx {
                let mut len = 1;
                let mut p = 0;

                // reset buffer to the maximal size
                unistr.resize(buffer_size, 0);

                // is this a single character or a sequence of characters ?
                if self.image()[self.loc(x, y) as usize].rendition & RE_EXTEND_CHAR != 0 {
                    // sequence of characters
                    let mut extended_char_length = 0 as wchar_t;
                    let chars = ExtendedCharTable::instance()
                        .lookup_extended_char(
                            self.image()[self.loc(x, y) as usize].character_union.data(),
                            &mut extended_char_length,
                        )
                        .unwrap();
                    for index in 0..extended_char_length as usize {
                        assert!(p < buffer_size);
                        unistr[p] = chars[index];
                        p += 1;
                    }
                } else {
                    c = self.image()[self.loc(x, y) as usize].character_union.data();
                    if c != 0 {
                        assert!(p < buffer_size);
                        unistr[p] = c;
                        p += 1;
                    }
                }

                let line_draw = self.is_line_char(c);
                let double_width = self.image()[self.image_size.min(self.loc(x, y) + 1) as usize]
                    .character_union
                    .data()
                    == 0;

                let img = &self.image()[self.loc(x, y) as usize];
                let current_foreground = img.foreground_color;
                let current_background = img.background_color;
                let current_rendition = img.rendition;

                let mut img = &self.image()[self.loc(x + len, y) as usize];
                while x + len <= rlx
                    && img.foreground_color == current_foreground
                    && img.background_color == current_background
                    && img.rendition == current_rendition
                    && (self.image()[self.image_size.min(self.loc(x + len, y) + 1) as usize]
                        .character_union
                        .data()
                        == 0)
                        == double_width
                    && self.is_line_char(img.character_union.data()) == line_draw
                {
                    c = img.character_union.data();
                    if c != 0 {
                        unistr[p] = c;
                        p += 1;
                    }

                    if double_width {
                        len += 1;
                    }
                    len += 1;

                    img = &self.image()[self.loc(x + len, y) as usize];
                }

                if x + len < self.used_columns
                    && self.image()[self.loc(x + len, y) as usize]
                        .character_union
                        .data()
                        == 0
                {
                    len += 1;
                }

                let save_fixed_font = self.fixed_font;
                if line_draw {
                    self.fixed_font = false;
                }
                unistr.resize(p as usize, 0);

                // Create a text scaling matrix for double width and double height lines.
                let mut text_scale = Matrix::new_identity();

                if y < self.line_properties.len() as i32 {
                    if self.line_properties[y as usize] & LINE_DOUBLE_WIDTH != 0 {
                        text_scale.set_scale_x(2.);
                    }
                    if self.line_properties[y as usize] & LINE_DOUBLE_HEIGHT != 0 {
                        text_scale.set_scale_y(2.);
                    }
                }

                // calculate the area in which the text will be drawn
                let mut text_area = self.calculate_text_area(tlx, tly, x, y, len);

                // move the calculated area to take account of scaling applied to the
                // painter. the position of the area from the origin (0,0) is scaled by
                // the opposite of whatever transformation has been applied to the
                // painter. this ensures that painting does actually start from
                // textArea.topLeft()
                //(instead of textArea.topLeft() * painter-scale)
                text_area.move_top_left(
                    &text_scale
                        .invert()
                        .unwrap()
                        .map_point(text_area.top_left())
                        .into(),
                );

                // Apply text scaling matrix.
                painter.set_transform(text_scale, true);

                // paint text fragment
                let style = self.image()[self.loc(x, y) as usize];
                let slice: Vec<uwchar_t> = unsafe { std::mem::transmute(unistr.clone()) };
                self.draw_text_fragment(painter, text_area, WideString::from_vec(slice), &style);

                self.fixed_font = save_fixed_font;

                // reset back to single-width, single-height lines.
                painter.set_transform(text_scale.invert().unwrap(), true);

                if y < self.line_properties.len() as i32 - 1 {
                    // double-height lines are represented by two adjacent lines
                    // containing the same characters
                    // both lines will have the LINE_DOUBLEHEIGHT attribute.
                    // If the current line has the LINE_DOUBLEHEIGHT attribute,
                    // we can therefore skip the next line
                    if self.line_properties[y as usize] & LINE_DOUBLE_HEIGHT != 0 {
                        y += 1;
                    }
                }

                x += len - 1;
                x += 1;
            }
            y += 1;
        }
    }
    /// draws a section of text, all the text in this section
    /// has a common color and style
    fn draw_text_fragment(
        &mut self,
        painter: &mut Painter,
        rect: FRect,
        text: WideString,
        style: &Character,
    ) {
        painter.save_pen();

        let foreground_color = style.foreground_color.color(&self.color_table);
        let background_color = style.background_color.color(&self.color_table);

        if background_color != self.background() {
            self.draw_background(painter, rect, background_color, false);
        }

        let mut invert_character_color = false;

        // draw text
        self.draw_characters(painter, rect, &text, style, invert_character_color);

        if style.rendition & RE_CURSOR != 0 {
            self.draw_cursor(painter, rect, foreground_color, &mut invert_character_color);
        }

        painter.restore_pen();
    }
    /// draws the background for a text fragment
    /// if useOpacitySetting is true then the color's alpha value will be set to
    /// the display's transparency (set with setOpacity()), otherwise the
    /// background will be drawn fully opaque
    fn draw_background(
        &mut self,
        painter: &mut Painter,
        rect: FRect,
        color: Color,
        _use_opacity_setting: bool,
    ) {
        // TODO: Return if there is a background image
        // TODO: Set the opacity
        painter.save();
        painter.fill_rect(rect, color);
        painter.restore();
    }
    /// draws the cursor character.
    fn draw_cursor(
        &mut self,
        painter: &mut Painter,
        rect: FRect,
        foreground_color: Color,
        invert_colors: &mut bool,
    ) {
        painter.set_antialiasing(false);
        let mut cursor_rect: FRect = rect.into();
        cursor_rect.set_height(self.font_height as f32 - self.line_spacing as f32 - 1.);

        if !self.cursor_blinking {
            if self.cursor_color.valid {
                painter.set_color(self.cursor_color);
            } else {
                painter.set_color(foreground_color);
            }

            if self.cursor_shape == KeyboardCursorShape::BlockCursor {
                // draw the cursor outline, adjusting the area so that
                // it is draw entirely inside 'rect'
                let line_width = painter.line_width().max(1.);
                let adjusted_cursor_rect = cursor_rect.adjusted(
                    line_width * 0.7,
                    line_width * 0.,
                    -line_width * 0.7,
                    -line_width * 0.,
                );

                painter.draw_rect(adjusted_cursor_rect);

                if self.is_focus() {
                    painter.fill_rect(
                        adjusted_cursor_rect,
                        if self.cursor_color.valid {
                            self.cursor_color
                        } else {
                            foreground_color
                        },
                    );

                    if !self.cursor_color.valid {
                        // invert the colour used to draw the text to ensure that the
                        // character at the cursor position is readable
                        *invert_colors = true;
                    }
                }
            } else if self.cursor_shape == KeyboardCursorShape::UnderlineCursor {
                painter.draw_line_f(
                    cursor_rect.left(),
                    cursor_rect.bottom(),
                    cursor_rect.right(),
                    cursor_rect.bottom(),
                )
            } else if self.cursor_shape == KeyboardCursorShape::IBeamCursor {
                painter.draw_line_f(
                    cursor_rect.left(),
                    cursor_rect.top(),
                    cursor_rect.left(),
                    cursor_rect.bottom(),
                )
            }
        }
        painter.set_antialiasing(true);
    }
    /// draws the characters or line graphics in a text fragment.
    fn draw_characters(
        &mut self,
        painter: &mut Painter,
        rect: FRect,
        text: &WideString,
        style: &Character,
        invert_character_color: bool,
    ) {
        // Don't draw text which is currently blinking.
        if self.blinking && style.rendition & RE_BLINK != 0 {
            return;
        }

        // Don't draw concealed characters.
        if style.rendition & RE_CONCEAL != 0 {
            return;
        }

        // Setup bold, underline, intalic, strkeout and overline
        let use_bold = style.rendition & RE_BOLD != 0 && self.bold_intense;
        let use_underline = style.rendition & RE_UNDERLINE != 0;
        let use_italic = style.rendition & RE_ITALIC != 0;
        let use_strike_out = style.rendition & RE_STRIKEOUT != 0;
        let use_overline = style.rendition & RE_OVERLINE != 0;

        let font = self.font_mut();
        let mut typeface = font.typeface().unwrap();
        if typeface.bold() != use_bold || typeface.italic() != use_italic {
            typeface.set_bold(use_bold);
            typeface.set_italic(use_italic);
            font.set_typeface(typeface);
        }
        painter.set_font(font.to_skia_font());

        let text_color = if invert_character_color {
            style.background_color
        } else {
            style.foreground_color
        };
        let color = text_color.color(&self.color_table);
        painter.set_color(color);

        // Draw text
        if self.is_line_char_string(text) {
            self.draw_line_char_string(painter, rect.x(), rect.y(), text, style);
        } else {
            let text = text
                .to_string()
                .expect("`draw_characters()` transfer wchar_t text to utf-8 failed.");

            if self.bidi_enable {
                painter.fill_rect(rect, style.background_color.color(&self.color_table));
                painter.draw_paragraph(
                    &text,
                    (rect.x() as f32, rect.y() as f32),
                    0.,
                    rect.width() as f32,
                    Some(1),
                    false
                );
            } else {
                let mut draw_rect = FRect::new(rect.x(), rect.y(), rect.width(), rect.height());
                draw_rect.set_height(draw_rect.height() + self.draw_text_addition_height);

                painter.fill_rect(draw_rect, style.background_color.color(&self.color_table));
                // Draw the text start at the left-bottom.
                painter.draw_paragraph(&text, (rect.x(), rect.y()), 0., self.size().width() as f32, Some(1), false);

                if use_underline {
                    let y = draw_rect.bottom() as f32 - 0.5;
                    painter.draw_line_f(draw_rect.left() as f32, y, draw_rect.right() as f32, y)
                }

                if use_strike_out {
                    let y = (draw_rect.top() as f32 + draw_rect.bottom() as f32) / 2.;
                    painter.draw_line_f(draw_rect.left() as f32, y, draw_rect.right() as f32, y)
                }

                if use_overline {
                    let y = draw_rect.top() as f32 + 0.5;
                    painter.draw_line_f(draw_rect.left() as f32, y, draw_rect.right() as f32, y)
                }
            }
        }
    }
    /// draws a string of line graphics.
    fn draw_line_char_string(
        &mut self,
        painter: &mut Painter,
        x: f32,
        y: f32,
        str: &WideString,
        attributes: &Character,
    ) {
        painter.save_pen();

        if attributes.rendition & RE_BOLD != 0 && self.bold_intense {
            painter.set_line_width(3.);
        }

        let wchar_t_bytes = str.as_vec();
        for i in 0..wchar_t_bytes.len() {
            let code = (wchar_t_bytes[i] & 0xff) as u8;
            if LINE_CHARS[code as usize] != 0 {
                draw_line_char(
                    painter,
                    x + (self.font_width * i as f32),
                    y,
                    self.font_width,
                    self.font_height,
                    code,
                )
            } else {
                draw_other_char(
                    painter,
                    x + (self.font_width * i as f32),
                    y,
                    self.font_width,
                    self.font_height,
                    code,
                )
            }
        }

        painter.restore_pen();
    }
    /// draws the preedit string for input methods.
    fn draw_input_method_preedit_string(&mut self, painter: &mut Painter, rect: &Rect) {
        // TODO
    }

    fn paint_filters(&mut self, painter: &mut Painter) {
        let cursor_pos = self.map_to_widget_f(&Cursor::position().into());

        let (cursor_line, cursor_column) = self.get_character_position(cursor_pos);
        let cursor_character = self.image()[self.loc(cursor_column, cursor_line) as usize];

        painter.set_color(
            cursor_character
                .foreground_color
                .color(self.get_color_table()),
        );

        let spots = self.filter_chain.hotspots();
        for spot in spots.iter() {
            let mut region = FRegion::default();

            if spot.type_() == HotSpotType::Link {
                self.calc_hotspot_link_region(spot, &mut region)
            }

            for line in spot.start_line()..=spot.end_line() {
                self.paint_hotspot_each_line(line, spot, &region, painter)
            }
        }
    }

    fn calc_hotspot_link_region(&self, spot: &Rc<Box<dyn HotSpotImpl>>, region: &mut FRegion) {
        let mut r = FRect::default();
        if spot.start_line() == spot.end_line() {
            r.set_coords(
                spot.start_column() as f32 * self.font_width + 1. + self.left_base_margin,
                spot.start_line() as f32 * self.font_height + 1. + self.top_base_margin,
                spot.end_column() as f32 * self.font_width - 1. + self.left_base_margin,
                (spot.end_line() as f32 + 1.) * self.font_height - 1. + self.top_base_margin,
            );
            region.add_rect(r);
        } else {
            r.set_coords(
                spot.start_column() as f32 * self.font_width + 1. + self.left_base_margin,
                spot.start_line() as f32 * self.font_height + 1. + self.top_base_margin,
                self.columns as f32 * self.font_width - 1. + self.left_base_margin,
                (spot.start_line() as f32 + 1.) * self.font_height - 1. + self.top_base_margin,
            );
            region.add_rect(r);

            for line in spot.start_line() + 1..spot.end_line() {
                r.set_coords(
                    0. * self.font_width + 1. + self.left_base_margin,
                    line as f32 * self.font_height + 1. + self.top_base_margin,
                    self.columns as f32 * self.font_width - 1. + self.left_base_margin,
                    (line as f32 + 1.) * self.font_height - 1. + self.top_base_margin,
                );
                region.add_rect(r);
            }
            r.set_coords(
                0. * self.font_width + 1. + self.left_base_margin,
                spot.end_line() as f32 * self.font_height + 1. + self.top_base_margin,
                spot.end_column() as f32 * self.font_width - 1. + self.left_base_margin,
                (spot.end_line() as f32 + 1.) * self.font_height - 1. + self.top_base_margin,
            );
            region.add_rect(r);
        }
    }

    fn paint_hotspot_each_line(
        &self,
        line: i32,
        spot: &Rc<Box<dyn HotSpotImpl>>,
        region: &FRegion,
        painter: &mut Painter,
    ) {
        let mut start_column = 0;
        let mut end_column = self.columns - 1;

        // ignore whitespace at the end of the lines:
        while self.image()[self.loc(end_column, line) as usize]
            .character_union
            .equals(wch!(' '))
            && end_column > 0
        {
            end_column -= 1;
        }

        // increment here because the column which we want to set 'endColumn' to
        // is the first whitespace character at the end of the line:
        end_column += 1;

        if line == spot.start_line() {
            start_column = spot.start_column();
        }
        if line == spot.end_line() {
            end_column = spot.end_column();
        }

        // subtract one pixel from the right and bottom so that
        // we do not overdraw adjacent hotspots.
        //
        // subtracting one pixel from all sides also prevents an edge case where
        // moving the mouse outside a link could still leave it underlined
        // because the check below for the position of the cursor finds it on the border of the target area.
        let mut r = FRect::default();
        r.set_coords(
            start_column as f32 * self.font_width + 1. + self.left_base_margin,
            line as f32 * self.font_height + 1. + self.top_base_margin,
            end_column as f32 * self.font_width - 1. + self.left_base_margin,
            (line as f32 + 1.) * self.font_height - 1. + self.top_base_margin,
        );

        match spot.type_() {
            HotSpotType::Link => {
                let (_, metrics) = self.font().to_skia_font().metrics();
                let base_line = r.bottom() - metrics.descent;
                let under_line_pos = base_line + metrics.underline_position().unwrap();
                if region.contains_point(&self.map_to_widget_f(&Cursor::position().into())) {
                    painter.draw_line_f(r.left(), under_line_pos, r.right(), under_line_pos);
                }
            }
            HotSpotType::Marker => painter.fill_rect(r, Color::from_rgba(255, 0, 0, 120)),
            _ => {}
        }
    }

    fn cal_draw_text_addition_height(&mut self, painter: &mut Painter) {
        // let test_rect = Rect::new(1, 1, self.font_width * 4, self.font_height);
        // painter.draw_text(LTR_OVERRIDE_CHAR, origin)
    }
    //////////////////////////////////////////////// Drawing functions end.  ////////////////////////////////////////////////

    #[inline]
    fn when_resized(&mut self, size: Size) {
        if size.width() == 0 || size.height() == 0 {
            return;
        }
        self.update_image_size();
        self.process_filters();
    }

    /// Returns the terminal color palette used by the view.
    #[inline]
    pub fn get_color_table(&self) -> &[ColorEntry] {
        &self.color_table
    }
    /// Sets the terminal color palette used by the view.
    #[inline]
    pub fn set_color_table(&mut self, table: &[ColorEntry]) {
        for i in 0..TABLE_COLORS {
            self.color_table[i] = table[i];
        }

        self.set_background_color(self.color_table[DEFAULT_BACK_COLOR as usize].color)
    }

    /// Sets the seed used to generate random colors for the view
    /// (in color schemes that support them).
    #[inline]
    pub fn set_random_seed(&mut self, seed: u32) {
        self.random_seed = seed
    }
    /// Returns the seed used to generate random colors for the view
    /// (in color schemes that support them).
    #[inline]
    pub fn random_seed(&self) -> u32 {
        self.random_seed
    }

    /// Sets the opacity of the terminal view.
    #[inline]
    pub fn set_opacity(&mut self, opacity: f64) {
        self.opacity = bound64(0., opacity, 1.);
    }

    /// Sets the background image of the terminal view.
    pub fn set_background_image(&mut self, image: &str) {
        if !image.is_empty() {
            // TODO: load background image to Pixmap
        } else {
            // TODO: create a empty Pixmap
        }
    }
    /// Sets the background image mode of the terminal view.
    #[inline]
    pub fn set_background_mode(&mut self, mode: BackgroundMode) {
        self.background_mode = mode
    }

    /// Specifies whether the terminal display has a vertical scroll bar, and if so
    /// whether it is shown on the left or right side of the view.
    pub fn set_scroll_bar_position(&mut self, position: ScrollBarState) {
        if self.scroll_bar_location == position {
            return;
        }

        if position == ScrollBarState::NoScrollBar {
            nonnull_mut!(self.scroll_bar).hide();
        } else {
            nonnull_mut!(self.scroll_bar).show();
        }

        self.top_margin = 1.;
        self.left_margin = 1.;
        self.scroll_bar_location = position;

        self.propagate_size();
        self.update();
    }
    /// Setting the current position and range of the display scroll bar.
    pub fn set_scroll(&mut self, cursor: i32, lines: i32) {
        let scroll_bar = nonnull_mut!(self.scroll_bar);
        if scroll_bar.minimum() == 0
            && scroll_bar.maximum() == (lines - self.lines)
            && scroll_bar.value() == cursor
        {
            return;
        }
        disconnect!(scroll_bar, value_changed(), self, null);
        scroll_bar.set_range(0, lines - self.lines);
        scroll_bar.set_single_step(1);
        scroll_bar.set_page_step(lines);
        scroll_bar.set_value(cursor);
        connect!(
            scroll_bar,
            value_changed(),
            self,
            scroll_bar_position_changed(i32)
        );
    }
    /// Scroll to the bottom of the terminal (reset scrolling).
    pub fn scroll_to_end(&mut self) {
        let scroll_bar = nonnull_mut!(self.scroll_bar);
        disconnect!(scroll_bar, value_changed(), self, null);
        scroll_bar.set_value(scroll_bar.maximum());
        connect!(
            scroll_bar,
            value_changed(),
            self,
            scroll_bar_position_changed(i32)
        );

        let screen_window = nonnull_mut!(self.screen_window);
        screen_window.scroll_to(scroll_bar.value() + 1);
        screen_window.set_track_output(screen_window.at_end_of_output());
    }

    /// Returns the display's filter chain.  When the image for the display is
    /// updated, the text is passed through each filter in the chain.  Each filter
    /// can define hotspots which correspond to certain strings (such as URLs or
    /// particular words). Depending on the type of the hotspots created by the
    /// filter ( returned by Filter::Hotspot::type() ) the view will draw visual
    /// cues such as underlines on mouse-over for links or translucent rectangles
    /// for markers.
    ///
    /// To add a new filter to the view, call:
    ///      view->filter_chain()->add_filter( filterObject );
    pub fn filter_chain(&self) -> &impl FilterChainImpl {
        self.filter_chain.as_ref()
    }

    /// Updates the filters in the display's filter chain.  This will cause
    /// the hotspots to be updated to match the current image.
    ///
    /// TODO: This function can be expensive depending on the
    /// image size and number of filters in the filterChain()
    pub fn process_filters(&mut self) {
        if self.screen_window.is_none() {
            return;
        }
        let screen_window = nonnull_mut!(self.screen_window);

        let mut pre_update_hotspots = self.hotspot_region();

        // use [`ScreenWindow::get_image()`] here rather than `image` because
        // other classes may call process_filters() when this view's
        // ScreenWindow emits a scrolled() signal - which will happen before
        // update_image() is called on the display and therefore _image is
        // out of date at this point
        let window_lines = screen_window.window_lines();
        let window_columns = screen_window.window_columns();
        let line_properties = &screen_window.get_line_properties();
        let image = screen_window.get_image();
        self.filter_chain
            .set_image(image, window_lines, window_columns, line_properties);
        self.filter_chain.process();

        let post_update_hotspots = self.hotspot_region();

        // Should only update the region in pre_update_hotspots|post_update_hotspots
        pre_update_hotspots.or(&post_update_hotspots);
        if pre_update_hotspots.is_valid() {
            self.update_rect(CoordRect::new(pre_update_hotspots, Coordinate::Widget));
        }
    }

    /// Returns a list of menu actions created by the filters for the content
    /// at the given @p position.
    pub fn filter_actions(&self, _position: Point) -> Vec<Action> {
        todo!()
    }

    pub fn handle_mouse_pressed(&mut self, evt: &MouseEvent) {
        if self.screen_window().is_none() {
            return;
        }

        let modifier = evt.modifier();
        let (char_line, char_column) = self.get_character_position(evt.position().into());

        if evt.mouse_button() == MouseButton::LeftButton {
            self.line_selection_mode = false;
            self.word_selection_mode = false;

            let selected = self
                .screen_window()
                .unwrap()
                .is_selected(char_column, char_line);

            if (!self.ctrl_drag || modifier.has(KeyboardModifier::ControlModifier)) && selected {
                self.drag_info.state = DragState::DiPending;
                self.drag_info.start = evt.position().into();
            } else {
                self.drag_info.state = DragState::DiNone;

                self.preserve_line_breaks = !modifier.has(KeyboardModifier::ControlModifier)
                    && !modifier.has(KeyboardModifier::AltModifier);
                self.column_selection_mode = modifier.has(KeyboardModifier::AltModifier)
                    && modifier.has(KeyboardModifier::ControlModifier);

                if self.mouse_marks || modifier.has(KeyboardModifier::ShiftModifier) {
                    self.screen_window_mut().unwrap().clear_selection();

                    let mut pos = Point::new(char_column, char_line);
                    *pos.y_mut() += self.scroll_bar().unwrap().value();
                    self.pnt_sel = pos;
                    self.i_pnt_sel = pos;
                    self.act_sel = 1;
                } else {
                    let scroll_bar = self.scroll_bar().unwrap();
                    emit!(
                        self.mouse_signal(),
                        0,
                        char_column + 1,
                        char_line + 1 + scroll_bar.value() - scroll_bar.maximum(),
                        0u8
                    )
                }

                let spot = self.filter_chain.hotspot_at(char_line, char_column);
                if let Some(spot) = spot {
                    spot.activate("click-action");
                }
            }
        } else if evt.mouse_button() == MouseButton::MiddleButton {
            if self.mouse_marks || modifier.has(KeyboardModifier::ShiftModifier) {
                self.emit_selection(true, modifier.has(KeyboardModifier::ControlModifier));
            } else {
                let scroll_bar = self.scroll_bar().unwrap();
                emit!(
                    self.mouse_signal(),
                    1,
                    char_column + 1,
                    char_line + 1 + scroll_bar.value() - scroll_bar.maximum(),
                    0u8
                );
            }
        } else if evt.mouse_button() == MouseButton::RightButton {
            if self.mouse_marks || modifier.has(KeyboardModifier::ShiftModifier) {
                let pos: Point = evt.position().into();
                emit!(self.configure_request(), pos);
            } else {
                let scroll_bar = self.scroll_bar().unwrap();
                emit!(
                    self.mouse_signal(),
                    2,
                    char_column + 1,
                    char_line + 1 + scroll_bar.value() - scroll_bar.maximum(),
                    0u8
                );
            }
        }
    }

    pub fn handle_mouse_double_click(&mut self, evt: &MouseEvent) {
        if evt.mouse_button() != MouseButton::LeftButton {
            return;
        }
        if self.screen_window.is_none() {
            return;
        }
        let modifier = evt.modifier();

        let (char_line, char_column) = self.get_character_position(evt.position().into());
        let pos = Point::new(char_column, char_line);

        if !self.mouse_marks && !modifier.has(KeyboardModifier::ShiftModifier) {
            let scroll_bar = self.scroll_bar().unwrap();
            emit!(
                self.mouse_signal(),
                0,
                pos.x() + 1,
                pos.y() + 1 + scroll_bar.value() - scroll_bar.maximum(),
                0u8
            );
            return;
        }

        self.screen_window_mut().unwrap().clear_selection();
        let mut bgn_sel = pos;
        let mut end_sel = pos;
        let mut i = self.loc(bgn_sel.x(), bgn_sel.y());
        self.i_pnt_sel = bgn_sel;
        *self.i_pnt_sel.y_mut() += self.scroll_bar().unwrap().value();

        self.word_selection_mode = true;

        // find word boundaries:
        let sel_class = self.char_class(self.image()[i as usize].character_union.data());

        // find the start of the word:
        let mut x = bgn_sel.x();
        while (x > 0
            || (bgn_sel.y() > 0
                && self.line_properties[bgn_sel.y() as usize - 1] & LINE_WRAPPED != 0))
            && self.char_class(self.image()[i as usize - 1].character_union.data()) == sel_class
        {
            i -= 1;
            if x > 0 {
                x -= 1;
            } else {
                x = self.used_columns - 1;
                *bgn_sel.y_mut() -= 1;
            }
        }

        bgn_sel.set_x(x);
        self.screen_window_mut()
            .unwrap()
            .set_selection_start(bgn_sel.x(), bgn_sel.y(), false);

        // find the end of the word:
        i = self.loc(end_sel.x(), end_sel.y());
        x = end_sel.x();
        while (x < self.used_columns - 1
            || (end_sel.y() < self.used_lines - 1
                && self.line_properties[end_sel.y() as usize] & LINE_WRAPPED != 0))
            && self.char_class(self.image()[i as usize + 1].character_union.data()) == sel_class
        {
            i += 1;
            if x < self.used_columns - 1 {
                x += 1;
            } else {
                x = 0;
                *end_sel.y_mut() += 1;
            }
        }

        end_sel.set_x(x);

        // In word selection mode don't select @ (64) if at end of word.
        if self.image()[i as usize].character_union.data() == wch!('@')
            && end_sel.x() - bgn_sel.x() > 0
        {
            end_sel.set_x(x - 1);
        }

        self.act_sel = 2;

        self.screen_window_mut()
            .unwrap()
            .set_selection_end(end_sel.x(), end_sel.y());

        self.set_selection(
            self.screen_window()
                .unwrap()
                .selected_text(self.preserve_line_breaks),
        );
    }

    pub fn handle_mouse_triple_click(&mut self, evt: &MouseEvent) {
        if self.screen_window().is_none() {
            return;
        }

        let (char_line, char_column) = self.get_character_position(evt.position().into());
        self.i_pnt_sel = Point::new(char_column, char_line);

        self.screen_window_mut().unwrap().clear_selection();

        self.line_selection_mode = true;
        self.word_selection_mode = false;

        self.act_sel = 2;

        while self.i_pnt_sel.y() > 0
            && self.line_properties[self.i_pnt_sel.y() as usize - 1] & LINE_WRAPPED != 0
        {
            *self.i_pnt_sel.y_mut() -= 1;
        }

        match self.triple_click_mode {
            TripleClickMode::SelectForwardsFromCursor => {
                let mut i = self.loc(self.i_pnt_sel.x(), self.i_pnt_sel.y());
                let sel_class = self.char_class(self.image()[i as usize].character_union.data());
                let mut x = self.i_pnt_sel.x();

                while (x > 0
                    || (self.i_pnt_sel.y() > 0
                        && self.line_properties[self.i_pnt_sel.y() as usize - 1] & LINE_WRAPPED
                            != 0))
                    && self.char_class(self.image()[i as usize - 1].character_union.data())
                        == sel_class
                {
                    i -= 1;
                    if x > 0 {
                        x -= 1;
                    } else {
                        x = self.columns - 1;
                        *self.i_pnt_sel.y_mut() -= 1;
                    }
                }

                let y = self.i_pnt_sel.y();
                self.screen_window_mut()
                    .unwrap()
                    .set_selection_start(x, y, false);
                self.triple_sel_begin = Point::new(x, y);
            }
            TripleClickMode::SelectWholeLine => {
                let y = self.i_pnt_sel.y();
                self.screen_window_mut()
                    .unwrap()
                    .set_selection_start(0, y, false);
                self.triple_sel_begin = Point::new(0, y);
            }
        }

        while self.i_pnt_sel.y() < self.lines - 1
            && self.line_properties[self.i_pnt_sel.y() as usize] & LINE_WRAPPED != 0
        {
            *self.i_pnt_sel.y_mut() += 1;
        }

        let column = self.columns - 1;
        let line = self.i_pnt_sel.y();
        self.screen_window_mut()
            .unwrap()
            .set_selection_end(column, line);

        self.set_selection(
            self.screen_window()
                .unwrap()
                .selected_text(self.preserve_line_breaks),
        );

        let scroll_bar = self.scroll_bar().unwrap();
        *self.i_pnt_sel.y_mut() += scroll_bar.value();
    }

    /// Returns true if the cursor is set to blink or false otherwise.
    #[inline]
    pub fn blinking_cursor(&self) -> bool {
        self.has_blinking_cursor
    }

    /// Specifies whether or not the cursor blinks.
    #[inline]
    pub fn set_blinking_cursor(&mut self, blink: bool) {
        self.has_blinking_cursor = blink;

        if blink && !self.blink_cursor_timer.is_active() {
            self.blink_cursor_timer.start(Duration::from_millis(
                application::cursor_blinking_time() as u64,
            ))
        }

        if !blink && self.blink_cursor_timer.is_active() {
            self.blink_cursor_timer.stop();
            if self.cursor_blinking {
                self.blink_cursor_event()
            }
        }
    }

    /// Specifies whether or not text can blink.
    #[inline]
    pub fn set_blinking_text_enable(&mut self, blink: bool) {
        self.allow_blinking_text = blink
    }

    #[inline]
    pub fn set_ctrl_drag(&mut self, enable: bool) {
        self.ctrl_drag = enable
    }
    #[inline]
    pub fn ctrl_drag(&self) -> bool {
        self.ctrl_drag
    }

    /// Sets how the text is selected when the user triple clicks within the view.
    #[inline]
    pub fn set_triple_click_mode(&mut self, mode: TripleClickMode) {
        self.triple_click_mode = mode
    }
    #[inline]
    pub fn get_triple_click_mode(&self) -> TripleClickMode {
        self.triple_click_mode
    }

    #[inline]
    pub fn set_line_spacing(&mut self, spacing: u32) {
        self.line_spacing = spacing;

        // line spacing was changed, should recalculate the font_width
        self.set_vt_font(self.font().to_skia_font())
    }
    #[inline]
    pub fn line_spacing(&self) -> u32 {
        self.line_spacing
    }

    #[inline]
    pub fn set_margin(&mut self, margin: i32) {
        self.top_base_margin = margin as f32;
        self.left_base_margin = margin as f32;
    }
    #[inline]
    pub fn margin(&mut self) -> i32 {
        self.top_base_margin as i32
    }

    #[inline]
    pub fn set_scroll_bar(&mut self, scroll_bar: &mut ScrollBar) {
        self.scroll_bar = NonNull::new(scroll_bar)
    }

    /// @param [`use_x_selection`] Store and retrieve data from global mouse selection.
    /// Support for selection is only available on systems with global mouse selection (such as X11).
    pub fn emit_selection(&mut self, use_x_selection: bool, append_return: bool) {
        if self.screen_window.is_none() {
            return;
        }

        // Paste Clipboard by simulating keypress events
        let text = if use_x_selection {
            // TODO: Paste selections
            None
        } else {
            System::clipboard().text(ClipboardLevel::Os)
        };
        if let Some(mut text) = text {
            if text.is_empty() {
                return;
            }

            text = text.replace("\r\n", "\n").replace("\n", "\r");

            if self.trim_pasted_trailing_new_lines {
                text = REGULAR_EXPRESSION.replace(&text, "").to_string();
            }

            if self.confirm_multiline_paste && text.contains('\r') {
                if !self.multiline_confirmation(&text) {
                    return;
                }
            }

            self.bracket_text(&mut text);

            // appendReturn is intentionally handled _after_ enclosing texts with
            // brackets as that feature is used to allow execution of commands
            // immediately after paste. Ref: https://bugs.kde.org/show_bug.cgi?id=16179
            if append_return {
                text.push('\r');
            }

            let e = KeyPressedEvent::new(KeyCode::Unknown, text, KeyboardModifier::NoModifier);
            emit!(self.key_pressed_signal(), e, true);

            let screen_window = nonnull_mut!(self.screen_window);
            screen_window.clear_selection();

            match self.motion_after_pasting {
                MotionAfterPasting::MoveStartScreenWindow => {
                    screen_window.set_track_output(true);
                    screen_window.scroll_to(0);
                }
                MotionAfterPasting::MoveEndScreenWindow => {
                    self.scroll_to_end();
                }
                MotionAfterPasting::NoMoveScreenWindow => {}
            }
        }
    }

    /// change and wrap text corresponding to paste mode.
    #[inline]
    pub fn bracket_text(&self, text: &mut String) {
        if self.bracketed_paste_mode() && !self.disable_bracketed_paste_mode {
            text.insert_str(0, "\u{001b}[200~");
            text.push_str("\u{001b}[201~");
        }
    }

    /// Sets the shape of the keyboard cursor. This is the cursor drawn
    /// at the position in the terminal where keyboard input will appear.
    ///
    /// In addition the terminal display widget also has a cursor for
    /// the mouse pointer, which can be set using the QWidget::setCursor() method.
    ///
    /// Defaults to BlockCursor
    #[inline]
    pub fn set_keyboard_cursor_shape(&mut self, shape: KeyboardCursorShape) {
        self.cursor_shape = shape
    }
    /// Returns the shape of the keyboard cursor.
    #[inline]
    pub fn keyboard_cursor_shape(&self) -> KeyboardCursorShape {
        self.cursor_shape
    }
    /// Sets the color used to draw the keyboard cursor.
    ///
    /// The keyboard cursor defaults to using the foreground color of the character
    /// underneath it.
    ///
    /// @param [`use_foreground_color`] If true, the cursor color will change to match
    /// the foreground color of the character underneath it as it is moved, in this
    /// case, the @p color parameter is ignored and the color of the character
    /// under the cursor is inverted to ensure that it is still readable.
    ///
    /// @param [`color`] The color to use to draw the cursor.  This is only taken into
    /// account if @p [`use_foreground_color`] is false.
    #[inline]
    pub fn set_keyboard_cursor_color(&mut self, use_foreground_color: bool, color: Color) {
        if use_foreground_color {
            self.cursor_color = Color::new();
        } else {
            self.cursor_color = color
        }
    }
    /// Returns the color of the keyboard cursor, or an invalid color if the
    /// keyboard cursor color is set to change according to the foreground color of
    /// the character underneath it.
    #[inline]
    pub fn keyboard_cursor_color(&self) -> Color {
        self.cursor_color
    }

    /// Returns the number of lines of text which can be displayed in the widget.
    ///
    /// This will depend upon the height of the widget and the current font.
    /// See [`font_height()`]
    #[inline]
    pub fn lines(&self) -> i32 {
        self.lines
    }
    /// Returns the number of characters of text which can be displayed on
    /// each line in the widget.
    ///
    /// This will depend upon the width of the widget and the current font.
    /// See [`font_width()`]
    #[inline]
    pub fn columns(&self) -> i32 {
        self.columns
    }

    /// Returns the height of the characters in the font used to draw the text in
    /// the view.
    #[inline]
    pub fn font_height(&self) -> f32 {
        self.font_height
    }
    /// Returns the width of the characters in the view.
    /// This assumes the use of a fixed-width font.
    #[inline]
    pub fn font_width(&self) -> f32 {
        self.font_width
    }

    pub fn set_size(&mut self, _cols: i32, _lins: i32) {
        let horizontal_margin = 2 * self.left_base_margin as i32;
        let vertical_margin = 2 * self.top_base_margin as i32;

        let new_size = Size::new(
            horizontal_margin + (self.columns * self.font_width.ceil() as i32),
            vertical_margin + (self.lines * self.font_height.ceil() as i32),
        );

        if new_size != self.size() {
            self.size = new_size;
            self.resize(Some(self.size.width()), Some(self.size.height()));
        }
    }
    pub fn set_fixed_size(&mut self, cols: i32, lins: i32) {
        self.is_fixed_size = true;

        // ensure that display is at least one line by one column in size.
        self.set_columns(1.max(cols));
        self.set_lines(1.max(lins));
        self.set_used_columns(self.used_columns.min(self.columns));
        self.set_used_lines(self.used_lines.min(self.lines));

        if self.image.is_some() {
            self.make_image();
        }
        self.set_size(cols, lins);
    }

    /// Sets which characters, in addition to letters and numbers,
    /// are regarded as being part of a word for the purposes
    /// of selecting words in the display by double clicking on them.
    ///
    /// The word boundaries occur at the first and last characters which
    /// are either a letter, number, or a character in @p [`wc`]
    ///
    /// @param [`wc`] An array of characters which are to be considered parts
    /// of a word ( in addition to letters and numbers ).
    #[inline]
    pub fn set_word_characters(&mut self, wc: String) {
        self.word_characters = wc
    }
    /// Returns the characters which are considered part of a word for the
    /// purpose of selecting words in the display with the mouse.
    #[inline]
    pub fn word_characters(&self) -> &str {
        &self.word_characters
    }

    /// Sets the type of effect used to alert the user when a 'bell' occurs in the
    /// terminal session.
    ///
    /// The terminal session can trigger the bell effect by calling bell() with
    /// the alert message.
    #[inline]
    pub fn set_bell_mode(&mut self, mode: BellMode) {
        self.bell_mode = mode
    }
    /// Returns the type of effect used to alert the user when a 'bell' occurs in
    /// the terminal session.
    #[inline]
    pub fn bell_mode(&self) -> BellMode {
        self.bell_mode
    }

    pub fn set_selection(&mut self, t: String) {
        System::clipboard().set_text(t, ClipboardLevel::Os);
    }

    /// Returns the font used to draw characters in the view.
    pub fn get_vt_font(&self) -> Font {
        *self.font()
    }
    /// Sets the font used to draw the display.  Has no effect if @p [`font`]
    /// is larger than the size of the display itself.
    pub fn set_vt_font(&mut self, mut font: skia_safe::Font) {
        if let Some(typeface) = font.typeface() {
            if !typeface.is_fixed_pitch() {
                warn!(
                    "Using a variable-width font in the terminal.  This may cause 
performance degradation and display/alignment errors."
                )
            }
        }

        // hint that text should be drawn with/without anti-aliasing.
        // depending on the user's font configuration, this may not be respected
        if ANTIALIAS_TEXT.load(Ordering::SeqCst) {
            font.set_edging(tmui::skia_safe::font::Edging::AntiAlias);
        } else {
            font.set_edging(tmui::skia_safe::font::Edging::Alias);
        }

        self.set_font(font.into());
        self.font_change();
    }

    /// Specify whether line chars should be drawn by ourselves or left to
    /// underlying font rendering libraries.
    #[inline]
    pub fn set_draw_line_chars(&mut self, draw_line_chars: bool) {
        self.draw_line_chars = draw_line_chars
    }

    /// Specifies whether characters with intense colors should be rendered
    /// as bold. Defaults to true.
    #[inline]
    pub fn set_bold_intense(&mut self, bold_intense: bool) {
        self.bold_intense = bold_intense
    }
    /// Returns true if characters with intense colors are rendered in bold.
    #[inline]
    pub fn get_bold_intense(&self) -> bool {
        self.bold_intense
    }

    /// Sets whether or not the current height and width of the terminal in lines
    /// and columns is displayed whilst the widget is being resized.
    #[inline]
    pub fn set_terminal_size_hint(&mut self, on: bool) {
        self.terminal_size_hint = on
    }
    /// Returns whether or not the current height and width of the terminal in lines
    /// and columns is displayed whilst the widget is being resized.
    #[inline]
    pub fn terminal_size_hint(&self) -> bool {
        self.terminal_size_hint
    }

    /// Sets whether the terminal size display is shown briefly
    /// after the widget is first shown.
    ///
    /// See [`set_terminal_size_hint()`] , [`is_terminal_size_hint()`]
    #[inline]
    pub fn set_terminal_size_startup(&mut self, on: bool) {
        self.terminal_size_start_up = on
    }

    /// Sets the status of the BiDi rendering inside the terminal view.
    /// Defaults to disabled.
    #[inline]
    pub fn set_bidi_enable(&mut self, enable: bool) {
        self.bidi_enable = enable
    }
    /// Returns the status of the BiDi rendering in this widget.
    #[inline]
    pub fn is_bidi_enable(&mut self) -> bool {
        self.bidi_enable
    }

    /// Sets the terminal screen section which is displayed in this widget.
    /// When [`update_image()`] is called, the view fetches the latest character
    /// image from the the associated terminal screen window.
    ///
    /// In terms of the model-view paradigm, the ScreenWindow is the model which is
    /// rendered by the TerminalView.
    pub fn set_screen_window(&mut self, window: &mut ScreenWindow) {
        // The old ScreenWindow will be disconnected in emulation
        self.screen_window = NonNull::new(window);

        if self.screen_window.is_some() {
            connect!(window, output_changed(), self, update_line_properties());
            connect!(window, output_changed(), self, update_image());
            connect!(window, output_changed(), self, update_filters());
            connect!(window, scrolled(), self, update_filters());
            connect!(window, scroll_to_end(), self, scroll_to_end());
            window.set_window_lines(self.lines);
        }
    }
    /// Returns the terminal screen section which is displayed in this widget.
    /// See [`set_screen_window()`]
    #[inline]
    pub fn screen_window(&self) -> Option<&ScreenWindow> {
        match self.screen_window.as_ref() {
            Some(window) => unsafe { Some(window.as_ref()) },
            None => None,
        }
    }

    #[inline]
    pub fn screen_window_mut(&mut self) -> Option<&mut ScreenWindow> {
        match self.screen_window.as_mut() {
            Some(window) => unsafe { Some(window.as_mut()) },
            None => None,
        }
    }

    #[inline]
    pub fn scroll_bar(&self) -> Option<&ScrollBar> {
        match self.scroll_bar.as_ref() {
            Some(scroll_bar) => unsafe { Some(scroll_bar.as_ref()) },
            None => None,
        }
    }

    #[inline]
    pub fn scroll_bar_mut(&mut self) -> Option<&mut ScrollBar> {
        match self.scroll_bar.as_mut() {
            Some(scroll_bar) => unsafe { Some(scroll_bar.as_mut()) },
            None => None,
        }
    }

    #[inline]
    pub fn set_motion_after_pasting(&mut self, action: MotionAfterPasting) {
        self.motion_after_pasting = action
    }
    #[inline]
    pub fn motion_after_pasting(&self) -> MotionAfterPasting {
        self.motion_after_pasting
    }
    #[inline]
    pub fn set_confirm_multiline_paste(&mut self, confirm_multiline_paste: bool) {
        self.confirm_multiline_paste = confirm_multiline_paste
    }
    #[inline]
    pub fn set_trim_pasted_trailing_new_lines(&mut self, trim_pasted_trailing_new_lines: bool) {
        self.trim_pasted_trailing_new_lines = trim_pasted_trailing_new_lines
    }

    /// maps a point on the widget to the position ( ie. line and column )
    /// of the character at that point.
    pub fn get_character_position(&self, widget_point: FPoint) -> (i32, i32) {
        let content_rect = self.contents_rect_f(Some(Coordinate::Widget));
        let mut line =
            ((widget_point.y() - content_rect.top() - self.top_margin) / self.font_height) as i32;
        let mut column;
        if line < 0 {
            line = 0;
        }
        if line >= self.used_lines {
            line = self.used_lines - 1;
        }

        let x = widget_point.x() + self.font_width / 2. - content_rect.left() - self.left_margin;
        if self.fixed_font {
            column = (x / self.font_width) as i32;
        } else {
            column = 0;
            while column + 1 < self.used_columns && x > self.text_width(0, column + 1, line) {
                column += 1;
            }
        }

        if column < 0 {
            column = 0;
        }

        // the column value returned can be equal to _usedColumns, which
        // is the position just after the last character displayed in a line.
        //
        // this is required so that the user can select characters in the right-most
        // column (or left-most for right-to-left input)
        if column > self.used_columns {
            column = self.used_columns;
        }
        (line, column)
    }

    #[inline]
    pub fn disable_bracketed_paste_mode(&mut self, disable: bool) {
        self.disable_bracketed_paste_mode = disable
    }
    #[inline]
    pub fn is_disable_bracketed_paste_mode(&self) -> bool {
        self.disable_bracketed_paste_mode
    }

    #[inline]
    pub fn set_bracketed_paste_mode(&mut self, bracketed_paste_mode: bool) {
        self.bracketed_paste_mode = bracketed_paste_mode
    }
    #[inline]
    pub fn bracketed_paste_mode(&self) -> bool {
        self.bracketed_paste_mode
    }

    ////////////////////////////////////// Slots. //////////////////////////////////////
    /// Causes the terminal view to fetch the latest character image from the
    /// associated terminal screen ( see [`set_screen_window()`] ) and redraw the view.
    pub fn update_image(&mut self) {
        if self.screen_window.is_none() {
            return;
        }
        let screen_window = nonnull_mut!(self.screen_window);

        // optimization - scroll the existing image where possible and
        // avoid expensive text drawing for parts of the image that
        // can simply be moved up or down
        self.scroll_image(screen_window.scroll_count(), &screen_window.scroll_region());
        screen_window.reset_scroll_count();

        if self.image.is_none() {
            // Create _image.
            // The emitted changedContentSizeSignal also leads to getImage being
            // recreated, so do this first.
            self.update_image_size()
        }

        let lines = screen_window.window_lines();
        let columns = screen_window.window_columns();

        self.set_scroll(screen_window.current_line(), screen_window.line_count());

        let image = ptr_mut!(self.image.as_mut().unwrap() as *mut Vec<Character>);
        let new_img = screen_window.get_image();

        assert!(self.used_lines <= self.lines);
        assert!(self.used_columns <= self.columns);

        let tl = self.contents_rect(Some(Coordinate::Widget)).top_left();
        let tlx = tl.x();
        let tly = tl.y();
        self.has_blinker = false;

        let mut len;

        let mut cf = CharacterColor::default();
        let mut clipboard;
        let mut cr;

        let lines_to_update = self.lines.min(0.max(lines));
        let columns_to_update = self.columns.min(0.max(columns));

        // let mut disstr_u = vec![0 as wchar_t; columns_to_update as usize];
        let mut dirty_region = FRect::default();

        // debugging variable, this records the number of lines that are found to
        // be 'dirty' ( ie. have changed from the old _image to the new _image ) and
        // which therefore need to be repainted
        let mut _dirty_line_count = 0;

        for y in 0..lines_to_update {
            let current_line = &mut image[(y * self.columns) as usize..];
            let new_line = &new_img[(y * columns) as usize..];

            // The dirty mask indicates which characters need repainting. We also
            // mark surrounding neighbours dirty, in case the character exceeds
            // its cell boundaries
            let mut dirty_mask = vec![false; columns_to_update as usize + 2];

            let mut update_line = false;

            for x in 0..columns_to_update as usize {
                if new_line[x] != current_line[x] {
                    dirty_mask[x] = true;
                }
            }

            if !self.resizing {
                let mut x = 0usize;
                while x < columns_to_update as usize {
                    self.has_blinker = self.has_blinker || (new_line[x].rendition & RE_BLINK != 0);

                    // Start drawing if this character or the next one differs.
                    // We also take the next one into account to handle the situation
                    // where characters exceed their cell width.
                    if dirty_mask[x] {
                        let c = new_line[x + 0].character_union.data();
                        if c == 0 {
                            continue;
                        }

                        let line_draw = self.is_line_char(c);
                        let double_width = if x + 1 == columns_to_update as usize {
                            false
                        } else {
                            new_line[x + 1].character_union.data() == 0
                        };
                        cr = new_line[x].rendition;
                        clipboard = new_line[x].background_color;

                        if new_line[x].foreground_color != cf {
                            cf = new_line[x].foreground_color;
                        }

                        let lln = columns_to_update as usize - x;
                        len = 1;
                        while len < lln {
                            let ch = new_line[x + len];
                            if ch.character_union.data() == 0 {
                                continue;
                            }

                            let next_is_double_width = if x + len + 1 == columns_to_update as usize
                            {
                                false
                            } else {
                                new_line[x + len + 1].character_union.data() == 0
                            };

                            if ch.foreground_color != cf
                                || ch.background_color != clipboard
                                || ch.rendition != cr
                                || !dirty_mask[x + len]
                                || self.is_line_char(c) != line_draw
                                || next_is_double_width != double_width
                            {
                                break;
                            }

                            len += 1;
                        }

                        let save_fixed_font = self.fixed_font;
                        if line_draw {
                            self.fixed_font = false;
                        }
                        if double_width {
                            self.fixed_font = false;
                        }

                        update_line = true;

                        self.fixed_font = save_fixed_font;
                        x += len - 1;
                    }
                    x += 1;
                }
            }

            // both the top and bottom halves of double height _lines must always be
            // redrawn although both top and bottom halves contain the same characters,
            // only the top one is actually drawn.
            if self.line_properties.len() > y as usize {
                update_line =
                    update_line || (self.line_properties[y as usize] & LINE_DOUBLE_HEIGHT != 0);
            }

            // if the characters on the line are different in the old and the new _image
            // then this line must be repainted.
            if update_line {
                _dirty_line_count += 1;
                let dirty_rect = FRect::new(
                    self.left_margin + tlx as f32,
                    self.top_margin + tly as f32 + self.font_height * y as f32,
                    self.font_width * columns_to_update as f32,
                    self.font_height,
                );

                dirty_region.or(&dirty_rect);
            }

            current_line[0..columns_to_update as usize]
                .copy_from_slice(&new_line[0..columns_to_update as usize]);
        }

        // if the new _image is smaller than the previous _image, then ensure that the
        // area outside the new _image is cleared
        if lines_to_update < self.used_lines {
            let rect = FRect::new(
                self.left_margin + tlx as f32,
                self.top_margin + tly as f32 + self.font_height * lines_to_update as f32,
                self.font_width * self.columns as f32,
                self.font_height * (self.used_lines - lines_to_update) as f32,
            );
            dirty_region.or(&rect);
        }
        self.set_used_lines(lines_to_update);

        if columns_to_update < self.used_columns {
            let rect = FRect::new(
                self.left_margin + tlx as f32 + columns_to_update as f32 * self.font_width,
                self.top_margin + tly as f32,
                self.font_width * (self.used_columns - columns_to_update) as f32,
                self.font_height * self.lines as f32,
            );
            dirty_region.or(&rect);
        }
        self.set_used_columns(columns_to_update);

        dirty_region.or(&self.input_method_data.previous_preedit_rect);

        // update the parts of the view which have changed
        if dirty_region.width() > 0. && dirty_region.height() > 0. {
            self.update_rect(CoordRect::new(dirty_region, Coordinate::Widget));
        }

        if self.has_blinker && !self.blink_timer.is_active() {
            self.blink_timer.start(Duration::from_millis(
                TEXT_BLINK_DELAY.load(Ordering::SeqCst),
            ));
        }
        if !self.has_blinker && self.blink_timer.is_active() {
            self.blink_timer.stop();
            self.blinking = false;
        }
    }

    /// Essentially calls [`process_filters()`].
    pub fn update_filters(&mut self) {
        if self.screen_window.is_none() {
            return;
        }

        self.process_filters();
    }

    /// Causes the terminal view to fetch the latest line status flags from the
    /// associated terminal screen ( see [`set_screen_window()`] ).
    pub fn update_line_properties(&mut self) {
        if self.screen_window.is_none() {
            return;
        }

        self.line_properties = self.screen_window().unwrap().get_line_properties();
    }

    /// Copies the selected text to the clipboard.
    pub fn copy_clipboard(&mut self) {
        if self.screen_window.is_none() {
            return;
        }

        let text = self
            .screen_window()
            .unwrap()
            .selected_text(self.preserve_line_breaks);
        System::clipboard().set_text(text, ClipboardLevel::Os);
    }

    /// Pastes the content of the clipboard into the view.
    pub fn paste_clipboard(&mut self) {
        self.emit_selection(false, false)
    }

    /// Pastes the content of the selection into the view.
    pub fn paste_selection(&mut self) {
        self.emit_selection(true, false)
    }

    /// Causes the widget to display or hide a message informing the user that
    /// terminal output has been suspended (by using the flow control key
    /// combination Ctrl+S)
    ///
    /// @param [`suspended`] True if terminal output has been suspended and the warning
    /// message should be shown or false to indicate that terminal output has been
    /// resumed and that the warning message should disappear.
    pub fn output_suspended(&mut self, suspended: bool) {
        todo!()
    }

    /// Sets whether the program whose output is being displayed in the view
    /// is interested in mouse events.
    ///
    /// If this is set to true, mouse signals will be emitted by the view when the
    /// user clicks, drags or otherwise moves the mouse inside the view. The user
    /// interaction needed to create selections will also change, and the user will
    /// be required to hold down the shift key to create a selection or perform
    /// other mouse activities inside the view area - since the program running in
    /// the terminal is being allowed to handle normal mouse events itself.
    ///
    /// @param [`uses_mouse`] Set to true if the program running in the terminal is
    /// interested in mouse events or false otherwise.
    pub fn set_uses_mouse(&mut self, uses_mouse: bool) {
        if self.mouse_marks != uses_mouse {
            self.mouse_marks = uses_mouse;
            self.set_cursor_shape(if self.mouse_marks {
                SystemCursorShape::TextCursor
            } else {
                SystemCursorShape::ArrowCursor
            });
            emit!(self.uses_mouse_changed());
        }
    }

    /// See [`set_uses_mouse()`]
    #[inline]
    pub fn uses_mouse(&mut self) -> bool {
        self.mouse_marks
    }

    /// Shows a notification that a bell event has occurred in the terminal.
    pub fn bell(&mut self, message: &str) {
        if self.bell_mode == BellMode::NoBell {
            return;
        }

        // limit the rate at which bells can occur
        //...mainly for sound effects where rapid bells in sequence
        // produce a horrible noise
        if self.allow_bell {
            self.allow_bell = false;
            Timer::once(|mut timer| {
                connect!(timer, timeout(), self, enable_bell());
                timer.start(Duration::from_millis(500));
            });

            match self.bell_mode {
                BellMode::SystemBeepBell => {
                    System::beep();
                }
                BellMode::NotifyBell => {
                    emit!(self.notify_bell(), message)
                }
                BellMode::VisualBell => {
                    self.swap_color_table();
                    Timer::once(|mut timer| {
                        connect!(timer, timeout(), self, swap_color_table());
                        timer.start(Duration::from_millis(200));
                    });
                }
                _ => {}
            }
        }
    }

    /// Sets the background of the view to the specified color.
    /// @see [`set_color_table()`], [`set_foreground_color()`]
    pub fn set_background_color(&mut self, color: Color) {
        let scroll_bar = nonnull_mut!(self.scroll_bar);
        self.color_table[DEFAULT_BACK_COLOR as usize].color = color;

        self.set_background(color);
        scroll_bar.set_background(color);

        self.update();
    }

    /// Sets the text of the view to the specified color.
    /// @see [`set_color_table()`], [`set_background_color()`]
    pub fn set_foreground_color(&mut self, color: Color) {
        self.color_table[DEFAULT_FORE_COLOR as usize].color = color;
        self.update();
    }

    pub fn selection_changed(&mut self) {
        if self.screen_window.is_none() {
            return;
        }
        emit!(
            self.copy_avaliable(),
            self.screen_window()
                .unwrap()
                .selected_text(false)
                .is_empty()
                == false
        );
    }

    fn scroll_bar_position_changed(&mut self, _value: i32) {
        if self.screen_window.is_none() {
            return;
        }

        let scroll_bar = nonnull_mut!(self.scroll_bar);
        let screen_window = nonnull_mut!(self.screen_window);
        screen_window.scroll_to(scroll_bar.value());

        // if the thumb has been moved to the bottom of the _scrollBar then set
        // the display to automatically track new output,
        // that is, scroll down automatically to how new _lines as they are added.
        let at_end_of_output = scroll_bar.value() == scroll_bar.maximum();
        screen_window.set_track_output(at_end_of_output);

        self.update_image();
    }

    fn blink_event(&mut self) {
        if !self.allow_blinking_text {
            return;
        }

        self.blinking = !self.blinking;

        // TODO:  Optimize to only repaint the areas of the widget
        // where there is blinking text
        // rather than repainting the whole widget.
        self.update();
    }

    fn blink_cursor_event(&mut self) {
        self.cursor_blinking = !self.cursor_blinking;
        self.update_cursor();
    }

    /// Renables bell noises and visuals.  Used to disable further bells for a
    /// short period of time after emitting the first in a sequence of bell events.
    fn enable_bell(&mut self) {
        self.allow_bell = true;
    }

    fn swap_color_table(&mut self) {
        let color = self.color_table[1];
        self.color_table[1] = self.color_table[0];
        self.color_table[0] = color;
        self.colors_inverted = !self.colors_inverted;
        self.update();
    }

    ////////////////////////////////////// Private functions. //////////////////////////////////////
    #[inline]
    fn set_used_columns(&mut self, used_columns: i32) {
        self.used_columns = used_columns;
    }

    #[inline]
    fn set_used_lines(&mut self, used_lines: i32) {
        self.used_lines = used_lines;
    }

    #[inline]
    fn set_columns(&mut self, columns: i32) {
        self.columns = columns;
    }

    #[inline]
    fn set_lines(&mut self, lines: i32) {
        self.lines = lines;
    }

    fn font_change(&mut self) {
        let font = self.font().to_skia_font();
        let typeface = font.typeface().unwrap();

        let mut typeface_provider = TypefaceFontProvider::new();
        let family = typeface.family_name();
        typeface_provider.register_typeface(typeface, Some(family.clone()));

        let mut font_collection = FontCollection::new();
        font_collection.set_asset_font_manager(Some(typeface_provider.clone().into()));

        // define text style
        let mut style = ParagraphStyle::new();
        let mut text_style = TextStyle::new();
        text_style.set_font_size(font.size());
        text_style.set_font_families(&vec![family]);
        text_style.set_letter_spacing(0.);
        style.set_text_style(&text_style);

        // layout the paragraph
        let mut paragraph_builder = ParagraphBuilder::new(&style, font_collection);
        paragraph_builder.add_text(REPCHAR);
        let mut paragraph = paragraph_builder.build();
        paragraph.layout(f32::MAX);

        self.font_width = paragraph.max_intrinsic_width() / REPCHAR.len() as f32;
        self.font_height = paragraph.height();

        self.fixed_font = true;

        // "Base character width on widest ASCII character. This prevents too wide
        // characters in the presence of double wide (e.g. Chinese) characters."
        // Get the width from representative normal width characters
        let wchar_t_repchar: Vec<u16> = REPCHAR.encode_utf16().collect();
        let mut widths = vec![0f32; wchar_t_repchar.len()];
        font.get_widths(&wchar_t_repchar, &mut widths);
        let fw = widths[0];
        for i in 1..widths.len() {
            if fw != widths[i] {
                self.fixed_font = false;
                break;
            }
        }

        if self.font_width < 1. {
            self.font_width = 1.;
        }

        emit!(
            self.changed_font_metrics_signal(),
            (self.font_height, self.font_width)
        );
        self.propagate_size();

        // We will run paint event testing procedure.
        // Although this operation will destroy the original content,
        // the content will be drawn again after the test.
        self.draw_text_test_flag = true;
        self.update();
    }

    fn extend_selection(&mut self, mut pos: FPoint) {
        if self.screen_window.is_none() {
            return;
        }
        let scroll_bar = nonnull_mut!(self.scroll_bar);

        let tl = self.contents_rect(Some(Coordinate::Widget)).top_left();
        let tlx = tl.x();
        let tly = tl.y();
        let scroll = scroll_bar.value();

        // we're in the process of moving the mouse with the left button pressed
        // the mouse cursor will kept caught within the bounds of the text in this widget.
        let mut lines_beyond_widget;

        let text_bounds = FRect::new(
            tlx as f32 + self.left_margin,
            tly as f32 + self.top_margin,
            self.used_columns as f32 * self.font_width - 1.,
            self.used_lines as f32 * self.font_height - 1.,
        );

        // Adjust position within text area bounds.
        let old_pos = pos;

        pos.set_x(tmui::tlib::global::bound32(
            text_bounds.left(),
            pos.x(),
            text_bounds.right(),
        ));
        pos.set_y(tmui::tlib::global::bound32(
            text_bounds.top(),
            pos.y(),
            text_bounds.bottom(),
        ));

        if old_pos.y() > text_bounds.bottom() {
            lines_beyond_widget = (old_pos.y() - text_bounds.bottom()) / self.font_height;
            // Scroll forward
            scroll_bar.set_value(scroll_bar.value() + lines_beyond_widget as i32 + 1);
        }
        if old_pos.y() < text_bounds.top() {
            lines_beyond_widget = (text_bounds.top() - old_pos.y()) / self.font_height;
            scroll_bar.set_value(scroll_bar.value() - lines_beyond_widget as i32 - 1);
        }

        let (char_line, char_column) = self.get_character_position(pos);

        let mut here = Point::new(char_column, char_line);
        let mut ohere = Point::default();
        let mut i_pnt_sel_corr = self.i_pnt_sel;
        i_pnt_sel_corr.set_y(i_pnt_sel_corr.y() - scroll_bar.value());
        let mut pnt_sel_corr = self.pnt_sel;
        pnt_sel_corr.set_y(pnt_sel_corr.y() - scroll_bar.value());
        let mut swapping = false;

        if self.word_selection_mode {
            // Extend to word boundaries.
            let mut i;
            let mut sel_class;

            let left_not_right = here.y() < i_pnt_sel_corr.y()
                || (here.y() == i_pnt_sel_corr.y() && here.x() < i_pnt_sel_corr.x());
            let old_left_not_right = pnt_sel_corr.y() < i_pnt_sel_corr.y()
                || (pnt_sel_corr.y() == i_pnt_sel_corr.y()
                    && pnt_sel_corr.x() < i_pnt_sel_corr.x());
            swapping = left_not_right != old_left_not_right;

            // Find left (left_not_right ? from here : from start)
            let mut left = if left_not_right { here } else { i_pnt_sel_corr };
            i = self.loc(left.x(), left.y());
            if i >= 0 && i <= self.image_size {
                sel_class = self.char_class(self.image()[i as usize].character_union.data());

                while (left.x() > 0
                    || (left.y() > 0
                        && self.line_properties[left.y() as usize - 1] & LINE_WRAPPED != 0))
                    && self.char_class(self.image()[i as usize - 1].character_union.data())
                        == sel_class
                {
                    i -= 1;
                    if left.x() > 0 {
                        left.set_x(left.x() - 1)
                    } else {
                        left.set_x(self.used_columns - 1);
                        left.set_y(left.y() - 1);
                    }
                }
            }

            // Find right (left_not_right ? from start : from here)
            let mut right = if left_not_right { i_pnt_sel_corr } else { here };
            i = self.loc(right.x(), right.y());
            if i >= 0 && i <= self.image_size {
                sel_class = self.char_class(self.image()[i as usize].character_union.data());
                while (right.x() < self.used_columns - 1
                    || (right.y() < self.used_lines - 1
                        && self.line_properties[right.y() as usize] & LINE_WRAPPED != 0))
                    && self.char_class(self.image()[i as usize + 1].character_union.data())
                        == sel_class
                {
                    i += 1;
                    if right.x() < self.used_columns - 1 {
                        right.set_x(right.x() + 1);
                    } else {
                        right.set_x(0);
                        right.set_y(right.y() + 1);
                    }
                }
            }

            // Pick which is start (ohere) and which is extension (here).
            ohere.set_x(ohere.x() + 1);
        }

        if self.line_selection_mode {
            // Extend to complete line.
            let above_not_below = here.y() < i_pnt_sel_corr.y();

            let mut above = if above_not_below {
                here
            } else {
                i_pnt_sel_corr
            };
            let mut below = if above_not_below {
                i_pnt_sel_corr
            } else {
                here
            };

            while above.y() > 0 && self.line_properties[above.y() as usize - 1] & LINE_WRAPPED != 0
            {
                above.set_y(above.y() - 1);
            }
            while below.y() < self.used_lines - 1
                && self.line_properties[below.y() as usize] & LINE_WRAPPED != 0
            {
                below.set_y(below.y() + 1);
            }

            above.set_x(0);
            below.set_x(self.used_columns - 1);

            // Pick which is start (ohere) and which is extension (here)
            if above_not_below {
                here = above;
                ohere = below;
            } else {
                here = below;
                ohere = above;
            }

            let new_sel_begin = Point::new(ohere.x(), ohere.y());
            swapping = !(self.triple_sel_begin == new_sel_begin);
            self.triple_sel_begin = new_sel_begin;

            ohere.set_x(ohere.x() + 1);
        }

        let mut offset = 0;
        if !self.word_selection_mode && !self.line_selection_mode {
            let i;
            let _sel_class;

            let left_not_right = here.y() < i_pnt_sel_corr.y()
                || (here.y() == i_pnt_sel_corr.y() && here.x() < i_pnt_sel_corr.x());
            let old_left_not_right = pnt_sel_corr.y() < i_pnt_sel_corr.y()
                || (pnt_sel_corr.y() == i_pnt_sel_corr.y()
                    && pnt_sel_corr.x() < i_pnt_sel_corr.x());
            swapping = left_not_right != old_left_not_right;

            // Find left (left_not_right ? from here : from start)
            let left = if left_not_right { here } else { i_pnt_sel_corr };

            // Find right (left_not_right ? from start : from here)
            let right = if left_not_right { i_pnt_sel_corr } else { here };

            if right.x() > 0 && !self.column_selection_mode {
                i = self.loc(right.x(), right.y());
                if i >= 0 && i <= self.image_size {
                    _sel_class =
                        self.char_class(self.image()[i as usize - 1].character_union.data());
                }
            }

            // Pick which is start (ohere) and which is extension (here)
            if left_not_right {
                here = left;
                ohere = right;
                offset = 0;
            } else {
                here = right;
                ohere = left;
                offset = -1;
            }
        }

        if here == pnt_sel_corr && scroll == scroll_bar.value() {
            // Not moved.
            return;
        }

        if here == ohere {
            // It's not left, it's not right.
            return;
        }

        let screen_window = nonnull_mut!(self.screen_window);
        if self.act_sel < 2 || swapping {
            if self.column_selection_mode && !self.line_selection_mode && !self.word_selection_mode
            {
                screen_window.set_selection_start(ohere.x(), ohere.y(), true);
            } else {
                screen_window.set_selection_start(ohere.x() - 1 - offset, ohere.y(), false);
            }
        }

        self.act_sel = 2;
        self.pnt_sel = here;
        self.pnt_sel.set_y(self.pnt_sel.y() + scroll_bar.value());

        if self.column_selection_mode && !self.line_selection_mode && !self.word_selection_mode {
            screen_window.set_selection_end(here.x(), here.y());
        } else {
            screen_window.set_selection_end(here.x() + offset, here.y());
        }
    }

    #[inline]
    fn do_drag(&mut self) {
        self.drag_info.state = DragState::DiDragging;
    }

    /// classifies the 'ch' into one of three categories
    /// and returns a character to indicate which category it is in
    ///
    ///     - A space (returns ' ')
    ///     - Part of a word (returns 'a')
    ///     - Other characters (returns the input character)
    fn char_class(&mut self, ch: wchar_t) -> wchar_t {
        if ch == b' ' as wchar_t {
            return b' ' as wchar_t;
        }

        if (ch >= b'0' as wchar_t && ch <= b'9' as wchar_t)
            || (ch >= b'a' as wchar_t && ch <= b'z' as wchar_t)
            || (ch >= b'A' as wchar_t && ch <= b'Z' as wchar_t
                || self.word_characters.contains(ch as u8 as char))
        {
            return b'a' as wchar_t;
        }

        ch
    }

    fn clear_image(&mut self) {
        if self.image.is_none() {
            return;
        }
        // Initialize image[image_size] too. See make_image()
        for i in 0..self.image_size as usize {
            let image = self.image_mut();
            image[i].character_union.set_data(wch!(' '));
            image[i].foreground_color = CharacterColor::default_foreground();
            image[i].background_color = CharacterColor::default_background();
            image[i].rendition = DEFAULT_RENDITION;
        }
    }

    fn mouse_triple_click_event(&mut self, ev: MouseEvent) {
        if self.screen_window.is_none() {
            return;
        }
        let scroll_bar = nonnull_mut!(self.scroll_bar);
        let screen_window = nonnull_mut!(self.screen_window);

        let (char_line, char_column) = self.get_character_position(ev.position().into());
        self.pnt_sel = Point::new(char_column, char_line);

        screen_window.clear_selection();

        self.line_selection_mode = true;
        self.word_selection_mode = false;

        self.act_sel = 2;
        emit!(self.is_busy_selecting(), true);

        while self.pnt_sel.y() > 0
            && self.line_properties[self.pnt_sel.y() as usize - 1] & LINE_WRAPPED != 0
        {
            self.pnt_sel.set_y(self.pnt_sel.y() - 1);
        }

        if self.triple_click_mode == TripleClickMode::SelectForwardsFromCursor {
            // find word boundary start
            let mut i = self.loc(self.pnt_sel.x(), self.pnt_sel.y());
            let sel_class = self.char_class(self.image()[i as usize].character_union.data());

            let mut x = self.pnt_sel.x();

            while (x > 0
                || (self.pnt_sel.y() > 0
                    && self.line_properties[self.pnt_sel.y() as usize - 1] & LINE_WRAPPED != 0))
                && self.char_class(self.image()[i as usize - 1].character_union.data()) == sel_class
            {
                i -= 1;
                if x > 0 {
                    x -= 1;
                } else {
                    x = self.columns - 1;
                    self.pnt_sel.set_y(self.pnt_sel.y() - 1);
                }
            }

            screen_window.set_selection_start(x, self.pnt_sel.y(), false);
            self.triple_sel_begin = Point::new(x, self.pnt_sel.y());
        } else if self.triple_click_mode == TripleClickMode::SelectWholeLine {
            screen_window.set_selection_start(0, self.pnt_sel.y(), false);
            self.triple_sel_begin = Point::new(0, self.pnt_sel.y());
        }

        while self.pnt_sel.y() < self.lines - 1
            && self.line_properties[self.pnt_sel.y() as usize] & LINE_WRAPPED != 0
        {
            self.pnt_sel.set_y(self.pnt_sel.y() + 1);
        }

        screen_window.set_selection_end(self.columns - 1, self.pnt_sel.y());

        self.set_selection(screen_window.selected_text(self.preserve_line_breaks));

        self.i_pnt_sel
            .set_y(self.i_pnt_sel.y() + scroll_bar.value());
    }

    /// determine the width of this text.
    fn text_width(&self, start_column: i32, length: i32, line: i32) -> f32 {
        if self.image.is_none() {
            return 0.;
        }
        let image = self.image();
        let font = self.font().to_skia_font();
        let mut result = 0.;
        let mut widths = vec![];
        for column in 0..length {
            let c: &[uwchar_t; 1] = unsafe {
                std::mem::transmute(&[image[self.loc(start_column + column, line) as usize]
                    .character_union
                    .data()])
            };
            let ws16: U16String;
            #[cfg(not(windows))]
            let c = {
                let ws32 = U32String::from_vec(c.to_vec());
                let str = ws32.to_string().unwrap();
                ws16 = U16String::from_str(&str);
                ws16.as_slice()
            };
            #[cfg(windows)]
            let c = {
                ws16 = U16String::from_vec(c.to_vec());
                ws16.as_slice()
            };
            font.get_widths(c, &mut widths);
            let width: f32 = widths.iter().sum();
            result += width;
        }
        result
    }
    /// determine the area that encloses this series of characters.
    fn calculate_text_area(
        &self,
        top_left_x: i32,
        top_left_y: i32,
        start_column: i32,
        line: i32,
        length: i32,
    ) -> FRect {
        let left = if self.fixed_font {
            self.font_width * start_column as f32
        } else {
            self.text_width(0, start_column, line)
        };
        let top = self.font_height * line as f32;
        let width = if self.fixed_font {
            self.font_width * length as f32
        } else {
            self.text_width(start_column, length, line)
        };

        FRect::new(
            self.left_margin + top_left_x as f32 + left,
            self.top_margin + top_left_y as f32 + top,
            width,
            self.font_height,
        )
    }

    /// maps an area in the character image to an area on the widget.
    fn image_to_widget(&self, image_area: &FRect) -> FRect {
        let mut result = FRect::default();
        result.set_left(self.left_margin + self.font_width * image_area.left());
        result.set_top(self.top_margin + self.font_height * image_area.top());
        result.set_width(self.font_width * image_area.width());
        result.set_height(self.font_height * image_area.height());

        result
    }

    /// the area where the preedit string for input methods will be draw.
    fn preedit_rect(&mut self) -> FRect {
        let preedit_length = string_width(&self.input_method_data.preedit_string);

        if preedit_length == 0 {
            return FRect::default();
        }

        FRect::new(
            self.left_margin + self.font_width * self.cursor_position().x() as f32,
            self.top_margin + self.font_height * self.cursor_position().y() as f32,
            self.font_width * preedit_length as f32,
            self.font_height,
        )
    }

    /// shows a notification window in the middle of the widget indicating the
    /// terminal's current size in columns and lines
    fn show_resize_notification(&self) {
        // TODO: show resize notification.
    }

    /// scrolls the image by a number of lines.
    /// 'lines' may be positive ( to scroll the image down )
    /// or negative ( to scroll the image up )
    /// 'region' is the part of the image to scroll - currently only
    /// the top, bottom and height of 'region' are taken into account,
    /// the left and right are ignored.
    fn scroll_image(&mut self, lines: i32, screen_window_region: &Rect) {
        // if the flow control warning is enabled this will interfere with the
        // scrolling optimizations and cause artifacts.  the simple solution here
        // is to just disable the optimization whilst it is visible
        if self.output_suspend_label.visible() {
            return;
        }

        // constrain the region to the display
        // the bottom of the region is capped to the number of lines in the display's
        // internal image - 2, so that the height of 'region' is strictly less
        // than the height of the internal image.
        let mut region = *screen_window_region;
        region.set_bottom(region.bottom().min(self.lines - 2));

        // return if there is nothing to do
        if lines == 0
            || self.image.is_none()
            || !region.is_valid()
            || region.top() + lines.abs() >= region.bottom()
            || self.lines <= region.height()
        {
            return;
        }

        // hide terminal size label to prevent it being scrolled.
        if self.resize_widget.visible() {
            self.resize_widget.hide()
        }

        let mut scroll_rect = FRect::default();
        scroll_rect.set_left(0.);
        scroll_rect.set_right(self.size().width() as f32);

        let first_char_pos = &mut self.image.as_mut().unwrap()
            [(region.top() * self.columns) as usize] as *mut Character
            as *mut c_void;
        let last_char_pos = &mut self.image.as_mut().unwrap()
            [((region.top() + lines.abs()) * self.columns) as usize]
            as *mut Character as *mut c_void;

        let top = self.top_margin.ceil() + region.top() as f32 * self.font_height.ceil();
        let lines_to_move = region.height() - lines.abs();
        let bytes_to_move = lines_to_move * self.columns * size_of::<Character>() as i32;

        assert!(lines_to_move > 0);
        assert!(bytes_to_move > 0);

        // Scroll internal image
        if lines > 0 {
            // Scroll down:
            unsafe { memmove(first_char_pos, last_char_pos, bytes_to_move as usize) };
            scroll_rect.set_top(top);
        } else {
            // Scroll up:
            unsafe { memmove(last_char_pos, first_char_pos, bytes_to_move as usize) };
            scroll_rect.set_top(top + lines.abs() as f32 * self.font_height);
        }
        scroll_rect.set_height(lines_to_move as f32 * self.font_height);

        self.update_rect(CoordRect::new(scroll_rect, Coordinate::Widget));
    }

    /// shows the multiline prompt
    fn multiline_confirmation(&mut self, text: &str) -> bool {
        // TODO: shows the multiline prompt.
        false
    }

    fn calc_geometry(&mut self) {
        let contents_rect = self.contents_rect(Some(Coordinate::Widget));

        match self.scroll_bar_location {
            ScrollBarState::NoScrollBar => {}
            ScrollBarState::ScrollBarLeft => {
                nonnull_mut!(self.scroll_bar).set_scroll_bar_position(ScrollBarPosition::Start);
            }
            ScrollBarState::ScrollBarRight => {
                nonnull_mut!(self.scroll_bar).set_scroll_bar_position(ScrollBarPosition::End);
            }
        }

        self.top_margin = self.top_base_margin;
        self.content_width = contents_rect.width() - 2 * self.left_base_margin as i32;
        self.content_height = contents_rect.height() - 2 * self.top_base_margin as i32;

        if !self.is_fixed_size {
            // ensure that display is always at least one column wide
            self.set_columns(((self.content_width as f32 / self.font_width) as i32).max(1));
            self.set_used_columns(self.used_columns.min(self.columns));

            // ensure that display is always at least one line high
            self.set_lines(((self.content_height as f32 / self.font_height) as i32).max(1));
            self.set_used_lines(self.used_lines.min(self.lines));
        }
    }

    fn propagate_size(&mut self) {
        if self.is_fixed_size {
            self.set_size(self.columns, self.lines);
            // TODO: Maybe should adjust parent size?
            return;
        }
        if self.image.is_some() {
            self.update_image_size();
        }
    }

    fn update_image_size(&mut self) {
        let old_line = self.lines;
        let old_col = self.columns;

        let old_image = self.make_image();

        // copy the old image to reduce flicker
        let mlines = old_line.min(self.lines);
        let mcolumns = old_col.min(self.columns);

        if old_image.is_some() {
            for line in 0..mlines {
                let dist_start = (self.columns * line) as usize;
                let dist_end = dist_start + mcolumns as usize;
                let src_start = (old_col * line) as usize;
                let src_end = src_start + mcolumns as usize;
                self.image_mut()[dist_start..dist_end]
                    .copy_from_slice(&old_image.as_ref().unwrap()[src_start..src_end]);
            }
        }

        if self.screen_window.is_some() {
            nonnull_mut!(self.screen_window).set_window_lines(self.lines)
        }

        self.resizing = (old_line != self.lines) || (old_col != self.columns);

        if self.resizing {
            self.show_resize_notification();
            emit!(
                self.changed_content_size_signal(),
                self.content_height,
                self.content_width
            );
        }

        self.resizing = false
    }
    /// Make new image and return the old one.
    fn make_image(&mut self) -> Option<Vec<Character>> {
        self.calc_geometry();

        // confirm that array will be of non-zero size, since the painting code
        // assumes a non-zero array length
        assert!(self.lines > 0 && self.columns > 0);
        assert!(self.used_lines <= self.lines && self.used_columns <= self.columns);

        self.image_size = self.lines * self.columns;

        // We over-commit one character so that we can be more relaxed in dealing with
        // certain boundary conditions: _image[_imageSize] is a valid but unused position.
        let old_img = self
            .image
            .replace(vec![Character::default(); (self.image_size + 1) as usize]);

        self.clear_image();

        old_img
    }

    /// returns a region covering all of the areas of the widget which contain a hotspot.
    fn hotspot_region(&self) -> FRect {
        let mut region = FRect::default();
        let hotspots = self.filter_chain.hotspots();

        hotspots.iter().for_each(|hotspot| {
            let mut r = FRect::default();
            if hotspot.start_line() == hotspot.end_line() {
                r.set_left(hotspot.start_column() as f32);
                r.set_top(hotspot.start_line() as f32);
                r.set_right(hotspot.end_column() as f32);
                r.set_bottom(hotspot.end_line() as f32);
                region.or(&self.image_to_widget(&r))
            } else {
                r.set_left(hotspot.start_column() as f32);
                r.set_top(hotspot.start_line() as f32);
                r.set_right(self.columns as f32);
                r.set_bottom(hotspot.start_line() as f32);
                region.or(&self.image_to_widget(&r));

                for line in hotspot.start_line() + 1..hotspot.end_line() {
                    r.set_left(0.);
                    r.set_top(line as f32);
                    r.set_right(self.columns as f32);
                    r.set_bottom(line as f32);
                    region.or(&self.image_to_widget(&r));
                }

                r.set_left(0.);
                r.set_top(hotspot.end_line() as f32);
                r.set_right(hotspot.end_column() as f32);
                r.set_bottom(hotspot.end_line() as f32);
                region.or(&self.image_to_widget(&r));
            }
        });

        region
    }

    /// returns the position of the cursor in columns and lines.
    fn cursor_position(&self) -> Point {
        if self.screen_window.is_some() {
            nonnull_ref!(self.screen_window).cursor_position()
        } else {
            Point::new(0, 0)
        }
    }

    /// redraws the cursor.
    fn update_cursor(&mut self) {
        let rect = FRect::from_point_size(self.cursor_position().into(), Size::new(1, 1).into());
        let cursor_rect = self.image_to_widget(&rect);
        self.update_rect(CoordRect::new(cursor_rect, Coordinate::Widget));
    }

    fn handle_shortcut_override_event(&mut self, event: KeyEvent) {
        todo!()
    }

    #[inline]
    fn is_line_char(&self, c: wchar_t) -> bool {
        self.draw_line_chars && ((c & 0xFF80) == 0x2500)
    }
    #[inline]
    fn is_line_char_string(&self, string: &WideString) -> bool {
        string.len() > 0 && self.is_line_char(string.as_slice()[0] as wchar_t)
    }

    #[inline]
    /// Get reference of image without option check, may cause panic.
    fn image(&self) -> &[Character] {
        self.image.as_ref().unwrap()
    }
    #[inline]
    /// Get mutable reference of image without option check, may cause panic.
    fn image_mut(&mut self) -> &mut [Character] {
        self.image.as_mut().unwrap()
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////////////
/// Predefine
//////////////////////////////////////////////////////////////////////////////////////////////////////////
static ANTIALIAS_TEXT: AtomicBool = AtomicBool::new(true);
static HAVE_TRANSPARENCY: AtomicBool = AtomicBool::new(true);
static TEXT_BLINK_DELAY: AtomicU64 = AtomicU64::new(500);

const REPCHAR: &'static str = concat!(
    "ABCDEFGHIJKLMNOPQRSTUVWXYZ",
    "abcdefgjijklmnopqrstuvwxyz",
    "0123456789./+@"
);

const LTR_OVERRIDE_CHAR: wchar_t = 0x202D;

//////////////////////////////////////////////////////////////////////////////////////////////////////////
/// Display Operations
//////////////////////////////////////////////////////////////////////////////////////////////////////////
#[repr(u32)]
enum LineEncode {
    TopL = 1 << 1,
    TopC = 1 << 2,
    TopR = 1 << 3,

    LeftT = 1 << 5,
    Int11 = 1 << 6,
    Int12 = 1 << 7,
    Int13 = 1 << 8,
    RightT = 1 << 9,

    LeftC = 1 << 10,
    Int21 = 1 << 11,
    Int22 = 1 << 12,
    Int23 = 1 << 13,
    RightC = 1 << 14,

    LeftB = 1 << 15,
    Int31 = 1 << 16,
    Int32 = 1 << 17,
    Int33 = 1 << 18,
    RightB = 1 << 19,

    BotL = 1 << 21,
    BotC = 1 << 22,
    BotR = 1 << 23,
}

const LINE_CHARS: [u32; 128] = [
    0x00007c00, 0x000fffe0, 0x00421084, 0x00e739ce, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00427000, 0x004e7380, 0x00e77800, 0x00ef7bc0,
    0x00421c00, 0x00439ce0, 0x00e73c00, 0x00e7bde0, 0x00007084, 0x000e7384, 0x000079ce, 0x000f7bce,
    0x00001c84, 0x00039ce4, 0x00003dce, 0x0007bdee, 0x00427084, 0x004e7384, 0x004279ce, 0x00e77884,
    0x00e779ce, 0x004f7bce, 0x00ef7bc4, 0x00ef7bce, 0x00421c84, 0x00439ce4, 0x00423dce, 0x00e73c84,
    0x00e73dce, 0x0047bdee, 0x00e7bde4, 0x00e7bdee, 0x00427c00, 0x0043fce0, 0x004e7f80, 0x004fffe0,
    0x004fffe0, 0x00e7fde0, 0x006f7fc0, 0x00efffe0, 0x00007c84, 0x0003fce4, 0x000e7f84, 0x000fffe4,
    0x00007dce, 0x0007fdee, 0x000f7fce, 0x000fffee, 0x00427c84, 0x0043fce4, 0x004e7f84, 0x004fffe4,
    0x00427dce, 0x00e77c84, 0x00e77dce, 0x0047fdee, 0x004e7fce, 0x00e7fde4, 0x00ef7f84, 0x004fffee,
    0x00efffe4, 0x00e7fdee, 0x00ef7fce, 0x00efffee, 0x00000000, 0x00000000, 0x00000000, 0x00000000,
    0x000f83e0, 0x00a5294a, 0x004e1380, 0x00a57800, 0x00ad0bc0, 0x004390e0, 0x00a53c00, 0x00a5a1e0,
    0x000e1384, 0x0000794a, 0x000f0b4a, 0x000390e4, 0x00003d4a, 0x0007a16a, 0x004e1384, 0x00a5694a,
    0x00ad2b4a, 0x004390e4, 0x00a52d4a, 0x00a5a16a, 0x004f83e0, 0x00a57c00, 0x00ad83e0, 0x000f83e4,
    0x00007d4a, 0x000f836a, 0x004f93e4, 0x00a57d4a, 0x00ad836a, 0x00000000, 0x00000000, 0x00000000,
    0x00000000, 0x00000000, 0x00000000, 0x00000000, 0x00001c00, 0x00001084, 0x00007000, 0x00421000,
    0x00039ce0, 0x000039ce, 0x000e7380, 0x00e73800, 0x000e7f80, 0x00e73884, 0x0003fce0, 0x004239ce,
];

fn draw_line_char(painter: &mut Painter, x: f32, y: f32, w: f32, h: f32, code: u8) {
    // Calculate cell midpoints, end points.
    let cx = x + w / 2.;
    let cy = y + h / 2.;
    let ex = x + w - 1.;
    let ey = y + h - 1.;

    let to_draw = LINE_CHARS[code as usize];

    // Top lines:
    if to_draw & TopL as u32 != 0 {
        painter.draw_line_f(cx - 1., y, cx - 1., cy - 2.);
    }
    if to_draw & TopC as u32 != 0 {
        painter.draw_line_f(cx, y, cx, cy - 2.);
    }
    if to_draw & TopR as u32 != 0 {
        painter.draw_line_f(cx + 1., y, cx + 1., cy - 2.);
    }

    // Bot lines:
    if to_draw & BotL as u32 != 0 {
        painter.draw_line_f(cx - 1., cy + 2., cx - 1., ey);
    }
    if to_draw & BotC as u32 != 0 {
        painter.draw_line_f(cx, cy + 2., cx, ey);
    }
    if to_draw & BotR as u32 != 0 {
        painter.draw_line_f(cx + 1., cy + 2., cx + 1., ey);
    }

    // Left lines:
    if to_draw & LeftT as u32 != 0 {
        painter.draw_line_f(x, cy - 1., cx - 2., cy - 1.);
    }
    if to_draw & LeftC as u32 != 0 {
        painter.draw_line_f(x, cy, cx - 2., cy);
    }
    if to_draw & LeftB as u32 != 0 {
        painter.draw_line_f(x, cy + 1., cx - 2., cy + 1.);
    }

    // Right lines:
    if to_draw & RightT as u32 != 0 {
        painter.draw_line_f(cx + 2., cy - 1., ex, cy - 1.);
    }
    if to_draw & RightC as u32 != 0 {
        painter.draw_line_f(cx + 2., cy, ex, cy);
    }
    if to_draw & RightB as u32 != 0 {
        painter.draw_line_f(cx + 2., cy + 1., ex, cy + 1.);
    }

    // Intersection points.
    if to_draw & Int11 as u32 != 0 {
        painter.draw_point_f(cx - 1., cy - 1.);
    }
    if to_draw & Int12 as u32 != 0 {
        painter.draw_point_f(cx, cy - 1.);
    }
    if to_draw & Int13 as u32 != 0 {
        painter.draw_point_f(cx + 1., cy - 1.);
    }

    if to_draw & Int21 as u32 != 0 {
        painter.draw_point_f(cx - 1., cy);
    }
    if to_draw & Int22 as u32 != 0 {
        painter.draw_point_f(cx, cy);
    }
    if to_draw & Int23 as u32 != 0 {
        painter.draw_point_f(cx + 1., cy);
    }

    if to_draw & Int31 as u32 != 0 {
        painter.draw_point_f(cx - 1., cy + 1.);
    }
    if to_draw & Int32 as u32 != 0 {
        painter.draw_point_f(cx, cy + 1.);
    }
    if to_draw & Int33 as u32 != 0 {
        painter.draw_point_f(cx + 1., cy + 1.);
    }
}

fn draw_other_char(painter: &mut Painter, x: f32, y: f32, w: f32, h: f32, code: u8) {
    // Calculate cell midpoints, end points.
    let cx = x + w / 2.;
    let cy = y + h / 2.;
    let ex = x + w - 1.;
    let ey = y + h - 1.;

    // Double dashes
    if 0x4C <= code && code <= 0x4F {
        let x_half_gap = 1f32.max(w / 15.);
        let y_half_gap = 1f32.max(h / 15.);

        match code {
            0x4D => {
                // BOX DRAWINGS HEAVY DOUBLE DASH HORIZONTAL
                painter.draw_line_f(x, cy - 1., cx - x_half_gap - 1., cy - 1.);
                painter.draw_line_f(x, cy + 1., cx - x_half_gap - 1., cy + 1.);
                painter.draw_line_f(cx + x_half_gap, cy - 1., ex, cy - 1.);
                painter.draw_line_f(cx + x_half_gap, cy + 1., ex, cy + 1.);
            }
            0x4C => {
                // BOX DRAWINGS LIGHT DOUBLE DASH HORIZONTAL
                painter.draw_line_f(x, cy, cx - x_half_gap - 1., cy);
                painter.draw_line_f(cx + x_half_gap, cy, ex, cy);
            }
            0x4F => {
                // BOX DRAWINGS HEAVY DOUBLE DASH VERTICAL
                painter.draw_line_f(cx - 1., y, cx - 1., cy - y_half_gap - 1.);
                painter.draw_line_f(cx + 1., y, cx + 1., cy - y_half_gap - 1.);
                painter.draw_line_f(cx - 1., cy + y_half_gap, cx - 1., ey);
                painter.draw_line_f(cx + 1., cy + y_half_gap, cx + 1., ey);
            }
            0x4E => {
                // BOX DRAWINGS LIGHT DOUBLE DASH VERTICAL
                painter.draw_line_f(cx, y, cx, cy - y_half_gap - 1.);
                painter.draw_line_f(cx, cy + y_half_gap, cx, ey);
            }
            _ => {}
        }

    // Rounded corner characters
    } else if 0x6D <= code && code <= 0x70 {
        let r = w * 3. / 8.;
        let d = 2. * r;

        match code {
            0x6D => {
                // BOX DRAWINGS LIGHT ARC DOWN AND RIGHT
                painter.draw_line_f(cx, cy + r, cx, ey);
                painter.draw_line_f(cx + r, cy, ex, cy);
                painter.draw_arc_f(cx, cy, d, d, 90. * 16., 90. * 16., false);
            }
            0x6E => {
                // BOX DRAWINGS LIGHT ARC DOWN AND LEFT
                painter.draw_line_f(cx, cy + r, cx, ey);
                painter.draw_line_f(x, cy, cx - r, cy);
                painter.draw_arc_f(cx - d, cy, d, d, 0. * 16., 90. * 16., false);
            }
            0x6F => {
                // BOX DRAWINGS LIGHT ARC UP AND LEFT
                painter.draw_line_f(cx, y, cx, cy - r);
                painter.draw_line_f(x, cy, cx - r, cy);
                painter.draw_arc_f(cx - d, cy - d, d, d, 270. * 16., 90. * 16., false);
            }
            0x70 => {
                // BOX DRAWINGS LIGHT ARC UP AND RIGHT
                painter.draw_line_f(cx, y, cx, cy - r);
                painter.draw_line_f(cx + r, cy, ex, cy);
                painter.draw_arc_f(cx, cy - d, d, d, 180. * 16., 90. * 16., false);
            }
            _ => {}
        }

    // Diagonals
    } else if 0x71 <= code && code <= 0x73 {
        match code {
            0x71 => {
                // BOX DRAWINGS LIGHT DIAGONAL UPPER RIGHT TO LOWER LEFT
                painter.draw_line_f(ex, y, x, ey);
            }
            0x72 => {
                // BOX DRAWINGS LIGHT DIAGONAL UPPER LEFT TO LOWER RIGHT
                painter.draw_line_f(x, y, ex, ey);
            }
            0x73 => {
                // BOX DRAWINGS LIGHT DIAGONAL CROSS
                painter.draw_line_f(ex, y, x, ey);
                painter.draw_line_f(x, y, ex, ey);
            }
            _ => {}
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////////////
/// Enums
//////////////////////////////////////////////////////////////////////////////////////////////////////////
#[repr(u8)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum ScrollBarState {
    #[default]
    NoScrollBar = 0,
    ScrollBarLeft,
    ScrollBarRight,
}
impl From<u8> for ScrollBarState {
    fn from(x: u8) -> Self {
        match x {
            0 => Self::NoScrollBar,
            1 => Self::ScrollBarLeft,
            2 => Self::ScrollBarRight,
            _ => unimplemented!(),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum TripleClickMode {
    #[default]
    SelectWholeLine = 0,
    SelectForwardsFromCursor,
}
impl From<u8> for TripleClickMode {
    fn from(x: u8) -> Self {
        match x {
            0 => Self::SelectWholeLine,
            1 => Self::SelectForwardsFromCursor,
            _ => unimplemented!(),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum MotionAfterPasting {
    #[default]
    NoMoveScreenWindow = 0,
    MoveStartScreenWindow,
    MoveEndScreenWindow,
}
impl From<u8> for MotionAfterPasting {
    fn from(x: u8) -> Self {
        match x {
            0 => Self::NoMoveScreenWindow,
            1 => Self::MoveStartScreenWindow,
            2 => Self::MoveEndScreenWindow,
            _ => unimplemented!(),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum KeyboardCursorShape {
    #[default]
    BlockCursor = 0,
    UnderlineCursor,
    IBeamCursor,
}
impl From<u8> for KeyboardCursorShape {
    fn from(x: u8) -> Self {
        match x {
            0 => Self::BlockCursor,
            1 => Self::UnderlineCursor,
            2 => Self::IBeamCursor,
            _ => unimplemented!(),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum BackgroundMode {
    #[default]
    None = 0,
    Stretch,
    Zoom,
    Fit,
    Center,
}
impl From<u8> for BackgroundMode {
    fn from(x: u8) -> Self {
        match x {
            0 => Self::None,
            1 => Self::Stretch,
            2 => Self::Zoom,
            3 => Self::Fit,
            4 => Self::Center,
            _ => unimplemented!(),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum BellMode {
    #[default]
    SystemBeepBell = 0,
    NotifyBell,
    VisualBell,
    NoBell,
}
impl From<u8> for BellMode {
    fn from(x: u8) -> Self {
        match x {
            0 => Self::SystemBeepBell,
            1 => Self::NotifyBell,
            2 => Self::VisualBell,
            3 => Self::NoBell,
            _ => unimplemented!(),
        }
    }
}

#[repr(u8)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum DragState {
    #[default]
    DiNone = 0,
    DiPending,
    DiDragging,
}
impl From<u8> for DragState {
    fn from(x: u8) -> Self {
        match x {
            0 => Self::DiNone,
            1 => Self::DiPending,
            2 => Self::DiDragging,
            _ => unimplemented!(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regular_replace() {
        let str = "hello\r";
        assert_eq!(REGULAR_EXPRESSION.replace(str, ""), "hello");
    }
}
