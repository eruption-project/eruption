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

// use tracing::*;
use mlua::prelude::*;
use std::any::Any;

use crate::constants;

use crate::plugins::{self, Plugin};

// pub type Result<T> = std::result::Result<T, eyre::Error>;

/// A plugin that listens for key events
/// Registered events can be subsequently processed by Lua scripts
pub struct CanvasPlugin {}

impl CanvasPlugin {
    pub fn new() -> Self {
        CanvasPlugin {}
    }

    /// Returns the number of "pixels" on the canvas
    pub(crate) fn get_canvas_size() -> usize {
        constants::CANVAS_SIZE
    }

    /// Returns the height of the canvas
    pub(crate) fn get_canvas_height() -> usize {
        constants::CANVAS_HEIGHT
    }

    /// Returns the width of the canvas
    pub(crate) fn get_canvas_width() -> usize {
        constants::CANVAS_WIDTH
    }
}

#[async_trait::async_trait]
impl Plugin for CanvasPlugin {
    fn get_name(&self) -> String {
        "Canvas".to_string()
    }

    fn get_description(&self) -> String {
        "Canvas related functions".to_string()
    }

    fn initialize(&mut self) -> plugins::Result<()> {
        Ok(())
    }

    fn register_lua_funcs(&self, lua_ctx: &Lua) -> mlua::Result<()> {
        let globals = lua_ctx.globals();

        // canvas related functions
        let get_canvas_size =
            lua_ctx.create_function(|_, ()| Ok(CanvasPlugin::get_canvas_size()))?;
        globals.set("get_canvas_size", get_canvas_size)?;

        let get_canvas_width =
            lua_ctx.create_function(|_, ()| Ok(CanvasPlugin::get_canvas_width()))?;
        globals.set("get_canvas_width", get_canvas_width)?;

        let get_canvas_height =
            lua_ctx.create_function(|_, ()| Ok(CanvasPlugin::get_canvas_height()))?;
        globals.set("get_canvas_height", get_canvas_height)?;

        Ok(())
    }

    async fn main_loop_hook(&self, _ticks: u64) {}

    fn sync_main_loop_hook(&self, _ticks: u64) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}