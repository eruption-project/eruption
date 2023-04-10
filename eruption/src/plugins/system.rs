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
use sysinfo::{ProcessExt, ProcessStatus, System, SystemExt};
// use tracing::*;

use std::process::Command;
use std::sync::atomic::Ordering;

use crate::plugins;
use crate::plugins::Plugin;

// pub type Result<T> = std::result::Result<T, eyre::Error>;

// #[derive(Debug, Fail)]
// pub enum SystemPluginError {
//     #[error("Unknown error: {}", description)]
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
    pub(crate) fn get_current_load_avg_1() -> f64 {
        System::new().load_average().one
    }

    /// Get the system's load average of the last 5 minutes
    pub(crate) fn get_current_load_avg_5() -> f64 {
        System::new().load_average().five
    }

    /// Get the system's load average of the last 15 minutes
    pub(crate) fn get_current_load_avg_15() -> f64 {
        System::new().load_average().fifteen
    }

    /// Get the number of runnable tasks
    pub(crate) fn get_runnable_tasks() -> usize {
        System::new()
            .processes()
            .iter()
            .filter(|(_pid, process)| process.status() == ProcessStatus::Run)
            .count()
    }

    /// Get the number of tasks on the system
    pub(crate) fn get_total_tasks() -> usize {
        System::new().processes().len()
    }

    /// Execute a shell command
    pub(crate) fn system(command: &str, args: &[String]) -> i32 {
        Command::new(command)
            .args(args)
            // .envs(&envs)
            .status()
            .map_or(Ok(std::i32::MIN), |v| v.code().ok_or(std::i32::MIN))
            .unwrap()
    }

    /// Terminate the Eruption daemon
    pub(crate) fn exit() {
        crate::QUIT.store(true, Ordering::SeqCst);
    }
}

#[async_trait::async_trait]
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

    fn register_lua_funcs(&self, lua_ctx: &Lua) -> mlua::Result<()> {
        let globals = lua_ctx.globals();

        let get_current_load_avg_1 =
            lua_ctx.create_function(|_, ()| Ok(SystemPlugin::get_current_load_avg_1()))?;
        globals.set("get_current_load_avg_1", get_current_load_avg_1)?;

        let get_current_load_avg_5 =
            lua_ctx.create_function(|_, ()| Ok(SystemPlugin::get_current_load_avg_5()))?;
        globals.set("get_current_load_avg_5", get_current_load_avg_5)?;

        let get_current_load_avg_15 =
            lua_ctx.create_function(|_, ()| Ok(SystemPlugin::get_current_load_avg_15()))?;
        globals.set("get_current_load_avg_15", get_current_load_avg_15)?;

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

        let exit = lua_ctx.create_function(|_, (): ()| {
            SystemPlugin::exit();
            Ok(())
        })?;
        globals.set("exit", exit)?;

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
