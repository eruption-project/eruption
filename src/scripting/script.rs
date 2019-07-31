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

use failure::Fail;
use lazy_static::lazy_static;
use log::*;
use rand::Rng;
use rlua::{Context, Function, Lua};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::mpsc::Receiver;
use std::sync::{Arc, Mutex, RwLock};
use std::vec::Vec;

use crate::plugin_manager;
use crate::rvdevice::{RvDeviceState, NUM_KEYS, RGB};
use crate::scripting::manifest::{ConfigParam, Manifest};

pub enum Message {
    // Startup, // Not passed via message but invoked directly
    Quit(u32),
    Tick(u32),
    KeyDown(u8),

    LoadScript(PathBuf),
}

lazy_static! {
    /// Global LED state of the managed device
    pub static ref LED_MAP: Arc<RwLock<Vec<RGB>>> = Arc::new(RwLock::new(vec![RGB {
        r: 0x00,
        g: 0x00,
        b: 0x00,
    }; NUM_KEYS]));
}

pub type Result<T> = std::result::Result<T, ScriptingError>;

#[derive(Debug, Fail)]
pub enum ScriptingError {
    #[fail(display = "Could not read script file")]
    OpenError {},

    #[fail(display = "Lua errors present")]
    LuaError {},

    #[fail(display = "Invalid or inaccessible manifest file")]
    InaccessibleManifest {},
    // #[fail(display = "Unknown error: {}", description)]
    // UnknownError { description: String },
}

/// These functions are intended to be used from within lua scripts
mod callbacks {
    use log::*;
    use std::convert::TryFrom;
    use std::sync::{Arc, Mutex};
    use std::thread;
    use std::time::Duration;

    use super::LED_MAP;

    use crate::rvdevice::{RvDeviceState, NUM_KEYS, RGB};

    /// Log a message with severity level `trace`.
    pub(crate) fn log_trace(x: &str) {
        trace!("{}", x);
    }

    /// Log a message with severity level `debug`.
    pub(crate) fn log_debug(x: &str) {
        debug!("{}", x);
    }

    /// Log a message with severity level `info`.
    pub(crate) fn log_info(x: &str) {
        info!("{}", x);
    }

    /// Log a message with severity level `warn`.
    pub(crate) fn log_warn(x: &str) {
        warn!("{}", x);
    }

    /// Log a message with severity level `error`.
    pub(crate) fn log_error(x: &str) {
        error!("{}", x);
    }

    /// Delay execution of the lua script by `millis` milliseconds.
    pub(crate) fn delay(millis: u64) {
        thread::sleep(Duration::from_millis(millis));
    }

    /// Get RGB components of a 32 bits color value.
    pub(crate) fn color_to_rgb(c: u32) -> (u8, u8, u8) {
        let r = u8::try_from((c >> 16) & 0xff).unwrap();
        let g = u8::try_from((c >> 8) & 0xff).unwrap();
        let b = u8::try_from(c & 0xff).unwrap();

        (r, g, b)
    }

    /// Convert RGB components to a 32 bits color value.
    pub(crate) fn rgb_to_color(r: u8, g: u8, b: u8) -> u32 {
        (u32::from(r) << 16) + (u32::from(g) << 8) + u32::from(b)
    }

    /// Generate a linear RGB color gradient from start to dest color,
    /// where p must lie in the range from 0..1.
    pub fn linear_gradient(start: u32, dest: u32, p: f64) -> u32 {
        let scr: f64 = f64::from((start >> 16) & 0xff);
        let scg: f64 = f64::from((start >> 8) & 0xff);
        let scb: f64 = f64::from((start) & 0xff);

        let dcr: f64 = f64::from((dest >> 16) & 0xff);
        let dcg: f64 = f64::from((dest >> 8) & 0xff);
        let dcb: f64 = f64::from((dest) & 0xff);

        let r: f64 = (scr as f64) + (((dcr - scr) as f64) * p);
        let g: f64 = (scg as f64) + (((dcg - scg) as f64) * p);
        let b: f64 = (scb as f64) + (((dcb - scb) as f64) * p);

        rgb_to_color(r.round() as u8, g.round() as u8, b.round() as u8)
    }

    /// Get the number of keys of the managed device.
    pub(crate) fn get_num_keys() -> usize {
        NUM_KEYS
    }

