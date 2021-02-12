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

use callbacks::CallbacksError;
use crossbeam::channel::Receiver;
use lazy_static::lazy_static;
use log::*;
use mlua::prelude::*;
use mlua::Function;
use parking_lot::RwLock;
use rand::Rng;
use std::cell::RefCell;
use std::collections::HashMap;
use std::error::Error;
use std::fmt;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::vec::Vec;

use crate::constants;
use crate::hwdevices::{KeyboardDevice, KeyboardHidEvent, MouseDevice, MouseHidEvent, RGBA};
use crate::plugin_manager;
use crate::scripting::manifest::{ConfigParam, Manifest};

use crate::{ACTIVE_PROFILE, ACTIVE_SCRIPTS};

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

    /// Frame generation counter, used to detect if we need to submit the LED_MAP to the keyboard
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

    #[error("Invalid or inaccessible manifest file")]
    InaccessibleManifest {},

    #[error("Invalid value")]
    ValueError {},
}

#[derive(Debug)]
pub struct UnknownError {}

impl std::error::Error for UnknownError {}

impl fmt::Display for UnknownError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Unknown error occurred")
    }
}

/// These functions are intended to be used from within Lua scripts
mod callbacks {
    use byteorder::{ByteOrder, LittleEndian};
    use log::*;
    use noise::{NoiseFn, Seedable};
    use palette::ConvertFrom;
    use palette::{Hsl, Srgb};
    use std::convert::TryFrom;
    use std::sync::atomic::Ordering;
    use std::time::Duration;
    use std::{cell::RefCell, thread};

    use super::{LED_MAP, LOCAL_LED_MAP, LOCAL_LED_MAP_MODIFIED};

    use crate::plugins::macros;
    use crate::{constants, hwdevices::RGBA};

    pub type Result<T> = std::result::Result<T, eyre::Error>;

    #[derive(Debug, Clone, thiserror::Error)]
    pub enum CallbacksError {
        #[error("Invalid handle supplied")]
        InvalidHandle {},

        #[error("Could not parse param value")]
        ParseParamError {},
    }

    fn seed() -> u32 {
        use std::time::{SystemTime, UNIX_EPOCH};

        let start = SystemTime::now();
        let since_the_epoch = start
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");

        since_the_epoch.as_millis() as u32
    }

    thread_local! {
        pub static PERLIN_NOISE: RefCell<noise::Perlin> = {
            let noise = noise::Perlin::new();
            RefCell::new(noise.set_seed(seed()))
        };

        pub static BILLOW_NOISE: RefCell<noise::Billow> = {
            let noise = noise::Billow::new();
            RefCell::new(noise.set_seed(seed()))
        };

        pub static VORONOI_NOISE: RefCell<noise::Worley> = {
            let noise = noise::Worley::new();
            RefCell::new(noise.set_seed(seed()))
        };

        pub static RIDGED_MULTIFRACTAL_NOISE: RefCell<noise::RidgedMulti> = {
            let noise = noise::RidgedMulti::new();

            // noise.octaves = 6;
            // noise.frequency = 2.0; // default: 1.0
            // noise.lacunarity = std::f64::consts::PI * 2.0 / 3.0;
            // noise.persistence = 1.0;
            // noise.attenuation = 4.0; // default: 2.0

            RefCell::new(noise.set_seed(seed()))
        };

        pub static FBM_NOISE: RefCell<noise::Fbm> = {
            let noise = noise::Fbm::new();
            RefCell::new(noise.set_seed(seed()))
        };

        pub static OPEN_SIMPLEX_NOISE: RefCell<noise::OpenSimplex> = {
            let noise = noise::OpenSimplex::new();
            RefCell::new(noise.set_seed(seed()))
        };

        pub static SUPER_SIMPLEX_NOISE: RefCell<noise::SuperSimplex> = {
            let noise = noise::SuperSimplex::new();
            RefCell::new(noise.set_seed(seed()))
        };
    }

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

    /// Returns the target framerate
    pub(crate) fn get_target_fps() -> u64 {
        constants::TARGET_FPS
    }

    /// Returns the Lua support scripts for all connected devices
    pub(crate) fn get_support_script_files() -> Vec<String> {
        let mut result = Vec::new();

        for device in crate::KEYBOARD_DEVICES.lock().iter() {
            result.push(device.read().get_support_script_file());
        }

        for device in crate::MOUSE_DEVICES.lock().iter() {
            result.push(device.read().get_support_script_file());
        }

        result
    }

    /// Returns the number of "pixels" on the canvas
    pub(crate) fn get_canvas_size() -> usize {
        constants::CANVAS_SIZE
    }

    /// Returns the height of the canvas
    pub(crate) fn get_canvas_height() -> usize {
        constants::CANVAS_HEIGHT
    }

