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

mod roccat_kone_pure_ultra;
mod roccat_kone_aimo;
mod roccat_nyth;

use hidapi::HidDevice;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Error, Debug)]
enum HwDeviceError {
    #[error("The device is not bound")]
    DeviceNotBound,

    #[error("Invalid result")]
    InvalidResult {},

    #[error("Invalid status code")]
    InvalidStatusCode {},

    #[error("The device is not supported")]
    DeviceNotSupported,
}

#[derive(Debug, Copy, Clone)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

pub trait DeviceTrait {
    fn send_init_sequence(&self) -> Result<()>;

    fn write_data_raw(&self, buf: &[u8]) -> Result<()>;
    fn read_data_raw(&self, size: usize) -> Result<Vec<u8>>;

    fn send_led_map(&self, led_map: &[RGBA]) -> Result<()>;
    fn send_test_pattern(&self) -> Result<()>;
}

pub fn bind_device(
    hiddev: HidDevice,
    vendor_id: u16,
    product_id: u16,
) -> Result<Box<dyn DeviceTrait>> {
    hiddev.set_blocking_mode(true)?;

    match (vendor_id, product_id) {
        // ROCCAT Kone Pure Ultra
        (0x1e7d, 0x2dd2) => Ok(Box::new(roccat_kone_pure_ultra::RoccatKonePureUltra::bind(
            hiddev,
        ))),

        // ROCCAT Kone Aimo
        (0x1e7d, 0x2e27) => Ok(Box::new(roccat_kone_aimo::RoccatKoneAimo::bind(hiddev))),

        // ROCCAT Nyth
        (0x1e7d, 0x2e7c) | (0x1e7d, 0x2e7d) => Ok(Box::new(roccat_nyth::RoccatNyth::bind(hiddev))),

        _ => Err(HwDeviceError::DeviceNotSupported.into()),
    }
}