    /// Get the current color of the key `idx`.
    pub(crate) fn get_key_color(rvdevid: &str, idx: usize) -> u32 {
        error!("{}: {}", rvdevid, idx);
        0
    }

    /// Set the color of the key `idx` to `c`.
    pub(crate) fn set_key_color(rvdev: &Arc<Mutex<RvDeviceState>>, idx: usize, c: u32) {
        match LED_MAP.write() {
            Ok(mut led_map) => {
                led_map[idx] = RGB {
                    r: u8::try_from((c >> 16) & 0xff).unwrap(),
                    g: u8::try_from((c >> 8) & 0xff).unwrap(),
                    b: u8::try_from(c & 0xff).unwrap(),
                };

                let mut rvdev = rvdev.lock().unwrap_or_else(|e| {
                    error!("Could not lock a shared data structure: {}", e);
                    panic!();
                });
                rvdev.send_led_map(&*led_map).unwrap_or_else(|e| {
                    error!("Could not send the LED map to the keyboard: {}", e)
                });
                thread::sleep(Duration::from_millis(
                    crate::constants::DEVICE_SETTLE_MILLIS,
                ));
            }

            Err(e) => {
                error!("Could not lock a shared data structure. {}", e);
            }
        }
    }

    /// Set all leds at once.
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

        let mut rvdev = rvdev.lock().unwrap_or_else(|e| {
            error!("Could not lock a shared data structure: {}", e);
            panic!();
        });
        rvdev
            .send_led_map(&led_map)
            .unwrap_or_else(|e| error!("Could not send the LED map to the keyboard: {}", e));
        thread::sleep(Duration::from_millis(
            crate::constants::DEVICE_SETTLE_MILLIS,
        ));
    }
}

/// Action requests for `run_script`
pub enum RunScriptResult {
    /// Currently running interpreter will be shut down, to execute another Lua script
    ReExecuteOtherScript(PathBuf),
}

/// Loads and runs a lua script.
/// Initializes a lua environment, loads the script and executes it
pub fn run_script(
    file: PathBuf,
    rvdevice: RvDeviceState,
    rx: &Receiver<Message>,
) -> Result<RunScriptResult> {
    match fs::read_to_string(file.clone()) {
        Ok(script) => {
            let lua = Lua::new();

            let manifest = Manifest::from(&file);
            if manifest.is_err() {
                error!(
                    "Could not parse manifest file for script '{}': {}",
                    file.to_string_lossy(),
                    manifest.clone().unwrap_err()
                );

                return Err(ScriptingError::InaccessibleManifest {});
            }

            let rvdevice = rvdevice.clone();

            let result: rlua::Result<RunScriptResult> = lua.context::<_, _>(|lua_ctx| {
                register_support_globals(lua_ctx, &rvdevice)?;
                register_support_funcs(lua_ctx, &rvdevice)?;
                register_script_config(lua_ctx, &manifest.unwrap())?;

                // start execution of the Lua script
                lua_ctx.load(&script).eval::<()>()?;

                // call startup event handler, iff present
                if let Ok(handler) = lua_ctx.globals().get::<_, Function>("on_startup") {
                    handler.call::<_, ()>(()).or_else(|e| {
                        error!("Lua error: {}", e);
                        Err(e)
                    })?;
                }

                loop {
                    if let Ok(msg) = rx.recv() {
                        match msg {
                            Message::Quit(param) => {
                                if let Ok(handler) = lua_ctx.globals().get::<_, Function>("on_quit")
                                {
                                    handler.call::<_, ()>(param).or_else(|e| {
                                        error!("Lua error: {}", e);
                                        Err(e)
                                    })?;
                                }
                            }

                            Message::Tick(param) => {
                                if let Ok(handler) = lua_ctx.globals().get::<_, Function>("on_tick")
                                {
                                    handler.call::<_, ()>(param).or_else(|e| {
                                        error!("Lua error: {}", e);
                                        Err(e)
                                    })?;
                                }
                            }

                            Message::KeyDown(param) => {
                                if let Ok(handler) =
                                    lua_ctx.globals().get::<_, Function>("on_key_down")
                                {
                                    handler.call::<_, ()>(param).or_else(|e| {
                                        error!("Lua error: {}", e);
                                        Err(e)
                                    })?;
                                }
                            }

                            Message::LoadScript(script_path) => {
                                return Ok(RunScriptResult::ReExecuteOtherScript(script_path))
                            }
                        }
                    }
                }
            });

            match result {
                Ok(action) => Ok(action),

                Err(_e) => Err(ScriptingError::LuaError {}),
            }
        }

        Err(_e) => Err(ScriptingError::OpenError {}),
    }
}