    /// Returns the width of the canvas
    pub(crate) fn get_canvas_width() -> usize {
        constants::CANVAS_WIDTH
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

    /// Inject a button event on the eruption virtual mouse.
    pub(crate) fn inject_mouse_button(button_index: u32, down: bool) {
        // calling inject_mouse_button(..) from Lua will drop the current input;
        // the original mouse event from the hardware mouse will not be
        // mirrored on the virtual mouse.
        macros::DROP_CURRENT_MOUSE_INPUT.store(true, Ordering::SeqCst);

        macros::UINPUT_TX
            .lock()
            .as_ref()
            .unwrap()
            .send(macros::Message::InjectButtonEvent {
                button: button_index,
                down,
            })
            .unwrap();
    }

    /// Inject a mouse wheel scroll event on the eruption virtual mouse.
    pub(crate) fn inject_mouse_wheel(direction: u32) {
        // calling inject_mouse_wheel(..) from Lua will drop the current input;
        // the original mouse event from the hardware mouse will not be
        // mirrored on the virtual mouse.
        macros::DROP_CURRENT_MOUSE_INPUT.store(true, Ordering::SeqCst);

        macros::UINPUT_TX
            .lock()
            .as_ref()
            .unwrap()
            .send(macros::Message::InjectMouseWheelEvent { direction })
            .unwrap();
    }

    /// Inject a key on the eruption virtual keyboard after sleeping for `millis` milliseconds.
    pub(crate) fn inject_key_with_delay(ev_key: u32, down: bool, millis: u64) {
        // calling inject_key(..) from Lua will drop the current input;
        // the original key event from the hardware keyboard will not be
        // mirrored on the virtual keyboard.
        macros::DROP_CURRENT_KEY.store(true, Ordering::SeqCst);

        thread::Builder::new()
            .name("uinput/delayed".to_owned())
            .spawn(move || {
                thread::sleep(Duration::from_millis(millis));

                macros::UINPUT_TX
                    .lock()
                    .as_ref()
                    .unwrap()
                    .send(macros::Message::InjectKey { key: ev_key, down })
                    .unwrap();
            })
            .unwrap();
    }

    // pub(crate) fn set_status_led(keyboard_device: &KeyboardDevice, led_id: u8, on: bool) {
    //     keyboard_device
    //         .read()
    //         .set_status_led(LedKind::from_id(led_id).unwrap(), on)
    //         .unwrap_or_else(|e| error!("{}", e));
    // }

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
            Srgb::from_components(((r as f64 / 255.0), (g as f64 / 255.0), (b as f64 / 255.0)))
                .into_linear();

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
        let rgb = Srgb::convert_from(Hsl::new(h, s, l)).into_linear();
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
        let rgb = Srgb::convert_from(Hsl::new(h, s, l)).into_linear();
        let rgb = rgb.into_components();
        rgba_to_color(
            (rgb.0 * 255.0) as u8,
            (rgb.1 * 255.0) as u8,
            (rgb.2 * 255.0) as u8,
            a,
        )
    }

    /// Convert a CSS color value to a 32 bits color value.
    pub(crate) fn parse_color(val: &str) -> Result<u32> {
        match csscolorparser::parse(&val) {
            Ok(color) => {
                let (r, g, b, a) = color.rgba_u8();

                Ok(rgba_to_color(r, g, b, a))
            }

            Err(e) => {
                error!(
                    "Could not parse value, not a valid CSS color definition: {}",
                    e
                );
                Err(CallbacksError::ParseParamError {}.into())
            }
        }
    }

    /// Convert a gradient name to an opaque handle, representing that gradient
    pub(crate) fn gradient_from_name(val: &str) -> Result<usize> {
        match val {
            "rainbow-smooth" => super::ALLOCATED_GRADIENTS.with(|f| {
                let mut m = f.borrow_mut();
                let idx = m.len() + 1;

                let gradient = colorgrad::rainbow();
                m.insert(idx, gradient);

                Ok(idx)
            }),

            "sinebow-smooth" => super::ALLOCATED_GRADIENTS.with(|f| {
                let mut m = f.borrow_mut();
                let idx = m.len() + 1;

                let gradient = colorgrad::sinebow();
                m.insert(idx, gradient);

                Ok(idx)
            }),

            "spectral-smooth" => super::ALLOCATED_GRADIENTS.with(|f| {
                let mut m = f.borrow_mut();
                let idx = m.len() + 1;

                let gradient = colorgrad::spectral();
                m.insert(idx, gradient);

                Ok(idx)
            }),

            "rainbow-sharp" => super::ALLOCATED_GRADIENTS.with(|f| {
                let mut m = f.borrow_mut();
                let idx = m.len() + 1;

                let gradient = colorgrad::rainbow().sharp(5, 0.15);
                m.insert(idx, gradient);

                Ok(idx)
            }),

            "sinebow-sharp" => super::ALLOCATED_GRADIENTS.with(|f| {
                let mut m = f.borrow_mut();
                let idx = m.len() + 1;

                let gradient = colorgrad::sinebow().sharp(5, 0.15);
                m.insert(idx, gradient);

                Ok(idx)
            }),

            "spectral-sharp" => super::ALLOCATED_GRADIENTS.with(|f| {
                let mut m = f.borrow_mut();
                let idx = m.len() + 1;

                let gradient = colorgrad::spectral().sharp(5, 0.15);
                m.insert(idx, gradient);

                Ok(idx)
            }),

            _ => {
                error!("Could not parse value, not a valid stock-gradient");

                Err(CallbacksError::ParseParamError {}.into())
            }
        }
    }

