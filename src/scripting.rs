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
use rlua::{Context, Function, Lua, Result};
use std::collections::HashMap;
use std::fs;
use std::num::*;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex};
use std::vec::Vec;

use crate::plugin_manager;
use crate::rvdevice::{RvDeviceState, NUM_KEYS, RGB};

pub enum Message {
    Startup,
    Quit(u32),
    Tick(u32),
    KeyDown(u8),
}

lazy_static! {
    /// Global LED state of the managed device
    pub static ref LED_MAP: Arc<Mutex<Vec<RGB>>> = Arc::new(Mutex::new(vec![RGB {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    }; NUM_KEYS]));
}

mod callbacks {
    use log::*;
    use std::convert::TryFrom;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    use super::LED_MAP;

    use crate::rvdevice::{RvDeviceState, NUM_KEYS, RGB};

    /// Log a message with severity level `trace`
    /// this function is intended to be used from within lua scripts
    pub(crate) fn log_trace(x: &str) {
        trace!("{}", x);
    }

    /// Log a message with severity level `debug`
    /// this function is intended to be used from within lua scripts
    pub(crate) fn log_debug(x: &str) {
        debug!("{}", x);
    }

    /// Log a message with severity level `info`
    /// this function is intended to be used from within lua scripts
    pub(crate) fn log_info(x: &str) {
        info!("{}", x);
    }

    /// Log a message with severity level `warn`
    /// this function is intended to be used from within lua scripts
    pub(crate) fn log_warn(x: &str) {
        warn!("{}", x);
    }

    /// Log a message with severity level `error`
    /// this function is intended to be used from within lua scripts
    pub(crate) fn log_error(x: &str) {
        error!("{}", x);
    }

    /// Delay execution of the lua script by `millis` milliseconds
    /// this function is intended to be used from within lua scripts
    pub(crate) fn delay(millis: u64) {
        thread::sleep(Duration::from_millis(millis));
    }

    /// Get the number of keys of the managed device
    /// this function is intended to be used from within lua scripts
    pub(crate) fn get_num_keys() -> usize {
        NUM_KEYS
    }

    /// Get the current color of the key `idx`
    /// this function is intended to be used from within lua scripts
    pub(crate) fn get_key_color(rvdevid: &str, idx: usize) -> u32 {
        error!("{}: {}", rvdevid, idx);
        0
    }

    /// Set the color of the key `idx` to `c`
    /// this function is intended to be used from within lua scripts
    pub(crate) fn set_key_color(rvdev: &Arc<Mutex<RvDeviceState>>, idx: usize, c: u32) {
        match LED_MAP.lock() {
            Ok(mut led_map) => {
                led_map[idx] = RGB {
                    r: u8::try_from((c >> 16) & 0xff).unwrap(),
                    g: u8::try_from((c >> 8) & 0xff).unwrap(),
                    b: u8::try_from(c & 0xff).unwrap(),
                };

                let mut rvdev = rvdev.lock().unwrap();
                rvdev.send_led_map(&*led_map).unwrap();
                // thread::sleep(Duration::from_millis(20));
            }

            Err(e) => {
                error!("Could not lock a data structure. {}", e);
            }
        }
    }

    /// Set all leds at once
    /// this function is intended to be used from within lua scripts
    pub(crate) fn set_color_map(rvdev: &Arc<Mutex<RvDeviceState>>, map: &[u32]) {
        let mut led_map = [RGB { r: 0, g: 0, b: 0 }; NUM_KEYS];

        let mut i = 0;
        loop {
            led_map[i] = RGB {
                r: u8::try_from((map[i] >> 16) & 0xff).unwrap(),
                g: u8::try_from((map[i] >> 8) & 0xff).unwrap(),
                b: u8::try_from(map[i] & 0xff).unwrap(),
            };

            i += 1;
            if i >= NUM_KEYS - 1 {
                break;
            }
        }

        let mut rvdev = rvdev.lock().unwrap();
        rvdev.send_led_map(&led_map).unwrap();
        // thread::sleep(Duration::from_millis(20));
    }
}

