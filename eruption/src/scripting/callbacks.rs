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

/// These functions are intended to be used from within Lua scripts
use byteorder::{ByteOrder, LittleEndian};
use log::*;
use mlua::prelude::*;
use noise::NoiseFn;
use palette::convert::FromColor;
use palette::{Hsl, Srgb};
use rand::Rng;
use std::convert::TryFrom;
use std::fmt::Write;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Duration;
use std::vec::Vec;
use std::{cell::RefCell, thread};

use crate::{
    constants,
    hwdevices::RGBA,
    plugin_manager,
    plugins::macros,
    script::ScriptingError,
    script::{
        ALLOCATED_GRADIENTS, FRAME_GENERATION_COUNTER, LED_MAP, LOCAL_LED_MAP,
        LOCAL_LED_MAP_MODIFIED,
    },
    scripting::callbacks,
};

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
    let since_the_epoch = start.duration_since(UNIX_EPOCH).unwrap_or_default();

    since_the_epoch.as_millis() as u32
}

thread_local! {
    pub static PERLIN_NOISE: RefCell<noise::Perlin> = {
        let noise = noise::Perlin::new(seed());

        RefCell::new(noise)
    };

    pub static BILLOW_NOISE: RefCell<noise::Billow<noise::Perlin>> = {
        let noise = noise::Billow::new(seed());

        RefCell::new(noise)
    };

    pub static VORONOI_NOISE: RefCell<noise::Worley> = {
        let noise = noise::Worley::new(seed());

        RefCell::new(noise)
    };

    pub static RIDGED_MULTIFRACTAL_NOISE: RefCell<noise::RidgedMulti<noise::Perlin>> = {
        let noise = noise::RidgedMulti::new(seed());

        // noise.octaves = 6;
        // noise.frequency = 2.0; // default: 1.0
        // noise.lacunarity = std::f64::consts::PI * 2.0 / 3.0;
        // noise.persistence = 1.0;
        // noise.attenuation = 4.0; // default: 2.0

        RefCell::new(noise)
    };

    pub static FBM_NOISE: RefCell<noise::Fbm<noise::Perlin>> = {
        let noise = noise::Fbm::new(seed());

        RefCell::new(noise)
    };

    pub static OPEN_SIMPLEX_NOISE: RefCell<noise::OpenSimplex> = {
        let noise = noise::OpenSimplex::new(seed());

        RefCell::new(noise)
    };

    pub static SUPER_SIMPLEX_NOISE: RefCell<noise::SuperSimplex> = {
        let noise = noise::SuperSimplex::new(seed());

        RefCell::new(noise)
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

pub(crate) fn stringify(mut string: &mut String, value: mlua::Value) -> Option<()> {
    match value {
        LuaValue::Nil => write!(&mut string, "{}", "nil"),
        LuaValue::Boolean(b) => write!(string, "{}", b),
        LuaValue::LightUserData(lud) => write!(string, "[lightuserdata, {:?}]", lud),
        LuaValue::Integer(i) => write!(string, "{}", i),
        LuaValue::Number(f) => write!(string, "{}", f),
        LuaValue::String(s) => write!(string, "{}", s.to_string_lossy()),
        LuaValue::Function(f) => write!(string, "[function, {:?}]", f.info()),
        LuaValue::Thread(t) => write!(string, "[thread, {:?}]", t.status()),
        LuaValue::UserData(ud) => write!(string, "[userdata, {:?}]", ud),
        LuaValue::Error(e) => write!(string, "{}", e),
        LuaValue::Table(t) => {
            write!(string, "{{ ").unwrap();
            for pair in t.pairs::<mlua::Value, mlua::Value>() {
                // (Fingers crossed that the user doesn't cyclically nest tables.)
                let (key, value) = pair.unwrap();
                stringify(string, key)?;
                write!(string, " = ").unwrap();
                stringify(string, value)?;
                write!(string, ", ").unwrap();
            }
            write!(string, "}}")
        }
    }
    .ok()
}

/// Returns the target framerate
pub(crate) fn get_target_fps() -> u64 {
    constants::TARGET_FPS
}

/// Returns the Lua support scripts for all connected devices
pub(crate) fn get_support_script_files() -> Vec<String> {
    let mut result = Vec::new();

    for device in crate::KEYBOARD_DEVICES.read().iter() {
        result.push(device.read().get_support_script_file());
    }

    for device in crate::MOUSE_DEVICES.read().iter() {
        result.push(device.read().get_support_script_file());
    }

    for device in crate::MISC_DEVICES.read().iter() {
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
        .read()
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
        .read()
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
        .read()
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
                .read()
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
    let rgb = Srgb::from_components(((r as f64 / 255.0), (g as f64 / 255.0), (b as f64 / 255.0)))
        .into_linear();

    let (h, s, l) = Hsl::from_color(rgb).into_components();

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
    let rgb = Srgb::from_color(Hsl::new(h, s, l)).into_linear();
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
    let rgb = Srgb::from_color(Hsl::new(h, s, l)).into_linear();
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
    match csscolorparser::parse(val) {
        Ok(color) => {
            let (r, g, b, a) = color.to_linear_rgba_u8();

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
        "rainbow-smooth" => ALLOCATED_GRADIENTS.with(|f| {
            let mut m = f.borrow_mut();
            let idx = m.len() + 1;

            let gradient = colorgrad::rainbow();
            m.insert(idx, gradient);

            Ok(idx)
        }),

        "sinebow-smooth" => ALLOCATED_GRADIENTS.with(|f| {
            let mut m = f.borrow_mut();
            let idx = m.len() + 1;

            let gradient = colorgrad::sinebow();
            m.insert(idx, gradient);

            Ok(idx)
        }),

        "spectral-smooth" => ALLOCATED_GRADIENTS.with(|f| {
            let mut m = f.borrow_mut();
            let idx = m.len() + 1;

            let gradient = colorgrad::spectral();
            m.insert(idx, gradient);

            Ok(idx)
        }),

        "rainbow-sharp" => ALLOCATED_GRADIENTS.with(|f| {
            let mut m = f.borrow_mut();
            let idx = m.len() + 1;

            let gradient = colorgrad::rainbow().sharp(5, 0.15);
            m.insert(idx, gradient);

            Ok(idx)
        }),

        "sinebow-sharp" => ALLOCATED_GRADIENTS.with(|f| {
            let mut m = f.borrow_mut();
            let idx = m.len() + 1;

            let gradient = colorgrad::sinebow().sharp(5, 0.15);
            m.insert(idx, gradient);

            Ok(idx)
        }),

        "spectral-sharp" => ALLOCATED_GRADIENTS.with(|f| {
            let mut m = f.borrow_mut();
            let idx = m.len() + 1;

            let gradient = colorgrad::spectral().sharp(5, 0.15);
            m.insert(idx, gradient);

            Ok(idx)
        }),

        // special handling for the "system" palette
        "system" => ALLOCATED_GRADIENTS.with(|f| {
            let mut m = f.borrow_mut();
            let idx = m.len() + 1;

            let gradient = if let Some(color_scheme) = crate::NAMED_COLOR_SCHEMES.read().get(val) {
                colorgrad::CustomGradient::new()
                    // start at index 1, ignore the darkest/black part of the palette
                    .colors(&color_scheme.colors)
                    .build()?
            } else {
                // use sinebow gradient as a fallback
                colorgrad::sinebow()
            };

            m.insert(idx, gradient);

            Ok(idx)
        }),

        _ => {
            if let Some(color_scheme) = crate::NAMED_COLOR_SCHEMES.read().get(val) {
                // Create a gradient from the named color scheme
                ALLOCATED_GRADIENTS.with(|f| {
                    let mut m = f.borrow_mut();
                    let idx = m.len() + 1;

                    let gradient = colorgrad::CustomGradient::new()
                        .colors(&color_scheme.colors)
                        .build()?;

                    m.insert(idx, gradient);

                    Ok(idx)
                })
            } else {
                error!("Could not parse value, not a valid stock-gradient or named color scheme");

                Err(CallbacksError::ParseParamError {}.into())
            }
        }
    }
}

/// De-allocates a gradient from an opaque handle, representing that gradient
pub(crate) fn gradient_destroy(handle: usize) -> Result<()> {
    ALLOCATED_GRADIENTS.with(|f| {
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
    ALLOCATED_GRADIENTS.with(|f| {
        let m = f.borrow();

        if let Some(gradient) = m.get(&handle) {
            let color = gradient.at(pos);
            let (r, g, b, a) = color.to_linear_rgba_u8();

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
    simdnoise::NoiseBuilder::gradient_3d_offset(f1, 2, f2, 2, f3, 2).generate_scaled(0.0, 1.0)[0]
}

/// Compute Turbulence noise (SIMD)
pub(crate) fn turbulence_noise_2d_simd(f1: f32, f2: f32) -> f32 {
    simdnoise::NoiseBuilder::turbulence_2d_offset(f1, 2, f2, 2).generate_scaled(0.0, 1.0)[0]
}

pub(crate) fn turbulence_noise_3d_simd(f1: f32, f2: f32, f3: f32) -> f32 {
    simdnoise::NoiseBuilder::turbulence_3d_offset(f1, 2, f2, 2, f3, 2).generate_scaled(0.0, 1.0)[0]
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
    let devices = crate::KEYBOARD_DEVICES.read();

    if !devices.is_empty() {
        let result = devices[0].read().get_num_keys();
        result
    } else {
        constants::MAX_KEYS
    }
}

/// Get state of all LEDs
pub(crate) fn get_color_map() -> Vec<u32> {
    let global_led_map = LED_MAP.read();

    let result = global_led_map
        .iter()
        .map(|v| {
            ((v.r as u32).overflowing_shl(16).0 + (v.g as u32).overflowing_shl(8).0 + v.b as u32)
                as u32
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

    FRAME_GENERATION_COUNTER.fetch_add(1, Ordering::SeqCst);

    Ok(())
}

pub(crate) fn get_brightness() -> isize {
    crate::BRIGHTNESS.load(Ordering::SeqCst)
}

pub(crate) fn set_brightness(val: isize) {
    crate::BRIGHTNESS.store(val, Ordering::SeqCst);
    FRAME_GENERATION_COUNTER.fetch_add(1, Ordering::SeqCst);
}

pub fn register_support_funcs(lua_ctx: &Lua) -> mlua::Result<()> {
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

    let stringify = lua_ctx.create_function(|_, value: mlua::Value| {
        let mut s = String::new();
        stringify(&mut s, value);
        Ok(s)
    })?;
    globals.set("stringify", stringify)?;

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

    let rand = lua_ctx.create_function(|_, (l, h): (i64, i64)| {
        if h - l > 0 {
            Ok(rand::thread_rng().gen_range(l..h))
        } else {
            Ok(0)
        }
    })?;
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
        plugin.register_lua_funcs(lua_ctx).unwrap();
    }

    Ok(())
}