    /// De-allocates a gradient from an opaque handle, representing that gradient
    pub(crate) fn gradient_destroy(handle: usize) -> Result<()> {
        super::ALLOCATED_GRADIENTS.with(|f| {
            let mut m = f.borrow_mut();

            if m.remove(&handle).is_some() {
                Ok(())
            } else {
                Err(CallbacksError::InvalidHandle {}.into())
            }
        })
    }

    /// Returns the color at the position `pos`
    pub(crate) fn gradient_color_at(handle: usize, pos: f64) -> Result<u32> {
        super::ALLOCATED_GRADIENTS.with(|f| {
            let m = f.borrow();

            if let Some(gradient) = m.get(&handle) {
                let color = gradient.at(pos);
                let (r, g, b, a) = color.rgba_u8();

                Ok(rgba_to_color(r, g, b, a))
            } else {
                Err(CallbacksError::InvalidHandle {}.into())
            }
        })
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
        PERLIN_NOISE.with(|noise| noise.borrow().get([f1, f2, f3]))
    }

    /// Compute Billow noise
    pub(crate) fn billow_noise(f1: f64, f2: f64, f3: f64) -> f64 {
        BILLOW_NOISE.with(|noise| noise.borrow().get([f1, f2, f3]))
    }

    /// Compute Worley (Voronoi) noise
    pub(crate) fn voronoi_noise(f1: f64, f2: f64, f3: f64) -> f64 {
        VORONOI_NOISE.with(|noise| noise.borrow().get([f1, f2, f3]))
    }

    /// Compute Fractal Brownian Motion noise
    pub(crate) fn fractal_brownian_noise(f1: f64, f2: f64, f3: f64) -> f64 {
        FBM_NOISE.with(|noise| noise.borrow().get([f1, f2, f3]))
    }

    /// Compute Ridged Multifractal noise
    pub(crate) fn ridged_multifractal_noise(f1: f64, f2: f64, f3: f64) -> f64 {
        RIDGED_MULTIFRACTAL_NOISE.with(|noise| noise.borrow().get([f1, f2, f3]))
    }

    /// Compute Open Simplex noise (2D)
    pub(crate) fn open_simplex_noise_2d(f1: f64, f2: f64) -> f64 {
        OPEN_SIMPLEX_NOISE.with(|noise| noise.borrow().get([f1, f2]))
    }

    /// Compute Open Simplex noise (3D)
    pub(crate) fn open_simplex_noise_3d(f1: f64, f2: f64, f3: f64) -> f64 {
        OPEN_SIMPLEX_NOISE.with(|noise| noise.borrow().get([f1, f2, f3]))
    }

    /// Compute Open Simplex noise (4D)
    pub(crate) fn open_simplex_noise_4d(f1: f64, f2: f64, f3: f64, f4: f64) -> f64 {
        OPEN_SIMPLEX_NOISE.with(|noise| noise.borrow().get([f1, f2, f3, f4]))
    }

    /// Compute Super Simplex noise (3D)
    pub(crate) fn super_simplex_noise_3d(f1: f64, f2: f64, f3: f64) -> f64 {
        SUPER_SIMPLEX_NOISE.with(|noise| noise.borrow().get([f1, f2, f3]))
    }

    /// Compute Checkerboard noise (3D)
    pub(crate) fn checkerboard_noise_3d(f1: f64, f2: f64, f3: f64) -> f64 {
        // no seed needed
        let noise = noise::Checkerboard::new(0);
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
        let data: Vec<_> = (1..=100_u32).collect();

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
        // TODO: Return the number of keys of a specific device
        144
    }

    /// Get state of all LEDs
    pub(crate) fn get_color_map() -> Vec<u32> {
        let global_led_map = LED_MAP.write();

        let result = global_led_map
            .iter()
            .map(|v| {
                ((v.r as u32).overflowing_shl(16).0
                    + (v.g as u32).overflowing_shl(8).0
                    + v.b as u32) as u32
            })
            .collect::<Vec<u32>>();

        assert!(result.len() == constants::CANVAS_SIZE);

        result
    }

