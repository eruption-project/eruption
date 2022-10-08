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

use lazy_static::lazy_static;
// use log::*;
use mlua::prelude::*;
use std::any::Any;
use std::collections::HashMap;

use crate::plugins;
use crate::plugins::Plugin;

pub type Result<T> = std::result::Result<T, eyre::Error>;

// #[derive(Debug, Fail)]
// pub enum HwAccelerationPluginError {
//     #[error("Unknown error: {}", description)]
//     UnknownError { description: String },
// }

lazy_static! {}

/// A plugin that gives Lua scripts access to the systems sensor data
pub struct HwAccelerationPlugin {}

impl HwAccelerationPlugin {
    pub fn new() -> Self {
        HwAccelerationPlugin {}
    }

    pub fn query_hw_accel_info() -> HashMap<String, String> {
        let mut result = HashMap::new();

        result.insert("backend".to_string(), "vulkan".to_string());
        result.insert("acceleration".to_string(), "false".to_string());

        result
    }

    pub fn compile_shader_program() -> Result<()> {
        Ok(())
    }

    pub fn set_uniform_value(_value: u32) {}

    pub fn get_uniform_value() -> u32 {
        0
    }
}

#[async_trait::async_trait]
impl Plugin for HwAccelerationPlugin {
    fn get_name(&self) -> String {
        "Hardware Acceleration".to_string()
    }

    fn get_description(&self) -> String {
        "Hardware accelerated effects using shader programs".to_string()
    }

    fn initialize(&mut self) -> plugins::Result<()> {
        Ok(())
    }

    fn register_lua_funcs(&self, lua_ctx: &Lua) -> mlua::Result<()> {
        let globals = lua_ctx.globals();

        let query_hw_accel_info = lua_ctx
            .create_function(move |_, ()| Ok(HwAccelerationPlugin::query_hw_accel_info()))?;
        globals.set("query_hw_accel_info", query_hw_accel_info)?;

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
