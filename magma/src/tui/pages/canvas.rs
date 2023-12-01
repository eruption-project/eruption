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

use crate::custom_widgets;
use crate::themes::THEME;
use crossterm::event::Event;
use ratatui::buffer::Buffer;
use ratatui::layout::{Rect};
use ratatui::widgets::{Clear, Paragraph, Widget};

use super::Page;

#[derive(Default)]
pub struct CanvasPage {}

impl CanvasPage {
    pub fn new() -> Self {
        Self {}
    }
}

impl Page for CanvasPage {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);

        Paragraph::new("Canvas")
            .style(THEME.title)
            .render(area, buf);

        let canvas = custom_widgets::RgbCanvas {};
        canvas.render(area, buf);
    }

    fn handle_event(&mut self, _event: &Event) {}
}