    /// Submit LED color map for later realization, as soon as the
    /// next frame is rendered
    pub(crate) fn submit_color_map(map: &[u32]) -> Result<()> {
        // trace!("submit_color_map: {}/{}", map.len(), constants::CANVAS_SIZE);

        // assert!(
        //     map.len() == constants::CANVAS_SIZE,
        //     format!(
        //         "Assertion 'map.len() == constants::CANVAS_SIZE' failed: {} != {}",
        //         map.len(),
        //         constants::CANVAS_SIZE
        //     )
        // );

        let mut led_map = [RGBA {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }; constants::CANVAS_SIZE];

        let mut i = 0;
        loop {
            led_map[i] = RGBA {
                a: u8::try_from((map[i] >> 24) & 0xff)?,
                r: u8::try_from((map[i] >> 16) & 0xff)?,
                g: u8::try_from((map[i] >> 8) & 0xff)?,
                b: u8::try_from(map[i] & 0xff)?,
            };

            i += 1;
            if i >= led_map.len() || i >= map.len() {
                break;
            }
        }

        LOCAL_LED_MAP.with(|local_map| local_map.borrow_mut().copy_from_slice(&led_map));
        LOCAL_LED_MAP_MODIFIED.with(|f| *f.borrow_mut() = true);

        super::FRAME_GENERATION_COUNTER.fetch_add(1, Ordering::SeqCst);

        Ok(())
    }

    pub(crate) fn get_brightness() -> isize {
        crate::BRIGHTNESS.load(Ordering::SeqCst)
    }

