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

use flume::Receiver;
use lazy_static::lazy_static;
use log::*;
use mlua::prelude::*;
use mlua::Function;
use parking_lot::RwLock;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::vec::Vec;

use crate::{
    constants, hwdevices::KeyboardHidEvent, hwdevices::MouseHidEvent, hwdevices::RGBA, profiles,
    scripting::callbacks, scripting::constants::*, scripting::manifest,
};

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, Clone)]
pub enum Message {
    // Startup, // Not passed via message but invoked directly
    Quit(u32),
    Tick(u32),

    // Keyboard events
    KeyDown(u8),
    KeyUp(u8),

    // HID events
    KeyboardHidEvent(KeyboardHidEvent),
    MouseHidEvent(MouseHidEvent),

    // Mouse events
    MouseButtonDown(u8),
    MouseButtonUp(u8),
    MouseMove(i32, i32, i32),
    MouseWheelEvent(u8),

    //LoadScript(PathBuf),
    // Abort,
    Unload,

    /// blend LOCAL_LED_MAP with LED_MAP ("realize" the color map)
    RealizeColorMap,

    SetParameter {
        parameter_value: ParameterValue,
    },
}

lazy_static! {
    /// Global LED map, the "canvas"
    pub static ref LED_MAP: Arc<RwLock<Vec<RGBA>>> = Arc::new(RwLock::new(vec![RGBA {
        r: 0x00,
        g: 0x00,
        b: 0x00,
        a: 0x00,
    }; constants::CANVAS_SIZE]));

    /// The last successfully rendered canvas
    pub static ref LAST_RENDERED_LED_MAP: Arc<RwLock<Vec<RGBA>>> = Arc::new(RwLock::new(vec![RGBA {
        r: 0x00,
        g: 0x00,
        b: 0x00,
        a: 0x00,
    }; constants::CANVAS_SIZE]));

    /// Frame generation counter, used to detect if we need to submit the LED_MAP to the hardware
    pub static ref FRAME_GENERATION_COUNTER: AtomicUsize = AtomicUsize::new(0);
}

thread_local! {
    /// LED color map to be realized on the next render frame
    pub static LOCAL_LED_MAP: RefCell<Vec<RGBA>> = RefCell::new(vec![RGBA {
        r: 0x00,
        g: 0x00,
        b: 0x00,
        a: 0x00,
    }; constants::CANVAS_SIZE]);

    /// True, if LED color map was modified at least once in this thread
    pub static LOCAL_LED_MAP_MODIFIED: RefCell<bool> = RefCell::new(false);

    /// Vec of allocated gradient objects
    pub static ALLOCATED_GRADIENTS: RefCell<HashMap<usize, colorgrad::Gradient>> = RefCell::new(HashMap::new());
}

#[derive(Debug, thiserror::Error)]
pub enum ScriptingError {
    #[error("Could not read script file")]
    OpenError {},

    #[error("Invalid value")]
    ValueError {},

    #[error("Error invoking handler function")]
    HandlerError {},
}

#[derive(Debug)]
pub struct UnknownError {}

impl std::error::Error for UnknownError {}

impl fmt::Display for UnknownError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unknown error occurred")
    }
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct ParameterValue {
    pub name: String,
    pub value: TypedValue,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub enum TypedValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Color(u32),
}

impl ParameterValue {
    fn get_value_string(&self) -> String {
        match &self.value {
            TypedValue::Int(value) => format!("{}", value),
            TypedValue::Float(value) => format!("{}", value),
            TypedValue::Bool(value) => format!("{}", value),
            TypedValue::String(value) => format!("{}", value),
            TypedValue::Color(value) => format!("#{:06x}", value),
        }
    }
}

pub trait ToParameterValue {
    fn to_parameter_value(&self) -> ParameterValue;
}

impl ToParameterValue for profiles::ConfigParam {
    fn to_parameter_value(&self) -> ParameterValue {
        match &self {
            profiles::ConfigParam::Int { name, value, .. } => ParameterValue {
                name: name.to_string(),
                value: TypedValue::Int(*value),
            },
            profiles::ConfigParam::Float { name, value, .. } => ParameterValue {
                name: name.to_string(),
                value: TypedValue::Float(*value),
            },
            profiles::ConfigParam::Bool { name, value, .. } => ParameterValue {
                name: name.to_string(),
                value: TypedValue::Bool(*value),
            },
            profiles::ConfigParam::String { name, value, .. } => ParameterValue {
                name: name.to_string(),
                value: TypedValue::String(value.to_string()),
            },
            profiles::ConfigParam::Color { name, value, .. } => ParameterValue {
                name: name.to_string(),
                value: TypedValue::Color(*value),
            },
        }
    }
}

