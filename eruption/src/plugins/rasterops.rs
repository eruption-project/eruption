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

use mlua::prelude::*;
use raqote::{
    Color, DrawOptions, DrawTarget, ExtendMode, FilterMode, Image, IntPoint, IntRect, SolidSource,
    Source, Transform,
};
use rayon::prelude::{IntoParallelRefIterator, ParallelIterator};
use std::any::Any;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::Ordering;

use crate::hwdevices::RGBA;
use crate::plugins::Plugin;
use crate::scripting::script;
use crate::{constants, plugins};

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, Clone, thiserror::Error)]
pub enum RasterOpsPluginError {
    #[error("Canvas operation failed: {}", description)]
    CanvasOpFailed { description: String },
    //#[error("Unknown error: {}", description)]
    //UnknownError { description: String },
}

thread_local! {
    /// Allocated draw targets (canvases)
    static DRAW_TARGETS: RefCell<HashMap<usize, Box<RefCell<DrawTarget>>>> = RefCell::new(HashMap::new());

    /// Canvas' state tracking information
    static STATE: RefCell<HashMap<usize, CanvasState>> = RefCell::new(HashMap::new());
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
struct CanvasState {
    source_color: u32,
}

/// A plugin that provides basic image processing and drawing routines for 2d-graphics primitives
pub struct RasterOpsPlugin {}

impl RasterOpsPlugin {
    pub fn new() -> Self {
        RasterOpsPlugin {}
    }

    /// retrieve the handle of the current canvas
    pub(crate) fn get_canvas() -> Result<usize> {
        let mut result = None;

        DRAW_TARGETS.with(|dts| {
            let mut dts = dts.borrow_mut();

            dts.entry(0).or_insert_with(|| {
                Box::new(RefCell::new(DrawTarget::new(
                    constants::CANVAS_WIDTH as i32,
                    constants::CANVAS_HEIGHT as i32,
                )))
            });

            STATE.with(|dts| {
                let mut dts = dts.borrow_mut();

                dts.entry(0).or_insert_with(|| CanvasState {
                    source_color: 0x00000000,
                });
            });

            result = Some(0);
        });

        if let Some(result) = result {
            Ok(result)
        } else {
            Err(RasterOpsPluginError::CanvasOpFailed {
                description: "Could not allocate a canvas".to_string(),
            }
            .into())
        }
    }

    pub(crate) fn create_new_canvas() -> Result<usize> {
        let mut result = None;

        DRAW_TARGETS.with(|dts| {
            let mut dts = dts.borrow_mut();

            let index = dts.len();
            dts.insert(
                index,
                Box::new(RefCell::new(DrawTarget::new(
                    constants::CANVAS_WIDTH as i32,
                    constants::CANVAS_HEIGHT as i32,
                ))),
            );

            STATE.with(|dts| {
                let mut dts = dts.borrow_mut();

                dts.entry(index).or_insert_with(|| CanvasState {
                    source_color: 0x00000000,
                });
            });

            result = Some(dts.len() - 1);
        });

        if let Some(result) = result {
            Ok(result)
        } else {
            Err(RasterOpsPluginError::CanvasOpFailed {
                description: "Could not allocate a canvas".to_string(),
            }
            .into())
        }
    }

    pub(crate) fn destroy_canvas(canvas: usize) -> Result<()> {
        let mut result = Ok(());

        DRAW_TARGETS.with(|dts| {
            let mut dts = dts.borrow_mut();

            if dts.remove(&canvas).is_none() {
                result = Err(RasterOpsPluginError::CanvasOpFailed {
                    description: "Could not find the specified canvas".to_string(),
                }
                .into())
            }

            STATE.with(|dts| {
                let mut dts = dts.borrow_mut();

                dts.remove(&canvas);
            });
        });

        result
    }

    pub(crate) fn realize_canvas(canvas: usize) -> Result<()> {
        let mut result = Ok(());

        DRAW_TARGETS.with(|dts| {
            let mut dts = dts.borrow_mut();

            if let Some(canvas) = dts.get_mut(&canvas) {
                let canvas_data = canvas.get_mut().get_data_u8();

                let mut led_map = [RGBA {
                    r: 0,
                    g: 0,
                    b: 0,
                    a: 0,
                }; constants::CANVAS_SIZE];

                let mut i = 0;
                let mut cntr = 0;

                loop {
                    led_map[cntr] = RGBA {
                        r: canvas_data[i],
                        g: canvas_data[i + 1],
                        b: canvas_data[i + 2],
                        a: canvas_data[i + 3],
                    };

                    i += 4;
                    cntr += 1;

                    if cntr >= led_map.len() || i >= canvas_data.len() {
                        break;
                    }
                }

                script::LOCAL_LED_MAP.with(|m| m.borrow_mut().copy_from_slice(&led_map));
                script::LOCAL_LED_MAP_MODIFIED.with(|f| *f.borrow_mut() = true);

                script::FRAME_GENERATION_COUNTER.fetch_add(1, Ordering::SeqCst);

                result = Ok(())
            } else {
                result = Err(RasterOpsPluginError::CanvasOpFailed {
                    description: "Could not find the specified canvas".to_string(),
                }
                .into())
            }
        });

        result
    }

    pub(crate) fn alpha_blend(
        foreground_canvas: usize,
        background_canvas: usize,
        alpha: f32,
    ) -> Result<()> {
        let mut result = Ok(());

        DRAW_TARGETS.with(|dts| {
            let dts = dts.borrow();

            let foreground_canvas_optional = dts.get(&foreground_canvas);
            let background_canvas_optional = dts.get(&background_canvas);

            if let Some(foreground_canvas) = foreground_canvas_optional {
                if let Some(background_canvas) = background_canvas_optional {
                    let src_rect = IntRect::new(
                        IntPoint::new(0, 0),
                        IntPoint::new(
                            foreground_canvas.borrow().width(),
                            foreground_canvas.borrow().height(),
                        ),
                    );
                    let point = IntPoint::new(0, 0);

                    let canvas = &**background_canvas;
                    canvas.borrow_mut().blend_surface_with_alpha(
                        &foreground_canvas.borrow(),
                        src_rect,
                        point,
                        alpha,
                    );

                    result = Ok(())
                } else {
                    result = Err(RasterOpsPluginError::CanvasOpFailed {
                        description: "Could not find the specified canvas".to_string(),
                    }
                    .into())
                }
            } else {
                result = Err(RasterOpsPluginError::CanvasOpFailed {
                    description: "Could not find the specified canvas".to_string(),
                }
                .into())
            }
        });

        result
    }

    pub(crate) fn set_source_color(canvas: usize, color: u32) -> Result<()> {
        let mut result = Ok(());

        STATE.with(|dts| {
            let mut dts = dts.borrow_mut();

            if let Some(state) = dts.get_mut(&canvas) {
                state.source_color = color;

                result = Ok(())
            } else {
                result = Err(RasterOpsPluginError::CanvasOpFailed {
                    description: "Could not find the associated state of the specified canvas"
                        .to_string(),
                }
                .into())
            }
        });

        result
    }

    pub(crate) fn fill_rectangle(
        canvas: usize,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Result<()> {
        let mut result = Ok(());

        DRAW_TARGETS.with(|dts| {
            let mut dts = dts.borrow_mut();

            let color = STATE.with(|states| {
                let states = states.borrow();

                if let Some(state) = states.get(&canvas) {
                    state.source_color
                } else {
                    0
                }
            });

            if let Some(canvas) = dts.get_mut(&canvas) {
                let (r, g, b, a) = support::color_to_rgba(color);

                canvas.get_mut().fill_rect(
                    x,
                    y,
                    width,
                    height,
                    &Source::Solid(SolidSource::from(Color::new(a, r, g, b))),
                    &DrawOptions::default(),
                );

                result = Ok(())
            } else {
                result = Err(RasterOpsPluginError::CanvasOpFailed {
                    description: "Could not find the specified canvas".to_string(),
                }
                .into())
            }
        });

        result
    }

    pub(crate) fn draw_circle(
        canvas: usize,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) -> Result<()> {
        let mut result = Ok(());

        DRAW_TARGETS.with(|dts| {
            let mut dts = dts.borrow_mut();

            let color = STATE.with(|states| {
                let states = states.borrow();

                if let Some(state) = states.get(&canvas) {
                    state.source_color
                } else {
                    0
                }
            });

            if let Some(canvas) = dts.get_mut(&canvas) {
                let (r, g, b, a) = support::color_to_rgba(color);

                // TODO: Implement this
                canvas.get_mut().fill_rect(
                    x,
                    y,
                    width,
                    height,
                    &Source::Solid(SolidSource::from(Color::new(a, r, g, b))),
                    &DrawOptions::default(),
                );

                result = Ok(())
            } else {
                result = Err(RasterOpsPluginError::CanvasOpFailed {
                    description: "Could not find the specified canvas".to_string(),
                }
                .into())
            }
        });

        result
    }

    pub(crate) fn draw_simplex_noise(
        canvas: usize,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        offset_x: f32,
        offset_y: f32,
        time: f32,
        freq: f32,
    ) -> Result<()> {
        let mut result = Ok(());

        DRAW_TARGETS.with(|dts| {
            let mut dts = dts.borrow_mut();

            if let Some(canvas) = dts.get_mut(&canvas) {
                let noise = simdnoise::NoiseBuilder::gradient_3d_offset(
                    offset_x,
                    width.round() as usize,
                    offset_y,
                    height.round() as usize,
                    time,
                    1,
                )
                .with_freq(freq / 100.0)
                .generate_scaled(0.0, 360.0);

                let noise = noise
                    .par_iter()
                    .map(|&n| support::hsl_to_color(n, 1.0, 0.5))
                    .collect::<Vec<_>>();

                let image = Image {
                    data: &noise,
                    width: width.floor() as i32,
                    height: height.floor() as i32,
                };

                canvas.get_mut().fill_rect(
                    x,
                    y,
                    width,
                    height,
                    &Source::Image(
                        image,
                        ExtendMode::Pad,
                        FilterMode::Nearest,
                        Transform::identity(),
                    ),
                    &DrawOptions::default(),
                );

                result = Ok(())
            } else {
                result = Err(RasterOpsPluginError::CanvasOpFailed {
                    description: "Could not find the specified canvas".to_string(),
                }
                .into())
            }
        });

        result
    }

    pub(crate) fn draw_turbulence_noise(
        canvas: usize,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        offset_x: f32,
        offset_y: f32,
        time: f32,
    ) -> Result<()> {
        let mut result = Ok(());

        DRAW_TARGETS.with(|dts| {
            let mut dts = dts.borrow_mut();

            if let Some(canvas) = dts.get_mut(&canvas) {
                let noise = simdnoise::NoiseBuilder::turbulence_3d_offset(
                    offset_x,
                    width.round() as usize,
                    offset_y,
                    height.round() as usize,
                    time,
                    1,
                )
                // .with_freq(freq)
                // .with_gain(gain)
                // .with_lacunarity(lacunarity)
                // .with_octaves(octaves)
                // .with_seed(seed)
                .generate_scaled(0.0, 360.0);

                let noise = noise
                    .par_iter()
                    .map(|&n| support::hsl_to_color(n, 1.0, 0.5))
                    .collect::<Vec<_>>();

                let image = Image {
                    data: &noise,
                    width: width.floor() as i32,
                    height: height.floor() as i32,
                };

                canvas.get_mut().fill_rect(
                    x,
                    y,
                    width,
                    height,
                    &Source::Image(
                        image,
                        ExtendMode::Pad,
                        FilterMode::Nearest,
                        Transform::identity(),
                    ),
                    &DrawOptions::default(),
                );

                result = Ok(())
            } else {
                result = Err(RasterOpsPluginError::CanvasOpFailed {
                    description: "Could not find the specified canvas".to_string(),
                }
                .into())
            }
        });

        result
    }

    // pub(crate) fn flood_fill() {}
}

impl Plugin for RasterOpsPlugin {
    fn get_name(&self) -> String {
        "Rasterops".to_string()
    }

    fn get_description(&self) -> String {
        "2D-primitives rendering and high-level image-processing functionality".to_string()
    }

    fn initialize(&mut self) -> plugins::Result<()> {
        Ok(())
    }

    fn register_lua_funcs(&self, lua_ctx: &Lua) -> mlua::Result<()> {
        let globals = lua_ctx.globals();

        // canvas related functions
        let get_canvas = lua_ctx.create_function(move |_, ()| {
            let result = RasterOpsPlugin::get_canvas()
                .map_err(|e: eyre::Error| LuaError::RuntimeError(format!("{e}")))?;

            Ok(result)
        })?;
        globals.set("get_canvas", get_canvas)?;

        let create_new_canvas = lua_ctx.create_function(move |_, ()| {
            let result = RasterOpsPlugin::create_new_canvas()
                .map_err(|e: eyre::Error| LuaError::RuntimeError(format!("{e}")))?;

            Ok(result)
        })?;
        globals.set("create_new_canvas", create_new_canvas)?;

        let destroy_canvas = lua_ctx.create_function(move |_, canvas: usize| {
            RasterOpsPlugin::destroy_canvas(canvas)
                .map_err(|e: eyre::Error| LuaError::RuntimeError(format!("{e}")))?;

            Ok(())
        })?;
        globals.set("destroy_canvas", destroy_canvas)?;

        let realize_canvas = lua_ctx.create_function(move |_, canvas: usize| {
            RasterOpsPlugin::realize_canvas(canvas)
                .map_err(|e: eyre::Error| LuaError::RuntimeError(format!("{e}")))?;

            Ok(())
        })?;
        globals.set("realize_canvas", realize_canvas)?;

        let alpha_blend = lua_ctx.create_function(
            move |_, (foreground_canvas, background_canvas, alpha): (usize, usize, f32)| {
                RasterOpsPlugin::alpha_blend(foreground_canvas, background_canvas, alpha)
                    .map_err(|e: eyre::Error| LuaError::RuntimeError(format!("{e}")))?;

                Ok(())
            },
        )?;
        globals.set("alpha_blend", alpha_blend)?;

        // state tracking related functions
        let set_source_color =
            lua_ctx.create_function(move |_, (canvas, color): (usize, u32)| {
                RasterOpsPlugin::set_source_color(canvas, color)
                    .map_err(|e: eyre::Error| LuaError::RuntimeError(format!("{e}")))?;

                Ok(())
            })?;
        globals.set("set_source_color", set_source_color)?;

        // drawing operations
        let fill_rectangle = lua_ctx.create_function(
            move |_, (canvas, x, y, width, height): (usize, f32, f32, f32, f32)| {
                RasterOpsPlugin::fill_rectangle(canvas, x, y, width, height)
                    .map_err(|e: eyre::Error| LuaError::RuntimeError(format!("{e}")))?;

                Ok(())
            },
        )?;
        globals.set("fill_rectangle", fill_rectangle)?;

        let draw_circle = lua_ctx.create_function(
            move |_, (canvas, x, y, width, height): (usize, f32, f32, f32, f32)| {
                RasterOpsPlugin::draw_circle(canvas, x, y, width, height)
                    .map_err(|e: eyre::Error| LuaError::RuntimeError(format!("{e}")))?;

                Ok(())
            },
        )?;
        globals.set("draw_circle", draw_circle)?;

        // special drawing operations
        let draw_simplex_noise = lua_ctx.create_function(
            move |_,
                  (canvas, x, y, width, height, offset_x, offset_y, time, freq): (
                usize,
                f32,
                f32,
                f32,
                f32,
                f32,
                f32,
                f32,
                f32,
            )| {
                RasterOpsPlugin::draw_simplex_noise(
                    canvas, x, y, width, height, offset_x, offset_y, time, freq,
                )
                .map_err(|e: eyre::Error| LuaError::RuntimeError(format!("{e}")))?;

                Ok(())
            },
        )?;
        globals.set("draw_simplex_noise", draw_simplex_noise)?;

        let draw_turbulence_noise = lua_ctx.create_function(
            move |_,
                  (canvas, x, y, width, height, offset_x, offset_y, time): (
                usize,
                f32,
                f32,
                f32,
                f32,
                f32,
                f32,
                f32,
            )| {
                RasterOpsPlugin::draw_turbulence_noise(
                    canvas, x, y, width, height, offset_x, offset_y, time,
                )
                .map_err(|e: eyre::Error| LuaError::RuntimeError(format!("{e}")))?;

                Ok(())
            },
        )?;
        globals.set("draw_turbulence_noise", draw_turbulence_noise)?;

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

#[allow(dead_code)]
mod support {
    use byteorder::{ByteOrder, LittleEndian};
    use palette::{FromColor, Hsl, LinSrgb};

    use crate::hwdevices::RGBA;

    /// Convert RGB components to a 32 bits color value.
    pub(crate) fn rgb_to_color(r: u8, g: u8, b: u8) -> u32 {
        LittleEndian::read_u32(&[b, g, r, 255])
    }

    /// Convert RGBA components to a 32 bits color value.
    pub(crate) fn rgba_to_color(r: u8, g: u8, b: u8, a: u8) -> u32 {
        LittleEndian::read_u32(&[b, g, r, a])
    }

    #[allow(clippy::many_single_char_names)]
    pub(super) fn color_to_rgba(c: u32) -> (u8, u8, u8, u8) {
        let a = u8::try_from((c >> 24) & 0xff).unwrap_or(0);
        let r = u8::try_from((c >> 16) & 0xff).unwrap_or(0);
        let g = u8::try_from((c >> 8) & 0xff).unwrap_or(0);
        let b = u8::try_from(c & 0xff).unwrap_or(0);

        (r, g, b, a)
    }

    /// Convert HSL components to a struct RGBA value.
    pub(crate) fn hsl_to_color(h: f32, s: f32, l: f32) -> u32 {
        let rgb = LinSrgb::from_color(Hsl::new(h, s, l)).into_linear();
        let rgb = rgb.into_components();

        rgb_to_color(
            (255.0 * rgb.0).round() as u8,
            (255.0 * rgb.1).round() as u8,
            (255.0 * rgb.2).round() as u8,
        )
    }

    /// Convert HSL components to a struct RGBA value.
    pub(crate) fn hsl_to_rgba(h: f32, s: f32, l: f32) -> RGBA {
        let rgb = LinSrgb::from_color(Hsl::new(h, s, l)).into_linear();
        let rgb = rgb.into_components();

        RGBA {
            r: (255.0 * rgb.0).round() as u8,
            g: (255.0 * rgb.1).round() as u8,
            b: (255.0 * rgb.2).round() as u8,
            a: 255,
        }
    }
}