fn register_support_globals(lua_ctx: Context, _rvdevice: &RvDeviceState) -> rlua::Result<()> {
    let globals = lua_ctx.globals();

    lua_ctx
        .load("package.path = package.path .. ';src/scripts/lib/?.lua'")
        .exec()
        .unwrap();

    let mut config: HashMap<&str, &str> = HashMap::new();
    config.insert("daemon_name", "eruption");
    config.insert("daemon_version", "0.0.8");

    globals.set("config", config)?;

    Ok(())
}

fn register_support_funcs(lua_ctx: Context, rvdevice: &RvDeviceState) -> rlua::Result<()> {
    let rvdevid = rvdevice.get_dev_id();
    let rvdev = Arc::new(Mutex::new(rvdevice.clone()));

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

    let clamp =
        lua_ctx.create_function(|_, (val, f1, f2): (f64, f64, f64)| {
            let mut val = val;
            if val < f1 { val = f1; }
            if val > f2 { val = f2; }
            
            Ok(val)
        })?;
    globals.set("clamp", clamp)?;

    let abs = lua_ctx.create_function(|_, f: f64| Ok(f.abs()))?;
    globals.set("abs", abs)?;

    let sin = lua_ctx.create_function(|_, a: f64| Ok(a.sin()))?;
    globals.set("sin", sin)?;

    let pow = lua_ctx.create_function(|_, (val, p): (f64, f64)| Ok(val.powf(p)))?;
    globals.set("pow", pow)?;

    let sqrt = lua_ctx.create_function(|_, f: (f64)| Ok(f.sqrt()))?;
    globals.set("sqrt", sqrt)?;

    let rand =
        lua_ctx.create_function(|_, (l, h): (u64, u64)| Ok(rand::thread_rng().gen_range(l, h)))?;
    globals.set("rand", rand)?;

    let trunc = lua_ctx.create_function(|_, f: (f64)| Ok(f.trunc() as i64))?;
    globals.set("trunc", trunc)?;

    // color handling
    let color_to_rgb = lua_ctx.create_function(|_, c: (u32)| Ok(callbacks::color_to_rgb(c)))?;
    globals.set("color_to_rgb", color_to_rgb)?;

    let rgb_to_color = lua_ctx
        .create_function(|_, (r, g, b): (u8, u8, u8)| Ok(callbacks::rgb_to_color(r, g, b)))?;
    globals.set("rgb_to_color", rgb_to_color)?;

    let linear_gradient = lua_ctx.create_function(|_, (start, dest, p): (u32, u32, f64)| {
        Ok(callbacks::linear_gradient(start, dest, p))
    })?;
    globals.set("linear_gradient", linear_gradient)?;

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

    // finally, register Lua functions supplied by eruption plugins
    let plugin_manager = plugin_manager::PLUGIN_MANAGER.read().unwrap();
    let plugins = plugin_manager.get_plugins();

    for plugin in plugins.iter() {
        plugin.register_lua_funcs(lua_ctx).unwrap();
    }

    Ok(())
}

fn register_script_config(lua_ctx: Context, manifest: &Manifest) -> rlua::Result<()> {
    let globals = lua_ctx.globals();

    for param in manifest.config.iter() {
        debug!("Applying parameter {:?}", param);

        match param {
            ConfigParam::Int { name, default, .. } => {
                globals.raw_set::<&str, i64>(name, *default)?;
            }

            ConfigParam::Float { name, default, .. } => {
                globals.raw_set::<&str, f64>(name, *default)?;
            }

            ConfigParam::Bool { name, default, .. } => {
                globals.raw_set::<&str, bool>(name, *default)?;
            }

            ConfigParam::String { name, default, .. } => {
                globals.raw_set::<&str, &str>(name, default)?;
            }

            ConfigParam::Color { name, default, .. } => {
                globals.raw_set::<&str, u32>(name, *default)?;
            }
        }
    }

    Ok(())
}