impl ToParameterValue for manifest::ConfigParam {
    fn to_parameter_value(&self) -> ParameterValue {
        match &self {
            manifest::ConfigParam::Int { name, default, .. } => ParameterValue {
                name: name.to_string(),
                value: TypedValue::Int(*default),
            },
            manifest::ConfigParam::Float { name, default, .. } => ParameterValue {
                name: name.to_string(),
                value: TypedValue::Float(*default),
            },
            manifest::ConfigParam::Bool { name, default, .. } => ParameterValue {
                name: name.to_string(),
                value: TypedValue::Bool(*default),
            },
            manifest::ConfigParam::String { name, default, .. } => ParameterValue {
                name: name.to_string(),
                value: TypedValue::String(default.to_string()),
            },
            manifest::ConfigParam::Color { name, default, .. } => ParameterValue {
                name: name.to_string(),
                value: TypedValue::Color(*default),
            },
        }
    }
}

/// Action requests for `run_script`
pub enum RunScriptResult {
    /// Script terminated gracefully
    TerminatedGracefully,

    /// Error abort
    TerminatedWithErrors,

    // Currently running interpreter will be shut down and restarted
    RestartScript,
}

/// Used to control the message processing loop of `run_script`
pub enum RunningScriptResult {
    Continue,
    RestartScript,
    TerminateGracefully,
    TerminateWithErrors,
}

struct RunningScriptCallHelper<'lua> {
    file_name: String,
    lua_ctx: &'lua Lua,
    lua_functions: HashMap<String, Option<Function<'lua>>>,
    skip_on_tick: bool,
    skip_on_mouse_move: bool,
    skip_on_hid_event: bool,
}

enum RunningScriptCallHelperResult {
    Successful,
    NoHandler,
}

impl<'lua> RunningScriptCallHelper<'lua> {
    fn new(file: &Path, lua_ctx: &'lua Lua) -> RunningScriptCallHelper<'lua> {
        RunningScriptCallHelper {
            file_name: file.to_string_lossy().to_string(),
            lua_ctx: lua_ctx,
            lua_functions: HashMap::new(),
            skip_on_tick: false,
            skip_on_mouse_move: false,
            skip_on_hid_event: false,
        }
    }

    fn call<Args: ToLuaMulti<'lua>>(
        &mut self,
        function_name: &str,
        args: Args,
    ) -> Result<RunningScriptCallHelperResult> {
        match self.find_handler(function_name) {
            Some(handler) => match handler.call::<Args, ()>(args) {
                Ok(()) => Ok(RunningScriptCallHelperResult::Successful),
                Err(e) => {
                    let error = e.source().unwrap_or(&UnknownError {});
                    error!("Lua error in file {}: {}\n\t{:?}", self.file_name, e, error);
                    Err(ScriptingError::HandlerError {}.into())
                }
            },
            None => Ok(RunningScriptCallHelperResult::NoHandler),
        }
    }

    fn verify_handler_exists(&mut self, function_name: &str) -> bool {
        self.find_handler(function_name).is_some()
    }

    fn find_handler(&mut self, function_name: &str) -> &Option<Function<'lua>> {
        // Caching the handler functions like this removes the script's ability to dynamically reassign global
        // functions (e.g., `_G['on_tick'] = ...`). But since that's an insane thing to do we'll go ahead and cache.
        self.lua_functions
            .entry(function_name.to_string())
            .or_insert_with(|| {
                let func = self
                    .lua_ctx
                    .globals()
                    .get::<_, Function<'lua>>(function_name)
                    .ok();
                if func.is_none() {
                    match function_name {
                        // Special optimizations since they happen frequently.
                        FUNCTION_ON_MOUSE_MOVE => self.skip_on_mouse_move = true,
                        FUNCTION_ON_TICK => self.skip_on_tick = true,
                        FUNCTION_ON_HID_EVENT => self.skip_on_hid_event = true,
                        _ => (),
                    }
                }
                func
            })
    }
}

