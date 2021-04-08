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

use crate::dbus_client;

mod generic_misc_device;
mod null_misc_device;

type Result<T> = std::result::Result<T, eyre::Error>;

// #[derive(Debug, thiserror::Error)]
// pub enum HwDevicesError {
//     #[error("The device is not supported")]
//     UnsupportedDevice,
// }

pub fn get_misc_devices() -> Result<Box<dyn MiscDevice>> {
    match dbus_client::get_managed_devices()?.2.get(0) {
        Some(device) => match device {
            _ => Ok(Box::new(generic_misc_device::GenericMiscDevice::new())),
        },

        _ => Ok(Box::new(null_misc_device::NullMiscDevice::new())),
    }
}

pub trait MiscDevice {
    fn get_make_and_model(&self) -> (&'static str, &'static str);

    fn draw(&self, _da: &gtk::DrawingArea, context: &cairo::Context) -> Result<()>;
}
