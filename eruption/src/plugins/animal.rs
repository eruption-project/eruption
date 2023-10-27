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

/*
-------------------------------------------------------------------------------
-- This library was heavily inspired by the excellent work of
-- Philipp Emanuel Weidmann
-- Please see: https://github.com/p-e-w/ternimal
-------------------------------------------------------------------------------
*/

use byteorder::{ByteOrder, LittleEndian};
use mlua::prelude::*;
use std::{
    any::Any,
    cell::RefCell,
    cmp::min,
    collections::{HashMap, VecDeque},
    sync::Arc,
    time::{Instant, SystemTime, UNIX_EPOCH},
};
use tracing::*;

use crate::plugins::Plugin;
use crate::{constants, plugins};

pub type Result<T> = std::result::Result<T, eyre::Error>;

// #[derive(Debug, thiserror::Error)]
// pub enum AnimalPluginError {
//     #[error("Unknown error: {}", description)]
//     UnknownError { description: String },
// }

#[derive(Debug, Clone, thiserror::Error)]
pub enum AnimalError {
    #[error("Invalid handle supplied")]
    InvalidHandle {},

    #[error("Invalid thickness specification: There must be 1, or 5, or 9, ... coefficients")]
    InvalidCoefficients {},

    #[error("Invalid thickness specification: Must be between 0.5 and 1000")]
    InvalidThickness {},

    // #[error("Invalid height: Height must be a multiple of 2")]
    // InvalidHeight {},
    #[error("Invalid length: Length must be between 1 and 1000")]
    InvalidLength {},

    #[error("Invalid radius for the configured width, height, and padding")]
    InvalidRadius {},

    #[error("Insufficient arena size for path generation")]
    InsufficientArenaSize {},

    #[error("Runtime error: {0}")]
    RuntimeError(&'static str),
}

thread_local! {
    /// HashMap holding all allocated animal objects
    pub static ALLOCATED_ANIMALS: RefCell<HashMap<usize, Animal>> = RefCell::new(HashMap::new());
}

pub struct Animal {
    name: String,
    opacity: f64,
    width: usize,
    height: usize,
    speed: f64,
    last_position: f64,
    coefficients: Vec<f64>,
    gradient: ternimal::Gradient,
    model: ternimal::Model,
    path: ternimal::Path,
    length_range: ternimal::Range<f64>,
    path_range: ternimal::Range<f64>,
    start_time: Instant,
}

impl Animal {
    pub fn new(
        name: String,
        speed: f64,
        len_min: f64,
        len_max: f64,
        max_radius: f64,
        gradient_stop_1: (f64, u32),
        gradient_stop_2: (f64, u32),
        gradient_stop_3: (f64, u32),
        opacity: f64,
        coefficients: (f64, f64, f64, f64, f64),
    ) -> Result<Self> {
        tracing::trace!("Creating new animal: {}", name);

        // Seed value for random number generation
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as u32;

        // Coloration gradient of the model, from its spine (`0`) to its outline (`1`)
        let gradient = ternimal::Gradient(vec![
            (gradient_stop_1.0, ternimal::Color::from(gradient_stop_1.1)),
            (gradient_stop_2.0, ternimal::Color::from(gradient_stop_2.1)),
            (gradient_stop_3.0, ternimal::Color::from(gradient_stop_3.1)),
        ]);

        // Dimensions of the arena, in blocks
        let width = constants::CANVAS_WIDTH;
        let height = constants::CANVAS_HEIGHT;

        // Minimum and maximum length of the model, in blocks.
        // The program will animate between the two for a "creeping" motion.
        let length_range = ternimal::Range::new(len_min, len_max);
        if !(1.0 <= length_range.from && length_range.to <= 1000.0) {
            return Err(AnimalError::InvalidLength {}.into());
        }

        // Coefficients of the function determining the model's thickness,
        // in blocks.
        //
        // The function has the form
        //
        // ```
        // f(o, t) = a + b * sin(PI * (c * o + d * t + e)) + ...
        // ```
        //
        // where `o` is the offset (between `0` and `1`) from the head
        // of the model to its tail, and `t` is the time in seconds
        // since the program was started.
        let coefficients: Vec<f64> = vec![
            coefficients.0,
            coefficients.1,
            coefficients.2,
            coefficients.3,
            coefficients.4,
        ];
        if coefficients.len() % 4 != 1 {
            return Err(AnimalError::InvalidCoefficients {}.into());
        }

        // Calculate upper bound for value of thickness function
        let mut max_thickness = coefficients[0];
        for i in 0..((coefficients.len() - 1) / 4) {
            max_thickness += coefficients[4 * i + 1].abs();
        }
        if !(0.5..=1000.0).contains(&max_thickness) {
            return Err(AnimalError::InvalidThickness {}.into());
        }

        let max_padding = 0.1 * ((min(width, height) as f64) / 2.0);

        // Minimum distance between the path and the boundary of the arena, in blocks
        let padding = f64::min(max_thickness, max_padding); // 0.0, max_padding;

        // let max_radius = 0.8 * (((min(width, height) as f64) / 2.0) - padding);

        // Minimum and maximum radius of the arcs comprising the path, in blocks
        let radius_range =
            ternimal::Range::new(f64::min(1.2 * max_thickness, max_radius), max_radius);
        if !(0.5 <= radius_range.from && radius_range.to <= max_radius) {
            return Err(AnimalError::InvalidRadius {}.into());
        }

        // The dimensions of the arena must be such that it is always possible
        // to generate a new arc that is tangential to the last arc in the path
        // and whose radius lies within the permitted range (see `Path` for details).
        // In the worst case, an arc of the maximum permitted radius is placed
        // at the center of the arena, minimizing the available space for the next arc,
        // which must be at least the minimum radius specified above.
        let min_size = (2.0 * radius_range.to) + (4.0 * radius_range.from) + (2.0 * padding);
        if (width as f64) < min_size && (height as f64) < min_size {
            return Err(AnimalError::InsufficientArenaSize {}.into());
        }

        let mut path = ternimal::Path {
            random: ternimal::Random(seed),
            x_range: ternimal::Range::new(padding, (width as f64) - padding),
            y_range: ternimal::Range::new(padding, (height as f64) - padding),
            radius_range,
            start_position: 0.0,
            arcs: VecDeque::new(),
        };

        let path_range = ternimal::Range::new(0.0, length_range.from);

        let arcs = path.generate(path_range.from, path_range.to);
        let model = ternimal::Model::new(arcs, coefficients.clone(), max_thickness, 0.0);

        let start_time = Instant::now();

        Ok(Self {
            name,
            opacity,
            width,
            height,
            speed,
            last_position: 0.0,
            coefficients,
            gradient,
            model,
            path,
            length_range,
            path_range,
            start_time,
        })
    }

