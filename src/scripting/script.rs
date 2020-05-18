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
use parking_lot::Mutex;
use rand::Rng;
use rlua::{Context, Function, Lua};
use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc::Receiver;
use std::sync::Arc;
use std::vec::Vec;

use crate::plugin_manager;
use crate::rvdevice::{RvDeviceState, NUM_KEYS, RGBA};
use crate::scripting::manifest::{ConfigParam, Manifest};

use crate::{ACTIVE_PROFILE, ACTIVE_SCRIPTS};

pub enum Message {
    // Startup, // Not passed via message but invoked directly
    Quit(u32),
    Tick(u32),
    KeyDown(u8),
    KeyUp(u8),

    //LoadScript(PathBuf),
    // Abort,
    Unload,

    /// blend LOCAL_LED_MAP with LED_MAP ("realize" the color map)
    RealizeColorMap,
}

lazy_static! {
    /// Global LED state of the managed device
    pub static ref LED_MAP: Arc<Mutex<Vec<RGBA>>> = Arc::new(Mutex::new(vec![RGBA {
        r: 0x00,
        g: 0x00,
        b: 0x00,
        a: 0x00,
    }; NUM_KEYS]));

    // Frame generation counter, used to detect if we need to submit the LED_MAP to the keyboard
    pub static ref FRAME_GENERATION_COUNTER: Arc<AtomicUsize> = Arc::new(AtomicUsize::new(0));
}

thread_local! {
    /// LED color map to be realized on the next render frame
    pub static LOCAL_LED_MAP: RefCell<Vec<RGBA>> = RefCell::new(vec![RGBA {
        r: 0x00,
        g: 0x00,
        b: 0x00,
        a: 0x00,
    }; NUM_KEYS]);
}

pub type Result<T> = std::result::Result<T, ScriptingError>;

#[derive(Debug, Fail)]
pub enum ScriptingError {
    #[fail(display = "Could not read script file")]
    OpenError {},

    #[fail(display = "Lua errors present")]
    LuaError { e: rlua::Error },

    #[fail(display = "Invalid or inaccessible manifest file")]
    InaccessibleManifest {},
    // #[fail(display = "Unknown error: {}", description)]
    // UnknownError { description: String },
}

/// These functions are intended to be used from within lua scripts
mod callbacks {
    use byteorder::{ByteOrder, LittleEndian};
    use log::*;
    use noise::NoiseFn;
    use palette::ConvertFrom;
    use palette::{Hsl, Srgb};
    use parking_lot::Mutex;
    use std::convert::TryFrom;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;
    use std::thread;
    use std::time::Duration;

    use super::{LED_MAP, LOCAL_LED_MAP};

    use crate::plugins::macros;
    use crate::rvdevice::{RvDeviceState, NUM_KEYS, RGBA};

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

    /// Delays the execution of the lua script by `millis` milliseconds.
    pub(crate) fn delay(millis: u64) {
        // TODO: This will totally block the Lua VM, so not very useful currently.
        thread::sleep(Duration::from_millis(millis));
    }

    /// Inject a key on the eruption virtual keyboard.
    pub(crate) fn inject_key(ev_key: u32, down: bool) {
        // calling inject_key(..) from Lua will drop the current input;
        // the original key event from the hardware keyboard will not be
        // mirrored on the virtual keyboard.
        macros::DROP_CURRENT_KEY.store(true, Ordering::SeqCst);

        macros::UINPUT_TX
            .lock()
            .as_ref()
            .unwrap()
            .send(macros::Message::InjectKey { key: ev_key, down })
            .unwrap();
    }

    /// Inject a key on the eruption virtual keyboard after sleeping for `millis` milliseconds.
    pub(crate) fn inject_key_with_delay(ev_key: u32, down: bool, millis: u64) {
        // calling inject_key(..) from Lua will drop the current input;
        // the original key event from the hardware keyboard will not be
        // mirrored on the virtual keyboard.
        macros::DROP_CURRENT_KEY.store(true, Ordering::SeqCst);

        thread::Builder::new().name("uinput/delayed".to_owned()).spawn(move || {
            thread::sleep(Duration::from_millis(millis));

            macros::UINPUT_TX
                .lock()
                .as_ref()
                .unwrap()
                .send(macros::Message::InjectKey { key: ev_key, down })
                .unwrap();
        }).unwrap();
    }

    /// Get RGB components of a 32 bits color value.
    pub(crate) fn color_to_rgb(c: u32) -> (u8, u8, u8) {
        let r = u8::try_from((c >> 16) & 0xff).unwrap();
        let g = u8::try_from((c >> 8) & 0xff).unwrap();
        let b = u8::try_from(c & 0xff).unwrap();

        (r, g, b)
    }

