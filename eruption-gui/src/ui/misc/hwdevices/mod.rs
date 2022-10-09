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

mod generic_misc_device;
mod null_misc_device;
mod roccat_aimo_pad;
mod roccat_elo_71_air;

type Result<T> = std::result::Result<T, eyre::Error>;

// #[derive(Debug, thiserror::Error)]
// pub enum HwDevicesError {
//     #[error("The device is not supported")]
//     UnsupportedDevice,
// }

pub fn get_misc_device(device_handle: u64) -> Result<Box<dyn MiscDevice>> {
    let devices = dbus_client::get_managed_devices()?;

    match dbus_client::get_managed_devices()?
        .2
        .get(device_handle as usize - (devices.0.len() + devices.1.len()))
    {
        Some(device) => match device {
            // ROCCAT/Turtle Beach Elo 7.1 Air
            (0x1e7d, 0x3a37) => Ok(Box::new(roccat_elo_71_air::RoccatElo71Air::new(
                device_handle,
            ))),

            // ROCCAT Aimo Pad Wide
            (0x1e7d, 0x343b) => Ok(Box::new(roccat_aimo_pad::RoccatAimoPad::new(device_handle))),

            _ => Ok(Box::new(generic_misc_device::GenericMiscDevice::new(
                device_handle,
            ))),
        },

        _ => Ok(Box::new(null_misc_device::NullMiscDevice::new())),
    }
}

pub struct Rectangle {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

pub trait MiscDevice {
    fn get_device(&self) -> u64;

    fn get_make_and_model(&self) -> (&'static str, &'static str);

    fn draw(&self, _da: &gtk::DrawingArea, context: &cairo::Context) -> Result<()>;

    /// Paint a cell on the Misc device widget
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
