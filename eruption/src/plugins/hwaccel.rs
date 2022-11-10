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

    Copyright (c) 2019-2022, The Eruption Development Team
*/

use bytemuck::{Pod, Zeroable};
use lazy_static::lazy_static;
use mlua::prelude::*;
use parking_lot::RwLock;
use shaderc::CompilationArtifact;
use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Arc;
use std::{any::Any, sync::atomic::Ordering};
use vulkano::buffer::{BufferUsage, CpuAccessibleBuffer};
use vulkano::command_buffer::{AutoCommandBufferBuilder, CommandBufferUsage};
use vulkano::descriptor_set::{PersistentDescriptorSet, WriteDescriptorSet};
use vulkano::device::physical::PhysicalDeviceType;
use vulkano::device::{Device, DeviceCreateInfo, DeviceExtensions, QueueCreateInfo};
use vulkano::instance::{Instance, InstanceCreateInfo};
use vulkano::pipeline::{ComputePipeline, Pipeline, PipelineBindPoint};
use vulkano::shader::ShaderModule;
use vulkano::sync::GpuFuture;
use vulkano::{sync, VulkanLibrary};

use crate::plugins::{self, Plugin};
use crate::{constants, util};

pub type Result<T> = std::result::Result<T, eyre::Error>;

// const WIDTH: u32 = 1920;
// const HEIGHT: u32 = 1080;

#[derive(Debug, thiserror::Error)]
pub enum HwAccelerationPluginError {
    #[error("Initialization failed: {}", description)]
    InitError { description: String },

    // #[error("No devices found")]
    // NoDevicesFound,
    #[error("Compilation failed: {}", description)]
    ShaderCompilationError { description: String },
    // #[error("Unknown error: {}", description)]
    // UnknownError { description: String },
}

lazy_static! {
    pub static ref HWACCEL_STATE: Arc<RwLock<Option<State>>> = Arc::new(RwLock::new(None));
}

pub struct State {
    device: Arc<Device>,
}

// here we derive all these traits to ensure the data behaves as simple as possible
#[repr(C)]
#[derive(Default, Copy, Clone, Zeroable, Pod)]
struct DataStruct {
    a: u32,
    b: u32,
}

/// A plugin that provides access to hardware acceleration APIs
pub struct HwAccelerationPlugin {}

impl HwAccelerationPlugin {
    pub fn new() -> Self {
        HwAccelerationPlugin {}
    }