    /// Get RGB components of a 32 bits color value.
    #[allow(clippy::many_single_char_names)]
    pub(crate) fn color_to_rgba(c: u32) -> (u8, u8, u8, u8) {
        let a = u8::try_from((c >> 24) & 0xff).unwrap();
        let r = u8::try_from((c >> 16) & 0xff).unwrap();
        let g = u8::try_from((c >> 8) & 0xff).unwrap();
        let b = u8::try_from(c & 0xff).unwrap();

        (r, g, b, a)
    }

    /// Get HSL components of a 32 bits color value.
    #[allow(clippy::many_single_char_names)]
    pub(crate) fn color_to_hsl(c: u32) -> (f64, f64, f64) {
        let (r, g, b) = color_to_rgb(c);
        let rgb =
            Srgb::from_components(((r as f64 / 255.0), (g as f64 / 255.0), (b as f64 / 255.0)));

        let (h, s, l) = Hsl::from(rgb).into_components();

        (h.into(), s, l)
    }

    /// Convert RGB components to a 32 bits color value.
    pub(crate) fn rgb_to_color(r: u8, g: u8, b: u8) -> u32 {
        LittleEndian::read_u32(&[b, g, r, 255])
    }

    /// Convert RGBA components to a 32 bits color value.
    pub(crate) fn rgba_to_color(r: u8, g: u8, b: u8, a: u8) -> u32 {
        LittleEndian::read_u32(&[b, g, r, a])
    }

    /// Convert HSL components to a 32 bits color value.
    pub(crate) fn hsl_to_color(h: f64, s: f64, l: f64) -> u32 {
        let rgb = Srgb::convert_from(Hsl::new(h, s, l));
        let rgb = rgb.into_components();
        rgba_to_color(
            (rgb.0 * 255.0) as u8,
            (rgb.1 * 255.0) as u8,
            (rgb.2 * 255.0) as u8,
            255,
        )
    }

    /// Convert HSLA components to a 32 bits color value.
    pub(crate) fn hsla_to_color(h: f64, s: f64, l: f64, a: u8) -> u32 {
        let rgb = Srgb::convert_from(Hsl::new(h, s, l));
        let rgb = rgb.into_components();
        rgba_to_color(
            (rgb.0 * 255.0) as u8,
            (rgb.1 * 255.0) as u8,
            (rgb.2 * 255.0) as u8,
            a,
        )
    }

    /// Generate a linear RGB color gradient from start to dest color,
    /// where p must lie in the range from [0.0..1.0].
    #[allow(clippy::many_single_char_names)]
    pub(crate) fn linear_gradient(start: u32, dest: u32, p: f64) -> u32 {
        let sca: f64 = f64::from((start >> 24) & 0xff);
        let scr: f64 = f64::from((start >> 16) & 0xff);
        let scg: f64 = f64::from((start >> 8) & 0xff);
        let scb: f64 = f64::from((start) & 0xff);

        let dca: f64 = f64::from((dest >> 24) & 0xff);
        let dcr: f64 = f64::from((dest >> 16) & 0xff);
        let dcg: f64 = f64::from((dest >> 8) & 0xff);
        let dcb: f64 = f64::from((dest) & 0xff);

        let r: f64 = (scr as f64) + (((dcr - scr) as f64) * p);
        let g: f64 = (scg as f64) + (((dcg - scg) as f64) * p);
        let b: f64 = (scb as f64) + (((dcb - scb) as f64) * p);
        let a: f64 = (sca as f64) + (((dca - sca) as f64) * p);

        rgba_to_color(
            r.round() as u8,
            g.round() as u8,
            b.round() as u8,
            a.round() as u8,
        )
    }

    /// Clamp the value `v` to the range [`min`..`max`]
    #[inline]
    pub(crate) fn clamp(v: f64, min: f64, max: f64) -> f64 {
        let mut x = v;
        if x < min {
            x = min;
        }
        if x > max {
            x = max;
        }
        x
    }

    /// Compute Gradient noise (SIMD)
    pub(crate) fn gradient_noise_2d_simd(f1: f32, f2: f32) -> f32 {
        simdnoise::NoiseBuilder::gradient_2d_offset(f1, 2, f2, 2).generate_scaled(0.0, 1.0)[0]
    }

    pub(crate) fn gradient_noise_3d_simd(f1: f32, f2: f32, f3: f32) -> f32 {
        simdnoise::NoiseBuilder::gradient_3d_offset(f1, 2, f2, 2, f3, 2).generate_scaled(0.0, 1.0)
            [0]
    }

    /// Compute Turbulence noise (SIMD)
    pub(crate) fn turbulence_noise_2d_simd(f1: f32, f2: f32) -> f32 {
        simdnoise::NoiseBuilder::turbulence_2d_offset(f1, 2, f2, 2).generate_scaled(0.0, 1.0)[0]
    }

