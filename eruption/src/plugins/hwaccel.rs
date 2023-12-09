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

use bytemuck::{Pod, Zeroable};
use drm::*;
use lazy_static::lazy_static;
use raw_window_handle::{
    DisplayHandle, DrmDisplayHandle, DrmWindowHandle, HasDisplayHandle, HasRawDisplayHandle,
    HasRawWindowHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle, WindowHandle,
};
use std::collections::{BTreeMap, HashSet};
use std::ffi::{CStr, CString};
use std::fs::{File, OpenOptions};
use std::io;
use std::os::fd::{AsFd, AsRawFd, BorrowedFd};
use std::path::Path;
use std::ptr::{self, null};
use std::sync::Arc;
use std::{any::Any, sync::atomic::Ordering};
use tracing_mutex::stdsync::RwLock;

use mlua::prelude::*;
use mlua::Lua;

use crate::constants;
use crate::plugins::{self, Plugin};

pub type Result<T> = std::result::Result<T, eyre::Error>;

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

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

lazy_static! {}

struct DrmRenderNode {
    file: File,
}

impl DrmRenderNode {
    pub fn open<P: AsRef<Path>>(path: P) -> io::Result<Self> {
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(path.as_ref())?;

        Ok(Self { file })
    }
}

impl drm::Device for DrmRenderNode {}

impl AsFd for DrmRenderNode {
    fn as_fd(&self) -> BorrowedFd<'_> {
        self.file.as_fd()
    }
}

impl HasDisplayHandle for DrmRenderNode {
    fn display_handle(
        &self,
    ) -> std::prelude::v1::Result<DisplayHandle<'_>, raw_window_handle::HandleError> {
        Ok(RawDisplayHandle::Drm(DrmDisplayHandle::new(
            self.file.as_fd().as_raw_fd(),
        )))
    }
}

impl HasWindowHandle for DrmRenderNode {
    fn window_handle(
        &self,
    ) -> std::prelude::v1::Result<WindowHandle, raw_window_handle::HandleError> {
        Ok(RawWindowHandle::Drm(DrmWindowHandle::new(0)))
    }
}

#[derive(Copy, Clone, Pod, Zeroable)]
#[repr(C)]
pub struct ShaderConstants {
    pub width: u32,
    pub height: u32,
    pub time: f32,

    pub cursor_x: f32,
    pub cursor_y: f32,
    pub drag_start_x: f32,
    pub drag_start_y: f32,
    pub drag_end_x: f32,
    pub drag_end_y: f32,

    /// Bit mask of the pressed buttons (0 = Left, 1 = Middle, 2 = Right).
    pub mouse_button_pressed: u32,

    /// The last time each mouse button (Left, Middle or Right) was pressed,
    /// or `f32::NEG_INFINITY` for buttons which haven't been pressed yet.
    ///
    /// If this is the first frame after the press of some button, that button's
    /// entry in `mouse_button_press_time` will exactly equal `time`.
    pub mouse_button_press_time: [f32; 3],
}

/*
pub fn saturate(x: f32) -> f32 {
    x.clamp(0.0, 1.0)
}

pub fn pow(v: Vec3, power: f32) -> Vec3 {
    vec3(v.x.powf(power), v.y.powf(power), v.z.powf(power))
}

pub fn exp(v: Vec3) -> Vec3 {
    vec3(v.x.exp(), v.y.exp(), v.z.exp())
}

/// Based on: <https://seblagarde.wordpress.com/2014/12/01/inverse-trigonometric-functions-gpu-optimization-for-amd-gcn-architecture/>
pub fn acos_approx(v: f32) -> f32 {
    let x = v.abs();
    let mut res = -0.155972 * x + 1.56467; // p(x)
    res *= (1.0f32 - x).sqrt();

    if v >= 0.0 {
        res
    } else {
        PI - res
    }
}

pub fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    // Scale, bias and saturate x to 0..1 range
    let x = saturate((x - edge0) / (edge1 - edge0));
    // Evaluate polynomial
    x * x * (3.0 - 2.0 * x)
}
*/

