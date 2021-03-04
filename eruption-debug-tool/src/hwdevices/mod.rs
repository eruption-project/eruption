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

mod roccat_kone_aimo;
mod roccat_kone_aimo_remastered;
mod roccat_kone_pure_ultra;
mod roccat_kova_aimo;
mod roccat_nyth;
mod roccat_vulcan_1xx;
mod roccat_vulcan_pro;
mod roccat_vulcan_pro_tkl;
mod roccat_vulcan_tkl;

use hidapi::{HidApi, HidDevice};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Error, Debug)]
enum HwDeviceError {
    #[error("The device is not bound")]
    DeviceNotBound,

    #[error("The device is not opened")]
    DeviceNotOpened,

    #[error("Invalid result")]
    InvalidResult {},

    #[error("Write error")]
    WriteError {},

    #[error("Invalid status code")]
    InvalidStatusCode {},

    #[error("LED map error")]
    LedMapError {},

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
    hidapi: &HidApi,
    vendor_id: u16,
    product_id: u16,
) -> Result<Box<dyn DeviceTrait>> {
    hiddev.set_blocking_mode(true)?;

    match (vendor_id, product_id) {
        // Keyboard devices

        // ROCCAT Vulcan 1xx series
        (0x1e7d, 0x3098) | (0x1e7d, 0x307a) => {
            let leddev = hidapi
                .device_list()
                .find(|dev| {
                    dev.product_id() == product_id
                        && dev.vendor_id() == vendor_id
                        && dev.interface_number() == roccat_vulcan_1xx::LED_INTERFACE
                })
                .expect("Could not bind LED sub-device")
                .open_device(&hidapi)
                .expect("Could not open LED sub-device");

            Ok(Box::new(roccat_vulcan_1xx::RoccatVulcan1xx::bind(
                hiddev, leddev,
            )))
        }

        // ROCCAT Vulcan Pro series
        (0x1e7d, 0x30f7) => {
            let leddev = hidapi
                .device_list()
                .find(|dev| {
                    dev.product_id() == product_id
                        && dev.vendor_id() == vendor_id
                        && dev.interface_number() == roccat_vulcan_pro::LED_INTERFACE
                })
                .expect("Could not bind LED sub-device")
                .open_device(&hidapi)
                .expect("Could not open LED sub-device");

            Ok(Box::new(roccat_vulcan_pro::RoccatVulcanPro::bind(
                hiddev, leddev,
            )))
        }

        // ROCCAT Vulcan TKL series
        (0x1e7d, 0x2fee) => {
            let leddev = hidapi
                .device_list()
                .find(|dev| {
                    dev.product_id() == product_id
                        && dev.vendor_id() == vendor_id
                        && dev.interface_number() == roccat_vulcan_tkl::LED_INTERFACE
                })
                .expect("Could not bind LED sub-device")
                .open_device(&hidapi)
                .expect("Could not open LED sub-device");

            Ok(Box::new(roccat_vulcan_tkl::RoccatVulcanTKL::bind(
                hiddev, leddev,
            )))
        }

        // ROCCAT Vulcan Pro TKL series
        (0x1e7d, 0x311a) => {
            let leddev = hidapi
                .device_list()
                .find(|dev| {
                    dev.product_id() == product_id
                        && dev.vendor_id() == vendor_id
                        && dev.interface_number() == roccat_vulcan_pro_tkl::LED_INTERFACE
                })
                .expect("Could not bind LED sub-device")
                .open_device(&hidapi)
                .expect("Could not open LED sub-device");

            Ok(Box::new(roccat_vulcan_pro_tkl::RoccatVulcanProTKL::bind(
                hiddev, leddev,
            )))
        }

        // Mouse devices

        // ROCCAT Kone Pure Ultra
        (0x1e7d, 0x2dd2) => Ok(Box::new(roccat_kone_pure_ultra::RoccatKonePureUltra::bind(
            hiddev,
        ))),

        // ROCCAT Kone Aimo
        (0x1e7d, 0x2e27) => Ok(Box::new(roccat_kone_aimo::RoccatKoneAimo::bind(hiddev))),

        // ROCCAT Kone Aimo Remastered
        (0x1e7d, 0x2e2c) => Ok(Box::new(
            roccat_kone_aimo_remastered::RoccatKoneAimoRemastered::bind(hiddev),
        )),

        // ROCCAT Kova AIMO
        (0x1e7d, 0x2cf1) | (0x1e7d, 0x2cf3) => {
            Ok(Box::new(roccat_kova_aimo::RoccatKovaAimo::bind(hiddev)))
        }

        // ROCCAT Nyth
        (0x1e7d, 0x2e7c) | (0x1e7d, 0x2e7d) => Ok(Box::new(roccat_nyth::RoccatNyth::bind(hiddev))),

        _ => Err(HwDeviceError::DeviceNotSupported.into()),
    }
}
