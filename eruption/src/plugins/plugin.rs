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

use mlua::prelude::*;
use std::any::Any;

use crate::plugins::Result;

/// Represents a plugin

pub trait Plugin: Any {
    /// Get the user visible name of a plugin
    fn get_name(&self) -> String;

    /// Get the user visible short description of a plugin
    fn get_description(&self) -> String;

    /// Called upon initialization of the plugin
    fn initialize(&mut self) -> Result<()>;

    /// Register supplied lua functions and extensions
    fn register_lua_funcs(&self, lua_ctx: &Lua) -> mlua::Result<()>;

    /// Called on each iteration of the main loop,
    /// use this for really short lived operations only
    fn sync_main_loop_hook(&self, ticks: u64);

    /// Event handling entrypoint
    // fn process_event(&mut self, event: Event);

    /// Downcast support
    fn as_any(&self) -> &dyn Any;

    /// Downcast support (mutable)
    fn as_any_mut(&mut self) -> &mut dyn Any;
}