    pub(crate) fn set_brightness(val: isize) {
        crate::BRIGHTNESS.store(val, Ordering::SeqCst);
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
    rx: &Receiver<Message>,
    keyboard_devices: &[KeyboardDevice],
    _mouse_devices: &[MouseDevice],
) -> Result<RunScriptResult> {
    match fs::read_to_string(file.clone()) {
        Ok(script) => {
            let lua_ctx = unsafe { Lua::unsafe_new_with(mlua::StdLib::ALL) };

            let manifest = Manifest::from(&file);
            if let Err(error) = manifest {
                error!(
                    "Could not parse manifest file for script '{}': {}",
                    file.display(),
                    error
                );

                return Err(ScriptingError::InaccessibleManifest {}.into());
            } else {
                ACTIVE_SCRIPTS
                    .lock()
                    .push(manifest.as_ref().unwrap().clone());
            }

            let mut errors_present = false;

            if register_support_globals(&lua_ctx).is_err() {
                return Ok(RunScriptResult::TerminatedWithErrors);
            }

            if register_support_funcs(&lua_ctx).is_err() {
                return Ok(RunScriptResult::TerminatedWithErrors);
            }

            if register_script_config(&lua_ctx, &manifest.unwrap()).is_err() {
                return Ok(RunScriptResult::TerminatedWithErrors);
            }

            // start execution of the Lua script
            lua_ctx.load(&script).eval::<()>().unwrap_or_else(|e| {
                error!(
                    "Lua error in file {}: {}\n\t{:?}",
                    file.to_string_lossy(),
                    e,
                    e.source().unwrap_or(&UnknownError {})
                );
                errors_present = true;
            });

            if errors_present {
                return Ok(RunScriptResult::TerminatedWithErrors);
            }

            // call startup event handler, if present
            if let Ok(handler) = lua_ctx.globals().get::<_, Function>("on_startup") {
                handler.call::<_, ()>(()).unwrap_or_else(|e| {
                    error!(
                        "Lua error in file {}: {}\n\t{:?}",
                        file.to_string_lossy(),
                        e,
                        e.source().unwrap_or(&UnknownError {})
                    );
                    errors_present = true;
                });
            }

            if errors_present {
                return Ok(RunScriptResult::TerminatedWithErrors);
            }

            // reduce CPU load by caching the event handler status
            let mut has_tick_handler = true;
            let mut has_mouse_move_handler = true;

            loop {
                if let Ok(msg) = rx.recv() {
                    match msg {
                        Message::Quit(param) => {
                            let mut errors_present = false;

                            if let Ok(handler) = lua_ctx.globals().get::<_, Function>("on_quit") {
                                handler.call::<_, ()>(param).unwrap_or_else(|e| {
                                    error!(
                                        "Lua error in file {}: {}\n\t{:?}",
                                        file.to_string_lossy(),
                                        e,
                                        e.source().unwrap_or(&UnknownError {})
                                    );
                                    errors_present = true;
                                });
                            }

                            *crate::UPCALL_COMPLETED_ON_QUIT.0.lock() -= 1;
                            crate::UPCALL_COMPLETED_ON_QUIT.1.notify_all();

                            if errors_present {
                                return Ok(RunScriptResult::TerminatedWithErrors);
                            }
                        }

                        Message::Tick(param) => {
                            if has_tick_handler {
                                let mut errors_present = false;

                                if let Ok(handler) = lua_ctx.globals().get::<_, Function>("on_tick")
                                {
                                    handler.call::<_, ()>(param).unwrap_or_else(|e| {
                                        error!(
                                            "Lua error in file {}: {}\n\t{:?}",
                                            file.to_string_lossy(),
                                            e,
                                            e.source().unwrap_or(&UnknownError {})
                                        );
                                        errors_present = true;
                                    })
                                } else {
                                    has_tick_handler = false;
                                }

                                if errors_present {
                                    return Ok(RunScriptResult::TerminatedWithErrors);
                                }
                            }
                        }

                        Message::RealizeColorMap => {
                            if LOCAL_LED_MAP_MODIFIED.with(|f| *f.borrow()) {
                                LOCAL_LED_MAP.with(|foreground| {
                                        for (idx, background) in LED_MAP.write().iter_mut().enumerate() {
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
                            }

                            // signal readiness / notify the main thread that we are done
                            let val = { *crate::COLOR_MAPS_READY_CONDITION.0.lock() };

                            let val = val.checked_sub(1).unwrap_or_else(|| {
                                warn!("Incorrect state in locking code detected");
                                0
                            });

                            *crate::COLOR_MAPS_READY_CONDITION.0.lock() = val;

                            crate::COLOR_MAPS_READY_CONDITION.1.notify_one();
                        }

                        Message::KeyDown(param) => {
                            let mut errors_present = false;

                            if let Ok(handler) = lua_ctx.globals().get::<_, Function>("on_key_down")
                            {
                                handler.call::<_, ()>(param).unwrap_or_else(|e| {
                                    error!(
                                        "Lua error in file {}: {}\n\t{:?}",
                                        file.to_string_lossy(),
                                        e,
                                        e.source().unwrap_or(&UnknownError {})
                                    );
                                    errors_present = true;
                                });
                            }

                            *crate::UPCALL_COMPLETED_ON_KEY_DOWN.0.lock() -= 1;
                            crate::UPCALL_COMPLETED_ON_KEY_DOWN.1.notify_all();

                            if errors_present {
                                return Ok(RunScriptResult::TerminatedWithErrors);
                            }
                        }

                        Message::KeyUp(param) => {
                            let mut errors_present = false;

                            if let Ok(handler) = lua_ctx.globals().get::<_, Function>("on_key_up") {
                                handler.call::<_, ()>(param).unwrap_or_else(|e| {
                                    error!(
                                        "Lua error in file {}: {}\n\t{:?}",
                                        file.to_string_lossy(),
                                        e,
                                        e.source().unwrap_or(&UnknownError {})
                                    );
                                    errors_present = true;
                                });
                            }

                            *crate::UPCALL_COMPLETED_ON_KEY_UP.0.lock() -= 1;
                            crate::UPCALL_COMPLETED_ON_KEY_UP.1.notify_all();

                            if errors_present {
                                return Ok(RunScriptResult::TerminatedWithErrors);
                            }
                        }

                        Message::KeyboardHidEvent(param) => {
                            let mut errors_present = false;

                            if let Ok(handler) =
                                lua_ctx.globals().get::<_, Function>("on_hid_event")
                            {
                                let arg1: u8;
                                let event_type: u32 = match param {
                                    KeyboardHidEvent::KeyUp { code } => {
                                        arg1 = keyboard_devices[0]
                                            .read()
                                            .hid_event_code_to_report(&code);
                                        1
                                    }

                                    KeyboardHidEvent::KeyDown { code } => {
                                        arg1 = keyboard_devices[0]
                                            .read()
                                            .hid_event_code_to_report(&code);
                                        2
                                    }

                                    KeyboardHidEvent::MuteDown => {
                                        arg1 = 1;
                                        3
                                    }

                                    KeyboardHidEvent::MuteUp => {
                                        arg1 = 0;
                                        3
                                    }

                                    KeyboardHidEvent::VolumeDown => {
                                        arg1 = 1;
                                        4
                                    }

                                    KeyboardHidEvent::VolumeUp => {
                                        arg1 = 0;
                                        4
                                    }

                                    KeyboardHidEvent::BrightnessDown => {
                                        arg1 = 1;
                                        5
                                    }

                                    KeyboardHidEvent::BrightnessUp => {
                                        arg1 = 0;
                                        5
                                    }

                                    KeyboardHidEvent::SetBrightness(val) => {
                                        arg1 = val;
                                        6
                                    }

                                    KeyboardHidEvent::NextSlot => {
                                        arg1 = 1;
                                        7
                                    }

                                    KeyboardHidEvent::PreviousSlot => {
                                        arg1 = 0;
                                        7
                                    }

                                    _ => {
                                        arg1 = 0;
                                        0
                                    }
                                };

                                handler
                                    .call::<_, ()>((event_type, arg1))
                                    .unwrap_or_else(|e| {
                                        error!(
                                            "Lua error in file {}: {}\n\t{:?}",
                                            file.to_string_lossy(),
                                            e,
                                            e.source().unwrap_or(&UnknownError {})
                                        );
                                        errors_present = true;
                                    });
                            }

                            *crate::UPCALL_COMPLETED_ON_KEYBOARD_HID_EVENT.0.lock() -= 1;
                            crate::UPCALL_COMPLETED_ON_KEYBOARD_HID_EVENT.1.notify_all();

                            if errors_present {
                                return Ok(RunScriptResult::TerminatedWithErrors);
                            }
                        }

                        Message::MouseHidEvent(param) => {
                            let mut errors_present = false;

                            if let Ok(handler) =
                                lua_ctx.globals().get::<_, Function>("on_mouse_hid_event")
                            {
                                let arg1: u8;
                                let event_type: u32 = match param {
                                    MouseHidEvent::DpiChange(dpi_slot) => {
                                        arg1 = dpi_slot;
                                        1
                                    }

                                    MouseHidEvent::ButtonDown(index) => {
                                        arg1 = index + 1;
                                        2
                                    }

                                    MouseHidEvent::ButtonUp(index) => {
                                        arg1 = index + 1;
                                        3
                                    }

                                    _ => {
                                        arg1 = 0;
                                        0
                                    }
                                };

                                handler
                                    .call::<_, ()>((event_type, arg1))
                                    .unwrap_or_else(|e| {
                                        error!(
                                            "Lua error in file {}: {}\n\t{:?}",
                                            file.to_string_lossy(),
                                            e,
                                            e.source().unwrap_or(&UnknownError {})
                                        );
                                        errors_present = true;
                                    });
                            }

                            *crate::UPCALL_COMPLETED_ON_MOUSE_HID_EVENT.0.lock() -= 1;
                            crate::UPCALL_COMPLETED_ON_MOUSE_HID_EVENT.1.notify_all();

                            if errors_present {
                                return Ok(RunScriptResult::TerminatedWithErrors);
                            }
                        }

                        Message::MouseButtonDown(param) => {
                            let mut errors_present = false;

                            if let Ok(handler) =
                                lua_ctx.globals().get::<_, Function>("on_mouse_button_down")
                            {
                                handler.call::<_, ()>(param).unwrap_or_else(|e| {
                                    error!(
                                        "Lua error in file {}: {}\n\t{:?}",
                                        file.to_string_lossy(),
                                        e,
                                        e.source().unwrap_or(&UnknownError {})
                                    );
                                    errors_present = true;
                                });
                            }

                            *crate::UPCALL_COMPLETED_ON_MOUSE_BUTTON_DOWN.0.lock() -= 1;
                            crate::UPCALL_COMPLETED_ON_MOUSE_BUTTON_DOWN.1.notify_all();

                            if errors_present {
                                return Ok(RunScriptResult::TerminatedWithErrors);
                            }
                        }

                        Message::MouseButtonUp(param) => {
                            let mut errors_present = false;

                            if let Ok(handler) =
                                lua_ctx.globals().get::<_, Function>("on_mouse_button_up")
                            {
                                handler.call::<_, ()>(param).unwrap_or_else(|e| {
                                    error!(
                                        "Lua error in file {}: {}\n\t{:?}",
                                        file.to_string_lossy(),
                                        e,
                                        e.source().unwrap_or(&UnknownError {})
                                    );
                                    errors_present = true;
                                });
                            }

                            *crate::UPCALL_COMPLETED_ON_MOUSE_BUTTON_UP.0.lock() -= 1;
                            crate::UPCALL_COMPLETED_ON_MOUSE_BUTTON_UP.1.notify_all();

                            if errors_present {
                                return Ok(RunScriptResult::TerminatedWithErrors);
                            }
                        }

                        Message::MouseMove(rel_x, rel_y, rel_z) => {
                            if has_mouse_move_handler {
                                let mut errors_present = false;

                                if let Ok(handler) =
                                    lua_ctx.globals().get::<_, Function>("on_mouse_move")
                                {
                                    handler.call::<_, ()>((rel_x, rel_y, rel_z)).unwrap_or_else(
                                        |e| {
                                            error!(
                                                "Lua error in file {}: {}\n\t{:?}",
                                                file.to_string_lossy(),
                                                e,
                                                e.source().unwrap_or(&UnknownError {})
                                            );
                                            errors_present = true;
                                        },
                                    );
                                } else {
                                    has_mouse_move_handler = false;
                                }
                            }

                            *crate::UPCALL_COMPLETED_ON_MOUSE_MOVE.0.lock() -= 1;
                            crate::UPCALL_COMPLETED_ON_MOUSE_MOVE.1.notify_all();

                            if errors_present {
                                return Ok(RunScriptResult::TerminatedWithErrors);
                            }
                        }

                        Message::MouseWheelEvent(param) => {
                            let mut errors_present = false;

                            if let Ok(handler) =
                                lua_ctx.globals().get::<_, Function>("on_mouse_wheel")
                            {
                                handler.call::<_, ()>(param).unwrap_or_else(|e| {
                                    error!(
                                        "Lua error in file {}: {}\n\t{:?}",
                                        file.to_string_lossy(),
                                        e,
                                        e.source().unwrap_or(&UnknownError {})
                                    );
                                    errors_present = true;
                                });
                            }

                            *crate::UPCALL_COMPLETED_ON_MOUSE_EVENT.0.lock() -= 1;
                            crate::UPCALL_COMPLETED_ON_MOUSE_EVENT.1.notify_all();

                            if errors_present {
                                return Ok(RunScriptResult::TerminatedWithErrors);
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

                            if let Ok(handler) = lua_ctx.globals().get::<_, Function>("on_quit") {
                                handler.call::<_, ()>(()).unwrap_or_else(|e| {
                                    error!(
                                        "Lua error in file {}: {}\n\t{:?}",
                                        file.to_string_lossy(),
                                        e,
                                        e.source().unwrap_or(&UnknownError {})
                                    );
                                    errors_present = true;
                                })
                            }

                            if errors_present {
                                error!(
                                    "Lua script '{}' terminated with errors",
                                    file.file_name().unwrap().to_string_lossy()
                                );
                                return Ok(RunScriptResult::TerminatedWithErrors);
                            } else {
                                debug!(
                                    "Lua script '{}' terminated gracefully",
                                    file.file_name().unwrap().to_string_lossy()
                                );
                                return Ok(RunScriptResult::TerminatedGracefully);
                            }
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

    #[cfg(debug_assertions)]
    lua_ctx
        .load("package.path = package.path .. ';eruption/src/scripts/lib/?;eruption/src/scripts/lib/?.lua'")
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

fn register_support_funcs(lua_ctx: &Lua) -> mlua::Result<()> {
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

    // eruption engine status
    let get_target_fps = lua_ctx.create_function(|_, ()| Ok(callbacks::get_target_fps()))?;
    globals.set("get_target_fps", get_target_fps)?;

    let get_support_script_files =
        lua_ctx.create_function(|_, ()| Ok(callbacks::get_support_script_files()))?;
    globals.set("get_support_script_files", get_support_script_files)?;

    // canvas related functions
    let get_canvas_size = lua_ctx.create_function(|_, ()| Ok(callbacks::get_canvas_size()))?;
    globals.set("get_canvas_size", get_canvas_size)?;

    let get_canvas_width = lua_ctx.create_function(|_, ()| Ok(callbacks::get_canvas_width()))?;
    globals.set("get_canvas_width", get_canvas_width)?;

    let get_canvas_height = lua_ctx.create_function(|_, ()| Ok(callbacks::get_canvas_height()))?;
    globals.set("get_canvas_height", get_canvas_height)?;

    // math library
    let max = lua_ctx.create_function(|_, (f1, f2): (f64, f64)| Ok(f1.max(f2)))?;
    globals.set("max", max)?;

    let min = lua_ctx.create_function(|_, (f1, f2): (f64, f64)| Ok(f1.min(f2)))?;
    globals.set("min", min)?;

    let clamp = lua_ctx
        .create_function(|_, (val, f1, f2): (f64, f64, f64)| Ok(callbacks::clamp(val, f1, f2)))?;
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

    let asin = lua_ctx.create_function(|_, a: f64| Ok(a.asin()))?;
    globals.set("asin", asin)?;

    let atan2 = lua_ctx.create_function(|_, (y, x): (f64, f64)| Ok(y.atan2(x)))?;
    globals.set("atan2", atan2)?;

    let ceil = lua_ctx.create_function(|_, f: f64| Ok(f.ceil()))?;
    globals.set("ceil", ceil)?;

    let floor = lua_ctx.create_function(|_, f: f64| Ok(f.floor()))?;
    globals.set("floor", floor)?;

    let round = lua_ctx.create_function(|_, f: f64| Ok(f.round()))?;
    globals.set("round", round)?;

    let rand =
        lua_ctx.create_function(|_, (l, h): (i64, i64)| Ok(rand::thread_rng().gen_range(l..h)))?;
    globals.set("rand", rand)?;

    let trunc = lua_ctx.create_function(|_, f: f64| Ok(f.trunc() as i64))?;
    globals.set("trunc", trunc)?;

    // let lerp = lua_ctx.create_function(|_, (a0, a1, w): (f64, f64, f64)| Ok((1.0 - w) * a0 + w * a1))?; // precise version
    let lerp = lua_ctx.create_function(|_, (v0, v1, t): (f64, f64, f64)| Ok(v0 + t * (v1 - v0)))?;
    globals.set("lerp", lerp)?;

    let invlerp = lua_ctx.create_function(|_, (v0, v1, t): (f64, f64, f64)| {
        Ok(callbacks::clamp((t - v0) / (v0 - v1), 0.0, 1.0))
    })?;
    globals.set("invlerp", invlerp)?;

    let range = lua_ctx.create_function(|_, (v0, v1, v2, v3, t): (f64, f64, f64, f64, f64)| {
        Ok(v2 + callbacks::clamp((t - v0) / (v1 - v0), 0.0, 1.0) * (v3 - v2))
    })?;
    globals.set("range", range)?;

    // keyboard state and macros
    let inject_key = lua_ctx.create_function(|_, (ev_key, down): (u32, bool)| {
        callbacks::inject_key(ev_key, down);
        Ok(())
    })?;
    globals.set("inject_key", inject_key)?;

    let inject_key_with_delay =
        lua_ctx.create_function(|_, (ev_key, down, millis): (u32, bool, u64)| {
            callbacks::inject_key_with_delay(ev_key, down, millis);
            Ok(())
        })?;
    globals.set("inject_key_with_delay", inject_key_with_delay)?;

    // mouse state and macros
    let inject_mouse_button = lua_ctx.create_function(|_, (button_index, down): (u32, bool)| {
        callbacks::inject_mouse_button(button_index, down);
        Ok(())
    })?;
    globals.set("inject_mouse_button", inject_mouse_button)?;

    let inject_mouse_wheel = lua_ctx.create_function(|_, direction: u32| {
        callbacks::inject_mouse_wheel(direction);
        Ok(())
    })?;
    globals.set("inject_mouse_wheel", inject_mouse_wheel)?;

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

    let parse_color = lua_ctx.create_function(|_, val: String| {
        callbacks::parse_color(&val)
            .map_err(|_e| LuaError::ExternalError(Arc::new(CallbacksError::ParseParamError {})))
    })?;
    globals.set("parse_color", parse_color)?;

    let gradient_from_name = lua_ctx.create_function(|_, name: String| {
        callbacks::gradient_from_name(&name)
            .map_err(|_e| LuaError::ExternalError(Arc::new(CallbacksError::ParseParamError {})))
    })?;
    globals.set("gradient_from_name", gradient_from_name)?;

    let gradient_destroy = lua_ctx.create_function(|_, handle: usize| {
        callbacks::gradient_destroy(handle)
            .map_err(|_e| LuaError::ExternalError(Arc::new(CallbacksError::ParseParamError {})))
    })?;
    globals.set("gradient_destroy", gradient_destroy)?;

    let gradient_color_at = lua_ctx.create_function(|_, (handle, pos): (usize, f64)| {
        Ok(callbacks::gradient_color_at(handle, pos).unwrap_or(0))
    })?;
    globals.set("gradient_color_at", gradient_color_at)?;

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

    let checkerboard_noise = lua_ctx.create_function(|_, (f1, f2, f3): (f64, f64, f64)| {
        Ok(callbacks::checkerboard_noise_3d(f1, f2, f3))
    })?;
    globals.set("checkerboard_noise", checkerboard_noise)?;

    // transformation utilities
    let rotate = lua_ctx.create_function(|_, (map, theta): (Vec<u32>, f64)| {
        Ok(callbacks::rotate(&map, theta, (22, 6)))
    })?;
    globals.set("rotate", rotate)?;

    // device related
    let get_num_keys = lua_ctx.create_function(move |_, ()| Ok(callbacks::get_num_keys()))?;
    globals.set("get_num_keys", get_num_keys)?;

    let get_color_map = lua_ctx.create_function(move |_, ()| Ok(callbacks::get_color_map()))?;
    globals.set("get_color_map", get_color_map)?;

    let submit_color_map = lua_ctx.create_function(move |_, map: Vec<u32>| {
        callbacks::submit_color_map(&map)
            .map_err(|_e| LuaError::ExternalError(Arc::new(ScriptingError::ValueError {})))
    })?;
    globals.set("submit_color_map", submit_color_map)?;

    let get_brightness = lua_ctx.create_function(move |_, ()| Ok(callbacks::get_brightness()))?;
    globals.set("get_brightness", get_brightness)?;

    let set_brightness = lua_ctx.create_function(move |_, val: isize| {
        callbacks::set_brightness(val);
        Ok(())
    })?;
    globals.set("set_brightness", set_brightness)?;

    // finally, register Lua functions supplied by eruption plugins
    let plugin_manager = plugin_manager::PLUGIN_MANAGER.read();
    let plugins = plugin_manager.get_plugins();

    for plugin in plugins.iter() {
        plugin.register_lua_funcs(&lua_ctx).unwrap();
    }

    Ok(())
}

fn register_script_config(lua_ctx: &Lua, manifest: &Manifest) -> mlua::Result<()> {
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
                        if let Some(val) = profile.get_string_value(script_name, name) {
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
