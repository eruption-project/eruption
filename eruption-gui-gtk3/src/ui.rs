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

pub mod about_dialog;
pub mod assistants;
pub mod automation_rules;
pub mod canvas;
pub mod color_schemes;
pub mod hwdevices;
pub mod keyboards;
pub mod keymaps;
pub mod macros;
pub mod main_window;
pub mod mice;
pub mod misc;
pub mod profiles;
pub mod rule;
pub mod settings;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Pages {
    Canvas = 0,
    Keyboards = 1,
    Mice = 2,
    Misc = 3,
    ColorSchemes = 4,
    AutomationRules = 5,
    Profiles = 6,
    Macros = 7,
    Keymaps = 8,
    Settings = 9,
}

impl From<u8> for Pages {
    fn from(value: u8) -> Self {
        match value {
            0 => Pages::Canvas,
            1 => Pages::Keyboards,
            2 => Pages::Mice,
            3 => Pages::Misc,
            4 => Pages::ColorSchemes,
            5 => Pages::AutomationRules,
            6 => Pages::Profiles,
            7 => Pages::Macros,
            8 => Pages::Keymaps,
            9 => Pages::Settings,
            _ => panic!("Invalid page"),
        }
    }
}
