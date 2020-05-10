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
use rlua::Context;
use std::any::Any;
// use failure::Fail;

use std::process::Command;

use crate::plugins;
use crate::plugins::Plugin;

// pub type Result<T> = std::result::Result<T, SystemPluginError>;

// #[derive(Debug, Fail)]
// pub enum SystemPluginError {
//     #[fail(display = "Unknown error: {}", description)]
//     UnknownError { description: String },
// }

/// A plugin that gives Lua scripts access to the systems state like e.g.
/// the number of runnable processes or the load average
pub struct SystemPlugin {}

impl SystemPlugin {
    pub fn new() -> Self {
        SystemPlugin {}
    }

    /// Get the system's load average of the last minute
    pub(crate) fn get_current_load_avg_1() -> f32 {
        procinfo::loadavg()
            .unwrap_or_else(|e| {
                error!("Could not gather status information: {}", e);
                panic!();
            })
            .load_avg_1_min
    }

    /// Get the system's load average of the last 5 minutes
    pub(crate) fn get_current_load_avg_5() -> f32 {
        procinfo::loadavg()
            .unwrap_or_else(|e| {
                error!("Could not gather status information: {}", e);
                panic!();
            })
            .load_avg_5_min
    }

    /// Get the system's load average of the last 10 minutes
    pub(crate) fn get_current_load_avg_10() -> f32 {
        procinfo::loadavg()
            .unwrap_or_else(|e| {
                error!("Could not gather status information: {}", e);
                panic!();
            })
            .load_avg_10_min
    }

    /// Get the number of runnable tasks
    pub(crate) fn get_runnable_tasks() -> u32 {
        procinfo::loadavg()
            .unwrap_or_else(|e| {
                error!("Could not gather status information: {}", e);
                panic!();
            })
            .tasks_runnable
    }

    /// Get the number of tasks on the system
    pub(crate) fn get_total_tasks() -> u32 {
        procinfo::loadavg()
            .unwrap_or_else(|e| {
                error!("Could not gather status information: {}", e);
                panic!();
            })
            .tasks_total
    }

    /// Execute a shell command
    pub(crate) fn system(command: &String, args: &Vec<String>) -> i32 {
        Command::new(command)
            .args(args)
            // .envs(&envs)
            .status()
            .map_or(Ok(std::i32::MIN), |v| v.code().ok_or(std::i32::MIN))
            .unwrap()
    }
}

impl Plugin for SystemPlugin {
    fn get_name(&self) -> String {
        "System".to_string()
    }

    fn get_description(&self) -> String {
        "Basic system information and status".to_string()
    }

    fn initialize(&mut self) -> plugins::Result<()> {
        Ok(())
    }

    fn register_lua_funcs(&self, lua_ctx: Context) -> rlua::Result<()> {
        let globals = lua_ctx.globals();

        let get_current_load_avg_1 =
            lua_ctx.create_function(|_, ()| Ok(SystemPlugin::get_current_load_avg_1()))?;
        globals.set("get_current_load_avg_1", get_current_load_avg_1)?;

        let get_current_load_avg_5 =
            lua_ctx.create_function(|_, ()| Ok(SystemPlugin::get_current_load_avg_5()))?;
        globals.set("get_current_load_avg_5", get_current_load_avg_5)?;

        let get_current_load_avg_10 =
            lua_ctx.create_function(|_, ()| Ok(SystemPlugin::get_current_load_avg_10()))?;
        globals.set("get_current_load_avg_10", get_current_load_avg_10)?;

        let get_runnable_tasks =
            lua_ctx.create_function(|_, ()| Ok(SystemPlugin::get_runnable_tasks()))?;
        globals.set("get_runnable_tasks", get_runnable_tasks)?;

        let get_total_tasks =
            lua_ctx.create_function(|_, ()| Ok(SystemPlugin::get_total_tasks()))?;
        globals.set("get_total_tasks", get_total_tasks)?;

        let system = lua_ctx.create_function(|_, (command, args): (String, Vec<String>)| {
            Ok(SystemPlugin::system(&command, &args))
        })?;
        globals.set("system", system)?;

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
