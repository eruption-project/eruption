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

use console::Key;

use ratatui::{
    prelude::*,
    widgets::{Block, List, ListItem, Paragraph, Widget},
};

// use crate::translations::tr;

// type Result<T> = std::result::Result<T, eyre::Error>;

pub struct ComboBox<'a> {
    items: Vec<String>,
    selected_index: usize,
    is_open: bool,

    block: Option<Block<'a>>,
    alignment: Alignment,
    style: Style,
}

impl<'a> ComboBox<'a> {
    pub fn new(items: &[String]) -> Self {
        Self {
            items: items.to_vec(),
            selected_index: 0,
            is_open: false,

            block: Default::default(),
            alignment: Default::default(),
            style: Default::default(),
        }
    }

    pub fn toggle_open(&mut self) {
        self.is_open = !self.is_open;
    }

    pub fn _handle_input(&mut self, key: Key) {
        match key {
            Key::Enter => self.toggle_open(),

            Key::ArrowDown => {
                if !self.is_open {
                    self.toggle_open();
                } else {
                    self.selected_index = (self.selected_index + 1) % self.items.len();
                }
            }
            Key::ArrowUp => {
                if self.is_open {
                    self.selected_index = if self.selected_index == 0 {
                        self.items.len() - 1
                    } else {
                        self.selected_index - 1
                    };
                }
            }

            _ => { /* do nothing */ }
        }
    }

    pub fn alignment(mut self, alignment: Alignment) -> Self {
        self.alignment = alignment;
        self
    }

    pub fn block(mut self, block: Block<'a>) -> Self {
        self.block = Some(block);
        self
    }

    pub fn style(mut self, style: Style) -> Self {
        self.style = style;
        self
    }
}

impl<'a> Widget for ComboBox<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if !self.is_open {
            let text = Line::styled(self.items[self.selected_index].clone(), Style::default());

            let p = Paragraph::new(text)
                .alignment(self.alignment)
                .style(self.style);

            if let Some(block) = self.block {
                p.block(block).render(area, buf);
            } else {
                p.render(area, buf);
            }
        } else {
            let items = self
                .items
                .iter()
                .map(|e| ListItem::new(Text::from(e.clone())))
                .collect::<Vec<_>>();

            let p = List::new(items).style(self.style);

            if let Some(block) = self.block {
                Widget::render(p.block(block), area, buf);
            } else {
                Widget::render(p, area, buf);
            }
        }
    }
}

impl<'a> Styled for ComboBox<'a> {
    type Item = ComboBox<'a>;

    fn style(&self) -> Style {
        self.style
    }

    fn set_style(self, _style: Style) -> Self::Item {
        // self.style = style;
        self
    }
}
