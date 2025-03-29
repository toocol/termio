use crate::{
    core::terminal_view::TerminalViewSignals,
    tools::{
        character::LINE_WRAPPED,
        character_color::BASE_COLOR_TABLE,
        event::ToKeyPressedEvent,
        filter::{FilterChainImpl, HotSpotType},
    },
};
use std::time::Duration;
use tlib::namespace::KeyCode;
use tmui::{
    application::cursor_blinking_time,
    font::FontCalculation,
    opti::tracker::Tracker,
    prelude::*,
    skia_safe::ClipOp,
    tlib::{
        connect,
        events::{KeyEvent, MouseEvent},
        figure::FSize,
        namespace::{Align, KeyboardModifier, MouseButton},
    },
    widget::widget_ext::WidgetExt,
};
use wchar::wch;

use super::{DragState, TerminalView, TripleClickMode};

impl TerminalView {
    pub(super) fn handle_initialize(&mut self) {
        self.set_hexpand(true);
        self.set_vexpand(true);
        self.set_focus(true);

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

    pub(super) fn handle_paint(&mut self, painter: &mut Painter) {
        painter.set_antialiasing(true);

        // TODO: Process the background image.

        if self.clear_margin {
            let rect = self.terminal_rect();
            painter.save();
            painter.clip_rect(rect, ClipOp::Difference);
            painter.fill_rect(
                self.contents_rect_f(Some(Coordinate::Widget)),
                self.background(),
            );
            painter.restore();
            self.clear_margin = false;
        }

        let region = self.redraw_region().clone();
        if region.is_empty() {
            let rect = self.contents_rect_f(Some(Coordinate::Widget));
            self.draw_background(painter, rect, self.background(), true);
            self.draw_contents(painter, rect);
        } else {
            for rect in region.into_iter() {
                self.draw_background(painter, rect.rect(), self.background(), true);
                self.draw_contents(painter, rect.rect());
            }
        }

        // self.draw_input_method_preedit_string(&mut painter, &self.preddit_rect());
        self.paint_filters(painter);
    }

    pub(super) fn handle_key_pressed(&mut self, event: &KeyEvent) {
        self.act_sel = 0;

        if self.has_blinking_cursor {
            if !self.blink_cursor_timer.is_active() {
                self.blink_cursor_timer
                    .start(Duration::from_millis(cursor_blinking_time() as u64));
            }

            if self.cursor_blinking {
                self.blink_cursor_event()
            }
        }

        if event.modifier().has(KeyboardModifier::ControlModifier)
            && event.key_code() == KeyCode::KeyInsert
        {
            emit!(self, control_insert_detected());
        }

        if event.modifier().has(KeyboardModifier::ShiftModifier)
            && event.key_code() == KeyCode::KeyInsert
        {
            emit!(self, shift_insert_detected());
        }

        if event.key_code() != KeyCode::KeyControl {
            self.screen_window_mut().unwrap().clear_selection();
        }

        emit!(
            self,
            key_pressed_signal(event.to_key_pressed_event(), false)
        );
    }

    pub(super) fn handle_mouse_wheel(&mut self, event: &MouseEvent) {
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

    pub(super) fn handle_mouse_released(&mut self, event: &MouseEvent) {
        if self.screen_window().is_none() {
            return;
        }

        let (char_line, char_column) = self.get_character_position(event.position().into());
        if event.mouse_button() == MouseButton::LeftButton {
            if self.drag_info.state == DragState::DiPending {
                self.screen_window_mut().unwrap().clear_selection();
            } else if self.drag_info.state == DragState::DiDragging {
                if self.act_sel > 1 {
                    self.copy_selection(
                        self.screen_window()
                            .unwrap()
                            .selected_text(self.preserve_line_breaks),
                    );
                }

                self.act_sel = 0;

                if !self.mouse_marks && !event.modifier().has(KeyboardModifier::ShiftModifier) {
                    let scroll_bar = self.scroll_bar().unwrap();
                    emit!(
                        self,
                        mouse_signal(
                            0,
                            char_column + 1,
                            char_line + 1 + scroll_bar.value() - scroll_bar.maximum(),
                            2u8
                        )
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
                self,
                mouse_signal(
                    button,
                    char_column + 1,
                    char_line + 1 + scroll_bar.value() - scroll_bar.maximum(),
                    2u8
                )
            );
        }
    }

    pub(super) fn handle_mouse_move(&mut self, event: &MouseEvent) {
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
                        spot.start_column() as f32 * self.font_width + self.left_margin,
                        spot.start_line() as f32 * self.font_height + self.top_margin,
                        spot.end_column() as f32 * self.font_width + self.left_margin,
                        (spot.end_line() + 1) as f32 * self.font_height - 1. + self.top_margin,
                    );
                    mouse_over_hotspot_area.add_rect(CoordRect::new(r, Coordinate::Widget));
                } else {
                    r.set_coords(
                        spot.start_column() as f32 * self.font_width + self.left_margin,
                        spot.start_line() as f32 * self.font_height + self.top_margin,
                        self.columns as f32 * self.font_width - 1. + self.left_margin,
                        (spot.start_line() + 1) as f32 * self.font_height + self.top_margin,
                    );
                    mouse_over_hotspot_area.add_rect(CoordRect::new(r, Coordinate::Widget));

                    for line in spot.start_line() + 1..spot.end_line() {
                        r.set_coords(
                            self.left_margin,
                            line as f32 * self.font_height + self.top_margin,
                            self.columns as f32 * self.font_width + self.left_margin,
                            (line + 1) as f32 * self.font_height + self.top_margin,
                        );
                        mouse_over_hotspot_area.add_rect(CoordRect::new(r, Coordinate::Widget));
                    }

                    r.set_coords(
                        self.left_margin,
                        spot.end_line() as f32 * self.font_height + self.top_margin,
                        spot.end_column() as f32 * self.font_width + self.left_margin,
                        (spot.end_line() + 1) as f32 * self.font_height + self.top_margin,
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
                self,
                mouse_signal(
                    button,
                    char_column + 1,
                    char_line + 1 + scroll_bar.value() - scroll_bar.maximum(),
                    1u8
                )
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

        let _tracker = Tracker::start("terminal_view_extend_selection");
        self.extend_selection(event.position().into());
    }

    pub(super) fn handle_mouse_pressed(&mut self, evt: &MouseEvent) {
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
                        self,
                        mouse_signal(
                            0,
                            char_column + 1,
                            char_line + 1 + scroll_bar.value() - scroll_bar.maximum(),
                            0u8
                        )
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
                    self,
                    mouse_signal(
                        1,
                        char_column + 1,
                        char_line + 1 + scroll_bar.value() - scroll_bar.maximum(),
                        0u8
                    )
                );
            }
        } else if evt.mouse_button() == MouseButton::RightButton {
            if self.mouse_marks || modifier.has(KeyboardModifier::ShiftModifier) {
                let pos: Point = evt.position().into();
                emit!(self, configure_request(pos));
            } else {
                let scroll_bar = self.scroll_bar().unwrap();
                emit!(
                    self,
                    mouse_signal(
                        2,
                        char_column + 1,
                        char_line + 1 + scroll_bar.value() - scroll_bar.maximum(),
                        0u8
                    )
                );
            }
        }
    }

    pub(super) fn handle_mouse_double_click(&mut self, evt: &MouseEvent) {
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
                self,
                mouse_signal(
                    0,
                    pos.x() + 1,
                    pos.y() + 1 + scroll_bar.value() - scroll_bar.maximum(),
                    0u8
                )
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

        self.copy_selection(
            self.screen_window()
                .unwrap()
                .selected_text(self.preserve_line_breaks),
        );
    }

    pub(super) fn handle_mouse_triple_click(&mut self, evt: &MouseEvent) {
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

        self.copy_selection(
            self.screen_window()
                .unwrap()
                .selected_text(self.preserve_line_breaks),
        );

        let scroll_bar = self.scroll_bar().unwrap();
        *self.i_pnt_sel.y_mut() += scroll_bar.value();
    }

    #[inline]
    pub(super) fn when_resized(&mut self, size: Size) {
        let size: FSize = size.into();
        if size.width() == 0. || size.height() == 0. {
            return;
        }
        let rect = self.rect_record();
        if size.width() < rect.width() || size.height() < rect.height() {
            self.clear_margin = true;
        }
        if size == rect.size() {
            return;
        }
        self.update_image_size();
        self.process_filters();
    }

    pub(super) fn handle_font_change(&mut self) {
        let font = &self.font().to_skia_fonts()[0];

        let (width, height) = self.font().calc_font_dimension();

        self.font_width = width;
        self.font_height = height;

        self.fixed_font = font.typeface().is_fixed_pitch();

        // "Base character width on widest ASCII character. This prevents too wide
        // characters in the presence of double wide (e.g. Chinese) characters."
        // Get the width from representative normal width characters
        // let wchar_t_repchar: Vec<u16> = REPCHAR.encode_utf16().collect();
        // let mut widths = vec![0f32; wchar_t_repchar.len()];
        // font.get_widths(&wchar_t_repchar, &mut widths);
        // let fw = widths[0];
        // for w in widths.iter().skip(1) {
        //     if fw != *w {
        //         self.fixed_font = false;
        //         break;
        //     }
        // }

        if self.font_width < 1. {
            self.font_width = 1.;
        }

        emit!(
            self,
            changed_font_metrics_signal(self.font_height, self.font_width)
        );
        self.propagate_size();

        self.update();
    }
}
