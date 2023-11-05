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

use lazy_static::lazy_static;
use std::collections::{BTreeMap, HashMap};
use std::path::Path;
use std::sync::Arc;
use std::{any::Any, sync::atomic::Ordering};
use tracing_mutex::stdsync::RwLock;

use gpgpu::*;
use mlua::prelude::*;
use mlua::Lua;

use crate::constants;
use crate::plugins::{self, Plugin};

pub type Result<T> = std::result::Result<T, eyre::Error>;

// const WIDTH: u32 = 1920;
// const HEIGHT: u32 = 1080;

#[derive(Debug, thiserror::Error)]
pub enum HwAccelerationPluginError {
    #[error("Initialization failed: {}", description)]
    InitError { description: String },

    // #[error("No devices found")]
    // NoDevicesFound,

    // #[error("Compilation failed: {}", description)]
    // ShaderCompilationError { description: String },
    #[error("Invalid shader specified")]
    InvalidShader {},
    // #[error("Unknown error: {}", description)]
    // UnknownError { description: String },
}

lazy_static! {
    /// GPGPU framework handle
    pub static ref FRAMEWORK: gpgpu::Framework = gpgpu::Framework::default();

    /// Loaded SPIR-V shader programs and their associated state
    pub static ref SHADERS: Arc<RwLock<HashMap<usize, Box<ShaderState<'static>>>>> =
        Arc::new(RwLock::new(HashMap::new()));
}

pub struct ShaderState<'fw> {
    pub shader: Box<Shader>,
    pub uniforms: HashMap<String, UniformVariant>,
    pub input_buffers: Vec<Box<GpuBuffer<'fw, u8>>>,
    pub output_buffer: Box<GpuBuffer<'fw, u32>>,
}

impl<'fw> ShaderState<'fw> {
    pub fn new(shader: Shader) -> Self {
        let fw = &FRAMEWORK;

        Self {
            shader: Box::new(shader),
            uniforms: HashMap::new(),
            input_buffers: vec![],
            output_buffer: Box::new(GpuBuffer::with_capacity(&fw, constants::CANVAS_SIZE as u64)),
        }
    }

    #[allow(dead_code)]
    pub fn set_uniform(&mut self, name: String, value: UniformVariant) -> Option<UniformVariant> {
        self.uniforms.insert(name, value)
    }
}

#[allow(dead_code)]
pub enum UniformVariant {
    F32(f32),
}

/// A plugin that provides access to hardware acceleration APIs
pub struct HwAccelPlugin {}

impl HwAccelPlugin {
    pub fn new() -> Self {
        HwAccelPlugin {}
    }

    pub fn initialize_hwaccel() -> Result<()> {
        Ok(())
    }

    pub fn hwaccel_status() -> BTreeMap<String, String> {
        let mut result = BTreeMap::new();

        if crate::EXPERIMENTAL_FEATURES.load(Ordering::SeqCst) {
            // let state = HWACCEL_STATE.read().unwrap();
            // let state = state.as_ref().unwrap();

            let device_name = "<unknown>".to_string();

            result.insert("device".to_string(), device_name);
            result.insert("backend".to_string(), "Vulkan".to_string());
            result.insert("acceleration-available".to_string(), "true".to_string());
        } else {
            result.insert("device".to_string(), "none".to_string());
            result.insert("backend".to_string(), "disabled".to_string());
            result.insert("acceleration-available".to_string(), "false".to_string());
        }

        result
    }

    pub fn render(shader: usize) -> Result<()> {
        match SHADERS.read().unwrap().get(&shader) {
            Some(shader) => {
                let fw = &FRAMEWORK;

                // GPU buffer creation
                let buf1 = GpuBuffer::<u8>::with_capacity(&fw, constants::CANVAS_SIZE as u64); // Input
                let buf2 = GpuBuffer::<u8>::with_capacity(&fw, constants::CANVAS_SIZE as u64); // Input
                let output = GpuBuffer::<u32>::with_capacity(&fw, constants::CANVAS_SIZE as u64); // Output

                // Descriptor set and program creation
                let desc = DescriptorSet::default()
                    .bind_buffer(&buf1, GpuBufferUsage::ReadOnly)
                    .bind_buffer(&buf2, GpuBufferUsage::ReadOnly)
                    .bind_buffer(&output, GpuBufferUsage::ReadWrite);

                let program =
                    Box::new(Program::new(&shader.shader, "main").add_descriptor_set(desc));

                // Kernel creation and enqueuing
                Kernel::new(&fw, *program).enqueue(constants::CANVAS_SIZE as u32, 1, 1);

                Ok(())
            }

            None => Err(HwAccelerationPluginError::InvalidShader {}.into()),
        }
    }

