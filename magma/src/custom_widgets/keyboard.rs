/*  SPDX-License-Identifier: GPL-3.0-or-later  */

/*
    This file is part of Eruption.

    Eruption is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    Eruption is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Eruption.  If not, see <http://www.gnu.org/licenses/>.

    Copyright (c) 2019-2023, The Eruption Development Team
*/

use std::rc::Rc;

use palette::{
    convert::{FromColorUnclamped, IntoColorUnclamped},
    Okhsv, Srgb,
};
use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Stylize},
    widgets::{Paragraph, Widget},
};

pub struct Keyboard;

impl Widget for Keyboard {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Self::layout(area);
        Self::render_title(layout[0], buf);
        Self::render_colors(layout[1], buf);
    }
}

impl Keyboard {
    fn layout(area: Rect) -> Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(1), Constraint::Min(0)])
            .split(area)
    }

    fn render_title(area: Rect, buf: &mut Buffer) {
        Paragraph::new("colors_rgb example. Press q to quit")
            .dark_gray()
            .alignment(Alignment::Center)
            .render(area, buf);
    }

    /// Render a colored grid of half block characters (`"▀"`) each with a different RGB color.
    fn render_colors(area: Rect, buf: &mut Buffer) {
        for (xi, x) in (area.left()..area.right()).enumerate() {
            for (yi, y) in (area.top()..area.bottom()).enumerate() {
                let hue = xi as f32 * 360.0 / area.width as f32;

                let value_fg = (yi as f32) / (area.height as f32 - 0.5);
                let fg = Okhsv::<f32>::new(hue, Okhsv::max_saturation(), value_fg);
                let fg: Srgb = fg.into_color_unclamped();
                let fg: Srgb<u8> = fg.into_format();
                let fg = Color::Rgb(fg.red, fg.green, fg.blue);

                let value_bg = (yi as f32 + 0.5) / (area.height as f32 - 0.5);
                let bg = Okhsv::new(hue, Okhsv::max_saturation(), value_bg);
                let bg = Srgb::<f32>::from_color_unclamped(bg);
                let bg: Srgb<u8> = bg.into_format();
                let bg = Color::Rgb(bg.red, bg.green, bg.blue);

                buf.get_mut(x, y).set_char('▀').set_fg(fg).set_bg(bg);
            }
        }
    }
}
