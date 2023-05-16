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
use std::collections::HashMap;

use crate::constants;

use crate::hwdevices::Zone;
use crate::plugins::{self, Plugin};

// pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct Rectangle {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Rectangle {
    #[allow(dead_code)]
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Rectangle {
            x,
            y,
            width,
            height,
        }
    }
}

impl LuaUserData for Rectangle {}

/// A plugin that provides Lua support-functions related to the canvas
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

    /// Returns the allocated zones and their respective dimensions
    pub(crate) fn get_devices_zone_allocations() -> HashMap<u64, Zone> {
        let mut result = HashMap::new();
        let mut cntr = 0;

        let keyboards = crate::KEYBOARD_DEVICES.read();

        for device in keyboards.iter() {
            result.insert(cntr, device.read().get_allocated_zone());

            cntr += 1;
        }

        let mice = crate::MOUSE_DEVICES.read();

        for device in mice.iter() {
            result.insert(cntr, device.read().get_allocated_zone());

            cntr += 1;
        }

        let misc = crate::MISC_DEVICES.read();

        for device in misc.iter() {
            result.insert(cntr, device.read().get_allocated_zone());

            cntr += 1;
        }

        result
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

        let get_devices_zone_allocations =
            lua_ctx.create_function(|_, ()| Ok(CanvasPlugin::get_devices_zone_allocations()))?;
        globals.set("get_devices_zone_allocations", get_devices_zone_allocations)?;

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
