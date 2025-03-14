mod helper;
mod predefine;
mod render;
mod wiget_imp;

pub use self::helper::{
    BackgroundMode, BellMode, DragState, KeyboardCursorShape, MotionAfterPasting, ScrollBarState,
    TripleClickMode,
};

use self::{
    helper::{DragInfo, InputMethodData},
    predefine::ANTIALIAS_TEXT,
};
use super::screen_window::{ScreenWindow, ScreenWindowSignals};
use crate::tools::{
    character::{Character, ExtendedCharTable, LineProperty},
    character_color::{ColorEntry, DEFAULT_BACK_COLOR, DEFAULT_FORE_COLOR, TABLE_COLORS},
    event::KeyPressedEvent,
    filter::{FilterChainImpl, TerminalImageFilterChain},
};
use derivative::Derivative;
use std::{ptr::NonNull, sync::atomic::Ordering, time::Duration};
use tmui::{
    application,
    clipboard::ClipboardLevel,
    font::FontEdging,
    graphics::painter::Painter,
    label::Label,
    opti::tracker::Tracker,
    prelude::*,
    scroll_bar::ScrollBar,
    system::System,
    tlib::{
        connect, emit,
        events::{KeyEvent, MouseEvent},
        figure::{Color, Size},
        global::bound64,
        nonnull_mut,
        object::{ObjectImpl, ObjectSubclass},
        signals,
        timer::Timer,
    },
    widget::WidgetImpl,
};

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
    text_blinking: bool,
    // has character to blink.
    has_blinker_text: bool,
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
    blink_text_timer: Timer,
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

    clear_margin: bool,
}

//////////////////////////////////////////////////////////////////////////////////////////////////////////
/// Widget Implements
//////////////////////////////////////////////////////////////////////////////////////////////////////////
impl ObjectSubclass for TerminalView {
    const NAME: &'static str = "TerminalView";
}

impl ObjectImpl for TerminalView {
    #[inline]
    fn initialize(&mut self) {
        self.handle_initialize();
    }
}

impl WidgetImpl for TerminalView {
    #[inline]
    fn enable_focus(&self) -> bool {
        true
    }

    #[inline]
    fn paint(&mut self, painter: &mut Painter) {
        self.handle_paint(painter)
    }

    #[inline]
    fn on_key_pressed(&mut self, event: &KeyEvent) {
        self.handle_key_pressed(event)
    }

    #[inline]
    fn on_mouse_wheel(&mut self, event: &MouseEvent) {
        self.handle_mouse_wheel(event)
    }

    #[inline]
    fn on_mouse_pressed(&mut self, event: &MouseEvent) {
        if event.n_press() == 2 {
            self.handle_mouse_double_click(event)
        } else if event.n_press() == 3 {
            self.handle_mouse_triple_click(event)
        } else {
            self.handle_mouse_pressed(event)
        }
    }

    #[inline]
    fn on_mouse_released(&mut self, event: &MouseEvent) {
        self.handle_mouse_released(event)
    }

    #[inline]
    fn on_mouse_move(&mut self, event: &MouseEvent) {
        let _tracker = Tracker::start("terminal_view_mouse_move");
        self.handle_mouse_move(event)
    }

    #[inline]
    fn font_changed(&mut self) {
        // hint that text should be drawn with/without anti-aliasing.
        // depending on the user's font configuration, this may not be respected
        let font = self.font_mut();
        if ANTIALIAS_TEXT.load(Ordering::SeqCst) {
            font.set_edging(FontEdging::AntiAlias);
        } else {
            font.set_edging(FontEdging::Alias);
        }

        self.handle_font_change()
    }

    fn on_get_focus(&mut self) {
        if self.has_blinking_cursor && !self.blink_cursor_timer.is_active() {
            self.blink_cursor_timer.start(Duration::from_millis(
                application::cursor_blinking_time() as u64,
            ));
        }

        if self.cursor_blinking {
            self.blink_cursor_event()
        }
    }