    pub fn tick(&mut self, delta: u32) {
        tracing::trace!("{}: Timer tick: {}", self.name, delta);

        let time = ternimal::seconds(self.start_time.elapsed());
        let position = self.speed * time;

        let mut expand_path = true;

        if self.length_range.to > self.length_range.from {
            // "Creeping" motion
            let mut delta = position - self.last_position;
            self.last_position = position;

            while delta > 0.0 {
                let length = self.path_range.to - self.path_range.from;

                let max_delta = if expand_path {
                    self.length_range.to - length
                } else {
                    length - self.length_range.from
                };

                let actual_delta = f64::min(delta, max_delta);

                if expand_path {
                    self.path_range.to += actual_delta;
                } else {
                    self.path_range.from += actual_delta;
                }

                if delta >= max_delta {
                    expand_path = !expand_path;
                }

                delta -= actual_delta;
            }
        } else {
            // Linear motion
            self.path_range.from = position;
            self.path_range.to = self.path_range.from + self.length_range.from;
        }

        // Calculate upper bound for value of thickness function
        let mut max_thickness = self.coefficients[0];
        for i in 0..((self.coefficients.len() - 1) / 4) {
            max_thickness += self.coefficients[4 * i + 1].abs();
        }

        let model = ternimal::Model::new(
            self.path.generate(self.path_range.from, self.path_range.to),
            self.coefficients.clone(),
            max_thickness,
            time,
        );

        self.model = model;
    }

    pub fn render(&self) -> Vec<u32> {
        tracing::trace!("{}: Rendering", self.name);

        let mut result = vec![0x000000000; constants::CANVAS_SIZE];

        /// Convert RGBA components to a 32 bits color value.
        fn rgba_to_color(r: f64, g: f64, b: f64, a: f64) -> u32 {
            LittleEndian::read_u32(&[
                (b * 255.0) as u8,
                (g * 255.0) as u8,
                (r * 255.0) as u8,
                (a * 255.0) as u8,
            ])
        }

        let canvas = ternimal::rasterize(&self.model, &self.gradient, self.width, self.height);

        // let output = ternimal::render(&canvas, true);
        // print!("{output}");

        for i in 0..constants::CANVAS_SIZE {
            let x = i % constants::CANVAS_WIDTH;
            let y = i / constants::CANVAS_WIDTH;

            let color = canvas[y][x].unwrap_or_else(|| ternimal::Color::new(0.0, 0.0, 0.0));
            let color = rgba_to_color(color.red, color.green, color.blue, self.opacity);

            result[i] = color;
        }

        result
    }
}

/// Support library for animal.lua
pub struct AnimalPlugin {}

impl AnimalPlugin {
    pub fn new() -> Self {
        AnimalPlugin {}
    }
}

impl Plugin for AnimalPlugin {
    fn get_name(&self) -> String {
        "Animal".to_string()
    }

    fn get_description(&self) -> String {
        "Simulation of organic movements (support library)".to_string()
    }

    fn initialize(&mut self) -> plugins::Result<()> {
        Ok(())
    }

