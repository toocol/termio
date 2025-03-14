use super::TerminalView;
use crate::{
    core::{
        terminal_view::{
            predefine::{REGULAR_EXPRESSION, TEXT_BLINK_DELAY},
            TerminalViewSignals,
        },
        uwchar_t,
    },
    tools::{
        character::{Character, DEFAULT_RENDITION, LINE_DOUBLE_HEIGHT, LINE_WRAPPED, RE_BLINK},
        character_color::CharacterColor,
        event::KeyPressedEvent,
        filter::{FilterChainImpl, HotSpotImpl},
        system_ffi::string_width,
    },
};
use libc::{c_void, memmove, wchar_t};
use std::{mem::size_of, rc::Rc, sync::atomic::Ordering, time::Duration};
use tmui::{
    clipboard::ClipboardLevel,
    font::FontCalculation,
    prelude::*,
    scroll_bar::{ScrollBarPosition, ScrollBarSignal},
    system::System,
    tlib::{
        connect, disconnect,
        figure::{FRect, FRegion, Point},
        namespace::{KeyCode, KeyboardModifier},
        nonnull_mut, ptr_mut,
    },
};
use wchar::wch;
#[cfg(not(windows))]
use widestring::U32String;
use widestring::{U16String, WideString};

impl TerminalView {
    #[inline]
    pub(super) fn loc(&self, x: i32, y: i32) -> i32 {
        y * self.columns + x
    }

    #[inline]
    pub(super) fn set_used_columns(&mut self, used_columns: i32) {
        self.used_columns = used_columns;
    }

    #[inline]
    pub(super) fn set_used_lines(&mut self, used_lines: i32) {
        self.used_lines = used_lines;
    }

    #[inline]
    pub(super) fn set_columns(&mut self, columns: i32) {
        self.columns = columns;
    }

    #[inline]
    pub(super) fn set_lines(&mut self, lines: i32) {
        self.lines = lines;
    }

    #[inline]
    pub(super) fn do_drag(&mut self) {
        self.drag_info.state = DragState::DiDragging;
    }

    #[inline]
    pub(super) fn is_line_char(&self, c: wchar_t) -> bool {
        self.draw_line_chars && ((c & 0xFF80) == 0x2500)
    }
    #[inline]
    pub(super) fn is_line_char_string(&self, string: &WideString) -> bool {
        string.len() > 0 && self.is_line_char(string.as_slice()[0] as wchar_t)
    }

    #[inline]
    /// Get reference of image without option check, may cause panic.
    pub(super) fn image(&self) -> &[Character] {
        self.image.as_ref().unwrap()
    }
    #[inline]
    /// Get mutable reference of image without option check, may cause panic.
    pub(super) fn image_mut(&mut self) -> &mut [Character] {
        self.image.as_mut().unwrap()
    }

    pub(super) fn calc_hotspot_link_region(
        &self,
        spot: &Rc<Box<dyn HotSpotImpl>>,
        region: &mut FRegion,
    ) {
        let mut r = FRect::default();
        if spot.start_line() == spot.end_line() {
            r.set_coords(
                spot.start_column() as f32 * self.font_width + 1. + self.left_margin,
                spot.start_line() as f32 * self.font_height + 1. + self.top_margin,
                spot.end_column() as f32 * self.font_width - 1. + self.left_margin,
                (spot.end_line() as f32 + 1.) * self.font_height - 1. + self.top_margin,
            );
            region.add_rect(r);
        } else {
            r.set_coords(
                spot.start_column() as f32 * self.font_width + 1. + self.left_margin,
                spot.start_line() as f32 * self.font_height + 1. + self.top_margin,
                self.columns as f32 * self.font_width - 1. + self.left_margin,
                (spot.start_line() as f32 + 1.) * self.font_height - 1. + self.top_margin,
            );
            region.add_rect(r);

            for line in spot.start_line() + 1..spot.end_line() {
                r.set_coords(
                    0. * self.font_width + 1. + self.left_margin,
                    line as f32 * self.font_height + 1. + self.top_margin,
                    self.columns as f32 * self.font_width - 1. + self.left_margin,
                    (line as f32 + 1.) * self.font_height - 1. + self.top_margin,
                );
                region.add_rect(r);
            }
            r.set_coords(
                0. * self.font_width + 1. + self.left_margin,
                spot.end_line() as f32 * self.font_height + 1. + self.top_margin,
                spot.end_column() as f32 * self.font_width - 1. + self.left_margin,
                (spot.end_line() as f32 + 1.) * self.font_height - 1. + self.top_margin,
            );
            region.add_rect(r);
        }
    }

