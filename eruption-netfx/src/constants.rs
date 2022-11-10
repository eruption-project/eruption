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

/// Default host name
pub const DEFAULT_HOST: &str = "localhost";

/// Default port number
pub const DEFAULT_PORT: u16 = 2359;

/// The width of the canvas
pub const CANVAS_WIDTH: usize = 128;

/// The height of the canvas
pub const CANVAS_HEIGHT: usize = 64;

/// The number of "pixels" on the canvas
pub const CANVAS_SIZE: usize = CANVAS_WIDTH * CANVAS_HEIGHT;

/// Default delay between images, used for animation mode
pub const DEFAULT_ANIMATION_DELAY_MILLIS: u64 = 83;

/// Default delay between screenshots, used for ambient mode
pub const DEFAULT_FRAME_DELAY_MILLIS: u64 = 37;

/// Timeout value to use for D-Bus connections
pub const DBUS_TIMEOUT_MILLIS: u32 = 250;