    pub(crate) fn turbulence_noise_3d_simd(f1: f32, f2: f32, f3: f32) -> f32 {
        simdnoise::NoiseBuilder::turbulence_3d_offset(f1, 2, f2, 2, f3, 2).generate_scaled(0.0, 1.0)
            [0]
    }

    /// Compute Perlin noise
    pub(crate) fn perlin_noise(f1: f64, f2: f64, f3: f64) -> f64 {
        let noise = noise::Perlin::new();
        noise.get([f1, f2, f3]) / 2.0 + 0.5
    }

    /// Compute Billow noise
    pub(crate) fn billow_noise(f1: f64, f2: f64, f3: f64) -> f64 {
        let noise = noise::Billow::new();
        noise.get([f1, f2, f3]) / 2.0 + 0.5
    }

    /// Compute Worley (Voronoi) noise
    pub(crate) fn voronoi_noise(f1: f64, f2: f64, f3: f64) -> f64 {
        let noise = noise::Worley::new();
        noise.get([f1, f2, f3]) / 2.0 + 0.5
    }

    /// Compute Fractal Brownian Motion noise
    pub(crate) fn fractal_brownian_noise(f1: f64, f2: f64, f3: f64) -> f64 {
        let noise = noise::Fbm::new();
        noise.get([f1, f2, f3]) / 2.0 + 0.5
    }

    /// Compute Ridged Multifractal noise
    pub(crate) fn ridged_multifractal_noise(f1: f64, f2: f64, f3: f64) -> f64 {
        let noise = noise::RidgedMulti::new();
        noise.get([f1, f2, f3]) / 2.0 + 0.5
    }

    /// Compute Open Simplex noise (2D)
    pub(crate) fn open_simplex_noise_2d(f1: f64, f2: f64) -> f64 {
        let noise = noise::OpenSimplex::new();
        noise.get([f1, f2]) / 2.0 + 0.5
    }

    /// Compute Open Simplex noise (3D)
    pub(crate) fn open_simplex_noise_3d(f1: f64, f2: f64, f3: f64) -> f64 {
        let noise = noise::OpenSimplex::new();
        noise.get([f1, f2, f3]) / 2.0 + 0.5
    }

    /// Compute Open Simplex noise (4D)
    pub(crate) fn open_simplex_noise_4d(f1: f64, f2: f64, f3: f64, f4: f64) -> f64 {
        let noise = noise::OpenSimplex::new();
        noise.get([f1, f2, f3, f4]) / 2.0 + 0.5
    }

    /// Compute Super Simplex noise (3D)
    pub(crate) fn super_simplex_noise_3d(f1: f64, f2: f64, f3: f64) -> f64 {
        let noise = noise::SuperSimplex::new();
        noise.get([f1, f2, f3]) / 2.0 + 0.5
    }

    use nalgebra as na;

    pub(crate) fn rotate(map: &[u32], theta: f64, sizes: (usize, usize)) -> Vec<u32> {
        let mut result = vec![0; map.len()];

        let m_rot = na::Matrix3::new_rotation(theta);

        for i in 0..map.len() {
            let x = (i / sizes.0) as f64;
            let y = (i / sizes.1) as f64;

            let point = na::Vector2::new(x, y).to_homogeneous();
            let t = m_rot * point;

            let idx = (t.x * sizes.0 as f64 + t.y).round();
            let idx = clamp(idx, 0 as f64, sizes.0 as f64 * sizes.1 as f64) as usize;

            // println!("{} -> {}: {}", point, t, idx);

            result[i] = map[idx];
        }

        result
    }

    #[test]
    fn test_rotate() {
        let data: Vec<_> = (1..=100u32).collect();

        let x_size = 10;
        let y_size = 10;

        let result = rotate(&data, 90.0 * std::f64::consts::PI / 180.0, (x_size, y_size));

        for l in 0..y_size {
            let s = l * y_size;
            println!("{:2?}", &result[s..s + x_size]);
        }

        println!("{:2?}", &data);
        println!("{:2?}", &result);
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
        let mut led_map = LED_MAP.lock();
        led_map[idx] = RGBA {
            a: u8::try_from((c >> 24) & 0xff).unwrap(),
            r: u8::try_from((c >> 16) & 0xff).unwrap(),
            g: u8::try_from((c >> 8) & 0xff).unwrap(),
            b: u8::try_from(c & 0xff).unwrap(),
        };

        let mut rvdev = rvdev.lock();

        rvdev
            .send_led_map(&*led_map)
            .unwrap_or_else(|e| error!("Could not send the LED map to the keyboard: {}", e));

        thread::sleep(Duration::from_millis(
            crate::constants::DEVICE_SETTLE_MILLIS,
        ));
    }

