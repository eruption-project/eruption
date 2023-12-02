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

use ndarray::ArrayView2;
use std::rc::Rc;

// use palette::{
//     convert::{FromColorUnclamped, IntoColorUnclamped},
//     FromColor, IntoColor, Lighten, LinSrgba, Okhsv, Srgb,
// };
use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::Color,
    widgets::Widget,
};

use resize::Pixel::RGB8;
use resize::Type;
use rgb::RGB8;

use crate::constants::{self};

pub struct RgbCanvas;

impl Widget for RgbCanvas {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Self::layout(area);
        Self::render_colors(layout[0], buf);
    }
}

impl RgbCanvas {
    fn layout(area: Rect) -> Rc<[Rect]> {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(0)])
            .split(area)
    }

    fn render_colors(area: Rect, buf: &mut Buffer) {
        let canvas = &*crate::CANVAS.read().unwrap();

        let canvas =
            ArrayView2::from_shape((constants::CANVAS_HEIGHT, constants::CANVAS_WIDTH), canvas)
                .unwrap();

        // resize
        let (w1, h1) = (constants::CANVAS_WIDTH, constants::CANVAS_HEIGHT);
        let (w2, h2) = (area.width as usize, area.height as usize);

        let canvas = canvas.map(|v| RGB8::new(v.r, v.g, v.b));
        let mut result = vec![RGB8::new(0, 0, 0); w2 * h2];

        let mut resizer = resize::new(w1, h1, w2, h2, RGB8, Type::Point).unwrap();
        resizer
            .resize(canvas.as_slice().unwrap(), &mut result)
            .unwrap();

        for (index, color) in result.iter().enumerate() {
            let x = (index % area.width as usize) as u16 + area.x;
            let y = (index / area.width as usize) as u16 + area.y;

            // post-process color
            // let source_color = LinSrgba::new(
            //     color.r as f64 / 255.0,
            //     color.g as f64 / 255.0,
            //     color.b as f64 / 255.0,
            //     color.a as f64 / 255.0,
            // );

            // let hue_value = hsl.0;
            // let saturation_value = hsl.1;
            // let lighten_value = hsl.2;

            // image post processing
            // let fg = LinSrgba::from_color(source_color).into_components();

            let fg = Color::Rgb(color.r, color.g, color.b);
            let bg = fg;

            buf.get_mut(x, y).set_char('\u{259F}').set_fg(fg).set_bg(bg);
            buf.get_mut(x + 1, y)
                .set_char('\u{259F}')
                .set_fg(fg)
                .set_bg(bg);
        }
    }
}
