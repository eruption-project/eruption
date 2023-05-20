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

use crate::constants;
use image::{imageops::FilterType, GenericImageView};
use std::path::Path;

type Result<T> = std::result::Result<T, eyre::Error>;

/// Post-processing and conversion of an image buffer to be used with the Eruption SDK
pub fn process_image_buffer(buffer: Vec<u8>) -> Result<String> {
    let mut result = String::new();

    // let dimensions = image::image_dimensions(filename)?;
    let img = image::load_from_memory(&buffer)?;

    // resize to match the Eruption virtual canvas; this may change the aspect-ratio of the image
    let img = img.resize_exact(
        constants::CANVAS_WIDTH as u32,
        constants::CANVAS_HEIGHT as u32,
        FilterType::Nearest,
    );

    for y in 0..constants::CANVAS_HEIGHT {
        for x in 0..constants::CANVAS_WIDTH {
            let index: usize = x + (y * constants::CANVAS_WIDTH);

            let pixel = img.get_pixel(x as u32, y as u32);

            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];
            let a = pixel[3];

            result += &format!("{index}:{r}:{g}:{b}:{a}\n");
        }
    }

    Ok(result)
}

/// Loads and converts an image file to a Network FX command stream
pub fn process_image_file<P: AsRef<Path>>(filename: P) -> Result<String> {
    let mut result = String::new();

    let filename = filename.as_ref();

    let img = image::open(filename)?;
    let img = img.resize_exact(
        constants::CANVAS_WIDTH as u32,
        constants::CANVAS_HEIGHT as u32,
        FilterType::Lanczos3,
    );

    for y in 0..constants::CANVAS_HEIGHT {
        for x in 0..constants::CANVAS_WIDTH {
            let index: usize = x + (y * constants::CANVAS_WIDTH);

            let pixel = img.get_pixel(x as u32, y as u32);

            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];
            let a = pixel[3];

            result += &format!("{index}:{r}:{g}:{b}:{a}\n");
        }
    }

    Ok(result)
}
