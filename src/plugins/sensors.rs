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

use lazy_static::lazy_static;
use log::*;
use rlua;
use rlua::Context;
use std::any::Any;
use std::error;
use std::error::Error;
use std::fmt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use sysinfo::{ComponentExt, SystemExt};

use crate::plugins;
use crate::plugins::Plugin;

// pub type Result<T> = std::result::Result<T, SensorsPluginError>;

#[derive(Debug, Clone)]
pub struct SensorsPluginError {
    code: u32,
}

impl error::Error for SensorsPluginError {
    fn description(&self) -> &str {
        match self.code {
            _ => "Unknown error",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}

impl fmt::Display for SensorsPluginError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

lazy_static! {
    static ref DO_REFRESH: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
    static ref SYSTEM: Arc<Mutex<sysinfo::System>> = Arc::new(Mutex::new(sysinfo::System::new()));
}

pub struct SensorsPlugin {}

/// A plugin that gives Lua scripts access to the systems sensor data
impl SensorsPlugin {
    pub fn new() -> Self {
        SensorsPlugin {}
    }

    pub fn refresh() {
        // we need to spawn a thread here, since sensor updating is really slooow
        let builder = thread::Builder::new().name("sensors".into());
        builder
            .spawn(move || {
                let mut system = SYSTEM.lock().unwrap_or_else(|e| {
                    error!("Could not lock a shared data structure: {}", e);
                    panic!();
                });
                system.refresh_all();
            })
            .unwrap_or_else(|e| {
                error!("Could not spawn a thread: {}", e);
                panic!()
            });
    }

    pub fn get_package_temp() -> f32 {
        DO_REFRESH.store(true, Ordering::SeqCst);

        let system = SYSTEM.lock().unwrap_or_else(|e| {
            error!("Could not lock a shared data structure: {}", e);
            panic!();
        });;

        let components = system.get_components_list();
        components[components.len() - 1].get_temperature()
    }

    pub fn get_package_max_temp() -> f32 {
        DO_REFRESH.store(true, Ordering::SeqCst);

        let system = SYSTEM.lock().unwrap_or_else(|e| {
            error!("Could not lock a shared data structure: {}", e);
            panic!();
        });;

        let components = system.get_components_list();
        components[components.len() - 1].get_max()
    }

    pub fn get_mem_total_kb() -> u64 {
        DO_REFRESH.store(true, Ordering::SeqCst);

        let system = SYSTEM.lock().unwrap_or_else(|e| {
            error!("Could not lock a shared data structure: {}", e);
            panic!();
        });;

        system.get_total_memory()
    }

    pub fn get_mem_used_kb() -> u64 {
        DO_REFRESH.store(true, Ordering::SeqCst);

        let system = SYSTEM.lock().unwrap_or_else(|e| {
            error!("Could not lock a shared data structure: {}", e);
            panic!();
        });;

        system.get_used_memory()
    }

    pub fn get_swap_total_kb() -> u64 {
        DO_REFRESH.store(true, Ordering::SeqCst);

        let system = SYSTEM.lock().unwrap_or_else(|e| {
            error!("Could not lock a shared data structure: {}", e);
            panic!();
        });;

        system.get_total_swap()
    }

    pub fn get_swap_used_kb() -> u64 {
        DO_REFRESH.store(true, Ordering::SeqCst);

        let system = SYSTEM.lock().unwrap_or_else(|e| {
            error!("Could not lock a shared data structure: {}", e);
            panic!();
        });;

        system.get_used_swap()
    }
}

impl Plugin for SensorsPlugin {
    fn get_name(&self) -> String {
        "Sensors".to_string()
    }

    fn get_description(&self) -> String {
        "Query system sensor values".to_string()
    }

    fn initialize(&mut self) -> plugins::Result<()> {
        Ok(())
    }

    fn register_lua_funcs(&self, lua_ctx: Context) -> rlua::Result<()> {
        let globals = lua_ctx.globals();

        let get_package_temp =
            lua_ctx.create_function(move |_, ()| Ok(SensorsPlugin::get_package_temp()))?;
        globals.set("get_package_temp", get_package_temp)?;

        let get_package_max_temp =
            lua_ctx.create_function(move |_, ()| Ok(SensorsPlugin::get_package_max_temp()))?;
        globals.set("get_package_max_temp", get_package_max_temp)?;

        let get_mem_total_kb =
            lua_ctx.create_function(move |_, ()| Ok(SensorsPlugin::get_mem_total_kb()))?;
        globals.set("get_mem_total_kb", get_mem_total_kb)?;

        let get_mem_used_kb =
            lua_ctx.create_function(move |_, ()| Ok(SensorsPlugin::get_mem_used_kb()))?;
        globals.set("get_mem_used_kb", get_mem_used_kb)?;

        let get_swap_total_kb =
            lua_ctx.create_function(move |_, ()| Ok(SensorsPlugin::get_swap_total_kb()))?;
        globals.set("get_swap_total_kb", get_swap_total_kb)?;

        let get_swap_used_kb =
            lua_ctx.create_function(move |_, ()| Ok(SensorsPlugin::get_swap_used_kb()))?;
        globals.set("get_swap_used_kb", get_swap_used_kb)?;

        Ok(())
    }

    fn main_loop_hook(&self, ticks: u64) {
        // refresh sensor state every other second, but only
        // if the sensors have been used at least once
        if ticks % crate::constants::SENSOR_UPDATE_TICKS == 0 && DO_REFRESH.load(Ordering::SeqCst) {
            Self::refresh();
        }
    }

    fn as_any(&self) -> &Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self
    }
}