    /// Get state of all LEDs
    pub(crate) fn get_color_map() -> Vec<u32> {
        let global_led_map = LED_MAP.lock();

        let result = global_led_map
            .iter()
            .map(|v| {
                ((v.r as u32).overflowing_shl(16).0
                    + (v.g as u32).overflowing_shl(8).0
                    + v.b as u32) as u32
            })
            .collect::<Vec<u32>>();

        assert!(result.len() == NUM_KEYS);

        result
    }

    /// Set all LEDs at once.
    pub(crate) fn set_color_map(rvdev: &Arc<Mutex<RvDeviceState>>, map: &[u32]) {
        assert!(map.len() == NUM_KEYS);

        let mut led_map = [RGBA {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }; NUM_KEYS];

        let mut i = 0;
        loop {
            led_map[i] = RGBA {
                a: u8::try_from((map[i] >> 24) & 0xff).unwrap(),
                r: u8::try_from((map[i] >> 16) & 0xff).unwrap(),
                g: u8::try_from((map[i] >> 8) & 0xff).unwrap(),
                b: u8::try_from(map[i] & 0xff).unwrap(),
            };

            i += 1;
            if i >= NUM_KEYS - 1 {
                break;
            }
        }

        let mut global_led_map = LED_MAP.lock();
        *global_led_map = led_map.to_vec();

        let mut rvdev = rvdev.lock();
        rvdev
            .send_led_map(&led_map)
            .unwrap_or_else(|e| error!("Could not send the LED map to the keyboard: {}", e));

        thread::sleep(Duration::from_millis(
            crate::constants::DEVICE_SETTLE_MILLIS,
        ));
    }

    /// Submit LED color map for later realization, as soon as the
    /// next frame is rendered
    pub(crate) fn submit_color_map(map: &[u32]) {
        // trace!("submit_color_map: {}/{}", map.len(), NUM_KEYS);
        assert!(
            map.len() == NUM_KEYS,
            format!(
                "Assertion 'map.len() == NUM_KEYS' failed: {} != {}",
                map.len(),
                NUM_KEYS
            )
        );

        let mut led_map = [RGBA {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }; NUM_KEYS];

        let mut i = 0;
        loop {
            led_map[i] = RGBA {
                a: u8::try_from((map[i] >> 24) & 0xff).unwrap(),
                r: u8::try_from((map[i] >> 16) & 0xff).unwrap(),
                g: u8::try_from((map[i] >> 8) & 0xff).unwrap(),
                b: u8::try_from(map[i] & 0xff).unwrap(),
            };

            i += 1;
            if i >= NUM_KEYS - 1 {
                break;
            }
        }

        LOCAL_LED_MAP.with(|local_map| local_map.borrow_mut().copy_from_slice(&led_map));
        super::FRAME_GENERATION_COUNTER.fetch_add(1, Ordering::SeqCst);
    }
}

/// Action requests for `run_script`
pub enum RunScriptResult {
    /// Script terminated gracefully
    TerminatedGracefully,

    /// Error abort
    TerminatedWithErrors,
    // Currently running interpreter will be shut down, to execute another Lua script
    //ReExecuteOtherScript(PathBuf),
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
            if let Err(error) = manifest {
                error!(
                    "Could not parse manifest file for script '{}': {}",
                    file.display(),
                    error
                );

                return Err(ScriptingError::InaccessibleManifest {});
            } else {
                ACTIVE_SCRIPTS
                    .lock()
                    .push(manifest.as_ref().unwrap().clone());
            }