/// Loads and runs a lua script.
/// Initializes a lua environment, loads the script and executes it
pub fn run_script(
    script_file: &Path,
    parameter_values: &mut HashMap<String, ParameterValue>,
    rx: &Receiver<Message>,
) -> Result<RunScriptResult> {
    match fs::read_to_string(script_file) {
        Ok(script) => {
            let lua_ctx =
                unsafe { Lua::unsafe_new_with(mlua::StdLib::ALL, mlua::LuaOptions::default()) };

            // Prepare the Lua environment and eval the script
            let prepared = register_support_globals(&lua_ctx)
                .and_then(|()| register_support_funcs(&lua_ctx))
                .and_then(|()| register_script_config(&lua_ctx, parameter_values.values()))
                .and_then(|()| lua_ctx.load(&script).eval::<()>());

            if let Err(e) = prepared {
                error!(
                    "Lua error in file {}: {}\n\t{:?}",
                    script_file.to_string_lossy(),
                    e,
                    e.source().unwrap_or(&UnknownError {})
                );
                return Ok(RunScriptResult::TerminatedWithErrors);
            }

            let mut call_helper = RunningScriptCallHelper::new(script_file, &lua_ctx);

            if call_helper.call(FUNCTION_ON_STARTUP, ()).is_err() {
                return Ok(RunScriptResult::TerminatedWithErrors);
            }

            loop {
                if let Ok(msg) = rx.recv() {
                    if let Message::SetParameter { parameter_value } = &msg {
                        // Save the new value for next time
                        parameter_values
                            .insert(parameter_value.name.clone(), parameter_value.clone());
                    }

                    match process_message(&mut call_helper, msg) {
                        Ok(RunningScriptResult::Continue) => (),
                        Ok(RunningScriptResult::TerminateGracefully) => {
                            return Ok(RunScriptResult::TerminatedGracefully)
                        }
                        Ok(RunningScriptResult::TerminateWithErrors) => {
                            return Ok(RunScriptResult::TerminatedWithErrors)
                        }
                        Ok(RunningScriptResult::RestartScript) => {
                            return Ok(RunScriptResult::RestartScript)
                        }
                        Err(e) => {
                            let error = e.source().unwrap_or(&UnknownError {});
                            error!(
                                "Unexpected lua error in file {}: {}\n\t{:?}",
                                call_helper.file_name, e, error
                            );
                            return Ok(RunScriptResult::TerminatedWithErrors);
                        }
                    }
                }
            }
        }

        Err(_e) => Err(ScriptingError::OpenError {}.into()),
    }
}

fn register_support_globals(lua_ctx: &Lua) -> mlua::Result<()> {
    let globals = lua_ctx.globals();

    let config = crate::CONFIG.lock();
    let script_dirs = config
        .as_ref()
        .unwrap()
        .get::<Vec<String>>("global.script_dirs")
        .unwrap_or_else(|_| vec![constants::DEFAULT_SCRIPT_DIR.to_string()]);

    let mut path_spec = String::from("package.path = package.path .. '");

    for script_dir in script_dirs {
        path_spec += &format!(";{0}/lib/?;{0}/lib/?.lua", &script_dir);
    }

    path_spec += "'";

    lua_ctx.load(&path_spec).exec().unwrap();

    let mut config: HashMap<&str, &str> = HashMap::new();
    config.insert("daemon_name", "eruption");
    config.insert("daemon_version", env!("CARGO_PKG_VERSION"));
    config.insert("api_level", env!("CARGO_PKG_VERSION"));

    globals.set("config", config)?;

    Ok(())
}

fn register_support_funcs(lua_ctx: &Lua) -> mlua::Result<()> {
    callbacks::register_support_funcs(lua_ctx)
}

fn register_script_config<'a, I>(lua_ctx: &Lua, parameter_values: I) -> mlua::Result<()>
where
    I: Iterator<Item = &'a ParameterValue>,
{
    for parameter_value in parameter_values {
        debug!("Applying parameter {:?}", parameter_value);
        set_config_param(lua_ctx, parameter_value)?;
    }

    Ok(())
}

fn set_config_param(lua_ctx: &Lua, param: &ParameterValue) -> mlua::Result<()> {
    let globals = lua_ctx.globals();
    match &param.value {
        TypedValue::Int(value) => globals.raw_set::<&str, i64>(&param.name, *value),
        TypedValue::Float(value) => globals.raw_set::<&str, f64>(&param.name, *value),
        TypedValue::Bool(value) => globals.raw_set::<&str, bool>(&param.name, *value),
        TypedValue::String(value) => globals.raw_set::<&str, &str>(&param.name, value),
        TypedValue::Color(value) => globals.raw_set::<&str, u32>(&param.name, *value),
    }
}

