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

use crate::themes::THEME;

use crate::tui::pages::Page;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use itertools::Itertools;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::widgets::*;

#[derive(Default)]
pub struct HelpPage {
    vertical_scroll: usize,
}

impl HelpPage {
    pub fn new() -> Self {
        Self { vertical_scroll: 0 }
    }
}

impl Page for HelpPage {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);

        Paragraph::new("Key Bindings")
            .style(THEME.title)
            .render(area, buf);

        let area = area.inner(
            &(Margin {
                vertical: 1,
                horizontal: 0,
            }),
        );

        let layout = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Percentage(25),
                Constraint::Percentage(50),
                Constraint::Percentage(25),
            ],
        )
        .split(area);

        // help text
        let keys = [
            ("Esc", "Quit"),
            ("Tab", "Next Tab"),
            ("Backspace", "Previous Tab"),
            ("F1-F4", "Switch slots"),
            ("↑/k", "Up"),
            ("↓/j", "Down"),
            ("←/h", "Left"),
            ("→/l", "Right"),
            ("?/h", "help"),
        ];

        let spans = keys
            .iter()
            .flat_map(|(key, desc)| {
                let key = Span::styled(format!(" {} ", key), THEME.key_binding.key);
                let desc = Span::styled(format!(" {} ", desc), THEME.key_binding.description);
                [key, desc]
            })
            .collect_vec();

        let header_cells = ["Key(s)", "Action"]
            .iter()
            .map(|h| Cell::from(*h).style(THEME.content));

        let header = Row::new(header_cells).style(THEME.content).height(2);

        let rows = spans
            .iter()
            .chunks(2)
            .into_iter()
            .map(|s| {
                let contents = s.cloned().collect_vec();

                Row::new([
                    Cell::from(contents[0].clone()),
                    Cell::from(contents[1].clone()),
                ])
                .bottom_margin(1)
            })
            .collect_vec();

        let mut state = TableState::default();

        StatefulWidget::render(
            Table::new(rows.clone())
                .header(header)
                .widths([Constraint::Percentage(25), Constraint::Percentage(75)]),
            layout[1],
            buf,
            &mut state,
        );

        // let lines = spans
        //     .iter()
        //     .chunks(2)
        //     .into_iter()
        //     .map(|s| Line::from(s.cloned().collect_vec()))
        //     .collect_vec();

        // Paragraph::new(lines.clone())
        //     .alignment(Alignment::Center)
        //     .fg(Color::Indexed(236))
        //     .bg(Color::Indexed(232))
        //     .style(THEME.content)
        //     .scroll((self.vertical_scroll as u16, 0))
        //     .render(layout[1], buf);

        let scrollbar = Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight);

        let mut scrollbar_state =
            ScrollbarState::new(rows.clone().len()).position(self.vertical_scroll);

        scrollbar.render(area, buf, &mut scrollbar_state);
    }

    fn handle_event(&mut self, event: &Event) {
        if let Event::Key(key) = event {
            if key.code == KeyCode::Up {
                let _ = self.vertical_scroll.saturating_sub(1);
            } else if key.code == KeyCode::Down {
                let _ = self.vertical_scroll.saturating_add(1);
            }
        }
    }
}
