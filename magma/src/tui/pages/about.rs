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
use crate::translations::tr;
use crate::tui::pages::Page;
use crossterm::event::Event;
use crossterm::event::KeyCode;
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::prelude::*;
use ratatui::widgets::*;

#[derive(Default)]
pub struct AboutPage {
    vertical_scroll: usize,
}

impl AboutPage {
    pub fn new() -> Self {
        Self { vertical_scroll: 0 }
    }
}

impl Page for AboutPage {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        Clear.render(area, buf);

        Paragraph::new("About Magma TUI")
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
                Constraint::Percentage(35),
                Constraint::Percentage(50),
                Constraint::Percentage(15),
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

        Paragraph::new(text.clone())
            .style(THEME.content)
            .scroll((self.vertical_scroll as u16, 0))
            .render(layout[1], buf);

        let scrollbar = Scrollbar::default().orientation(ScrollbarOrientation::VerticalRight);

        let mut scrollbar_state =
            ScrollbarState::new(text.lines.len()).position(self.vertical_scroll);

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
