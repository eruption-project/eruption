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

//use failure::Fail;
use rlua::Context;
use std::any::Any;

use crate::plugins;
use crate::plugins::Plugin;

//pub type Result<T> = std::result::Result<T, ProfilesPluginError>;

//#[derive(Debug, Fail)]
//pub enum ProfilesPluginError {
////#[fail(display = "Unknown error: {}", description)]
////UnknownError { description: String },
//}

/// A plugin that enables Eruption to switch profiles, based on the current system state
pub struct ProfilesPlugin {}

impl ProfilesPlugin {
    pub fn new() -> Self {
        ProfilesPlugin {}
    }
}

impl Plugin for ProfilesPlugin {
    fn get_name(&self) -> String {
        "Profiles".to_string()
    }

    fn get_description(&self) -> String {
        "Switch profiles based on system state".to_string()
    }

    fn initialize(&mut self) -> plugins::Result<()> {
        Ok(())
    }

    fn register_lua_funcs(&self, _lua_ctx: Context) -> rlua::Result<()> {
        // let globals = lua_ctx.globals();

        Ok(())
    }

    fn main_loop_hook(&self, _ticks: u64) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
