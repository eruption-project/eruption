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
use crate::translations::tr;
use crate::tui::pages::Page;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::widgets::*;

#[derive(Default)]
pub struct AboutPage {
    vertical_scroll: usize,
    scroll_extents: (usize, usize),
}

impl AboutPage {
    pub fn new() -> Self {
        Self {
            vertical_scroll: 0,
            scroll_extents: (0, 0),
        }
    }
}

impl Page for AboutPage {
    fn render(&mut self, frame: &mut Frame<'_>, area: Rect) {
        frame.render_widget(Clear, area);

        frame.render_widget(Paragraph::new("About Magma TUI").style(THEME.title), area);

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

        let text = r"
    ********                          **   **
    /**/////                 ******   /**  //
    /**       ****** **   **/**///** ****** **  ******  *******
    /******* //**//*/**  /**/**  /**///**/ /** **////**//**///**
    /**////   /** / /**  /**/******   /**  /**/**   /** /**  /**
    /**       /**   /**  /**/**///    /**  /**/**   /** /**  /**
    /********/***   //******/**       //** /**//******  ***  /**
    //////// ///     ////// //         //  //  //////  ///   //

";

        let text = Text::raw(format!("{text}\n\n{}", tr!("license-header")));

        frame.render_widget(
            Paragraph::new(text.clone())
                .style(THEME.content)
                .scroll((self.vertical_scroll as u16, 0)),
            layout[1],
        );

        self.scroll_extents = (text.lines.len(), 0);

        let scrollbar = Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight);

        let mut scrollbar_state =
            ScrollbarState::new(text.lines.len()).position(self.vertical_scroll);

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
