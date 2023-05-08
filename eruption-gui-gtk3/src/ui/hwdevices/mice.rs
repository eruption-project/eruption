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

use crate::{dbus_client, util::RGBA};

pub mod generic_mouse;
pub mod null_mouse;
pub mod roccat_burst_pro;
pub mod roccat_kain_100;
pub mod roccat_kain_2xx;
pub mod roccat_kone_aimo;
pub mod roccat_kone_aimo_remastered;
pub mod roccat_kone_pro_air;
pub mod roccat_kone_pure_ultra;
pub mod roccat_kone_xp;
pub mod roccat_kone_xtd;
pub mod roccat_kova_2016;
pub mod roccat_kova_aimo;
pub mod roccat_nyth;

pub type Result<T> = std::result::Result<T, eyre::Error>;

// #[derive(Debug, thiserror::Error)]
// pub enum HwDevicesError {
//     #[error("The device is not supported")]
//     UnsupportedDevice,
// }

pub fn get_mouse_device(device_handle: u64) -> Result<Box<dyn Mouse>> {
    let devices = dbus_client::get_managed_devices()?;

    match devices.1.get(device_handle as usize - devices.0.len()) {
        Some(device) => match device {
            // ROCCAT Kone XP
            (0x1e7d, 0x2c8b) => Ok(Box::new(roccat_kone_xp::RoccatKoneXp::new(device_handle))),

            // ROCCAT Kone XTD
            (0x1e7d, 0x2e22) => Ok(Box::new(roccat_kone_xtd::RoccatKoneXtd::new(device_handle))),

            // ROCCAT Kone Pure Ultra
            (0x1e7d, 0x2dd2) => Ok(Box::new(roccat_kone_pure_ultra::RoccatKonePureUltra::new(
                device_handle,
            ))),

            // ROCCAT Kone Pro Air
            (0x1e7d, 0x2c8e) | (0x1e7d, 0x2c92) => Ok(Box::new(
                roccat_kone_pro_air::RoccatKoneProAir::new(device_handle),
            )),

            // ROCCAT Burst Pro
            (0x1e7d, 0x2de1) => Ok(Box::new(roccat_burst_pro::RoccatBurstPro::new(
                device_handle,
            ))),

            // ROCCAT Kain 100
            (0x1e7d, 0x2d00) => Ok(Box::new(roccat_kain_100::RoccatKain100::new(device_handle))),

            // ROCCAT Kain 2xx
            (0x1e7d, 0x2d5f) | (0x1e7d, 0x2d60) => {
                Ok(Box::new(roccat_kain_2xx::RoccatKain2xx::new(device_handle)))
            }

            // ROCCAT Kone Aimo
            (0x1e7d, 0x2e27) => Ok(Box::new(roccat_kone_aimo::RoccatKoneAimo::new(
                device_handle,
            ))),

            // ROCCAT Kone Aimo Remastered
            (0x1e7d, 0x2e2c) => Ok(Box::new(
                roccat_kone_aimo_remastered::RoccatKoneAimoRemastered::new(device_handle),
            )),

            // ROCCAT Kova 2016
            (0x1e7d, 0x2cee) | (0x1e7d, 0x2cef) | (0x1e7d, 0x2cf0) => Ok(Box::new(
                roccat_kova_2016::RoccatKova2016::new(device_handle),
            )),

            // ROCCAT Kova Aimo
            (0x1e7d, 0x2cf1) | (0x1e7d, 0x2cf3) => Ok(Box::new(
                roccat_kova_aimo::RoccatKovaAimo::new(device_handle),
            )),

            // ROCCAT Nyth
            (0x1e7d, 0x2e7c) | (0x1e7d, 0x2e7d) => {
                Ok(Box::new(roccat_nyth::RoccatNyth::new(device_handle)))
            }

            _ => Ok(Box::new(generic_mouse::GenericMouse::new(device_handle))),
        },

        None => Ok(Box::new(null_mouse::NullMouse::new())),
    }
}

pub struct Rectangle {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

pub trait Mouse {
    fn get_device(&self) -> u64;

    fn get_make_and_model(&self) -> (&'static str, &'static str);

    /// Draw an animated mouse with live action colors
    fn draw_mouse(&self, _da: &gtk::DrawingArea, context: &cairo::Context) -> Result<()>;

    /// Paint a cell on the Mouse widget
    fn paint_cell(
        &self,
        cell_index: usize,
        color: &RGBA,
        cr: &cairo::Context,
        width: f64,
        height: f64,
        scale_factor: f64,
    ) -> Result<()>;
}