/// Loads and runs a lua script
/// Initializes a lua environment, loads a script and executes it
pub fn run_scripts(rvdevice: RvDeviceState, rx: &Receiver<Message>) -> Result<()> {
    let lua = Lua::new();

    let script = fs::read_to_string("src/scripts/script").unwrap();

    lua.context(|lua_ctx| {
        register_support_globals(lua_ctx, &rvdevice)?;
        register_support_funcs(lua_ctx, rvdevice)?;

        // start execution of the Lua script
        lua_ctx.load(&script).eval::<()>()?;

        'EVENT_LOOP: loop {
            if let Ok(msg) = rx.recv() {
                match msg {
                    Message::Startup => {
                        if let Ok(handler) = lua_ctx.globals().get::<_, Function>("on_startup") {
                            handler.call::<_, ()>(()).or_else(|e| {
                                error!("{}", e);
                                Ok(())
                            })?;
                        }
                    }

                    Message::Quit(param) => {
                        if let Ok(handler) = lua_ctx.globals().get::<_, Function>("on_quit") {
                            handler.call::<_, ()>(param).or_else(|e| {
                                error!("{}", e);
                                Ok(())
                            })?;
                        }
                    }

                    Message::Tick(param) => {
                        if let Ok(handler) = lua_ctx.globals().get::<_, Function>("on_tick") {
                            handler.call::<_, ()>(param).or_else(|e| {
                                error!("{}", e);
                                Ok(())
                            })?;
                        }
                    }

                    Message::KeyDown(param) => {
                        if let Ok(handler) = lua_ctx.globals().get::<_, Function>("on_key_down") {
                            handler.call::<_, ()>(param).or_else(|e| {
                                error!("{}", e);
                                Ok(())
                            })?;
                        }
                    }

                    _ => break 'EVENT_LOOP,
                }
            }
        }

        Ok(())
    })?;

    Ok(())
}

fn register_support_globals(lua_ctx: Context, _rvdevice: &RvDeviceState) -> Result<()> {
    let globals = lua_ctx.globals();

    lua_ctx
        .load("package.path = package.path .. ';src/scripts/lib/?.lua'")
        .exec()
        .unwrap();

    let mut config: HashMap<&str, &str> = HashMap::new();
    config.insert("daemon_name", "eruption");
    config.insert("daemon_version", "0.0.1");

    globals.set("config", config)?;

    Ok(())
}

fn register_support_funcs(lua_ctx: Context, rvdevice: RvDeviceState) -> Result<()> {
    let rvdevid = rvdevice.get_dev_id();
    let rvdev = Arc::new(Mutex::new(rvdevice));

    let globals = lua_ctx.globals();

    // logging
    let trace = lua_ctx.create_function(|_, msg: String| {
        callbacks::log_trace(&msg);
        Ok(())
    })?;
    globals.set("trace", trace)?;

    let debug = lua_ctx.create_function(|_, msg: String| {
        callbacks::log_debug(&msg);
        Ok(())
    })?;
    globals.set("debug", debug)?;

    let info = lua_ctx.create_function(|_, msg: String| {
        callbacks::log_info(&msg);
        Ok(())
    })?;
    globals.set("info", info)?;

    let warn = lua_ctx.create_function(|_, msg: String| {
        callbacks::log_warn(&msg);
        Ok(())
    })?;
    globals.set("warn", warn)?;

    let error = lua_ctx.create_function(|_, msg: String| {
        callbacks::log_error(&msg);
        Ok(())
    })?;
    globals.set("error", error)?;

    let delay = lua_ctx.create_function(|_, millis: u64| {
        callbacks::delay(millis);
        Ok(())
    })?;
    globals.set("delay", delay)?;

    // math library
    let max = lua_ctx.create_function(|_, (f1, f2): (f64, f64)| Ok(f1.max(f2)))?;
    globals.set("max", max)?;

    let min = lua_ctx.create_function(|_, (f1, f2): (f64, f64)| Ok(f1.min(f2)))?;
    globals.set("min", min)?;

    let abs = lua_ctx.create_function(|_, f: f64| Ok(f.abs()))?;
    globals.set("abs", abs)?;

    let sin = lua_ctx.create_function(|_, a: f64| Ok(a.sin()))?;
    globals.set("sin", sin)?;

    let pow = lua_ctx.create_function(|_, (val, p): (f64, f64)| Ok(val.powf(p)))?;
    globals.set("pow", pow)?;

    // device related
    let get_num_keys = lua_ctx.create_function(move |_, ()| Ok(callbacks::get_num_keys()))?;
    globals.set("get_num_keys", get_num_keys)?;

    let rvdevid_tmp = rvdevid.clone();
    let get_key_color = lua_ctx
        .create_function(move |_, idx: usize| Ok(callbacks::get_key_color(&rvdevid_tmp, idx)))?;
    globals.set("get_key_color", get_key_color)?;

    let rvdev_tmp = rvdev.clone();
    let set_key_color = lua_ctx.create_function(move |_, (idx, c): (usize, u32)| {
        callbacks::set_key_color(&rvdev_tmp, idx, c);
        Ok(())
    })?;
    globals.set("set_key_color", set_key_color)?;

    let rvdev_tmp = rvdev.clone();
    let set_color_map = lua_ctx.create_function(move |_, map: (Vec<u32>)| {
        callbacks::set_color_map(&rvdev_tmp, &map);
        Ok(())
    })?;
    globals.set("set_color_map", set_color_map)?;


    // now register Lua functions supplied by eruption plugins
    let plugin_manager = plugin_manager::PLUGIN_MANAGER.read().unwrap();
    let plugins = plugin_manager.get_plugins();

    for plugin in plugins.iter() {
        plugin.register_lua_funcs(lua_ctx).unwrap();
    }

    Ok(())
}
