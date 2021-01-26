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

use crate::xwrap::Image;
use image::{imageops::FilterType, DynamicImage, GenericImageView};
use std::path::Path;

type Result<T> = std::result::Result<T, eyre::Error>;

/// Converts an image buffer to a Network FX command stream
pub fn process_image_buffer(buffer: &[u8]) -> Result<String> {
    let mut result = String::new();

    let img = image::load_from_memory(&buffer)?;
    let img = img.resize_exact(NUM_COLS as u32, NUM_ROWS as u32, FilterType::Gaussian);

    for x in 0..NUM_COLS {
        for y in 0..NUM_ROWS {
            let key_index: usize = (ROWS_TOPOLOGY[x + (y * (NUM_COLS + 1))]) as usize + 1;

            if !(1..=NUM_KEYS).contains(&key_index) {
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
pub fn process_image_file<P: AsRef<Path>>(filename: P) -> Result<String> {
    let mut result = String::new();

    let filename = filename.as_ref();

    let img = image::open(&filename)?;
    let img = img.resize_exact(NUM_COLS as u32, NUM_ROWS as u32, FilterType::Gaussian);

    for x in 0..NUM_COLS {
        for y in 0..NUM_ROWS {
            let key_index: usize = (ROWS_TOPOLOGY[x + (y * (NUM_COLS + 1))]) as usize + 1;

            if !(1..=NUM_KEYS).contains(&key_index) {
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
pub fn process_screenshot(image: &Image) -> Result<String> {
    let mut result = String::new();

    let buffer = image.into_image_buffer().unwrap();
    let img = DynamicImage::ImageRgba8(buffer);
    let img = img.resize_exact(NUM_COLS as u32, NUM_ROWS as u32, FilterType::Gaussian);

    for x in 0..NUM_COLS {
        for y in 0..NUM_ROWS {
            let key_index: usize = (ROWS_TOPOLOGY[x + (y * (NUM_COLS + 1))]) as usize + 1;

            if !(1..=NUM_KEYS).contains(&key_index) {
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

const NUM_KEYS: usize = 144;
const NUM_ROWS: usize = 6;
const NUM_COLS: usize = 21;

// const ARRAY_OFFSET: usize = 132;

#[rustfmt::skip]
const ROWS_TOPOLOGY: [u8; 264] = [
    // ISO model
    0x00, 0x0b, 0x11, 0x17, 0x1c, 0x30, 0x35, 0x3b, 0x41, 0x4e, 0x54, 0x55, 0x56, 0x63, 0x67, 0x6c,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01, 0x06, 0x0c, 0x12, 0x18, 0x1d, 0x21, 0x31, 0x36, 0x3c,
    0x42, 0x48, 0x4f, 0x57, 0x64, 0x68, 0x6d, 0x71, 0x77, 0x7c, 0x81, 0xff, 0x02, 0x07, 0x0d, 0x13,
    0x19, 0x1e, 0x22, 0x32, 0x37, 0x3d, 0x43, 0x49, 0x50, 0x58, 0x65, 0x69, 0x6e, 0x72, 0x78, 0x7d,
    0x82, 0xff, 0x03, 0x08, 0x0e, 0x14, 0x1a, 0x1f, 0x23, 0x33, 0x38, 0x3e, 0x44, 0x4a, 0x60, 0x73,
    0x79, 0x7e, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x04, 0x09, 0x0f, 0x15, 0x1b, 0x20, 0x24, 0x34,
    0x39, 0x3f, 0x45, 0x4b, 0x52, 0x6a, 0x74, 0x7a, 0x7f, 0x83, 0xff, 0xff, 0xff, 0xff, 0x05, 0x0a,
    0x10, 0x25, 0x46, 0x4c, 0x53, 0x59, 0x66, 0x6b, 0x6f, 0x75, 0x80, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff,

    // ANSI model
    0x00, 0x0b, 0x11, 0x17, 0x1c, 0x30, 0x35, 0x3b, 0x41, 0x4e, 0x54, 0x55, 0x56, 0x63, 0x67, 0x6c,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01, 0x06, 0x0c, 0x12, 0x18, 0x1d, 0x21, 0x31, 0x36, 0x3c,
    0x42, 0x48, 0x4f, 0x57, 0x64, 0x68, 0x6d, 0x71, 0x77, 0x7c, 0x81, 0xff, 0x02, 0x07, 0x0d, 0x13,
    0x19, 0x1e, 0x22, 0x32, 0x37, 0x3d, 0x43, 0x49, 0x50, 0x51, 0x65, 0x69, 0x6e, 0x72, 0x78, 0x7d,
    0x82, 0xff, 0x03, 0x08, 0x0e, 0x14, 0x1a, 0x1f, 0x23, 0x33, 0x38, 0x3e, 0x44, 0x4a, 0x58, 0x73,
    0x79, 0x7e, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x04, 0x0f, 0x15, 0x1b, 0x20, 0x24, 0x34, 0x39,
    0x3f, 0x45, 0x4b, 0x52, 0x6a, 0x74, 0x7a, 0x7f, 0x83, 0xff, 0xff, 0xff, 0xff, 0xff, 0x05, 0x0a,
    0x10, 0x25, 0x46, 0x4c, 0x53, 0x59, 0x66, 0x6b, 0x6f, 0x75, 0x80, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff,
];