            let result: rlua::Result<RunScriptResult> = lua.context::<_, _>(|lua_ctx| {
                let mut errors_present = false;

                if register_support_globals(lua_ctx, &rvdevice).is_err() {
                    return Ok(RunScriptResult::TerminatedWithErrors)
                }

                if register_support_funcs(lua_ctx, &rvdevice).is_err() {
                    return Ok(RunScriptResult::TerminatedWithErrors)
                }

                if register_script_config(lua_ctx, &manifest.unwrap()).is_err() {
                    return Ok(RunScriptResult::TerminatedWithErrors)
                }

                // start execution of the Lua script
                lua_ctx.load(&script).eval::<()>().unwrap_or_else(|e| {
                    error!("Lua error: {}", e);
                    errors_present = true;
                });

                if errors_present {
                    return Ok(RunScriptResult::TerminatedWithErrors)
                }

                // call startup event handler, iff present
                if let Ok(handler) = lua_ctx.globals().get::<_, Function>("on_startup") {
                    handler.call::<_, ()>(()).unwrap_or_else(|e| {
                        error!("Lua error: {}", e);
                        errors_present = true;
                    });
                }

                if errors_present {
                    return Ok(RunScriptResult::TerminatedWithErrors)
                }

                loop {
                    if let Ok(msg) = rx.recv() {
                        match msg {
                            Message::Quit(param) => {
                                let mut errors_present = false;

                                if let Ok(handler) = lua_ctx.globals().get::<_, Function>("on_quit")
                                {

                                    handler.call::<_, ()>(param).unwrap_or_else(|e| {
                                        error!("Lua error: {}", e);
                                        errors_present = true;
                                    });
                                }

                                if errors_present {
                                    return Ok(RunScriptResult::TerminatedWithErrors)
                                }
                            }

                            Message::Tick(param) => {
                                let mut errors_present = false;

                                if let Ok(handler) = lua_ctx.globals().get::<_, Function>("on_tick")
                                {
                                    handler.call::<_, ()>(param).unwrap_or_else(|e| {
                                        error!("Lua error: {}", e);
                                        errors_present = true;
                                    })
                                }

                                if errors_present {
                                    return Ok(RunScriptResult::TerminatedWithErrors)
                                }
                            }

                            Message::RealizeColorMap => {
                                LOCAL_LED_MAP.with(|foreground| {
                                    for (idx, background) in LED_MAP.lock().iter_mut().enumerate() {
                                        let bg = &background;
                                        let fg = foreground.borrow()[idx];

                                        let brightness = crate::BRIGHTNESS.load(Ordering::SeqCst);

                                        #[rustfmt::skip]
                                        let color = RGBA {
                                            r: ((((fg.a as f64) * fg.r as f64 + (255 - fg.a) as f64 * bg.r as f64).abs() * brightness as f64 / 100.0) as u32 >> 8) as u8,
                                            g: ((((fg.a as f64) * fg.g as f64 + (255 - fg.a) as f64 * bg.g as f64).abs() * brightness as f64 / 100.0) as u32 >> 8) as u8,
                                            b: ((((fg.a as f64) * fg.b as f64 + (255 - fg.a) as f64 * bg.b as f64).abs() * brightness as f64 / 100.0) as u32 >> 8) as u8,
                                            a: fg.a as u8,
                                        };

                                        *background = color;
                                    }
                                });

                                // signal readiness / notify the main thread that we are done
                                crate::COLOR_MAPS_READY_CONDITION
                                    .0
                                    .lock()
                                    .checked_sub(1)
                                    .unwrap_or_else(|| {
                                        warn!("Incorrect state in locking code detected");
                                        0
                                    });
                                crate::COLOR_MAPS_READY_CONDITION.1.notify_one();
                            }

                            Message::KeyDown(param) => {
                                let mut errors_present = false;

                                if let Ok(handler) =
                                    lua_ctx.globals().get::<_, Function>("on_key_down")
                                {
                                    handler.call::<_, ()>(param).unwrap_or_else(|e| {
                                        error!("Lua error: {}", e);
                                        errors_present = true;
                                    });
                                }

                                *crate::UPCALL_COMPLETED_ON_KEY_DOWN.0.lock() -= 1;
                                crate::UPCALL_COMPLETED_ON_KEY_DOWN.1.notify_all();

                                if errors_present {
                                    return Ok(RunScriptResult::TerminatedWithErrors)
                                }
                            }

                            Message::KeyUp(param) => {
                                let mut errors_present = false;

                                if let Ok(handler) =
                                    lua_ctx.globals().get::<_, Function>("on_key_up")
                                {
                                    handler.call::<_, ()>(param).unwrap_or_else(|e| {
                                        error!("Lua error: {}", e);
                                        errors_present = true;
                                    });
                                }

                                *crate::UPCALL_COMPLETED_ON_KEY_UP.0.lock() -= 1;
                                crate::UPCALL_COMPLETED_ON_KEY_UP.1.notify_all();

                                if errors_present {
                                    return Ok(RunScriptResult::TerminatedWithErrors)
                                }
                            }

                            //Message::LoadScript(script_path) => {
                            //return Ok(RunScriptResult::ReExecuteOtherScript(script_path))
                            //}

                            // Message::Abort => {
                            //     error!("Lua script '{}' terminated with errors", file.file_name().unwrap().to_string_lossy());
                            //     return Ok(RunScriptResult::TerminatedWithErrors);
                            // }

                            Message::Unload => {
                                let mut errors_present = false;                                

                                if let Ok(handler) = lua_ctx.globals().get::<_, Function>("on_quit")
                                {
                                    handler.call::<_, ()>(()).unwrap_or_else(|e| {
                                        error!("Lua error: {}", e);
                                        errors_present = true;
                                    })
                                }

                                if errors_present {
                                    error!("Lua script '{}' terminated with errors", file.file_name().unwrap().to_string_lossy());
                                    return Ok(RunScriptResult::TerminatedWithErrors);
                                } else {
                                    debug!("Lua script '{}' terminated gracefully", file.file_name().unwrap().to_string_lossy());
                                    return Ok(RunScriptResult::TerminatedGracefully);
                                }
                            }
                        }
                    }
                }
            });

