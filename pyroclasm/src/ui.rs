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

pub mod hwdevices;
pub mod pages;

#[derive(Debug, Default, Clone, PartialEq, serde::Deserialize, serde::Serialize)]
pub enum TabPages {
    /// Startup page
    #[default]
    Start,

    /// Canvas page
    Canvas,

    /// Keyboard devices page
    Keyboards,

    /// Mouse devices page
    Mice,

    /// Misc devices page
    Misc,

    /// Profiles page
    Profiles,

    /// Macros page
    Macros,

    /// Rules page
    Rules,

    /// Color-schemes page
    ColorSchemes,

    /// Settings page
    Settings,

    /// The about page
    About,

    /// The logs page
    Logs,

    /// The debug page
    Debug,
}