fn process_message(
    call_helper: &mut RunningScriptCallHelper,
    msg: Message,
) -> Result<RunningScriptResult> {
    match msg {
        Message::Quit(param) => on_quit(call_helper, param),
        Message::Tick(param) => on_tick(call_helper, param),
        Message::RealizeColorMap => realize_color_map(),
        Message::KeyDown(param) => on_key_down(call_helper, param),
        Message::KeyUp(param) => on_key_up(call_helper, param),
        Message::KeyboardHidEvent(param) => on_keyboard_hid_event(call_helper, param),
        Message::MouseHidEvent(param) => on_mouse_hid_event(call_helper, param),
        Message::MouseButtonDown(param) => on_mouse_button_down(call_helper, param),
        Message::MouseButtonUp(param) => on_mouse_button_up(call_helper, param),
        Message::MouseMove(rel_x, rel_y, rel_z) => on_mouse_move(call_helper, rel_x, rel_y, rel_z),
        Message::MouseWheelEvent(param) => on_mouse_wheel_event(call_helper, param),
        Message::Unload => on_unload(call_helper),
        Message::SetParameter { parameter_value } => on_set_parameter(call_helper, parameter_value),
    }
}

fn continue_if_ok(
    call_result: Result<RunningScriptCallHelperResult>,
) -> Result<RunningScriptResult> {
    match call_result {
        Ok(_r) => Ok(RunningScriptResult::Continue),
        Err(_e) => Ok(RunningScriptResult::TerminateWithErrors),
    }
}

fn on_quit(call_helper: &mut RunningScriptCallHelper, param: u32) -> Result<RunningScriptResult> {
    let called = call_helper.call(FUNCTION_ON_QUIT, param);

    let mut val = crate::UPCALL_COMPLETED_ON_QUIT.0.lock();
    *val = val.saturating_sub(1);

    crate::UPCALL_COMPLETED_ON_QUIT.1.notify_all();

    continue_if_ok(called)
}

fn on_tick(call_helper: &mut RunningScriptCallHelper, param: u32) -> Result<RunningScriptResult> {
    let called = if call_helper.skip_on_tick {
        Ok(RunningScriptCallHelperResult::NoHandler)
    } else {
        call_helper.call(FUNCTION_ON_TICK, param)
    };

    continue_if_ok(called)
}

fn realize_color_map() -> Result<RunningScriptResult> {
    if LOCAL_LED_MAP_MODIFIED.with(|f| *f.borrow()) {
        LOCAL_LED_MAP.with(|foreground| {
            let brightness = crate::BRIGHTNESS.load(Ordering::SeqCst);

            let fader = crate::BRIGHTNESS_FADER.load(Ordering::SeqCst);
            let fader_base = crate::BRIGHTNESS_FADER_BASE.load(Ordering::SeqCst);

            let brightness = if fader_base > 0 && fader > 0 {
                (1.0 - (fader as f32 / fader_base as f32)) * brightness as f32
            } else {
                brightness as f32
            };

            for chunks in LED_MAP.write().chunks_exact_mut(constants::CANVAS_SIZE) {
                for (idx, background) in chunks.iter_mut().enumerate() {
                    let bg = &background;
                    let fg = foreground.borrow()[idx];

                    #[rustfmt::skip]
                    let color = RGBA {
                        r: ((((fg.a as f32) * fg.r as f32 + (255 - fg.a) as f32 * bg.r as f32).floor() * brightness as f32 / 100.0) as u32 >> 8) as u8,
                        g: ((((fg.a as f32) * fg.g as f32 + (255 - fg.a) as f32 * bg.g as f32).floor() * brightness as f32 / 100.0) as u32 >> 8) as u8,
                        b: ((((fg.a as f32) * fg.b as f32 + (255 - fg.a) as f32 * bg.b as f32).floor() * brightness as f32 / 100.0) as u32 >> 8) as u8,
                        a: fg.a as u8,
                    };

                    *background = color;
                }
            }
        });
    }

    // signal readiness / notify the main thread that we are done
    let val = { *crate::COLOR_MAPS_READY_CONDITION.0.lock() };

    let val = val.checked_sub(1).unwrap_or_else(|| {
        warn!("Incorrect state in locking code detected");
        0
    });

    *crate::COLOR_MAPS_READY_CONDITION.0.lock() = val;

    crate::COLOR_MAPS_READY_CONDITION.1.notify_one();

    Ok(RunningScriptResult::Continue)
}

