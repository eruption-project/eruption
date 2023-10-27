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

use crate::plugins::{self, Plugin};

// pub type Result<T> = std::result::Result<T, IntrospectionPluginError>;

// #[derive(Debug, Fail)]
// pub enum IntrospectionPluginError {
//     #[error("Unknown error: {}", description)]
//     UnknownError { description: String },
// }

pub struct IntrospectionPlugin {}

impl IntrospectionPlugin {
    pub fn new() -> Self {
        IntrospectionPlugin {}
    }
}

impl Plugin for IntrospectionPlugin {
    fn get_name(&self) -> String {
        "Introspection".to_string()
    }

    fn get_description(&self) -> String {
        "Provide runtime status of the Eruption daemon".to_string()
    }

    fn initialize(&mut self) -> plugins::Result<()> {
        Ok(())
    }

    fn register_lua_funcs(&self, _lua_ctx: &Lua) -> mlua::Result<()> {
        // let globals = lua_ctx.globals();

        Ok(())
    }

    fn sync_main_loop_hook(&self, _ticks: u64) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
