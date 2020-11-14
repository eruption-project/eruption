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
*/

// use log::*;
use mlua::prelude::*;
use std::any::Any;

use crate::plugins::{self, Plugin};

// pub type Result<T> = std::result::Result<T, eyre::Error>;

/// A plugin that listens for key events
/// Registered events can be subsequently processed by Lua scripts
pub struct KeyboardPlugin {}

impl KeyboardPlugin {
    pub fn new() -> Self {
        KeyboardPlugin {}
    }

    pub(crate) fn get_key_state(key_index: usize) -> bool {
        crate::KEY_STATES.lock()[key_index]
    }
}

#[async_trait::async_trait]
impl Plugin for KeyboardPlugin {
    fn get_name(&self) -> String {
        "Keyboard".to_string()
    }

    fn get_description(&self) -> String {
        "Keyboard related functions".to_string()
    }

    fn initialize(&mut self) -> plugins::Result<()> {
        Ok(())
    }

    fn register_lua_funcs(&self, lua_ctx: &Lua) -> mlua::Result<()> {
        let globals = lua_ctx.globals();

        let get_key_state = lua_ctx
            .create_function(|_, key_index: usize| Ok(KeyboardPlugin::get_key_state(key_index)))?;
        globals.set("get_key_state", get_key_state)?;

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