fn create_pipeline(
    device: &wgpu::Device,
    pipeline_layout: &wgpu::PipelineLayout,
    surface_format: wgpu::TextureFormat,
    compiled_shader_modules: CompiledShaderModules,
) -> wgpu::RenderPipeline {
    // FIXME(eddyb) automate this decision by default.
    let create_module = |module| {
        let wgpu::ShaderModuleDescriptorSpirV { label, source } = module;
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label,
            source: wgpu::ShaderSource::SpirV(source),
        })
    };

    let vs_entry_point = "main_vs";
    let fs_entry_point = "main_fs";

    let vs_module_descr = compiled_shader_modules.spv_module_for_entry_point(vs_entry_point);
    let fs_module_descr = compiled_shader_modules.spv_module_for_entry_point(fs_entry_point);

    // HACK(eddyb) avoid calling `device.create_shader_module` twice unnecessarily.
    let vs_fs_same_module = std::ptr::eq(&vs_module_descr.source[..], &fs_module_descr.source[..]);

    let vs_module = &create_module(vs_module_descr);
    let fs_module;
    let fs_module = if vs_fs_same_module {
        vs_module
    } else {
        fs_module = create_module(fs_module_descr);
        &fs_module
    };

    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
        layout: Some(pipeline_layout),
        vertex: wgpu::VertexState {
            module: vs_module,
            entry_point: vs_entry_point,
            buffers: &[],
        },
        primitive: wgpu::PrimitiveState {
            topology: wgpu::PrimitiveTopology::TriangleList,
            strip_index_format: None,
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: None,
            unclipped_depth: false,
            polygon_mode: wgpu::PolygonMode::Fill,
            conservative: false,
        },
        depth_stencil: None,
        multisample: wgpu::MultisampleState {
            count: 1,
            mask: !0,
            alpha_to_coverage_enabled: false,
        },
        fragment: Some(wgpu::FragmentState {
            module: fs_module,
            entry_point: fs_entry_point,
            targets: &[Some(wgpu::ColorTargetState {
                format: surface_format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview: None,
    })
}

/// A plugin that provides access to hardware acceleration APIs
pub struct HwAccelPlugin {}

impl HwAccelPlugin {
    pub fn new() -> Self {
        HwAccelPlugin {}
    }

    pub async fn initialize_hwaccel() -> Result<()> {
        let backends = wgpu::util::backend_bits_from_env().unwrap_or(wgpu::Backends::VULKAN);

        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends,
            dx12_shader_compiler: wgpu::util::dx12_shader_compiler_from_env().unwrap_or_default(),
            ..Default::default()
        });

        let drm_dev = DrmRenderNode::open("/dev/dri/card1").expect("Render node");
        println!("{:#?}", drm_dev.get_driver().unwrap());

        let initial_surface =
            unsafe { instance.create_surface(&drm_dev) }.expect("Failed to create surface ");

        let adapter = wgpu::util::initialize_adapter_from_env_or_default(
            &instance,
            // Request an adapter which can render to our surface
            Some(initial_surface).as_ref(),
        )
        .await
        .expect("Failed to find an appropriate adapter");

        let mut features = wgpu::Features::PUSH_CONSTANTS;
        // if options.force_spirv_passthru {
        //     features |= wgpu::Features::SPIRV_SHADER_PASSTHROUGH;
        // }

        let limits = wgpu::Limits {
            max_push_constant_size: 128,
            ..Default::default()
        };

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    features,
                    limits,
                },
                None,
            )
            .await
            .expect("Failed to create device");

        let auto_configure_surface = |adapter: &_, device: &_, surface: wgpu::Surface| {
            let mut surface_config = surface
                .get_default_config(adapter, WIDTH, HEIGHT)
                .unwrap_or_else(|| {
                    panic!(
                        "Missing formats/present modes in surface capabilities: {:#?}",
                        surface.get_capabilities(adapter)
                    )
                });

            // FIXME(eddyb) should this be toggled by a CLI arg?
            // NOTE(eddyb) VSync was disabled in the past, but without VSync,
            // especially for simpler shaders, you can easily hit thousands
            // of frames per second, stressing GPUs for no reason.
            surface_config.present_mode = wgpu::PresentMode::AutoVsync;

            surface.configure(device, &surface_config);

            (surface, surface_config)
        };
        let mut surface_with_config =
            initial_surface.map(|surface| auto_configure_surface(&adapter, &device, surface));

        // Load the shaders from disk

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[wgpu::PushConstantRange {
                stages: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                range: 0..std::mem::size_of::<ShaderConstants>() as u32,
            }],
        });

        let mut render_pipeline = create_pipeline(
            // &options,
            &device,
            &pipeline_layout,
            surface_with_config.as_ref().map_or_else(
                |pending| pending.preferred_format,
                |(_, surface_config)| surface_config.format,
            ),
            compiled_shader_modules,
        );

        /* let mut png_encoder = png::Encoder::new(File::create("out.png").unwrap(), WIDTH, HEIGHT);

        png_encoder.set_depth(png::BitDepth::Eight);
        png_encoder.set_color(png::ColorType::RGBA);

        let mut png_writer = png_encoder
            .write_header()
            .unwrap()
            .into_stream_writer_with_size((4 * WIDTH) as usize);

        for _ in 0..HEIGHT {
            let row = unsafe { std::slice::from_raw_parts(data, 4 * WIDTH as usize) };
            png_writer.write_all(row).unwrap();
            data = unsafe { data.offset(subresource_layout.row_pitch as isize) };
        }

        png_writer.finish().unwrap(); */

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
        Ok(())
    }

    fn set_uniform(shader: usize, name: &str) -> Result<()> {
        todo!()
    }

    // /// Load a shader program from a SPIR-V binary file
    // #[allow(dead_code)]
    // pub fn load_shader<P: AsRef<Path>>(shader_path: P) -> Result<usize> {
    //     let fw = &FRAMEWORK;

    //     let shader = Shader::from_spirv_file(&fw, &shader_path)?;
    //     let shader_state = ShaderState::new(shader);

    //     let index = SHADERS.read().unwrap().len();
    //     SHADERS
    //         .write()
    //         .unwrap()
    //         .insert(index, Box::new(shader_state));

    //     Ok(index)
    // }

    // /// Compiles GLSL or WGSL shader code to SPIR-V, and load the binary shader program
    // #[allow(dead_code)]
    // fn compile_shader<P: AsRef<Path>>(shader_path: P) -> Result<Shader> {
    //     let compiler = shaderc::Compiler::new().unwrap();
    //     let mut options = shaderc::CompileOptions::new().unwrap();

    //     options.add_macro_definition("EP", Some("main"));

    //     let shader_source = std::fs::read_to_string(shader_path.as_ref())?;

    //     let binary_result = compiler.compile_into_spirv(
    //         &shader_source,
    //         shaderc::ShaderKind::Compute,
    //         shader_path.as_ref().to_str().unwrap(),
    //         "main",
    //         Some(&options),
    //     )?;

    //     let fw = &FRAMEWORK;

    //     let shader = Shader::from_spirv_bytes(
    //         &fw,
    //         binary_result.as_binary_u8(),
    //         Some(
    //             &shader_path
    //                 .as_ref()
    //                 .file_name()
    //                 .map(|s| s.to_string_lossy().into_owned())
    //                 .unwrap_or_else(|| "<unknown>".to_string()),
    //         ),
    //     );

    //     Ok(shader)
    // }
}

impl Plugin for HwAccelPlugin {
    fn get_name(&self) -> String {
        "Hwaccel".to_string()
    }

    fn get_description(&self) -> String {
        "Hardware accelerated effects using Vulkan shader-programs".to_string()
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
