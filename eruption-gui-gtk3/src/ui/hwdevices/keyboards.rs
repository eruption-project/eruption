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

use crate::{dbus_client, util::RGBA};

pub mod corsair_strafe;
pub mod generic_keyboard;
pub mod null_keyboard;
pub mod roccat_magma;
pub mod roccat_pyro;
pub mod roccat_vulcan_1xx;
pub mod roccat_vulcan_pro;
pub mod roccat_vulcan_pro_tkl;
pub mod roccat_vulcan_tkl;
pub mod wooting_two_he_arm;

type Result<T> = std::result::Result<T, eyre::Error>;

// #[derive(Debug, thiserror::Error)]
// pub enum HwDevicesError {
//     #[error("The device is not supported")]
//     UnsupportedDevice,
// }

pub fn get_keyboard_device(device_handle: u64) -> Result<Box<dyn Keyboard>> {
    // let devices = dbus_client::get_managed_devices()?;

    match dbus_client::get_managed_devices()?
        .0
        .get(device_handle as usize)
    {
        Some(device) => match device {
            // Wooting Two HE (ARM)
            (0x31e3, 0x1230) => Ok(Box::new(wooting_two_he_arm::WootingTwoHeArm::new(
                device_handle,
            ))),

            // ROCCAT Vulcan 1xx series
            (0x1e7d, 0x3098) | (0x1e7d, 0x307a) => Ok(Box::new(
                roccat_vulcan_1xx::RoccatVulcan1xx::new(device_handle),
            )),

            // ROCCAT Vulcan Pro series
            (0x1e7d, 0x30f7) => Ok(Box::new(roccat_vulcan_pro::RoccatVulcanPro::new(
                device_handle,
            ))),

            // ROCCAT Vulcan Pro TKL series
            (0x1e7d, 0x311a) => Ok(Box::new(roccat_vulcan_pro_tkl::RoccatVulcanProTKL::new(
                device_handle,
            ))),

            // ROCCAT Vulcan TKL series
            (0x1e7d, 0x2fee) => Ok(Box::new(roccat_vulcan_tkl::RoccatVulcanTKL::new(
                device_handle,
            ))),

            // ROCCAT Magma
            (0x1e7d, 0x3124) => Ok(Box::new(roccat_magma::RoccatMagma::new(device_handle))),

            // ROCCAT Pyro
            (0x1e7d, 0x314C) => Ok(Box::new(roccat_pyro::RoccatPyro::new(device_handle))),

            // Corsair STRAFE series
            (0x1b1c, 0x1b15) => Ok(Box::new(corsair_strafe::CorsairStrafe::new(device_handle))),

            _ => Ok(Box::new(generic_keyboard::GenericKeyboard::new(
                device_handle,
            ))),
        },

        _ => Ok(Box::new(null_keyboard::NullKeyboard::new())),
    }
}

pub trait Keyboard {
    fn get_device(&self) -> u64;

    fn get_make_and_model(&self) -> (&'static str, &'static str);

    /// Draw an animated keyboard with live action colors
    fn draw_keyboard(&self, _da: &gtk::DrawingArea, context: &cairo::Context) -> Result<()>;

    fn paint_key(
        &self,
        key: usize,
        color: &RGBA,
        cr: &cairo::Context,
        layout: &pango::Layout,
    ) -> Result<()>;

    fn get_key_defs(&self, layout: &str) -> &[KeyDef];
}

#[derive(Debug, PartialEq)]
pub struct KeyDef<'a> {
    is_dummy: bool,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    caption: Caption<'a>,
    // index: usize,
}

impl<'a> KeyDef<'a> {
    const fn new(
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        caption: Caption<'a>,
        _index: usize,
    ) -> Self {
        Self {
            is_dummy: false,
            x,
            y,
            width,
            height,
            caption,
            // index, // currently only included for documentation purposes
        }
    }

    const fn dummy(_index: usize) -> Self {
        Self {
            is_dummy: true,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            caption: Caption::simple(""),
            // index, // currently only included for documentation purposes
        }
    }
}

#[derive(Debug, PartialEq)]
struct Caption<'a> {
    text: &'a str,
    x_offset: f64,
    y_offset: f64,
}

impl<'a> Caption<'a> {
    const fn new(text: &'a str, x_offset: f64, y_offset: f64) -> Self {
        Self {
            text,
            x_offset,
            y_offset,
        }
    }

    const fn simple(text: &'a str) -> Self {
        Self {
            text,
            x_offset: 0.0,
            y_offset: 0.0,
        }
    }
}