    pub fn initialize_hwaccel() -> Result<()> {
        // load Vulkan libraries
        let library = VulkanLibrary::new()?;

        // create the instance
        let create_info = InstanceCreateInfo {
            engine_name: Some("Eruption Hwaccel".to_string()),
            enumerate_portability: true,
            ..InstanceCreateInfo::application_from_cargo_toml()
        };

        let instance = Instance::new(library, create_info)?;

        // find a valid physical device
        let device_extensions = DeviceExtensions {
            khr_storage_buffer_storage_class: true,
            ..DeviceExtensions::empty()
        };

        let (physical_device, queue_family_index) = instance
            .enumerate_physical_devices()?
            .filter(|p| p.supported_extensions().contains(&device_extensions))
            .filter_map(|p| {
                // The Vulkan specs guarantee that a compliant implementation must provide at least one queue
                // that supports compute operations.
                p.queue_family_properties()
                    .iter()
                    .position(|q| q.queue_flags.compute)
                    .map(|i| (p, i as u32))
            })
            .min_by_key(|(p, _)| match p.properties().device_type {
                PhysicalDeviceType::DiscreteGpu => 0,
                PhysicalDeviceType::IntegratedGpu => 1,
                PhysicalDeviceType::VirtualGpu => 2,
                PhysicalDeviceType::Cpu => 3,
                PhysicalDeviceType::Other => 4,
                _ => 5,
            })
            .unwrap();

        log::info!(
            "Hwaccel using device: {} (type: {:?})",
            physical_device.properties().device_name,
            physical_device.properties().device_type
        );

        // initialize the device...
        let (device, mut queues) = Device::new(
            physical_device,
            DeviceCreateInfo {
                enabled_extensions: device_extensions,
                queue_create_infos: vec![QueueCreateInfo {
                    queue_family_index,
                    ..Default::default()
                }],
                ..Default::default()
            },
        )
        .unwrap();

        let queue = queues.next().unwrap();

        // let cs = {
        //     let shader_path = "lib/shaders/mandelbrot.comp.glsl";

        //     if let Ok(shader_path) = util::match_script_path(&shader_path) {
        //         let artifact = Self::compile_shader(shader_path)?;
        //         unsafe { ShaderModule::from_bytes(device.clone(), &artifact.as_binary_u8()) }?
        //     } else {
        //         return Err(HwAccelerationPluginError::ShaderCompilationError {
        //             description: shader_path.to_string(),
        //         }
        //         .into());
        //     }
        // };

        // Now let's get to the actual example.
        //
        // What we are going to do is very basic: we are going to fill a buffer with 64k integers
        // and ask the GPU to multiply each of them by 12.
        //
        // GPUs are very good at parallel computations (SIMD-like operations), and thus will do this
        // much more quickly than a CPU would do. While a CPU would typically multiply them one by one
        // or four by four, a GPU will do it by groups of 32 or 64.
        //
        // Note however that in a real-life situation for such a simple operation the cost of
        // accessing memory usually outweighs the benefits of a faster calculation. Since both the CPU
        // and the GPU will need to access data, there is no other choice but to transfer the data
        // through the slow PCI express bus.

        // We need to create the compute pipeline that describes our operation.
        //
        // If you are familiar with graphics pipeline, the principle is the same except that compute
        // pipelines are much simpler to create.
        let pipeline = {
            // mod cs {
            //     vulkano_shaders::shader! {
            //         ty: "compute",
            //         src: "
            //         #version 450
            //         layout(local_size_x = 64, local_size_y = 1, local_size_z = 1) in;
            //         layout(set = 0, binding = 0) buffer Data {
            //             uint data[];
            //         } data;
            //         void main() {
            //             uint idx = gl_GlobalInvocationID.x;
            //             data.data[idx] *= 12;
            //         }
            //     "
            //     }
            // }
            // let shader = cs::load(device.clone()).unwrap();

            let cs = {
                let shader_path = "lib/shaders/example.comp.glsl";

                let shader = if let Ok(shader_path) = util::match_script_path(&shader_path) {
                    let artifact = Self::compile_shader(&shader_path)?;
                    unsafe { ShaderModule::from_bytes(device.clone(), artifact.as_binary_u8()) }?
                } else {
                    return Err(HwAccelerationPluginError::ShaderCompilationError {
                        description: shader_path.to_string(),
                    }
                    .into());
                };

                shader
            };

            ComputePipeline::new(
                device.clone(),
                cs.entry_point("main").unwrap(),
                &(),
                None,
                |_| {},
            )
            .unwrap()
        };

        // We start by creating the buffer that will store the data.
        let data_buffer = {
            // Iterator that produces the data.
            let data_iter = 0..65536u32;
            // Builds the buffer and fills it with this iterator.
            CpuAccessibleBuffer::from_iter(
                device.clone(),
                BufferUsage {
                    storage_buffer: true,
                    ..BufferUsage::empty()
                },
                false,
                data_iter,
            )
            .unwrap()
        };

        // In order to let the shader access the buffer, we need to build a *descriptor set* that
        // contains the buffer.
        //
        // The resources that we bind to the descriptor set must match the resources expected by the
        // pipeline which we pass as the first parameter.
        //
        // If you want to run the pipeline on multiple different buffers, you need to create multiple
        // descriptor sets that each contain the buffer you want to run the shader on.
        let layout = pipeline.layout().set_layouts().get(0).unwrap();
        let _set = PersistentDescriptorSet::new(
            layout.clone(),
            [WriteDescriptorSet::buffer(0, data_buffer.clone())],
        )
        .unwrap();

        // In order to let the shader access the buffer, we need to build a *descriptor set* that
        // contains the buffer.
        //
        // The resources that we bind to the descriptor set must match the resources expected by the
        // pipeline which we pass as the first parameter.
        //
        // If you want to run the pipeline on multiple different buffers, you need to create multiple
        // descriptor sets that each contain the buffer you want to run the shader on.
        let layout = pipeline.layout().set_layouts().get(0).unwrap();
        let set = PersistentDescriptorSet::new(
            layout.clone(),
            [WriteDescriptorSet::buffer(0, data_buffer.clone())],
        )?;

        // In order to execute our operation, we have to build a command buffer.
        let mut builder = AutoCommandBufferBuilder::primary(
            device.clone(),
            queue.queue_family_index(),
            CommandBufferUsage::OneTimeSubmit,
        )?;

        builder
            // The command buffer only does one thing: execute the compute pipeline.
            // This is called a *dispatch* operation.
            //
            // Note that we clone the pipeline and the set. Since they are both wrapped around an
            // `Arc`, this only clones the `Arc` and not the whole pipeline or set (which aren't
            // cloneable anyway). In this example we would avoid cloning them since this is the last
            // time we use them, but in a real code you would probably need to clone them.
            .bind_pipeline_compute(pipeline.clone())
            .bind_descriptor_sets(
                PipelineBindPoint::Compute,
                pipeline.layout().clone(),
                0,
                set,
            )
            .dispatch([1024, 1, 1])?;
        // Finish building the command buffer by calling `build`.
        let command_buffer = builder.build()?;

        // Let's execute this command buffer now.
        // To do so, we TODO: this is a bit clumsy, probably needs a shortcut
        let future = sync::now(device.clone())
            .then_execute(queue, command_buffer)?
            // This line instructs the GPU to signal a *fence* once the command buffer has finished
            // execution. A fence is a Vulkan object that allows the CPU to know when the GPU has
            // reached a certain point.
            // We need to signal a fence here because below we want to block the CPU until the GPU has
            // reached that point in the execution.
            .then_signal_fence_and_flush()?;

        // Blocks execution until the GPU has finished the operation. This method only exists on the
        // future that corresponds to a signalled fence. In other words, this method wouldn't be
        // available if we didn't call `.then_signal_fence_and_flush()` earlier.
        // The `None` parameter is an optional timeout.
        //
        // Note however that dropping the `future` variable (with `drop(future)` for example) would
        // block execution as well, and this would be the case even if we didn't call
        // `.then_signal_fence_and_flush()`.
        // Therefore the actual point of calling `.then_signal_fence_and_flush()` and `.wait()` is to
        // make things more explicit. In the future, if the Rust language gets linear types vulkano may
        // get modified so that only fence-signalled futures can get destroyed like this.
        future.wait(None)?;

        // Now that the GPU is done, the content of the buffer should have been modified. Let's
        // check it out.
        // The call to `read()` would return an error if the buffer was still in use by the GPU.
        let data_buffer_content = data_buffer.read()?;
        for n in 0..65536u32 {
            assert_eq!(data_buffer_content[n as usize], n * 42);
        }

        // icecream::ice!(&data_buffer_content);

        *HWACCEL_STATE.write() = Some(State { device });

        Ok(())
    }