    fn register_lua_funcs(&self, lua_ctx: &Lua) -> mlua::Result<()> {
        let globals = lua_ctx.globals();

        let animal_create = lua_ctx.create_function(
            |_,
             (
                name,
                speed,
                len_min,
                len_max,
                max_radius,
                gradient_stop_1,
                gradient_stop_2,
                gradient_stop_3,
                opacity,
                coefficients,
            ): (
                String,
                f64,
                f64,
                f64,
                f64,
                mlua::Table,
                mlua::Table,
                mlua::Table,
                f64,
                mlua::Table,
            )| {
                ALLOCATED_ANIMALS
                    .with(|f| {
                        let mut m = f.borrow_mut();
                        let idx = m.len() + 1;

                        let gradient_stop_1_pos = gradient_stop_1.get(1).map_err(|_e| {
                            AnimalError::RuntimeError("Invalid color gradient definition")
                        })?;

                        let gradient_stop_1_color = gradient_stop_1
                            .get(2)
                            .map_err(|_e| AnimalError::RuntimeError("Invalid color definition"))?;

                        let gradient_stop_2_pos = gradient_stop_2.get(1).map_err(|_e| {
                            AnimalError::RuntimeError("Invalid color gradient definition")
                        })?;

                        let gradient_stop_2_color = gradient_stop_2
                            .get(2)
                            .map_err(|_e| AnimalError::RuntimeError("Invalid color definition"))?;

                        let gradient_stop_3_pos = gradient_stop_3.get(1).map_err(|_e| {
                            AnimalError::RuntimeError("Invalid color gradient definition")
                        })?;

                        let gradient_stop_3_color = gradient_stop_3
                            .get(2)
                            .map_err(|_e| AnimalError::RuntimeError("Invalid color definition"))?;

                        let coefficients = (
                            coefficients.get(1).map_err(|_e| {
                                AnimalError::RuntimeError("Invalid color definition")
                            })?,
                            coefficients.get(2).map_err(|_e| {
                                AnimalError::RuntimeError("Invalid color definition")
                            })?,
                            coefficients.get(3).map_err(|_e| {
                                AnimalError::RuntimeError("Invalid color definition")
                            })?,
                            coefficients.get(4).map_err(|_e| {
                                AnimalError::RuntimeError("Invalid color definition")
                            })?,
                            coefficients.get(5).map_err(|_e| {
                                AnimalError::RuntimeError("Invalid color definition")
                            })?,
                        );

                        let animal = Animal::new(
                            name,
                            speed,
                            len_min,
                            len_max,
                            max_radius,
                            (gradient_stop_1_pos, gradient_stop_1_color),
                            (gradient_stop_2_pos, gradient_stop_2_color),
                            (gradient_stop_3_pos, gradient_stop_3_color),
                            opacity,
                            coefficients,
                        )?;
                        m.insert(idx, animal);

                        Ok(idx)
                    })
                    .map_err(|e: eyre::Error| {
                        LuaError::RuntimeError(format!(
                            "Could not instantiate the animal object: {e}"
                        ))
                    })
            },
        )?;
        globals.set("animal_create", animal_create)?;

        let animal_tick = lua_ctx.create_function(|_, (handle, delta): (usize, u32)| {
            ALLOCATED_ANIMALS
                .with(|f| {
                    let mut m = f.borrow_mut();

                    if let Some(animal) = m.get_mut(&handle) {
                        animal.tick(delta);

                        Ok(())
                    } else {
                        Err(AnimalError::InvalidHandle {})
                    }
                })
                .map_err(|e| LuaError::ExternalError(Arc::new(e)))
        })?;
        globals.set("animal_tick", animal_tick)?;

        let animal_render = lua_ctx.create_function(|_, (handle,): (usize,)| {
            ALLOCATED_ANIMALS
                .with(|f| {
                    let m = f.borrow();

                    if let Some(animal) = m.get(&handle) {
                        let result = animal.render();

                        Ok(result)
                    } else {
                        Err(AnimalError::InvalidHandle {})
                    }
                })
                .map_err(|e| LuaError::ExternalError(Arc::new(e)))
        })?;
        globals.set("animal_render", animal_render)?;

        let animal_destroy = lua_ctx.create_function(|_, (handle,): (usize,)| {
            ALLOCATED_ANIMALS
                .with(|f| {
                    let mut m = f.borrow_mut();

                    if let Some(animal) = m.remove(&handle) {
                        tracing::trace!("Destroying animal object: {}", animal.name);

                        Ok(())
                    } else {
                        Err(AnimalError::InvalidHandle {})
                    }
                })
                .map_err(|e| LuaError::ExternalError(Arc::new(e)))
        })?;
        globals.set("animal_destroy", animal_destroy)?;

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
mod ternimal {
    // Ternimal - Simulate a lifeform in the terminal
    //
    // Copyright (c) 2017 Philipp Emanuel Weidmann <pew@worldwidemann.com>
    //
    // Nemo vir est qui mundum non reddat meliorem.
    //
    // Released under the terms of the GNU General Public License, version 3
    // (https://gnu.org/licenses/gpl.html)

    // Adapted for Eruption, 2021

    use std::collections::{HashMap, VecDeque};
    use std::f64::consts::PI;
    use std::f64::{INFINITY, NEG_INFINITY};
    use std::fmt::Display;
    use std::ops::{Add, Mul, Sub};
    use std::process;
    use std::str::FromStr;
    use std::time::Duration;

    const TWO_PI: f64 = 2.0 * PI;

    /// Prints its formatted arguments to standard error, then exits the program.
    macro_rules! exit {
    ($($arg:tt)*) => (
        // Red foreground color
        eprint!("\x1B[31m");
        eprint!($($arg)*);
        // Reset
        eprint!("\x1B[m\n");
        process::exit(1);
    );
}

    /// Wraps its formatted arguments in an `Err`.
    macro_rules! err {
    ($($arg:tt)*) => (
        Err(format!($($arg)*))
    );
}

    /// Evaluates to the minimum of its arguments.
    ///
    /// Note that unlike with the macro from https://rustbyexample.com/macros/repeat.html,
    /// the arguments need to implement only `PartialOrd`, not `Ord`.
    macro_rules! min {
    ($a:expr) => ($a);
    ($a:expr, $($b:expr),+) => (
        if $a < min!($($b),+) {
            $a
        } else {
            min!($($b),+)
        }
    );
}

    /// Evaluates to the maximum of its arguments.
    macro_rules! max {
    ($a:expr) => ($a);
    ($a:expr, $($b:expr),+) => (
        if $a > max!($($b),+) {
            $a
        } else {
            max!($($b),+)
        }
    );
}

    /*
    /// Program entry point
    fn main() {
        let args: Vec<String> = env::args().skip(1).collect();
        let arguments = unwrap_or_exit(Arguments::parse(args));

        // Sets a local variable to the value of the command line argument
        // with the same name as the variable.
        macro_rules! arg_var {
            ($name:ident, $default:expr) => {
                let $name = unwrap_or_exit(arguments.value(stringify!($name), $default));
            };
            ($name:ident, $default:expr, $min:expr, $max:expr) => {
                arg_var!($name, $default);
                if !($min <= $name && $name <= $max) {
                    exit!(
                        "Invalid value '{}' for argument '{}': Value must be between {} and {}.",
                        $name,
                        stringify!($name),
                        $min,
                        $max
                    );
                }
            };
        }

        // Seed value for random number generation
        arg_var!(
            seed,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs() as u32
        );

        // Linear speed of the model along its path, in blocks/second
        arg_var!(speed, 30.0, 0.0, 1000.0);
        // Animation frames per second
        arg_var!(fps, 30.0, 0.1, 600.0);

        // Coloration gradient of the model, from its spine (`0`) to its outline (`1`)
        arg_var!(
            gradient,
            Gradient(vec![
                (0.4, Color::new(1.0, 1.0, 1.0)),
                (0.6, Color::new(0.15, 0.15, 0.7)),
                (1.0, Color::new(0.3, 0.1, 0.3)),
            ])
        );

        // Use 24-bit RGB terminal colors (`true`) or the 256-color palette (`false`)
        arg_var!(true_color, true);

        // Dimensions of the arena, in blocks
        arg_var!(width, 60, 1, 500);
        arg_var!(height, 40, 1, 500);
        if height % 2 != 0 {
            exit!(
                "Invalid height '{}': Height must be a multiple of 2.",
                height
            );
        }

        // Minimum and maximum length of the model, in blocks.
        // The program will animate between the two for a "creeping" motion.
        let length_range = unwrap_or_exit(arguments.value("length", Range::new(10.0, 20.0)));
        if !(1.0 <= length_range.from && length_range.to <= 1000.0) {
            exit!(
                "Invalid length '{} to {}': Length must be between 1 and 1000.",
                length_range.from,
                length_range.to
            );
        }

        // Coefficients of the function determining the model's thickness,
        // in blocks.
        //
        // The function has the form
        //
        // ```
        // f(o, t) = a + b * sin(PI * (c * o + d * t + e)) + ...
        // ```
        //
        // where `o` is the offset (between `0` and `1`) from the head
        // of the model to its tail, and `t` is the time in seconds
        // since the program was started.
        let coefficients =
            unwrap_or_exit(arguments.values("thickness", vec![4.0, 1.0, 3.5, 0.0, 0.0]));
        if coefficients.len() % 4 != 1 {
            exit!("Invalid thickness specification: There must be 1, or 5, or 9, ... coefficients; {} were supplied.",
                coefficients.len());
        }

        let thickness = |offset: f64, time: f64| {
            assert!(0.0 <= offset && offset <= 1.0);
            let mut thickness = coefficients[0];
            for i in 0..((coefficients.len() - 1) / 4) {
                thickness += coefficients[4 * i + 1]
                    * (PI
                        * ((coefficients[4 * i + 2] * offset)
                            + (coefficients[4 * i + 3] * time)
                            + coefficients[4 * i + 4]))
                        .sin();
            }
            thickness
        };

        // Calculate upper bound for value of thickness function
        let mut max_thickness = coefficients[0];
        for i in 0..((coefficients.len() - 1) / 4) {
            max_thickness += coefficients[4 * i + 1].abs();
        }
        if !(0.5 <= max_thickness && max_thickness <= 1000.0) {
            exit!("Invalid thickness specification: Maximum thickness is {}; must be between 0.5 and 1000.", max_thickness);
        }

        let max_padding = 0.8 * ((min!(width, height) as f64) / 2.0);
        // Minimum distance between the path and the boundary of the arena, in blocks
        arg_var!(padding, min!(max_thickness, max_padding), 0.0, max_padding);

        let max_radius = 0.8 * (((min!(width, height) as f64) / 2.0) - padding);
        // Minimum and maximum radius of the arcs comprising the path, in blocks
        let radius_range = unwrap_or_exit(arguments.value(
            "radius",
            Range::new(min!(1.2 * max_thickness, max_radius), max_radius),
        ));
        if !(0.5 <= radius_range.from && radius_range.to <= max_radius) {
            exit!(
                "Invalid radius '{} to {}': For the configured width, height, and padding, \
                radius must be between 0.5 and {}.",
                radius_range.from,
                radius_range.to,
                max_radius
            );
        }

        // The dimensions of the arena must be such that it is always possible
        // to generate a new arc that is tangential to the last arc in the path
        // and whose radius lies within the permitted range (see `Path` for details).
        // In the worst case, an arc of the maximum permitted radius is placed
        // at the center of the arena, minimizing the available space for the next arc,
        // which must be at least the minimum radius specified above.
        let min_size = (2.0 * radius_range.to) + (4.0 * radius_range.from) + (2.0 * padding);
        if (width as f64) < min_size && (height as f64) < min_size {
            exit!("Insufficient arena size for path generation: For the configured radius and padding, \
                either width or height must be at least {}.", min_size);
        }

        let mut path = Path {
            random: Random(seed),
            x_range: Range::new(padding, (width as f64) - padding),
            y_range: Range::new(padding, (height as f64) - padding),
            radius_range,
            start_position: 0.0,
            arcs: VecDeque::new(),
        };

        let mut last_position = 0.0;
        let mut path_range = Range::new(0.0, length_range.from);
        let mut expand_path = true;

        let start_time = Instant::now();

        // Rendering loop
        loop {
            let time = seconds(start_time.elapsed());

            let position = speed * time;

            if length_range.to > length_range.from {
                // "Creeping" motion
                let mut delta = position - last_position;
                last_position = position;

                while delta > 0.0 {
                    let length = path_range.to - path_range.from;

                    let max_delta = if expand_path {
                        length_range.to - length
                    } else {
                        length - length_range.from
                    };

                    let actual_delta = min!(delta, max_delta);

                    if expand_path {
                        path_range.to += actual_delta;
                    } else {
                        path_range.from += actual_delta;
                    }

                    if delta >= max_delta {
                        expand_path = !expand_path;
                    }

                    delta -= actual_delta;
                }
            } else {
                // Linear motion
                path_range.from = position;
                path_range.to = path_range.from + length_range.from;
            }

            let model = Model::new(path.generate(path_range.from, path_range.to), max_thickness);

            let canvas = rasterize(&model, &gradient, width, height);
            let output = render(&canvas, true_color);

            // Hide cursor while printing canvas to avoid flickering
            print!("\x1B[?25l{}\x1B[?25h\n", output);

            // Sleep to compensate for difference between rendering time and frame time
            let sleep_time = (1.0 / fps) - (seconds(start_time.elapsed()) - time);
            if sleep_time > 0.0 {
                thread::sleep(Duration::new(
                    sleep_time.trunc() as u64,
                    (sleep_time.fract() * 1_000_000_000.0) as u32,
                ));
            }

            // Move cursor up to enable drawing of next frame over the current one
            print!("\x1B[{}A", height / 2);
        }
    }

    */

    //----------------------------------------------------------
    // Rendering
    //----------------------------------------------------------

    /// 2D model of a lifeform
    pub(super) struct Model {
        /// "Spine" of the model
        pub arcs: Vec<Arc>,
        pub coefficients: Vec<f64>,
        pub max_thickness: f64,
        pub length: f64,
        pub x_range: Range<f64>,
        pub y_range: Range<f64>,
        pub time: f64,
    }

    impl Model {
        /// Creates a new `Model` object.
        pub(super) fn new(
            arcs: Vec<Arc>,
            coefficients: Vec<f64>,
            max_thickness: f64,
            time: f64,
        ) -> Model {
            assert!(!arcs.is_empty());
            assert!(max_thickness > 0.0);

            let mut length = 0.0;

            let mut min_x = INFINITY;
            let mut max_x = NEG_INFINITY;
            let mut min_y = INFINITY;
            let mut max_y = NEG_INFINITY;

            // Calculate length and bounding box of model
            for arc in &arcs {
                length += arc.length;

                let (x_range, y_range) = arc.bounding_box();
                min_x = min!(x_range.from, min_x);
                max_x = max!(x_range.to, max_x);
                min_y = min!(y_range.from, min_y);
                max_y = max!(y_range.to, max_y);
            }

            assert!(length > 0.0);

            Model {
                arcs,
                coefficients,
                max_thickness,
                length,
                x_range: Range::new(min_x - max_thickness, max_x + max_thickness),
                y_range: Range::new(min_y - max_thickness, max_y + max_thickness),
                time,
            }
        }

        pub(super) fn thickness(&self, offset: f64) -> f64 {
            assert!((0.0..=1.0).contains(&offset));

            let mut thickness = self.coefficients[0];
            for i in 0..((self.coefficients.len() - 1) / 4) {
                thickness += self.coefficients[4 * i + 1]
                    * (PI
                        * ((self.coefficients[4 * i + 2] * offset)
                            + (self.coefficients[4 * i + 3] * self.time)
                            + self.coefficients[4 * i + 4]))
                        .sin();
            }

            thickness
        }

        /// Returns the color of the given point if it lies within the model,
        /// or `None` if it lies outside of it.
        pub(super) fn color(&self, point: &Point, gradient: &Gradient) -> Option<Color> {
            // Return early if `point` lies outside the bounding box.
            // This dramatically improves performance when the model
            // is small compared to the arena.
            if !(self.x_range.contains(point.x) && self.y_range.contains(point.y)) {
                return None;
            }

            let mut distance = INFINITY;
            let mut length = 0.0;

            // Determine minimum *relative* distance from point to model
            // (distance as a fraction of the corresponding local thickness)
            for (i, arc) in self.arcs.iter().enumerate() {
                let first_arc = i == 0;
                let last_arc = i == self.arcs.len() - 1;

                let (arc_distance, arc_offset) = match arc.location(point) {
                    ArcLocation::Inside(offset) => (point.distance(&arc.point(offset)), offset),
                    // "Round off" model shape by capping ends with circles
                    ArcLocation::Before if first_arc => (point.distance(&arc.point(0.0)), 0.0),
                    ArcLocation::After if last_arc => (point.distance(&arc.point(1.0)), 1.0),
                    _ => (-1.0, -1.0),
                };

                if 0.0 <= arc_distance && arc_distance <= self.max_thickness {
                    let offset = (length + (arc.length * arc_offset)) / self.length;

                    // The loop iterates over arcs from the tail of the model to its head,
                    // so the offset must be inverted before being passed to the thickness function
                    let thickness = self.thickness(1.0 - offset);

                    if thickness > 0.0 {
                        distance = min!(arc_distance / thickness, distance);
                    }
                }

                length += arc.length;
            }

            if distance > 1.0 {
                None
            } else {
                Some(gradient.color(distance))
            }
        }
    }

    /// Grid of colored blocks representing the arena
    pub(super) type Canvas = Vec<Vec<Option<Color>>>;

    /// Determines the color of each pixel-like block in the arena for the given model.
    pub(super) fn rasterize(
        model: &Model,
        gradient: &Gradient,
        width: usize,
        height: usize,
    ) -> Canvas {
        let mut grid = vec![];

        // Compute colors for grid points
        for i in 0..(height + 1) {
            let mut row = vec![];
            for j in 0..(width + 1) {
                let color = model.color(&Point::new(j as f64, i as f64), gradient);
                row.push(color);
            }
            grid.push(row);
        }

        let mut canvas = vec![];

        // Color blocks by averaging the colors of their corners
        for i in 0..height {
            let mut row = vec![];

            for j in 0..width {
                let mut corners = vec![];

                for ii in 0..2 {
                    for jj in 0..2 {
                        match grid[i + ii][j + jj] {
                            Some(color) => corners.push(color),
                            None => {}
                        }
                    }
                }

                row.push(if corners.len() >= 3 {
                    Some(Color::average(&corners))
                } else {
                    None
                });
            }

            canvas.push(row);
        }

        canvas
    }

    /// Returns a string that, when printed to the terminal, renders the given canvas.
    pub(super) fn render(canvas: &Canvas, true_color: bool) -> String {
        assert!(!canvas.is_empty());
        // The canvas must have an even number of rows because
        // two rows are represented by each line of output
        assert!(canvas.len() % 2 == 0);

        let mut output = String::new();

        let mut reset_required = true;

        let row_length = canvas[0].len();

        for i in 0..(canvas.len() / 2) {
            assert!(canvas[2 * i].len() == row_length);
            assert!(canvas[2 * i + 1].len() == row_length);

            for j in 0..row_length {
                let block = match (canvas[2 * i][j], canvas[2 * i + 1][j]) {
                    (Some(top), Some(bottom)) => {
                        // Unicode UPPER HALF BLOCK with foreground (top) and background (bottom) color
                        format!(
                            "\x1B[38;{};48;{}m\u{2580}",
                            top.sgr(true_color),
                            bottom.sgr(true_color)
                        )
                    }
                    (Some(top), None) => {
                        // Unicode UPPER HALF BLOCK with foreground (top) color
                        format!(
                            "\x1B[{}38;{}m\u{2580}",
                            if reset_required { "0;" } else { "" },
                            top.sgr(true_color)
                        )
                    }
                    (None, Some(bottom)) => {
                        // Unicode LOWER HALF BLOCK with foreground (bottom) color
                        format!(
                            "\x1B[{}38;{}m\u{2584}",
                            if reset_required { "0;" } else { "" },
                            bottom.sgr(true_color)
                        )
                    }
                    (None, None) => {
                        format!("{} ", if reset_required { "\x1B[m" } else { "" })
                    }
                };

                output.push_str(&block);
                reset_required = canvas[2 * i][j].is_some() && canvas[2 * i + 1][j].is_some();
            }

            let last_line = i == (canvas.len() / 2) - 1;

            // Always reset on the last line to restore foreground color
            if reset_required || last_line {
                output.push_str("\x1B[m");
            }
            if !last_line {
                output.push('\n');
            }

            reset_required = false;
        }

        output
    }

    //----------------------------------------------------------
    // Path generation
    //----------------------------------------------------------

    /// Extensible, differentiable curve in `R^2`
    pub(super) struct Path {
        pub random: Random,
        /// Permitted range for x coordinates of points on path
        pub x_range: Range<f64>,
        /// Permitted range for y coordinates of points on path
        pub y_range: Range<f64>,
        /// Permitted range for radii of arcs comprising the path
        pub radius_range: Range<f64>,
        /// Linear position associated with first arc in the path
        pub start_position: f64,
        pub arcs: VecDeque<Arc>,
    }

    impl Path {
        /// Extends the path with randomly generated arcs as needed to cover
        /// the range of positions between `start_position` and `end_position`,
        /// discards existing arcs not overlapping that range, and returns
        /// the segment of the path corresponding to the range.
        pub(super) fn generate(&mut self, start_position: f64, end_position: f64) -> Vec<Arc> {
            assert!(start_position < end_position);

            // Always leave at least one arc in the path
            // for newly generated arcs to connect to
            while self.arcs.len() > 1
                && (self.start_position + self.arcs[0].length) < start_position
            {
                let first_arc = self.arcs.pop_front().unwrap();
                self.start_position += first_arc.length;
            }

            while self.end_position() < end_position {
                let (center, radius, start, clockwise) = if self.arcs.is_empty() {
                    // Initial arc
                    let min_radius = self.radius_range.from;

                    // Leave room around center for a circle of radius at least `min_radius`
                    let center = Point::new(
                        self.random.real_range(
                            self.x_range.from + min_radius,
                            self.x_range.to - min_radius,
                        ),
                        self.random.real_range(
                            self.y_range.from + min_radius,
                            self.y_range.to - min_radius,
                        ),
                    );

                    // Largest radius that fits inside the rectangle
                    let max_radius = min!(
                        self.radius_range.to,
                        (center.x - self.x_range.from).abs(),
                        (center.x - self.x_range.to).abs(),
                        (center.y - self.y_range.from).abs(),
                        (center.y - self.y_range.to).abs()
                    );

                    (
                        center,
                        self.random.real_range(min_radius, max_radius),
                        self.random.real_range_except(0.0, TWO_PI, &[TWO_PI]),
                        self.random.flip(),
                    )
                } else {
                    // The goal is to construct an arc that:
                    // 1. Starts where the last arc ends
                    // 2. Continues from the last arc in such a way that
                    //    the resulting curve is differentiable
                    // 3. Has orientation opposite to the last arc
                    // 4. Lies completely inside the rectangle
                    let last_arc = &self.arcs[self.arcs.len() - 1];

                    let (endpoint, direction) = last_arc.endpoint_and_direction();

                    let max_radius = self.max_radius(endpoint, direction);
                    let radius = self.random.real_range(self.radius_range.from, max_radius);

                    (
                        endpoint + (direction * radius),
                        radius,
                        if last_arc.end < PI {
                            last_arc.end + PI
                        } else {
                            last_arc.end - PI
                        },
                        !last_arc.clockwise,
                    )
                };

                // Brute force an admissible endpoint angle for the arc, that is,
                // an angle that allows for the construction of a tangent arc with
                // radius at least the minimum radius specified for the path
                let arc = loop {
                    let end = self.random.real_range_except(0.0, TWO_PI, &[TWO_PI, start]);
                    let arc = Arc::new(center, radius, start, end, clockwise);

                    let (endpoint, direction) = arc.endpoint_and_direction();

                    if self.max_radius(endpoint, direction) >= self.radius_range.from {
                        break arc;
                    }
                };

                self.arcs.push_back(arc);
            }

            let mut arcs = vec![];

            let mut arc_start_position = self.start_position;

            for arc in &self.arcs {
                let arc_end_position = arc_start_position + arc.length;

                if start_position < arc_end_position && end_position > arc_start_position {
                    // Arc is at least partially contained in range
                    let start = if start_position > arc_start_position {
                        let position = (start_position - arc_start_position) / arc.length;
                        ((arc.start + (arc.end_difference * position)) + TWO_PI) % TWO_PI
                    } else {
                        arc.start
                    };

                    let end = if end_position < arc_end_position {
                        let position = (end_position - arc_start_position) / arc.length;
                        ((arc.start + (arc.end_difference * position)) + TWO_PI) % TWO_PI
                    } else {
                        arc.end
                    };

                    arcs.push(Arc::new(arc.center, arc.radius, start, end, arc.clockwise));
                }

                arc_start_position = arc_end_position;
            }

            arcs
        }

        /// Returns the last position in the path.
        pub(super) fn end_position(&self) -> f64 {
            self.start_position + self.arcs.iter().map(|arc| arc.length).sum::<f64>()
        }

        /// Returns the maximum radius allowed for an arc that is tangential to
        /// the arc ending at `endpoint` with the normalized center-endpoint
        /// vector `direction`.
        pub(super) fn max_radius(&self, endpoint: Point, direction: Point) -> f64 {
            // For an arc to be tangential to the given arc, the centers
            // of the two arcs must be collinear with the tangent point
            // (i.e. `endpoint`). Since the new arc has orientation
            // opposite to the given arc, its center lies on the extension of
            // the line from the given arc's center to its endpoint.
            //
            // The radius of the new arc must be chosen such that the arc lies
            // inside the rectangle. The new arc is bounded to the right
            // by the vertical line at
            //
            // ```
            // endpoint.x + (radius * direction.x) + radius
            // ```
            //
            // Setting this to be at most `x_range.to` and solving for `radius` gives
            //
            // ```
            // radius <= (x_range.to - endpoint.x) / (direction.x + 1)
            // ```
            //
            // and applying this argument mutatis mutandis to the other directions
            // results in the expressions below.
            min!(
                self.radius_range.to,
                // Left
                (self.x_range.from - endpoint.x) / (direction.x - 1.0),
                // Right
                (self.x_range.to - endpoint.x) / (direction.x + 1.0),
                // Top
                (self.y_range.from - endpoint.y) / (direction.y - 1.0),
                // Bottom
                (self.y_range.to - endpoint.y) / (direction.y + 1.0)
            )
        }
    }

    //----------------------------------------------------------
    // Geometric primitives
    //----------------------------------------------------------

    /// Point/vector in `R^2`
    #[derive(Copy, Clone)]
    pub(super) struct Point {
        pub x: f64,
        pub y: f64,
    }

    impl Point {
        /// Creates a new `Point` object.
        pub(super) fn new(x: f64, y: f64) -> Point {
            Point { x, y }
        }

        /// Returns the Euclidean distance between this point and the given point.
        pub(super) fn distance(&self, point: &Point) -> f64 {
            ((self.x - point.x).powi(2) + (self.y - point.y).powi(2)).sqrt()
        }
    }

    /// Vector addition
    impl Add for Point {
        type Output = Point;

        fn add(self, point: Point) -> Point {
            Point::new(self.x + point.x, self.y + point.y)
        }
    }

    /// Vector subtraction
    impl Sub for Point {
        type Output = Point;

        fn sub(self, point: Point) -> Point {
            Point::new(self.x - point.x, self.y - point.y)
        }
    }

    /// Scalar multiplication
    impl Mul<f64> for Point {
        type Output = Point;

        fn mul(self, scalar: f64) -> Point {
            Point::new(self.x * scalar, self.y * scalar)
        }
    }

    /// Circular arc in `R^2`
    pub(super) struct Arc {
        pub center: Point,
        pub radius: f64,
        /// Start angle of the arc (between `0` and `TWO_PI`)
        pub start: f64,
        /// End angle of the arc (between `0` and `TWO_PI`)
        pub end: f64,
        pub clockwise: bool,
        pub length: f64,
        pub end_difference: f64,
    }

    impl Arc {
        /// Creates a new `Arc` object.
        pub(super) fn new(
            center: Point,
            radius: f64,
            start: f64,
            end: f64,
            clockwise: bool,
        ) -> Arc {
            // Note that these assertions guarantee that the arc has positive length
            assert!(radius > 0.0);
            assert!((0.0..TWO_PI).contains(&start));
            assert!((0.0..TWO_PI).contains(&end));
            assert!((start - end).abs() > f64::EPSILON);

            let mut arc = Arc {
                center,
                radius,
                start,
                end,
                clockwise,
                length: 0.0,
                end_difference: 0.0,
            };

            arc.end_difference = arc.difference(arc.end);
            arc.length = arc.end_difference.abs() * arc.radius;

            arc
        }

        /// Returns the point at the given normalized (between `0` and `1`)
        /// linear position on the arc.
        pub(super) fn point(&self, position: f64) -> Point {
            assert!((0.0..=1.0).contains(&position));

            let angle = self.start + (self.end_difference * position);

            Point::new(
                self.center.x + (angle.cos() * self.radius),
                // Note that the canvas' y axis is flipped compared to
                // the standard Cartesian coordinate system
                self.center.y - (angle.sin() * self.radius),
            )
        }

        /// Returns the signed angular difference between `start` and the given angle,
        /// taking `clockwise` into account.
        pub(super) fn difference(&self, angle: f64) -> f64 {
            let difference = angle - self.start;

            if difference > 0.0 && self.clockwise {
                difference - TWO_PI
            } else if difference < 0.0 && !self.clockwise {
                difference + TWO_PI
            } else {
                difference
            }
        }

        /// Returns the endpoint of the arc, as well as the unit vector
        /// pointing from the center to the endpoint.
        pub(super) fn endpoint_and_direction(&self) -> (Point, Point) {
            let endpoint = self.point(1.0);

            let direction = endpoint - self.center;
            let direction = direction * (1.0 / direction.distance(&Point::new(0.0, 0.0)));

            (endpoint, direction)
        }

        /// Returns the horizontal and vertical extent of the arc.
        pub(super) fn bounding_box(&self) -> (Range<f64>, Range<f64>) {
            let start_point = self.point(0.0);
            let end_point = self.point(1.0);

            let min_x = if self.contains_angle(PI) {
                self.center.x - self.radius
            } else {
                min!(start_point.x, end_point.x)
            };

            let max_x = if self.contains_angle(0.0) {
                self.center.x + self.radius
            } else {
                max!(start_point.x, end_point.x)
            };

            let min_y = if self.contains_angle(PI / 2.0) {
                self.center.y - self.radius
            } else {
                min!(start_point.y, end_point.y)
            };

            let max_y = if self.contains_angle(1.5 * PI) {
                self.center.y + self.radius
            } else {
                max!(start_point.y, end_point.y)
            };

            (Range::new(min_x, max_x), Range::new(min_y, max_y))
        }

        /// Returns `true` if the arc covers the given angle and `false` otherwise.
        pub(super) fn contains_angle(&self, angle: f64) -> bool {
            self.difference(angle).abs() <= self.end_difference.abs()
        }

        /// Returns the location of the given point relative to the arc.
        pub(super) fn location(&self, point: &Point) -> ArcLocation {
            let direction = *point - self.center;

            let angle = ((-direction.y).atan2(direction.x) + TWO_PI) % TWO_PI;

            let angle_difference = self.difference(angle).abs();
            let end_difference = self.end_difference.abs();

            if angle_difference <= end_difference {
                ArcLocation::Inside(angle_difference / end_difference)
            } else if (angle_difference - end_difference) > (TWO_PI - end_difference) / 2.0 {
                ArcLocation::Before
            } else {
                ArcLocation::After
            }
        }
    }

    /// Location of a `Point` relative to an `Arc`
    pub(super) enum ArcLocation {
        /// The point lies inside the circular sector traced out by the arc
        /// (extended to an infinite radius), at the given position (between `0` and `1`)
        Inside(f64),
        /// The point lies inside the inverse of the circular sector traced out by the arc,
        /// within the half that touches the start of the arc
        Before,
        /// The point lies inside the inverse of the circular sector traced out by the arc,
        /// within the half that touches the end of the arc
        After,
    }

    //----------------------------------------------------------
    // Colors
    //----------------------------------------------------------

    /// Color in RGB color space
    #[derive(Copy, Clone)]
    pub(super) struct Color {
        /// Red component of the color (between `0` and `1`)
        pub red: f64,
        /// Green component of the color (between `0` and `1`)
        pub green: f64,
        /// Blue component of the color (between `0` and `1`)
        pub blue: f64,
    }

    impl Color {
        /// Creates a new `Color` object.
        pub(super) fn new(red: f64, green: f64, blue: f64) -> Color {
            assert!((0.0..=1.0).contains(&red));
            assert!((0.0..=1.0).contains(&green));
            assert!((0.0..=1.0).contains(&blue));
            Color { red, green, blue }
        }

        /// Returns the component-wise arithmetic mean of the given colors.
        pub(super) fn average(colors: &[Color]) -> Color {
            assert!(!colors.is_empty());

            let mut red = 0.0;
            let mut green = 0.0;
            let mut blue = 0.0;

            for color in colors {
                red += color.red;
                green += color.green;
                blue += color.blue;
            }

            let count = colors.len() as f64;

            Color::new(red / count, green / count, blue / count)
        }

        /// Returns the component-wise weighted linear interpolation between
        /// this color (`balance = 0`) and the given color (`balance = 1`).
        pub(super) fn interpolate(&self, color: &Color, balance: f64) -> Color {
            assert!((0.0..=1.0).contains(&balance));

            let inverse_balance = 1.0 - balance;

            Color::new(
                (self.red * inverse_balance) + (color.red * balance),
                (self.green * inverse_balance) + (color.green * balance),
                (self.blue * inverse_balance) + (color.blue * balance),
            )
        }

        /// Returns an ANSI Select Graphic Rendition representation of the color.
        pub(super) fn sgr(&self, true_color: bool) -> String {
            if true_color {
                let r = (self.red * 255.0).round() as u8;
                let g = (self.green * 255.0).round() as u8;
                let b = (self.blue * 255.0).round() as u8;
                format!("2;{r};{g};{b}")
            } else {
                let r = (self.red * 5.0).round() as u8;
                let g = (self.green * 5.0).round() as u8;
                let b = (self.blue * 5.0).round() as u8;
                // Formula from https://en.wikipedia.org/wiki/ANSI_escape_code
                format!("5;{}", 16 + (36 * r) + (6 * g) + b)
            }
        }
    }

    /// Multi-step linear color gradient
    ///
    /// Positions must be between `0` (start) and `1` (end).
    /// Steps must be ordered by position (ascending).
    pub(super) struct Gradient(pub Vec<(f64, Color)>);

    impl Gradient {
        /// Returns the interpolated color at the given position in the gradient.
        pub(super) fn color(&self, position: f64) -> Color {
            assert!((0.0..=1.0).contains(&position));

            let steps = &self.0;
            assert!(!steps.is_empty());
            let length = steps.len();

            if position <= steps[0].0 {
                steps[0].1
            } else if position >= steps[length - 1].0 {
                steps[length - 1].1
            } else {
                for i in 0..(length - 1) {
                    let start = steps[i].0;
                    let end = steps[i + 1].0;

                    assert!(start <= end);

                    if start <= position && position < end {
                        // Note that `start < end` per the above condition,
                        // so division by zero cannot occur here
                        let balance = (position - start) / (end - start);
                        return steps[i].1.interpolate(&steps[i + 1].1, balance);
                    }
                }

                unreachable!();
            }
        }
    }

    //----------------------------------------------------------
    // Command line parsing
    //----------------------------------------------------------

    /// Basic command line argument parser
    struct Arguments(HashMap<String, String>);

    impl Arguments {
        /// Parses an argument vector whose elements are of the form `name=value`.
        fn parse(args: Vec<String>) -> Result<Arguments, String> {
            let mut arguments = HashMap::new();

            for arg in args {
                let parts: Vec<&str> = arg.splitn(2, '=').collect();
                if parts.len() < 2 {
                    return err!(
                        "Invalid argument '{}': Arguments must be of the form 'name=value'.",
                        arg
                    );
                }

                let name = parts[0].trim();
                if name.is_empty() {
                    return err!("Invalid argument '{}': Name must not be empty.", arg);
                }
                if arguments.contains_key(name) {
                    return err!("Duplicate argument '{}'.", name);
                }

                let value =
                    parts[1].trim_matches(|c: char| c.is_whitespace() || c == '"' || c == '\'');

                arguments.insert(name.to_owned(), value.to_owned());
            }

            Ok(Arguments(arguments))
        }

        /// Returns the value of the argument `name`, or `default` if it does not exist.
        fn value<T>(&self, name: &str, default: T) -> Result<T, String>
        where
            T: FromStr,
            <T as FromStr>::Err: Display,
        {
            match self.0.get(name) {
                Some(value_string) => match value_string.parse() {
                    Ok(value) => Ok(value),
                    Err(error) => err!(
                        "Invalid value '{}' for argument '{}': {}",
                        value_string,
                        name,
                        error
                    ),
                },
                None => Ok(default),
            }
        }

        /// Returns the value of the argument `name` as a comma-separated list,
        /// or `default` if it does not exist.
        fn values<T>(&self, name: &str, default: Vec<T>) -> Result<Vec<T>, String>
        where
            T: FromStr,
            <T as FromStr>::Err: Display,
        {
            match self.0.get(name) {
                Some(value_string) => {
                    let mut values = vec![];

                    for part in value_string.split(',') {
                        values.push(match part.parse::<T>() {
                            Ok(value) => value,
                            Err(error) => {
                                return err!(
                                    "Invalid part '{}' in argument '{}': {}",
                                    part,
                                    name,
                                    error
                                )
                            }
                        });
                    }

                    Ok(values)
                }
                None => Ok(default),
            }
        }
    }

    /// `Range` parser
    impl<T> FromStr for Range<T>
    where
        T: FromStr + Copy + PartialOrd,
        <T as FromStr>::Err: Display,
    {
        type Err = String;

        fn from_str(s: &str) -> Result<Range<T>, String> {
            let parts: Vec<&str> = s.split(',').collect();
            let len = parts.len();

            if len == 1 || len == 2 {
                let from = match parts[0].parse::<T>() {
                    Ok(value) => value,
                    Err(error) => return err!("Invalid range part '{}': {}", parts[0], error),
                };

                let to = if len == 1 {
                    from
                } else {
                    match parts[1].parse::<T>() {
                        Ok(value) => value,
                        Err(error) => return err!("Invalid range part '{}': {}", parts[1], error),
                    }
                };

                if from <= to {
                    Ok(Range::new(from, to))
                } else {
                    err!(
                        "Invalid range '{}': {} is greater than {}.",
                        s,
                        parts[0],
                        parts[1]
                    )
                }
            } else {
                err!(
                    "Invalid range '{}': There must be 1 or 2 parts; {} were supplied.",
                    s,
                    len
                )
            }
        }
    }

    impl From<u32> for Color {
        fn from(v: u32) -> Color {
            /// Get RGB components of a 32 bits color value.
            #[allow(clippy::many_single_char_names)]
            pub(crate) fn color_to_rgba(c: u32) -> (u8, u8, u8, u8) {
                let a = u8::try_from((c >> 24) & 0xff).unwrap();
                let r = u8::try_from((c >> 16) & 0xff).unwrap();
                let g = u8::try_from((c >> 8) & 0xff).unwrap();
                let b = u8::try_from(c & 0xff).unwrap();

                (r, g, b, a)
            }

            let c = color_to_rgba(v);

            Color::new(c.0 as f64 / 255.0, c.1 as f64 / 255.0, c.2 as f64 / 255.0)
        }
    }

    /// `Color` parser
    impl FromStr for Color {
        type Err = String;

        fn from_str(s: &str) -> Result<Color, String> {
            if s.len() == 7 && s.starts_with('#') {
                let mut rgb = [0.0, 0.0, 0.0];

                for i in 0..3 {
                    let part = &s[((2 * i) + 1)..((2 * i) + 3)];

                    if let Ok(value) = u8::from_str_radix(part, 16) {
                        rgb[i] = (value as f64) / 255.0;
                    } else {
                        return err!("Invalid color literal '{}': Color components must be hexadecimal numbers.", s);
                    }
                }

                Ok(Color::new(rgb[0], rgb[1], rgb[2]))
            } else {
                err!(
                    "Invalid color literal '{}': Colors must be of the form '#RRGGBB'.",
                    s
                )
            }
        }
    }

    /// `Gradient` parser
    impl FromStr for Gradient {
        type Err = String;

        fn from_str(s: &str) -> Result<Gradient, String> {
            let mut steps = vec![];

            let mut last_position = NEG_INFINITY;

            for step in s.split(',') {
                let parts: Vec<&str> = step.split(':').collect();
                if parts.len() != 2 {
                    return err!(
                        "Invalid gradient step '{}': Steps must be of the form '0.0:#RRGGBB'.",
                        step
                    );
                }

                let position = match parts[0].parse::<f64>() {
                    Ok(value) => value,
                    Err(_) => {
                        return err!(
                            "Invalid gradient step position '{}': Positions must be numbers.",
                            parts[0]
                        )
                    }
                };
                if !(0.0..=1.0).contains(&position) {
                    return err!(
                        "Invalid gradient step position '{}': Positions must be between 0 and 1.",
                        position
                    );
                }
                if position < last_position {
                    return err!(
                        "Invalid gradient step position '{}': \
                        Positions must not be less than preceding position.",
                        position
                    );
                }
                last_position = position;

                let color = match parts[1].parse::<Color>() {
                    Ok(value) => value,
                    Err(error) => {
                        return err!("Invalid gradient step color '{}': {}", parts[1], error)
                    }
                };

                steps.push((position, color));
            }

            Ok(Gradient(steps))
        }
    }

    //----------------------------------------------------------
    // Miscellaneous
    //----------------------------------------------------------

    /// Linear congruential pseudorandom number generator
    pub(super) struct Random(pub u32);

    impl Random {
        // 2^31 - 1
        const MODULUS: u32 = 2147483647;

        /// Returns the next number in the pseudorandom sequence.
        pub(super) fn next(&mut self) -> u32 {
            // Parameters from glibc (64-bit arithmetic is needed to avoid overflow)
            self.0 = (((1103515245 * (self.0 as u64)) + 12345) % (Random::MODULUS as u64)) as u32;
            self.0
        }

        /// Returns a pseudorandom number between `0` and `1` (inclusive).
        pub(super) fn real(&mut self) -> f64 {
            (self.next() as f64) / ((Random::MODULUS - 1) as f64)
        }

        /// Returns a pseudorandom number between `min` and `max` (inclusive).
        pub(super) fn real_range(&mut self, min: f64, max: f64) -> f64 {
            assert!(min <= max);
            (self.real() * (max - min)) + min
        }

        /// Returns a pseudorandom number between `min` and `max` (inclusive),
        /// but not equal to any number in `except`.
        pub(super) fn real_range_except(&mut self, min: f64, max: f64, except: &[f64]) -> f64 {
            loop {
                let number = self.real_range(min, max);
                if !except.contains(&number) {
                    break number;
                }
            }
        }

        /// Returns `true` or `false` at random with equal probability.
        pub(super) fn flip(&mut self) -> bool {
            self.real() < 0.5
        }
    }

    /// Ordered interval (not to be confused with `std::ops::Range`)
    pub(super) struct Range<T> {
        pub from: T,
        pub to: T,
    }

    impl<T: PartialOrd> Range<T> {
        /// Creates a new `Range` object.
        pub(super) fn new(from: T, to: T) -> Range<T> {
            assert!(from <= to);
            Range { from, to }
        }

        /// Returns `true` if `value` lies within the range and `false` otherwise.
        pub(super) fn contains(&self, value: T) -> bool {
            self.from <= value && value <= self.to
        }
    }

    /// Returns the success value of `result` if it exists.
    /// Otherwise, prints the error value and exits the program.
    fn unwrap_or_exit<T, E: Display>(result: Result<T, E>) -> T {
        result.unwrap_or_else(|error| {
            exit!("{}", error);
        })
    }

    /// Returns the given duration, converted to (fractional) seconds.
    pub(super) fn seconds(duration: Duration) -> f64 {
        (duration.as_secs() as f64) + ((duration.subsec_nanos() as f64) / 1_000_000_000.0)
    }
}