    #[allow(dead_code)]
    pub fn set_uniform(_shader: usize, _name: &str) -> Result<()> {
        Ok(())
    }

    /// Load a shader program from a SPIR-V binary file
    #[allow(dead_code)]
    pub fn load_shader<P: AsRef<Path>>(shader_path: P) -> Result<usize> {
        let fw = &FRAMEWORK;

        let shader = Shader::from_spirv_file(&fw, &shader_path)?;
        let shader_state = ShaderState::new(shader);

        let index = SHADERS.read().unwrap().len();
        SHADERS
            .write()
            .unwrap()
            .insert(index, Box::new(shader_state));

        Ok(index)
    }

    /// Compiles GLSL or WGSL shader code to SPIR-V, and load the binary shader program
    #[allow(dead_code)]
    fn compile_shader<P: AsRef<Path>>(shader_path: P) -> Result<Shader> {
        let compiler = shaderc::Compiler::new().unwrap();
        let mut options = shaderc::CompileOptions::new().unwrap();

        options.add_macro_definition("EP", Some("main"));

        let shader_source = std::fs::read_to_string(shader_path.as_ref())?;

        let binary_result = compiler.compile_into_spirv(
            &shader_source,
            shaderc::ShaderKind::Compute,
            shader_path.as_ref().to_str().unwrap(),
            "main",
            Some(&options),
        )?;

        let fw = &FRAMEWORK;

        let shader = Shader::from_spirv_bytes(
            &fw,
            binary_result.as_binary_u8(),
            Some(
                &shader_path
                    .as_ref()
                    .file_name()
                    .map(|s| s.to_string_lossy().into_owned())
                    .unwrap_or_else(|| "<unknown>".to_string()),
            ),
        );

        Ok(shader)
    }
}

impl Plugin for HwAccelPlugin {
    fn get_name(&self) -> String {
        "Hwaccel".to_string()
    }

    fn get_description(&self) -> String {
        "Hardware accelerated effects using WGSL and GLSL shader-programs".to_string()
    }

    fn initialize(&mut self) -> plugins::Result<()> {
        if crate::EXPERIMENTAL_FEATURES.load(Ordering::SeqCst) {
            Self::initialize_hwaccel().map_err(|e| HwAccelerationPluginError::InitError {
                description: e.to_string(),
            })?;
        }

        Ok(())
    }

    fn register_lua_funcs(&self, lua_ctx: &Lua) -> mlua::Result<()> {
        let globals = lua_ctx.globals();

        let hwaccel_status =
            lua_ctx.create_function(move |_, ()| Ok(HwAccelPlugin::hwaccel_status()))?;
        globals.set("hwaccel_status", hwaccel_status)?;

        let hwaccel_render = lua_ctx.create_function(move |_, shader: usize| {
            let result = HwAccelPlugin::render(shader)
                .map_err(|e: eyre::Error| LuaError::RuntimeError(format!("{e}")))?;

            Ok(result)
        })?;
        globals.set("hwaccel_render", hwaccel_render)?;

        let set_uniform = lua_ctx.create_function(move |_, (shader, name): (usize, String)| {
            let result = HwAccelPlugin::set_uniform(shader, &name)
                .map_err(|e: eyre::Error| LuaError::RuntimeError(format!("{e}")))?;

            Ok(result)
        })?;
        globals.set("set_uniform", set_uniform)?;

        Ok(())
    }

    fn sync_main_loop_hook(&self, _ticks: u64) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
