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

use crate::{hwdevices::KeyboardDevice, xwrap::Image};
use image::{imageops::FilterType, DynamicImage, GenericImageView};
use std::path::Path;

type Result<T> = std::result::Result<T, eyre::Error>;

/// Converts an image buffer to a Network FX command stream
pub fn process_image_buffer(buffer: &[u8], device: &KeyboardDevice) -> Result<String> {
    let mut result = String::new();

    let img = image::load_from_memory(&buffer)?;
    let img = img.resize_exact(
        device.get_num_cols() as u32,
        device.get_num_rows() as u32,
        FilterType::Gaussian,
    );

    for x in 0..device.get_num_cols() {
        for y in 0..device.get_num_rows() {
            let key_index: usize =
                (device.get_rows_topology()[x + (y * (device.get_num_cols() + 1))]) as usize + 1;

            if !(1..=device.get_num_keys()).contains(&key_index) {
                continue;
            }

            let pixel = img.get_pixel(x as u32, y as u32);

            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];
            let a = pixel[3];

            result += &format!("{}:{}:{}:{}:{}\n", key_index, r, g, b, a);
        }
    }

    Ok(result)
}

/// Loads and converts an image file to a Network FX command stream
pub fn process_image_file<P: AsRef<Path>>(filename: P, device: &KeyboardDevice) -> Result<String> {
    let mut result = String::new();

    let filename = filename.as_ref();

    let img = image::open(&filename)?;
    let img = img.resize_exact(
        device.get_num_cols() as u32,
        device.get_num_rows() as u32,
        FilterType::Gaussian,
    );

    for x in 0..device.get_num_cols() {
        for y in 0..device.get_num_rows() {
            let key_index: usize =
                (device.get_rows_topology()[x + (y * (device.get_num_cols() + 1))]) as usize + 1;

            if !(1..=device.get_num_keys()).contains(&key_index) {
                continue;
            }

            let pixel = img.get_pixel(x as u32, y as u32);

            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];
            let a = pixel[3];

            result += &format!("{}:{}:{}:{}:{}\n", key_index, r, g, b, a);
        }
    }

    Ok(result)
}

/// Converts an image buffer to a Network FX command stream
pub fn process_screenshot(image: &Image, device: &KeyboardDevice) -> Result<String> {
    let mut result = String::new();

    let buffer = image.into_image_buffer().unwrap();
    let img = DynamicImage::ImageRgba8(buffer);
    let img = img.resize_exact(
        device.get_num_cols() as u32,
        device.get_num_rows() as u32,
        FilterType::Gaussian,
    );

    for x in 0..device.get_num_cols() {
        for y in 0..device.get_num_rows() {
            let key_index: usize =
                (device.get_rows_topology()[x + (y * (device.get_num_cols() + 1))]) as usize + 1;

            if !(1..=device.get_num_keys()).contains(&key_index) {
                continue;
            }

            let pixel = img.get_pixel(x as u32, y as u32);

            let r = pixel[0];
            let g = pixel[1];
            let b = pixel[2];
            let a = pixel[3];

            result += &format!("{}:{}:{}:{}:{}\n", key_index, r, g, b, a);
        }
    }

    Ok(result)
}
