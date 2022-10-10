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

use lazy_static::lazy_static;
// use log::*;
use mlua::prelude::*;
use parking_lot::RwLock;
use pixels::wgpu::{PowerPreference, RequestAdapterOptions};
use pixels::Pixels;
use pixels::{raw_window_handle, PixelsBuilder};
use std::collections::HashMap;
use std::sync::Arc;
use std::{any::Any, sync::atomic::Ordering};

use crate::plugins::{self, Plugin};

pub type Result<T> = std::result::Result<T, eyre::Error>;

const WIDTH: u32 = 1024;
const HEIGHT: u32 = 768;

// #[derive(Debug, Fail)]
// pub enum HwAccelerationPluginError {
//     #[error("Unknown error: {}", description)]
//     UnknownError { description: String },
// }

lazy_static! {
    pub static ref PIXELS: Arc<RwLock<Option<Pixels>>> = Arc::new(RwLock::new(None));
}

/// A plugin that provides access to hardware acceleration APIs
pub struct HwAccelerationPlugin {}

impl HwAccelerationPlugin {
    pub fn new() -> Self {
        HwAccelerationPlugin {}
    }

    pub fn initialize_hwaccel() -> Result<()> {
        let window = Rwh;

        let surface_texture = pixels::SurfaceTexture::new(WIDTH, HEIGHT, &window);

        let mut pixels = PixelsBuilder::new(WIDTH, HEIGHT, surface_texture)
            .request_adapter_options(RequestAdapterOptions {
                power_preference: PowerPreference::HighPerformance,
                force_fallback_adapter: false,
                compatible_surface: None,
            })
            // .wgpu_backend(Backends::VULKAN)
            // .enable_vsync(false)
            .build()?;

        // Set clear color to red.
        // pixels.set_clear_color(Color::RED);

        // let result = pixels.render();

        // *PIXELS.write() = Some(pixels);

        Ok(())
    }

    pub fn query_hw_accel_info() -> HashMap<String, String> {
        let mut result = HashMap::new();

        if let Some(pixels) = &*PIXELS.read() {
            let _features = pixels.device().features();

            icecream::ice!(pixels.device());
        } else {
        }

        if crate::EXPERIMENTAL_FEATURES.load(Ordering::SeqCst) {
            result.insert("backend".to_string(), "vulkan".to_string());
            result.insert("hardware-acceleration".to_string(), "true".to_string());
        } else {
            result.insert("backend".to_string(), "disabled".to_string());
            result.insert("hardware-acceleration".to_string(), "false".to_string());
        }

        result
    }

    // pub fn compile_shader_program() -> Result<()> {
    //     Ok(())
    // }

    // pub fn set_uniform_value(_value: u32) {}

    // pub fn get_uniform_value() -> u32 {
    //     0
    // }
}

#[async_trait::async_trait]
impl Plugin for HwAccelerationPlugin {
    fn get_name(&self) -> String {
        "Hardware Acceleration".to_string()
    }

    fn get_description(&self) -> String {
        "Hardware accelerated effects".to_string()
    }

    fn initialize(&mut self) -> plugins::Result<()> {
        if crate::EXPERIMENTAL_FEATURES.load(Ordering::SeqCst) {
            Self::initialize_hwaccel()?;
        }

        Ok(())
    }

    fn register_lua_funcs(&self, lua_ctx: &Lua) -> mlua::Result<()> {
        let globals = lua_ctx.globals();

        let query_hw_accel_info = lua_ctx
            .create_function(move |_, ()| Ok(HwAccelerationPlugin::query_hw_accel_info()))?;
        globals.set("query_hw_accel_info", query_hw_accel_info)?;

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

pub struct Rwh;

unsafe impl raw_window_handle::HasRawWindowHandle for Rwh {
    fn raw_window_handle(&self) -> raw_window_handle::RawWindowHandle {
        #[cfg(target_os = "macos")]
        return raw_window_handle::RawWindowHandle::AppKit(raw_window_handle::AppKitHandle::empty());
        #[cfg(any(
            target_os = "linux",
            target_os = "dragonfly",
            target_os = "freebsd",
            target_os = "netbsd",
            target_os = "openbsd",
        ))]
        return raw_window_handle::RawWindowHandle::Xlib(
            // raw_window_handle::WaylandHandle::empty(),
            raw_window_handle::XlibHandle::empty(),
        );
        #[cfg(target_os = "windows")]
        return raw_window_handle::RawWindowHandle::Win32(raw_window_handle::Win32Handle::empty());
        #[cfg(target_os = "ios")]
        return raw_window_handle::RawWindowHandle::UiKit(raw_window_handle::UiKitHandle::empty());
    }
}