    fn on_lose_focus(&mut self) {
        self.blink_cursor_timer.stop();
        if !self.cursor_blinking {
            self.blink_cursor_event()
        }
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
        key_pressed_signal(KeyPressedEvent, bool);

        /// A mouse event occurred.
        /// @param [`i32`] button: The mouse button (0 for left button, 1 for middle button, 2
        /// for right button, 3 for release) <br>
        /// @param [`i32`] column: The character column where the event occurred <br>
        /// @param [`i32`] row: The character row where the event occurred <br>
        /// @param [`u8`] type: The type of event.  0 for a mouse press / release or 1 for
        /// mouse motion
        mouse_signal(i32, i32, i32, u8);

        changed_font_metrics_signal(f32, f32);
        changed_content_size_signal(i32, i32);

        /// Emitted when the user right clicks on the display, or right-clicks with the
        /// Shift key held down if [`uses_mouse()`] is true.
        ///
        /// This can be used to display a context menu.
        configure_request(Point);

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

        copy_avaliable(bool);
        term_get_focus();
        term_lost_focus();

        notify_bell(&str);
        uses_mouse_changed();
    );
}
impl TerminalViewSignals for TerminalView {}

//////////////////////////////////////////////////////////////////////////////////////////////////////////
/// TerminalView Implements
//////////////////////////////////////////////////////////////////////////////////////////////////////////
impl TerminalView {
    /// Constructor to build `TerminalView`
    pub fn new(session_id: ObjectId) -> Box<Self> {
        let mut view: Box<Self> = Object::new(&[]);
        view.bind_session = session_id;
        view
    }

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
    pub fn terminate(&mut self) {
        self.terminate = true
    }

    #[inline]
    pub fn is_terminate(&self) -> bool {
        self.terminate
    }

    /// Returns the terminal color palette used by the view.
    #[inline]
    pub fn get_color_table(&self) -> &[ColorEntry] {
        &self.color_table
    }
    /// Sets the terminal color palette used by the view.
    #[inline]
    pub fn set_color_table(&mut self, table: &[ColorEntry]) {
        self.color_table[..TABLE_COLORS].copy_from_slice(&table[..TABLE_COLORS]);

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
    #[allow(clippy::if_same_then_else)]
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

    /// Returns a list of menu actions created by the filters for the content
    /// at the given @p position.
    pub fn filter_actions(&self, _position: Point) -> Vec<Action> {
        todo!()
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
        self.handle_font_change()
    }
    #[inline]
    pub fn line_spacing(&self) -> u32 {
        self.line_spacing
    }

    #[inline]
    pub fn set_margin(&mut self, margin: i32) {
        self.top_margin = margin as f32;
        self.left_margin = margin as f32;
    }
    #[inline]
    pub fn margin(&mut self) -> i32 {
        self.top_margin as i32
    }

    #[inline]
    pub fn set_scroll_bar(&mut self, scroll_bar: &mut ScrollBar) {
        self.scroll_bar = NonNull::new(scroll_bar)
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

    pub fn set_size(&mut self) {
        let horizontal_margin = 2 * self.left_margin as i32;
        let vertical_margin = 2 * self.top_margin as i32;

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
        self.set_size();
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

    pub fn copy_selection(&mut self, t: String) {
        System::clipboard().set_text(t, ClipboardLevel::Os);
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
            connect!(
                window,
                screen_window_output_changed(),
                self,
                update_line_properties()
            );
            connect!(window, screen_window_output_changed(), self, update_image());
            connect!(
                window,
                screen_window_output_changed(),
                self,
                update_filters()
            );
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
            emit!(self, uses_mouse_changed());
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
            Timer::once(|timer| {
                connect!(timer, timeout(), self, enable_bell());
                timer.start(Duration::from_millis(500));
            });

            match self.bell_mode {
                BellMode::SystemBeepBell => {
                    System::beep();
                }
                BellMode::NotifyBell => {
                    emit!(self, notify_bell(message))
                }
                BellMode::VisualBell => {
                    self.swap_color_table();
                    Timer::once(|timer| {
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

    /// Renables bell noises and visuals.  Used to disable further bells for a
    /// short period of time after emitting the first in a sequence of bell events.
    #[inline]
    pub fn enable_bell(&mut self) {
        self.allow_bell = true;
    }
}
