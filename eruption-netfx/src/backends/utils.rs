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

use crate::constants;

#[cfg(feature = "backend-x11")]
use crate::backends::x11::xwrap::Image;

use crate::hwdevices::KeyboardDevice;
use image::{imageops::FilterType, GenericImageView};

#[allow(unused_imports)]
use image::DynamicImage;

use std::path::Path;

#[allow(dead_code)]
type Result<T> = std::result::Result<T, eyre::Error>;

/// Converts an image buffer to a Network FX command stream
#[allow(dead_code)]
pub fn process_image_buffer(buffer: &[u8], _device: &KeyboardDevice) -> Result<String> {
    let mut result = String::new();

    let img = image::load_from_memory(buffer)?;
    let img = img.resize_exact(
        constants::CANVAS_WIDTH as u32,
        constants::CANVAS_HEIGHT as u32,
        FilterType::Lanczos3,
    );

    for y in 0..constants::CANVAS_HEIGHT {
        for x in 0..constants::CANVAS_WIDTH {
            let index: usize = x + (y * (constants::CANVAS_WIDTH));

            let pixel = img.get_pixel(x as u32, y as u32);

            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];
            let a = pixel[3];

            result += &format!("{}:{}:{}:{}:{}\n", index, r, g, b, a);
        }
    }

    Ok(result)
}

/// Loads and converts an image file to a Network FX command stream
#[allow(dead_code)]
pub fn process_image_file<P: AsRef<Path>>(filename: P, _device: &KeyboardDevice) -> Result<String> {
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

            result += &format!("{}:{}:{}:{}:{}\n", index, r, g, b, a);
        }
    }

    Ok(result)
}

/// Converts an image buffer to a Network FX command stream
#[cfg(feature = "backend-x11")]
pub fn process_screenshot(image: &Image, _device: &KeyboardDevice) -> Result<String> {
    let buffer = image.into_image_buffer().unwrap();
    let img = DynamicImage::ImageRgba8(buffer);
    let mut result = String::new();

    let img = img.resize_exact(
        constants::CANVAS_WIDTH as u32,
        constants::CANVAS_HEIGHT as u32,
        FilterType::Lanczos3,
    );

    for y in 0..constants::CANVAS_HEIGHT {
        for x in 0..constants::CANVAS_WIDTH {
            let index: usize = x + (y * (constants::CANVAS_WIDTH));

            let pixel = img.get_pixel(x as u32, y as u32);

            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];
            let a = pixel[3];

            result += &format!("{}:{}:{}:{}:{}\n", index, r, g, b, a);
        }
    }

    Ok(result)
}
