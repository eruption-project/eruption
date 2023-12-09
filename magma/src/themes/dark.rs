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

#![allow(dead_code)]

use ratatui::prelude::*;

pub struct Theme {
    pub root: Style,
    pub content: Style,
    pub app_title: Style,
    pub tabs: Style,
    pub tabs_selected: Style,
    pub borders: Style,
    pub borders_highlight: Style,
    pub title: Style,
    pub title_highlight: Style,
    pub description: Style,
    pub description_highlight: Style,
    pub slot: Style,
    pub active_slot: Style,
    pub key_binding: KeyBinding,
}

pub struct KeyBinding {
    pub key: Style,
    pub description: Style,
}

pub const THEME: Theme = Theme {
    root: Style::new().bg(BLACK),
    content: Style::new().bg(BLACK).fg(LIGHT_GRAY),
    app_title: Style::new()
        .fg(WHITE)
        .bg(DARK_GRAY)
        .add_modifier(Modifier::BOLD),
    tabs: Style::new().fg(DARK_GRAY).bg(BLACK),
    tabs_selected: Style::new()
        .fg(BLACK)
        .bg(LIGHT_GRAY)
        .add_modifier(Modifier::BOLD),
    borders: Style::new().fg(DARK_GRAY),
    borders_highlight: Style::new().fg(LIGHT_GRAY),
    title: Style::new()
        .fg(DARK_GRAY)
        .bg(BLACK)
        .add_modifier(Modifier::BOLD),
    title_highlight: Style::new().fg(LIGHT_GRAY).add_modifier(Modifier::BOLD),
    description: Style::new().fg(LIGHT_GRAY).bg(DARK_GRAY),
    description_highlight: Style::new().fg(LIGHT_GRAY).bg(DARK_GRAY),
    slot: Style::new().fg(DARK_GRAY).bg(BLACK),
    active_slot: Style::new()
        .fg(LIGHT_GRAY)
        .bg(DARK_GRAY)
        .add_modifier(Modifier::BOLD),
    key_binding: KeyBinding {
        key: Style::new().fg(BLACK).bg(DARK_GRAY),
        description: Style::new().fg(DARK_GRAY).bg(BLACK),
    },
};

const LIGHT_YELLOW: Color = Color::Rgb(192, 192, 96);
const LIGHT_GREEN: Color = Color::Rgb(64, 192, 96);
const LIGHT_RED: Color = Color::Rgb(192, 96, 96);
const RED: Color = Color::Indexed(160);
const BLACK: Color = Color::Indexed(234); // not really black, often #080808
const DARK_GRAY: Color = Color::Indexed(242);
const MID_GRAY: Color = Color::Indexed(246);
const LIGHT_GRAY: Color = Color::Indexed(250);
const WHITE: Color = Color::Indexed(255); // not really white, often #eeeeee