            match result {
                Ok(action) => Ok(action),

                Err(e) => Err(ScriptingError::LuaError { e }),
            }
        }

        Err(_e) => Err(ScriptingError::OpenError {}),
    }
}

fn register_support_globals(lua_ctx: Context, _rvdevice: &RvDeviceState) -> rlua::Result<()> {
    let globals = lua_ctx.globals();

    #[cfg(debug_assertions)]
    lua_ctx
        .load("package.path = package.path .. ';src/scripts/lib/?;src/scripts/lib/?.lua'")
        .exec()
        .unwrap();

    #[cfg(not(debug_assertions))]
    lua_ctx
        .load("package.path = package.path .. ';/usr/share/eruption/scripts/lib/?;/usr/share/eruption/scripts/lib/?.lua'")
        .exec()
        .unwrap();

    let mut config: HashMap<&str, &str> = HashMap::new();
    config.insert("daemon_name", "eruption");
    config.insert("daemon_version", env!("CARGO_PKG_VERSION"));
    config.insert("api_level", env!("CARGO_PKG_VERSION"));

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

    let clamp = lua_ctx.create_function(|_, (val, f1, f2): (f64, f64, f64)| Ok(callbacks::clamp(val, f1, f2)))?;
    globals.set("clamp", clamp)?;

    let abs = lua_ctx.create_function(|_, f: f64| Ok(f.abs()))?;
    globals.set("abs", abs)?;

    let sin = lua_ctx.create_function(|_, a: f64| Ok(a.sin()))?;
    globals.set("sin", sin)?;

    let cos = lua_ctx.create_function(|_, a: f64| Ok(a.cos()))?;
    globals.set("cos", cos)?;

    let pow = lua_ctx.create_function(|_, (val, p): (f64, f64)| Ok(val.powf(p)))?;
    globals.set("pow", pow)?;

    let sqrt = lua_ctx.create_function(|_, f: f64| Ok(f.sqrt()))?;
    globals.set("sqrt", sqrt)?;

    let rand =
        lua_ctx.create_function(|_, (l, h): (u64, u64)| Ok(rand::thread_rng().gen_range(l, h)))?;
    globals.set("rand", rand)?;

    let trunc = lua_ctx.create_function(|_, f: f64| Ok(f.trunc() as i64))?;
    globals.set("trunc", trunc)?;

    // let lerp = lua_ctx.create_function(|_, (a0, a1, w): (f64, f64, f64)| Ok((1.0 - w) * a0 + w * a1))?; // precise version
    let lerp = lua_ctx.create_function(|_, (v0, v1, t): (f64, f64, f64)| Ok(v0 + t * (v1 - v0)))?;
    globals.set("lerp", lerp)?;

    let invlerp = lua_ctx.create_function(|_, (v0, v1, t): (f64, f64, f64)| Ok(
        callbacks::clamp((t - v0) / (v0 - v1), 0.0, 1.0)
    ))?;
    globals.set("invlerp", invlerp)?;

    let range = lua_ctx.create_function(|_, (v0, v1, v2, v3, t): (f64, f64, f64, f64, f64)| Ok(
        v2 + callbacks::clamp((t - v0) / (v1 - v0), 0.0, 1.0) * (v3 - v2)
    ))?;
    globals.set("range", range)?;

    // keyboard state and macros
    let inject_key = lua_ctx.create_function(|_, (ev_key, down): (u32, bool)| {
        callbacks::inject_key(ev_key, down);
        Ok(())
    })?;
    globals.set("inject_key", inject_key)?;

    let inject_key_with_delay = lua_ctx.create_function(|_, (ev_key, down, millis): (u32, bool, u64)| {
        callbacks::inject_key_with_delay(ev_key, down, millis);
        Ok(())
    })?;
    globals.set("inject_key_with_delay", inject_key_with_delay)?;

    // color handling
    let color_to_rgb = lua_ctx.create_function(|_, c: u32| Ok(callbacks::color_to_rgb(c)))?;
    globals.set("color_to_rgb", color_to_rgb)?;

    let color_to_rgba = lua_ctx.create_function(|_, c: u32| Ok(callbacks::color_to_rgba(c)))?;
    globals.set("color_to_rgba", color_to_rgba)?;

    let color_to_hsl = lua_ctx.create_function(|_, c: u32| Ok(callbacks::color_to_hsl(c)))?;
    globals.set("color_to_hsl", color_to_hsl)?;

    let rgb_to_color = lua_ctx
        .create_function(|_, (r, g, b): (u8, u8, u8)| Ok(callbacks::rgb_to_color(r, g, b)))?;
    globals.set("rgb_to_color", rgb_to_color)?;

    let rgba_to_color = lua_ctx.create_function(|_, (r, g, b, a): (u8, u8, u8, u8)| {
        Ok(callbacks::rgba_to_color(r, g, b, a))
    })?;
    globals.set("rgba_to_color", rgba_to_color)?;

    let hsl_to_color = lua_ctx
        .create_function(|_, (h, s, l): (f64, f64, f64)| Ok(callbacks::hsl_to_color(h, s, l)))?;
    globals.set("hsl_to_color", hsl_to_color)?;

    let hsla_to_color = lua_ctx.create_function(|_, (h, s, l, a): (f64, f64, f64, u8)| {
        Ok(callbacks::hsla_to_color(h, s, l, a))
    })?;
    globals.set("hsla_to_color", hsla_to_color)?;

    let linear_gradient = lua_ctx.create_function(|_, (start, dest, p): (u32, u32, f64)| {
        Ok(callbacks::linear_gradient(start, dest, p))
    })?;
    globals.set("linear_gradient", linear_gradient)?;

    // noise utilities

    // fast implementations (SIMD)
    let gradient_noise_2d = lua_ctx
        .create_function(|_, (f1, f2): (f32, f32)| Ok(callbacks::gradient_noise_2d_simd(f1, f2)))?;
    globals.set("gradient_noise_2d", gradient_noise_2d)?;

    let gradient_noise_3d = lua_ctx.create_function(|_, (f1, f2, f3): (f32, f32, f32)| {
        Ok(callbacks::gradient_noise_3d_simd(f1, f2, f3))
    })?;
    globals.set("gradient_noise_3d", gradient_noise_3d)?;

    let turbulence_noise_2d = lua_ctx.create_function(|_, (f1, f2): (f32, f32)| {
        Ok(callbacks::turbulence_noise_2d_simd(f1, f2))
    })?;
    globals.set("turbulence_noise_2d", turbulence_noise_2d)?;

    let turbulence_noise_3d = lua_ctx.create_function(|_, (f1, f2, f3): (f32, f32, f32)| {
        Ok(callbacks::turbulence_noise_3d_simd(f1, f2, f3))
    })?;
    globals.set("turbulence_noise_3d", turbulence_noise_3d)?;

    // slow implementations (without use of SIMD)
    let perlin_noise = lua_ctx.create_function(|_, (f1, f2, f3): (f64, f64, f64)| {
        Ok(callbacks::perlin_noise(f1, f2, f3))
    })?;
    globals.set("perlin_noise", perlin_noise)?;

    let billow_noise = lua_ctx.create_function(|_, (f1, f2, f3): (f64, f64, f64)| {
        Ok(callbacks::billow_noise(f1, f2, f3))
    })?;
    globals.set("billow_noise", billow_noise)?;

    let voronoi_noise = lua_ctx.create_function(|_, (f1, f2, f3): (f64, f64, f64)| {
        Ok(callbacks::voronoi_noise(f1, f2, f3))
    })?;
    globals.set("voronoi_noise", voronoi_noise)?;

    let fractal_brownian_noise = lua_ctx.create_function(|_, (f1, f2, f3): (f64, f64, f64)| {
        Ok(callbacks::fractal_brownian_noise(f1, f2, f3))
    })?;
    globals.set("fractal_brownian_noise", fractal_brownian_noise)?;

    let ridged_multifractal_noise =
        lua_ctx.create_function(|_, (f1, f2, f3): (f64, f64, f64)| {
            Ok(callbacks::ridged_multifractal_noise(f1, f2, f3))
        })?;
    globals.set("ridged_multifractal_noise", ridged_multifractal_noise)?;

    let open_simplex_noise = lua_ctx.create_function(|_, (f1, f2, f3): (f64, f64, f64)| {
        Ok(callbacks::open_simplex_noise_3d(f1, f2, f3))
    })?;
    globals.set("open_simplex_noise", open_simplex_noise)?;

    let open_simplex_noise_2d = lua_ctx
        .create_function(|_, (f1, f2): (f64, f64)| Ok(callbacks::open_simplex_noise_2d(f1, f2)))?;
    globals.set("open_simplex_noise_2d", open_simplex_noise_2d)?;

    let open_simplex_noise_4d =
        lua_ctx.create_function(|_, (f1, f2, f3, f4): (f64, f64, f64, f64)| {
            Ok(callbacks::open_simplex_noise_4d(f1, f2, f3, f4))
        })?;
    globals.set("open_simplex_noise_4d", open_simplex_noise_4d)?;

    let super_simplex_noise = lua_ctx.create_function(|_, (f1, f2, f3): (f64, f64, f64)| {
        Ok(callbacks::super_simplex_noise_3d(f1, f2, f3))
    })?;
    globals.set("super_simplex_noise", super_simplex_noise)?;

    // transformation utilities
    let rotate = lua_ctx.create_function(|_, (map, theta): (Vec<u32>, f64)| {
        Ok(callbacks::rotate(&map, theta, (22, 6)))
    })?;
    globals.set("rotate", rotate)?;

    // device related
    let get_num_keys = lua_ctx.create_function(move |_, ()| Ok(callbacks::get_num_keys()))?;
    globals.set("get_num_keys", get_num_keys)?;

    let rvdevid_tmp = rvdevid;
    let get_key_color = lua_ctx
        .create_function(move |_, idx: usize| Ok(callbacks::get_key_color(&rvdevid_tmp, idx)))?;
    globals.set("get_key_color", get_key_color)?;

    let rvdev_tmp = rvdev.clone();
    let set_key_color = lua_ctx.create_function(move |_, (idx, c): (usize, u32)| {
        callbacks::set_key_color(&rvdev_tmp, idx, c);
        Ok(())
    })?;
    globals.set("set_key_color", set_key_color)?;

    let get_color_map = lua_ctx.create_function(move |_, ()| Ok(callbacks::get_color_map()))?;
    globals.set("get_color_map", get_color_map)?;

    let rvdev_tmp = rvdev;
    let set_color_map = lua_ctx.create_function(move |_, map: Vec<u32>| {
        callbacks::set_color_map(&rvdev_tmp, &map);
        Ok(())
    })?;
    globals.set("set_color_map", set_color_map)?;

    let submit_color_map = lua_ctx.create_function(move |_, map: Vec<u32>| {
        callbacks::submit_color_map(&map);
        Ok(())
    })?;
    globals.set("submit_color_map", submit_color_map)?;

    // finally, register Lua functions supplied by eruption plugins
    let plugin_manager = plugin_manager::PLUGIN_MANAGER.read();
    let plugins = plugin_manager.get_plugins();

    for plugin in plugins.iter() {
        plugin.register_lua_funcs(lua_ctx).unwrap();
    }

    Ok(())
}

