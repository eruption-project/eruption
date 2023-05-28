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

use crate::{
    dbus_client,
    hwdevices::{self, KeyboardDevice},
};
use eruption_sdk::{canvas::Canvas, color::Color};
use image::{imageops::FilterType, DynamicImage, GenericImageView, ImageBuffer, Rgba};

type Result<T> = std::result::Result<T, eyre::Error>;

/// Converts an image buffer to fit a specific device topology
pub fn process_image_buffer(
    buffer: ImageBuffer<Rgba<u8>, Vec<u8>>,
    device: &KeyboardDevice,
) -> Result<Canvas> {
    let mut result = Canvas::new();

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

            result[key_index] = Color::new(r, g, b, a);
        }
    }

    Ok(result)
}

// pub fn blend(_canvas: &mut Canvas, _src: &Canvas) {
//     let brightness = 1.0;

//     for (idx, bg) in canvas.iter_mut().enumerate() {
//         let fg = src[idx];

//         #[rustfmt::skip]
//         let color = Color::new(
//             ((((fg.a() as f32) * fg.r() as f32 + (255 - fg.a()) as f32 * bg.r() as f32).floor() * brightness as f32 / 100.0) as u32 >> 8) as u8,
//             ((((fg.a() as f32) * fg.g() as f32 + (255 - fg.a()) as f32 * bg.g() as f32).floor() * brightness as f32 / 100.0) as u32 >> 8) as u8,
//             ((((fg.a() as f32) * fg.b() as f32 + (255 - fg.a()) as f32 * bg.b() as f32).floor() * brightness as f32 / 100.0) as u32 >> 8) as u8,
//             fg.a(),
//         );

//         *bg = color;
//     }
// }

pub fn get_primary_keyboard_device() -> Result<KeyboardDevice> {
    let (keyboards, _mice, _misc) = dbus_client::get_managed_devices()?;

    let usb_vid = keyboards.get(0).unwrap_or_else(|| &(0, 0)).0;
    let usb_pid = keyboards.get(0).unwrap_or_else(|| &(0, 0)).1;

    let device = hwdevices::get_keyboard_device(usb_vid, usb_pid)?;

    Ok(device)
}
