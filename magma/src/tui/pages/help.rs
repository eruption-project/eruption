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

use crate::constants;
use crate::themes::THEME;

use crate::tui::pages::Page;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use itertools::Itertools;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::widgets::*;

#[derive(Default)]
pub struct HelpPage {
    vertical_scroll: usize,
    scroll_extents: (usize, usize),
}

impl HelpPage {
    pub fn new() -> Self {
        Self {
            vertical_scroll: 0,
            scroll_extents: (0, 0),
        }
    }
}

impl Page for HelpPage {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        frame.render_widget(Clear, area);

        frame.render_widget(Paragraph::new("Key Bindings").style(THEME.title), area);

        let area = area.inner(
            &(Margin {
                vertical: 1,
                horizontal: 0,
            }),
        );

        let layout = Layout::new(
            Direction::Horizontal,
            [
                Constraint::Percentage(15),
                Constraint::Percentage(70),
                Constraint::Percentage(15),
            ],
        )
        .split(area);

        // help text
        let keys = [
            ("Esc", "Quit"),
            ("Tab", "Next Tab"),
            ("Backspace", "Previous Tab"),
            ("F1-F4", "Switch slots"),
            ("F5-F6", "Toggle effects settings"),
            ("F8-F9", "Adjust global brightness"),
            (
                "Shift+F8-Shift+F9",
                "Adjust global brightness (fine grained)",
            ),
            (
                "Ctrl+F8-Ctrl+F9",
                "Adjust brightness of the currently visible device",
            ),
            ("Ctrl+Tab", "Next Tab on the currently active page"),
            (
                "Ctrl+Backspace",
                "Previous Tab on the currently active page",
            ),
            ("Return", "Confirm/Edit value"),
            ("Space", "Toggle"),
            ("↑", "Up/Increase value"),
            ("↓", "Down/Decrease value"),
            ("Page↑", "Page Up"),
            ("Page↓", "Page Down"),
            ("←", "Left"),
            ("→", "Right"),
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

        frame.render_stateful_widget(
            Table::new(
                rows.clone(),
                [Constraint::Percentage(20), Constraint::Percentage(80)],
            )
            .header(header)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .border_style(THEME.borders)
                    .padding(Padding {
                        left: 2,
                        right: 2,
                        top: 0,
                        bottom: 0,
                    }),
            ),
            layout[1],
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

        self.scroll_extents = (rows.len(), 0);

        let scrollbar = Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight);

        let mut scrollbar_state = ScrollbarState::new(rows.len()).position(self.vertical_scroll);

        frame.render_stateful_widget(scrollbar, layout[1], &mut scrollbar_state);
    }

    fn handle_event(&mut self, event: &Event) {
        if let Event::Key(key) = event {
            if key.code == KeyCode::Up {
                self.vertical_scroll = self.vertical_scroll.saturating_sub(constants::SCROLL_LINES);
                self.vertical_scroll = self.vertical_scroll.clamp(0, self.scroll_extents.0)
            } else if key.code == KeyCode::Down {
                self.vertical_scroll = self.vertical_scroll.saturating_add(constants::SCROLL_LINES);
                self.vertical_scroll = self.vertical_scroll.clamp(0, self.scroll_extents.0)
            }

            if key.code == KeyCode::PageUp {
                self.vertical_scroll = self
                    .vertical_scroll
                    .saturating_sub(constants::PAGE_SCROLL_LINES);
                self.vertical_scroll = self.vertical_scroll.clamp(0, self.scroll_extents.0)
            } else if key.code == KeyCode::PageDown {
                self.vertical_scroll = self
                    .vertical_scroll
                    .saturating_add(constants::PAGE_SCROLL_LINES);
                self.vertical_scroll = self.vertical_scroll.clamp(0, self.scroll_extents.0)
            }
        }
    }
}