fn register_script_config(lua_ctx: Context, manifest: &Manifest) -> rlua::Result<()> {
    let profile = &*ACTIVE_PROFILE.lock();
    let script_name = &manifest.name;

    let globals = lua_ctx.globals();
    if let Some(config) = &manifest.config {
        for param in config.iter() {
            debug!("Applying parameter {:?}", param);

            match param {
                ConfigParam::Int { name, default, .. } => {
                    if let Some(profile) = profile {
                        if let Some(val) = profile.get_int_value(script_name, name) {
                            globals.raw_set::<&str, i64>(name, *val)?;
                        } else {
                            globals.raw_set::<&str, i64>(name, *default)?;
                        }
                    } else {
                        globals.raw_set::<&str, i64>(name, *default)?;
                    }
                }

                ConfigParam::Float { name, default, .. } => {
                    if let Some(profile) = profile {
                        if let Some(val) = profile.get_float_value(script_name, name) {
                            globals.raw_set::<&str, f64>(name, *val)?;
                        } else {
                            globals.raw_set::<&str, f64>(name, *default)?;
                        }
                    } else {
                        globals.raw_set::<&str, f64>(name, *default)?;
                    }
                }

                ConfigParam::Bool { name, default, .. } => {
                    if let Some(profile) = profile {
                        if let Some(val) = profile.get_bool_value(script_name, name) {
                            globals.raw_set::<&str, bool>(name, *val)?;
                        } else {
                            globals.raw_set::<&str, bool>(name, *default)?;
                        }
                    } else {
                        globals.raw_set::<&str, bool>(name, *default)?;
                    }
                }

                ConfigParam::String { name, default, .. } => {
                    if let Some(profile) = profile {
                        if let Some(val) = profile.get_str_value(script_name, name) {
                            globals.raw_set::<&str, &str>(name, &*val)?;
                        } else {
                            globals.raw_set::<&str, &str>(name, &*default)?;
                        }
                    } else {
                        globals.raw_set::<&str, &str>(name, &*default)?;
                    }
                }

                ConfigParam::Color { name, default, .. } => {
                    if let Some(profile) = profile {
                        if let Some(val) = profile.get_color_value(script_name, name) {
                            globals.raw_set::<&str, u32>(name, *val)?;
                        } else {
                            globals.raw_set::<&str, u32>(name, *default)?;
                        }
                    } else {
                        globals.raw_set::<&str, u32>(name, *default)?;
                    }
                }
            }
        }
    }

    Ok(())
}
