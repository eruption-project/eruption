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

    Copyright (c) 2019-2022, The Eruption Development Team
*/

#![allow(dead_code)]

/// Default delay between images, used for animation mode
pub const DEFAULT_ANIMATION_DELAY_MILLIS: u64 = 83;

/// Default delay between screenshots, used for ambient mode
pub const DEFAULT_FRAME_DELAY_MILLIS: u64 = 37;

/// Timeout value to use for D-Bus connections
pub const DBUS_TIMEOUT_MILLIS: u64 = 5000;

/// Main loop delay
pub const MAIN_LOOP_SLEEP_MILLIS: u64 = 250;

/// Timeout value to use for D-Bus connections
/// that may involve interactivity like e.g.:
/// PolicyKit authentication
pub const DBUS_TIMEOUT_MILLIS_INTERACTIVE: u64 = 30000;

/// Default X11 display used by the X11 sensor plugin
pub const DEFAULT_X11_DISPLAY: &str = ":0";
