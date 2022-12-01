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

/// These are all the names of supported Lua function handlers invoked by eruption at certain events.
pub const FUNCTION_ON_STARTUP: &str = "on_startup";
pub const FUNCTION_ON_QUIT: &str = "on_quit";
pub const FUNCTION_ON_TICK: &str = "on_tick";
pub const FUNCTION_ON_APPLY_PARAMETER: &str = "on_apply_parameter";
pub const FUNCTION_ON_KEY_DOWN: &str = "on_key_down";
pub const FUNCTION_ON_KEY_UP: &str = "on_key_up";
pub const FUNCTION_ON_MOUSE_BUTTON_DOWN: &str = "on_mouse_button_down";
pub const FUNCTION_ON_MOUSE_BUTTON_UP: &str = "on_mouse_button_up";
pub const FUNCTION_ON_MOUSE_WHEEL: &str = "on_mouse_wheel";
pub const FUNCTION_ON_MOUSE_MOVE: &str = "on_mouse_move";
pub const FUNCTION_ON_HID_EVENT: &str = "on_hid_event";
pub const FUNCTION_ON_MOUSE_HID_EVENT: &str = "on_mouse_hid_event";
