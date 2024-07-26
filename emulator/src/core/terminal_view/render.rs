use super::{
    helper::LineEncode::*, predefine::LINE_CHARS, FilterChainImpl, KeyboardCursorShape,
    TerminalView,
};
use crate::{
    core::uwchar_t,
    tools::{
        character::{
            Character, ExtendedCharTable, LINE_DOUBLE_HEIGHT, LINE_DOUBLE_WIDTH, RE_BLINK, RE_BOLD,
            RE_CONCEAL, RE_CURSOR, RE_EXTEND_CHAR, RE_ITALIC, RE_OVERLINE, RE_STRIKEOUT,
            RE_UNDERLINE,
        },
        filter::{HotSpotImpl, HotSpotType},
    },
};
use libc::wchar_t;
use std::rc::Rc;
use tmui::{cursor::Cursor, graphics::painter::Painter, prelude::*, skia_safe::Matrix};
use wchar::wch;
use widestring::WideString;

impl TerminalView {
    /// divides the part of the display specified by 'rect' into
    /// fragments according to their colors and styles and calls
    /// drawTextFragment() to draw the fragments
    pub(super) fn draw_contents(&mut self, painter: &mut Painter, rect: FRect) {
        let _image = self.image();

        let tl = self.contents_rect(Some(Coordinate::Widget)).top_left();
        let tlx = tl.x();
        let tly = tl.y();

        let lux = (self.used_columns - 1)
            .min(0.max(((rect.left() - tlx as f32 - self.left_margin) / self.font_width) as i32));
        let luy = (self.used_lines - 1)
            .min(0.max(((rect.top() - tly as f32 - self.top_margin) / self.font_height) as i32));
        let rlx = (self.used_columns - 1)
            .min(0.max(((rect.right() - tlx as f32 - self.left_margin) / self.font_width) as i32));
        let rly = (self.used_lines - 1)
            .min(0.max(((rect.bottom() - tly as f32 - self.top_margin) / self.font_height) as i32));

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

                    for c in chars.iter().take(extended_char_length as usize) {
                        assert!(p < buffer_size);
                        unistr[p] = *c;
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
                unistr.resize(p, 0);

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
                #[allow(clippy::useless_transmute)]
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
    pub(super) fn draw_text_fragment(
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

        if style.rendition & RE_CURSOR != 0 {
            self.draw_cursor(painter, rect, foreground_color, &mut invert_character_color);
        }

        // draw text
        self.draw_characters(painter, rect, &text, style, invert_character_color);

        painter.restore_pen();
    }

    /// draws the background for a text fragment
    /// if useOpacitySetting is true then the color's alpha value will be set to
    /// the display's transparency (set with setOpacity()), otherwise the
    /// background will be drawn fully opaque
    pub(super) fn draw_background(
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
    pub(super) fn draw_cursor(
        &mut self,
        painter: &mut Painter,
        rect: FRect,
        foreground_color: Color,
        invert_colors: &mut bool,
    ) {
        if self.cursor_blinking {
            return;
        }
        if !self.is_focus() {
            return;
        }

        painter.set_antialiasing(false);
        let mut cursor_rect: FRect = rect;
        cursor_rect.set_height(self.font_height - self.line_spacing as f32 - 1.);

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

            painter.fill_rect(
                adjusted_cursor_rect,
                if self.cursor_color.valid {
                    self.cursor_color
                } else {
                    foreground_color
                },
            );

            // invert the colour used to draw the text to ensure that the
            // character at the cursor position is readable
            *invert_colors = true;
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

        painter.set_antialiasing(true);
    }

    /// draws the characters or line graphics in a text fragment.
    pub(super) fn draw_characters(
        &mut self,
        painter: &mut Painter,
        mut rect: FRect,
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

        let mut font = self.font().clone();
        if font.bold() != use_bold || font.italic() != use_italic {
            font.set_bold(use_bold);
            font.set_italic(use_italic);
        }
        painter.set_font(font);

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
                if !invert_character_color {
                    painter.fill_rect(rect, style.background_color.color(&self.color_table));
                }
                painter.draw_paragraph(
                    &text,
                    (rect.x(), rect.y()),
                    0.,
                    rect.width(),
                    Some(1),
                    false,
                );
            } else {
                rect.set_height(rect.height() + self.draw_text_addition_height);
                if !invert_character_color {
                    painter.fill_rect(rect, style.background_color.color(&self.color_table));
                }
                // Draw the text start at the left-bottom.
                painter.draw_paragraph(
                    &text,
                    (rect.x(), rect.y()),
                    0.,
                    self.size().width() as f32,
                    Some(1),
                    false,
                );

                if use_underline {
                    let y = rect.bottom() - 0.5;
                    painter.draw_line_f(rect.left(), y, rect.right(), y)
                }

                if use_strike_out {
                    let y = (rect.top() + rect.bottom()) / 2.;
                    painter.draw_line_f(rect.left(), y, rect.right(), y)
                }

                if use_overline {
                    let y = rect.top() + 0.5;
                    painter.draw_line_f(rect.left(), y, rect.right(), y)
                }
            }
        }
    }

    /// draws a string of line graphics.
    pub(super) fn draw_line_char_string(
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
        for (i, b) in wchar_t_bytes.iter().enumerate() {
            let code = (*b & 0xff) as u8;
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
    pub(super) fn draw_input_method_preedit_string(&mut self, painter: &mut Painter, rect: &Rect) {
        // TODO
    }

    pub(super) fn paint_filters(&mut self, painter: &mut Painter) {
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

    pub(super) fn paint_hotspot_each_line(
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
            start_column as f32 * self.font_width + 1. + self.left_margin,
            line as f32 * self.font_height + 1. + self.top_margin,
            end_column as f32 * self.font_width - 1. + self.left_margin,
            (line as f32 + 1.) * self.font_height - 1. + self.top_margin,
        );

        match spot.type_() {
            HotSpotType::Link => {
                let (_, metrics) = self.font().to_skia_fonts()[0].metrics();
                let base_line = r.bottom() - metrics.descent;
                let under_line_pos = base_line + metrics.underline_position().unwrap();
                if region.contains_point(&self.map_to_widget_f(&Cursor::position().into())) {
                    painter.draw_line_f(r.left(), under_line_pos, r.right(), under_line_pos);
                }
            }
            HotSpotType::Marker => painter.fill_rect(r, Color::rgba(255, 0, 0, 120)),
            _ => {}
        }
    }
}

//////////////////////////////////////////////////////////////////////////////////////////////////////////
/// Display Operations
//////////////////////////////////////////////////////////////////////////////////////////////////////////
pub(super) fn draw_line_char(painter: &mut Painter, x: f32, y: f32, w: f32, h: f32, code: u8) {
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

pub(super) fn draw_other_char(painter: &mut Painter, x: f32, y: f32, w: f32, h: f32, code: u8) {
    // Calculate cell midpoints, end points.
    let cx = x + w / 2.;
    let cy = y + h / 2.;
    let ex = x + w - 1.;
    let ey = y + h - 1.;

    // Double dashes
    if (0x4C..=0x4F).contains(&code) {
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
    } else if (0x6D..=0x70).contains(&code) {
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
    } else if (0x71..=0x73).contains(&code) {
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