fn on_key_down(
    call_helper: &mut RunningScriptCallHelper,
    param: u8,
) -> Result<RunningScriptResult> {
    let called = call_helper.call(FUNCTION_ON_KEY_DOWN, param);

    let mut val = crate::UPCALL_COMPLETED_ON_KEY_DOWN.0.lock();
    *val = val.saturating_sub(1);

    crate::UPCALL_COMPLETED_ON_KEY_DOWN.1.notify_all();

    continue_if_ok(called)
}

fn on_key_up(call_helper: &mut RunningScriptCallHelper, param: u8) -> Result<RunningScriptResult> {
    let called = call_helper.call(FUNCTION_ON_KEY_UP, param);

    let mut val = crate::UPCALL_COMPLETED_ON_KEY_UP.0.lock();
    *val = val.saturating_sub(1);

    crate::UPCALL_COMPLETED_ON_KEY_UP.1.notify_all();

    continue_if_ok(called)
}

fn on_keyboard_hid_event(
    call_helper: &mut RunningScriptCallHelper,
    param: KeyboardHidEvent,
) -> Result<RunningScriptResult> {
    let called = if call_helper.skip_on_hid_event {
        // (Don't read the keyboard state if the script doesn't use it.)
        Ok(RunningScriptCallHelperResult::NoHandler)
    } else {
        let call_args: (u8, u32) = match param {
            KeyboardHidEvent::KeyUp { code } => (
                crate::KEYBOARD_DEVICES.read()[0]
                    .read()
                    .hid_event_code_to_report(&code),
                1,
            ),
            KeyboardHidEvent::KeyDown { code } => (
                crate::KEYBOARD_DEVICES.read()[0]
                    .read()
                    .hid_event_code_to_report(&code),
                2,
            ),
            KeyboardHidEvent::MuteDown => (1, 3),
            KeyboardHidEvent::MuteUp => (0, 3),
            KeyboardHidEvent::VolumeDown => (1, 4),
            KeyboardHidEvent::VolumeUp => (0, 4),
            KeyboardHidEvent::BrightnessDown => (1, 5),
            KeyboardHidEvent::BrightnessUp => (0, 5),
            KeyboardHidEvent::SetBrightness(val) => (val, 6),
            KeyboardHidEvent::NextSlot => (1, 7),
            KeyboardHidEvent::PreviousSlot => (0, 5),
            _ => (0, 0),
        };

        call_helper.call(FUNCTION_ON_HID_EVENT, call_args)
    };

    let mut val = crate::UPCALL_COMPLETED_ON_KEYBOARD_HID_EVENT.0.lock();
    *val = val.saturating_sub(1);

    crate::UPCALL_COMPLETED_ON_KEYBOARD_HID_EVENT.1.notify_all();

    continue_if_ok(called)
}

fn on_mouse_hid_event(
    call_helper: &mut RunningScriptCallHelper,
    param: MouseHidEvent,
) -> Result<RunningScriptResult> {
    let call_args: (u8, u32) = match param {
        MouseHidEvent::DpiChange(dpi_slot) => (dpi_slot, 1),
        MouseHidEvent::ButtonDown(index) => (index + 1, 2),
        MouseHidEvent::ButtonUp(index) => (index + 1, 3),
        _ => (0, 0),
    };
    let called = call_helper.call(FUNCTION_ON_MOUSE_HID_EVENT, call_args);

    let mut val = crate::UPCALL_COMPLETED_ON_MOUSE_HID_EVENT.0.lock();
    *val = val.saturating_sub(1);

    crate::UPCALL_COMPLETED_ON_MOUSE_HID_EVENT.1.notify_all();

    continue_if_ok(called)
}

fn on_mouse_button_down(
    call_helper: &mut RunningScriptCallHelper,
    param: u8,
) -> Result<RunningScriptResult> {
    let called = call_helper.call(FUNCTION_ON_MOUSE_BUTTON_DOWN, param);

    let mut val = crate::UPCALL_COMPLETED_ON_MOUSE_BUTTON_DOWN.0.lock();
    *val = val.saturating_sub(1);

    crate::UPCALL_COMPLETED_ON_MOUSE_BUTTON_DOWN.1.notify_all();

    continue_if_ok(called)
}

