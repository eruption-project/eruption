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
// use log::*;
use mlua::prelude::*;
use parking_lot::Mutex;
use std::any::Any;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use sysinfo::{ComponentExt, RefreshKind, SystemExt};

use crate::plugins;
use crate::plugins::Plugin;

// pub type Result<T> = std::result::Result<T, eyre::Error>;

// #[derive(Debug, Fail)]
// pub enum SensorsPluginError {
//     #[error("Unknown error: {}", description)]
//     UnknownError { description: String },
// }

lazy_static! {
    /// If set to true, sensors are refreshed every SENSOR_UPDATE_TICKS main loop ticks
    static ref DO_REFRESH: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    /// System state and sensor information
    static ref SYSTEM: Arc<Mutex<sysinfo::System>> = Arc::new(Mutex::new(sysinfo::System::new_with_specifics(RefreshKind::default().with_components().with_memory())));
}

/// A plugin that gives Lua scripts access to the systems sensor data
pub struct SensorsPlugin {}

impl SensorsPlugin {
    pub fn new() -> Self {
        let mut system = SYSTEM.lock();

        system.refresh_memory();
        system.refresh_components_list();
        system.refresh_components();

        SensorsPlugin {}
    }

    /// Refresh state of sensors
    pub fn refresh() {
        let mut system = SYSTEM.lock();

        system.refresh_memory();
        system.refresh_components();
    }

    /// Get the temperature of the CPU package
    pub fn get_package_temp() -> f32 {
        DO_REFRESH.store(true, Ordering::SeqCst);

        let system = SYSTEM.lock();

        let components = system.get_components();
        if components.len() > 1 {
            components
                .iter()
                .find(|c| c.get_label().contains("Package id 0"))
                .and_then(|c| Some(c.get_temperature()))
                .unwrap_or_else(|| 0.0)
        } else {
            0.0
        }
    }

    /// Get the max. temperature of the CPU package
    pub fn get_package_max_temp() -> f32 {
        DO_REFRESH.store(true, Ordering::SeqCst);

        let system = SYSTEM.lock();

        let components = system.get_components();
        if components.len() > 1 {
            components
                .iter()
                .find(|c| c.get_label().contains("Package id 0"))
                .and_then(|c| Some(c.get_temperature()))
                .unwrap_or_else(|| 0.0)
        } else {
            0.0
        }
    }

    /// Get the total installed memory size
    pub fn get_mem_total_kb() -> u64 {
        DO_REFRESH.store(true, Ordering::SeqCst);

        let system = SYSTEM.lock();
        system.get_total_memory()
    }

    /// Get the amount of used memory
    pub fn get_mem_used_kb() -> u64 {
        DO_REFRESH.store(true, Ordering::SeqCst);

        let system = SYSTEM.lock();
        system.get_used_memory()
    }

    /// Get the total amount of swap space in kilobytes
    pub fn get_swap_total_kb() -> u64 {
        DO_REFRESH.store(true, Ordering::SeqCst);

        let system = SYSTEM.lock();
        system.get_total_swap()
    }

    /// Get the amount of used swap space in kilobytes
    pub fn get_swap_used_kb() -> u64 {
        DO_REFRESH.store(true, Ordering::SeqCst);

        let system = SYSTEM.lock();
        system.get_used_swap()
    }
}

#[async_trait::async_trait]
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

    fn register_lua_funcs(&self, lua_ctx: &Lua) -> mlua::Result<()> {
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

    async fn main_loop_hook(&self, ticks: u64) {
        // refresh sensor state (default: every other second), but only
        // if the sensors have been used at least once
        if ticks % crate::constants::SENSOR_UPDATE_TICKS == 0 && DO_REFRESH.load(Ordering::SeqCst) {
            Self::refresh();
        }
    }

    fn sync_main_loop_hook(&self, _ticks: u64) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
