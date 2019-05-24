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

use log::*;
use procinfo;
use rlua;
use rlua::Context;
use std::any::Any;
use std::error;
use std::error::Error;
use std::fmt;

use crate::plugins;
use crate::plugins::Plugin;

// pub type Result<T> = std::result::Result<T, DbusPluginError>;

#[derive(Debug, Clone)]
pub struct DbusPluginError {
    code: u32,
}

impl error::Error for DbusPluginError {
    fn description(&self) -> &str {
        match self.code {
            _ => "Unknown error",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}

impl fmt::Display for DbusPluginError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// A plugin that gives Lua scripts access to the systems state like e.g.
/// the number of runnable processes or the load average
pub struct DbusPlugin {}

impl DbusPlugin {
    pub fn new() -> Self {
        DbusPlugin {}
    }
}

impl Plugin for DbusPlugin {
    fn get_name(&self) -> String {
        "DBUS".to_string()
    }

    fn get_description(&self) -> String {
        "DBUS support plugin".to_string()
    }

    fn initialize(&mut self) -> plugins::Result<()> {
        Ok(())
    }

    fn register_lua_funcs(&self, lua_ctx: Context) -> rlua::Result<()> {
        let globals = lua_ctx.globals();

        // let get_current_load_avg_1 =
        //     lua_ctx.create_function(|_, ()| Ok(DbusPlugin::get_current_load_avg_1()))?;
        // globals.set("get_current_load_avg_1", get_current_load_avg_1)?;

        Ok(())
    }

    fn main_loop_hook(&self, _ticks: u64) {}

    fn as_any(&self) -> &Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self
    }
}