fn on_mouse_button_up(
    call_helper: &mut RunningScriptCallHelper,
    param: u8,
) -> Result<RunningScriptResult> {
    let called = call_helper.call(FUNCTION_ON_MOUSE_BUTTON_UP, param);

    let mut val = crate::UPCALL_COMPLETED_ON_MOUSE_BUTTON_UP.0.lock();
    *val = val.saturating_sub(1);

    crate::UPCALL_COMPLETED_ON_MOUSE_BUTTON_UP.1.notify_all();

    continue_if_ok(called)
}

fn on_mouse_move(
    call_helper: &mut RunningScriptCallHelper,
    rel_x: i32,
    rel_y: i32,
    rel_z: i32,
) -> Result<RunningScriptResult> {
    let called = if call_helper.skip_on_mouse_move {
        Ok(RunningScriptCallHelperResult::NoHandler)
    } else {
        call_helper.call(FUNCTION_ON_MOUSE_MOVE, (rel_x, rel_y, rel_z))
    };

    let mut val = crate::UPCALL_COMPLETED_ON_MOUSE_MOVE.0.lock();
    *val = val.saturating_sub(1);

    crate::UPCALL_COMPLETED_ON_MOUSE_MOVE.1.notify_all();

    continue_if_ok(called)
}

fn on_mouse_wheel_event(
    call_helper: &mut RunningScriptCallHelper,
    param: u8,
) -> Result<RunningScriptResult> {
    let called = call_helper.call(FUNCTION_ON_MOUSE_WHEEL, param);

    let mut val = crate::UPCALL_COMPLETED_ON_MOUSE_EVENT.0.lock();
    *val = val.saturating_sub(1);

    crate::UPCALL_COMPLETED_ON_MOUSE_EVENT.1.notify_all();

    continue_if_ok(called)
}

fn on_unload(call_helper: &mut RunningScriptCallHelper) -> Result<RunningScriptResult> {
    let called = call_helper.call(FUNCTION_ON_QUIT, ());
    match called {
        Ok(_) => {
            debug!("Lua script {} terminated gracefully", call_helper.file_name);
            Ok(RunningScriptResult::TerminateGracefully)
        }
        Err(_) => {
            error!(
                "Lua script {} terminated with errors",
                call_helper.file_name
            );
            Ok(RunningScriptResult::TerminateWithErrors)
        }
    }
}

fn on_set_parameter(
    call_helper: &mut RunningScriptCallHelper,
    parameter_value: ParameterValue,
) -> Result<RunningScriptResult> {
    if !call_helper.verify_handler_exists(FUNCTION_ON_APPLY_PARAMETER) {
        debug!(
            "Lua script {}: No {} function present.  Restarting script.",
            &call_helper.file_name, FUNCTION_ON_APPLY_PARAMETER,
        );

        // Before we restart, call on_quit to let the script know.
        // No need to decrement UPCALL_COMPLETED_ON_QUIT, since the message channel will still be active for the next Lua VM.
        let called_on_quit = call_helper.call(FUNCTION_ON_QUIT, 0);
        match called_on_quit {
            Ok(_r) => Ok(RunningScriptResult::RestartScript),
            Err(_e) => Ok(RunningScriptResult::TerminateWithErrors),
        }
    } else {
        let set = set_config_param(&call_helper.lua_ctx, &parameter_value);
        match set {
            Ok(_) => {
                debug!(
                    "Lua script {}: Successfully applied parameter",
                    &call_helper.file_name,
                );

                let call_args = (&*parameter_value.name, &*parameter_value.get_value_string());
                let called = call_helper.call(FUNCTION_ON_APPLY_PARAMETER, call_args);
                if let Ok(_) = called {
                    // (the handler must exist, as we already verified it before updating Lua's global table)
                    debug!(
                        "Lua script {}: Successfully called {}",
                        &call_helper.file_name, FUNCTION_ON_APPLY_PARAMETER,
                    );
                    Ok(RunningScriptResult::Continue)
                } else {
                    Ok(RunningScriptResult::TerminateWithErrors)
                }
            }
            Err(_) => Ok(RunningScriptResult::TerminateWithErrors),
        }
    }
}