    /// Setting the current position and range of the display scroll bar.
    pub(super) fn set_scroll(&mut self, cursor: i32, lines: i32) {
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
    pub(super) fn scroll_to_end(&mut self) {
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
    pub(super) fn filter_chain(&self) -> &impl FilterChainImpl {
        self.filter_chain.as_ref()
    }

    /// Updates the filters in the display's filter chain.  This will cause
    /// the hotspots to be updated to match the current image.
    ///
    /// TODO: This function can be expensive depending on the
    /// image size and number of filters in the filterChain()
    pub(super) fn process_filters(&mut self) {
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

    /// @param [`use_x_selection`] Store and retrieve data from global mouse selection.
    /// Support for selection is only available on systems with global mouse selection (such as X11).
    pub(super) fn emit_selection(&mut self, use_x_selection: bool, append_return: bool) {
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

            text = text.replace("\r\n", "\n").replace('\n', "\r");

            if self.trim_pasted_trailing_new_lines {
                text = REGULAR_EXPRESSION.replace(&text, "").to_string();
            }

            if self.confirm_multiline_paste
                && text.contains('\r')
                && !self.multiline_confirmation(&text)
            {
                return;
            }

            self.bracket_text(&mut text);

            // appendReturn is intentionally handled _after_ enclosing texts with
            // brackets as that feature is used to allow execution of commands
            // immediately after paste. Ref: https://bugs.kde.org/show_bug.cgi?id=16179
            if append_return {
                text.push('\r');
            }

            let e = KeyPressedEvent::new(KeyCode::Unknown, text, KeyboardModifier::NoModifier);
            emit!(self, key_pressed_signal(e, true));

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
    pub(super) fn bracket_text(&self, text: &mut String) {
        if self.bracketed_paste_mode() && !self.disable_bracketed_paste_mode {
            text.insert_str(0, "\u{001b}[200~");
            text.push_str("\u{001b}[201~");
        }
    }

    /// maps a point on the widget to the position ( ie. line and column )
    /// of the character at that point.
    pub(super) fn get_character_position(&self, widget_point: FPoint) -> (i32, i32) {
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

    pub(super) fn blink_text_event(&mut self) {
        if !self.allow_blinking_text {
            return;
        }

        self.text_blinking = !self.text_blinking;

        // TODO:  Optimize to only repaint the areas of the widget
        // where there is blinking text
        // rather than repainting the whole widget.
        self.update();
    }

    pub(super) fn selection_changed(&mut self) {
        if self.screen_window.is_none() {
            return;
        }
        emit!(
            self,
            copy_avaliable(
                !self
                    .screen_window()
                    .unwrap()
                    .selected_text(false)
                    .is_empty()
            )
        );
    }

    pub(super) fn extend_selection(&mut self, mut pos: FPoint) {
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

    /// classifies the 'ch' into one of three categories
    /// and returns a character to indicate which category it is in
    ///
    ///     - A space (returns ' ')
    ///     - Part of a word (returns 'a')
    ///     - Other characters (returns the input character)
    pub(super) fn char_class(&mut self, ch: wchar_t) -> wchar_t {
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

    pub(super) fn clear_image(&mut self) {
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

    /// determine the width of this text.
    pub(super) fn text_width(&self, start_column: i32, length: i32, line: i32) -> f32 {
        if self.image.is_none() {
            return 0.;
        }
        let image = self.image();
        let mut str = WideString::new();
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

            str.push_char(char::from_u32(c[0] as u32).unwrap());
        }

        self.font()
            .calc_text_dimension(&str.to_string().unwrap(), 0.)
            .0
    }

    /// determine the area that encloses this series of characters.
    pub(super) fn calculate_text_area(
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
    #[inline]
    fn image_to_widget(&self, image_area: &FRect) -> FRect {
        let mut result = FRect::default();
        result.set_left(self.left_margin + self.font_width * image_area.left());
        result.set_top(self.top_margin + self.font_height * image_area.top());
        result.set_width(self.font_width * image_area.width());
        result.set_height(self.font_height * image_area.height());

        result
    }

    /// the area where the preedit string for input methods will be draw.
    pub(super) fn preedit_rect(&mut self) -> FRect {
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
    pub(super) fn show_resize_notification(&self) {
        // TODO: show resize notification.
    }

    /// scrolls the image by a number of lines.
    /// 'lines' may be positive ( to scroll the image down )
    /// or negative ( to scroll the image up )
    /// 'region' is the part of the image to scroll - currently only
    /// the top, bottom and height of 'region' are taken into account,
    /// the left and right are ignored.
    pub(super) fn scroll_image(&mut self, lines: i32, screen_window_region: &Rect) {
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
    pub(super) fn multiline_confirmation(&mut self, text: &str) -> bool {
        // TODO: shows the multiline prompt.
        false
    }

    pub(super) fn calc_geometry(&mut self) {
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

        self.content_width = contents_rect.width() - 2 * self.left_margin as i32;
        self.content_height = contents_rect.height() - 2 * self.top_margin as i32;

        if !self.is_fixed_size {
            // ensure that display is always at least one column wide
            self.set_columns(((self.content_width as f32 / self.font_width) as i32).max(1));
            self.set_used_columns(self.used_columns.min(self.columns));

            // ensure that display is always at least one line high
            self.set_lines(((self.content_height as f32 / self.font_height) as i32).max(1));
            self.set_used_lines(self.used_lines.min(self.lines));
        }
    }

    pub(super) fn propagate_size(&mut self) {
        if self.is_fixed_size {
            self.set_fixed_size(self.columns, self.lines);
            // TODO: Maybe should adjust parent size?
            return;
        }
        if self.image.is_some() {
            self.update_image_size();
        }
    }

    pub(super) fn update_image_size(&mut self) {
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
                self,
                changed_content_size_signal(self.content_height, self.content_width)
            );
        }

        self.resizing = false
    }

    /// Make new image and return the old one.
    pub(super) fn make_image(&mut self) -> Option<Vec<Character>> {
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
    pub(super) fn hotspot_region(&self) -> FRect {
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
    #[inline]
    pub(super) fn cursor_position(&self) -> Point {
        if let Some(sw) = self.screen_window() {
            sw.cursor_position()
        } else {
            Point::new(0, 0)
        }
    }

    /// redraws the cursor.
    #[inline]
    pub(super) fn update_cursor(&mut self) {
        let rect = FRect::from_point_size(self.cursor_position().into(), Size::new(1, 1).into());
        let cursor_rect = self.image_to_widget(&rect);
        self.update_rect(CoordRect::new(cursor_rect, Coordinate::Widget));
    }

    #[inline]
    pub(super) fn terminal_rect(&self) -> FRect {
        let mut rect = self.contents_rect_f(Some(Coordinate::Widget));
        rect.offset(self.left_margin, self.top_margin);
        rect.set_width(self.columns as f32 * self.font_width);
        rect.set_height(self.lines as f32 * (self.font_height + self.line_spacing as f32));
        rect
    }
}

////////////////////////////////////// Slots. //////////////////////////////////////
impl TerminalView {
    /// Causes the terminal view to fetch the latest character image from the
    /// associated terminal screen ( see [`set_screen_window()`] ) and redraw the view.
    pub(super) fn update_image(&mut self) {
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
            // Create image.
            // The emitted changedContentSizeSignal also leads to getImage being
            // recreated, so do this first.
            self.update_image_size()
        }

        let lines = screen_window.window_lines();
        let columns = screen_window.window_columns();

        self.set_scroll(screen_window.current_line(), screen_window.line_count());

        // Skip the mutable reference borrow check.
        let image = ptr_mut!(self.image.as_mut().unwrap() as *mut Vec<Character>);
        let new_img = screen_window.get_image();

        debug_assert!(self.used_lines <= self.lines);
        debug_assert!(self.used_columns <= self.columns);

        let tl = self.contents_rect(Some(Coordinate::Widget)).top_left();
        let tlx = tl.x();
        let tly = tl.y();
        self.has_blinker_text = false;

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
                    self.has_blinker_text =
                        self.has_blinker_text || (new_line[x].rendition & RE_BLINK != 0);

                    // Start drawing if this character or the next one differs.
                    // We also take the next one into account to handle the situation
                    // where characters exceed their cell width.
                    if dirty_mask[x] {
                        let c = new_line[x].character_union.data();
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

        if self.has_blinker_text && !self.blink_text_timer.is_active() {
            self.blink_text_timer.start(Duration::from_millis(
                TEXT_BLINK_DELAY.load(Ordering::SeqCst),
            ));
        }
        if !self.has_blinker_text && self.blink_text_timer.is_active() {
            self.blink_text_timer.stop();
            self.text_blinking = false;
        }
    }

    /// Essentially calls [`process_filters()`].
    pub(super) fn update_filters(&mut self) {
        if self.screen_window.is_none() {
            return;
        }

        self.process_filters();
    }

    /// Causes the terminal view to fetch the latest line status flags from the
    /// associated terminal screen ( see [`set_screen_window()`] ).
    pub(super) fn update_line_properties(&mut self) {
        if self.screen_window.is_none() {
            return;
        }

        self.line_properties = self.screen_window().unwrap().get_line_properties();
    }

    pub(super) fn scroll_bar_position_changed(&mut self, _: i32) {
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

    pub(super) fn blink_cursor_event(&mut self) {
        self.cursor_blinking = !self.cursor_blinking;
        self.update_cursor();
    }

    pub(super) fn swap_color_table(&mut self) {
        self.color_table.swap(1, 0);
        self.colors_inverted = !self.colors_inverted;
        self.update();
    }
}

#[derive(Default)]
pub(super) struct InputMethodData {
    pub(super) preedit_string: WideString,
    pub(super) previous_preedit_rect: FRect,
}

#[derive(Default)]
pub(super) struct DragInfo {
    pub(super) state: DragState,
    pub(super) start: Point,
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

#[repr(u32)]
pub(super) enum LineEncode {
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
