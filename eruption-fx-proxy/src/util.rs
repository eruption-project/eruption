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

use eruption_sdk::canvas::Canvas;
use image::{imageops::FilterType, DynamicImage, ImageBuffer, Rgba};

use crate::constants;

type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum UtilError {
    // #[error("Invalid argument")]
    // InvalidArgument {},
}

/// Post-processing and conversion of an image buffer to be used with the Eruption SDK
pub fn process_image_buffer(buffer: ImageBuffer<Rgba<u8>, Vec<u8>>) -> Result<Canvas> {
    let img = DynamicImage::ImageRgba8(buffer);

    // resize to match the Eruption virtual canvas; this may change the aspect-ratio of the image
    let img = img.resize_exact(
        constants::CANVAS_WIDTH as u32,
        constants::CANVAS_HEIGHT as u32,
        FilterType::Lanczos3,
    );

    // apply post-processing filters
    // let img = img.adjust_contrast(1.25);
    // let img = img.blur(4.0);

    // convert to a Eruption SDK canvas
    let result = Canvas::from(img.into_bytes());

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