    pub fn compile_shader<P: AsRef<Path>>(shader_path: P) -> Result<CompilationArtifact> {
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

        Ok(binary_result)
    }

    pub fn hwaccel_status() -> BTreeMap<String, String> {
        let mut result = BTreeMap::new();

        if crate::EXPERIMENTAL_FEATURES.load(Ordering::SeqCst) {
            let state = HWACCEL_STATE.read();
            let state = state.as_ref().unwrap();

            let device_name = state
                .device
                .physical_device()
                .properties()
                .device_name
                .clone();

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

    pub fn hwaccel_tick(_delta: i32) {}

    pub fn color_map_from_render_surface() -> Vec<u32> {
        let color = 0xff000000;
        let result = vec![color; constants::CANVAS_SIZE];

        result
    }

    // pub fn set_uniform_value(_value: u32) {}

    // pub fn get_uniform_value() -> u32 {
    //     0
    // }
}

#[async_trait::async_trait]
impl Plugin for HwAccelerationPlugin {
    fn get_name(&self) -> String {
        "Hwaccel".to_string()
    }

    fn get_description(&self) -> String {
        "Hardware accelerated effects using Vulkan".to_string()
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
            lua_ctx.create_function(move |_, ()| Ok(HwAccelerationPlugin::hwaccel_status()))?;
        globals.set("hwaccel_status", hwaccel_status)?;

        let hwaccel_tick = lua_ctx.create_function(move |_, (delta,): (i32,)| {
            HwAccelerationPlugin::hwaccel_tick(delta);
            Ok(())
        })?;
        globals.set("hwaccel_tick", hwaccel_tick)?;

        let color_map_from_render_surface = lua_ctx.create_function(move |_, ()| {
            Ok(HwAccelerationPlugin::color_map_from_render_surface())
        })?;
        globals.set(
            "color_map_from_render_surface",
            color_map_from_render_surface,
        )?;

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
