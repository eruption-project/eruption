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

use log::*;
use parking_lot::Mutex;
use std::time::Duration;
use std::{sync::Arc, thread};

use crate::constants;

use super::{DeviceTrait, HwDeviceError, RGBA};

pub type Result<T> = super::Result<T>;

pub const NUM_KEYS: usize = 127;

// pub const CTRL_INTERFACE: i32 = 1; // Control USB sub device
pub const LED_INTERFACE: i32 = 3; // LED USB sub device

#[derive(Clone)]
/// Device specific code for the ROCCAT Vulcan TKL series keyboards
pub struct RoccatVulcanTKL {
    pub is_bound: bool,
    pub ctrl_hiddev: Arc<Mutex<Option<hidapi::HidDevice>>>,
    pub led_hiddev: Arc<Mutex<Option<hidapi::HidDevice>>>,
}

impl RoccatVulcanTKL {
    /// Binds the driver to the supplied HID devices
    pub fn bind(ctrl_dev: hidapi::HidDevice, led_dev: hidapi::HidDevice) -> Self {
        println!("Bound driver: ROCCAT Vulcan TKL");

        Self {
            is_bound: true,
            ctrl_hiddev: Arc::new(Mutex::new(Some(ctrl_dev))),
            led_hiddev: Arc::new(Mutex::new(Some(led_dev))),
        }
    }

    // pub(self) fn query_ctrl_report(&self, id: u8) -> Result<()> {
    //     trace!("Querying control device feature report");

    //     if !self.is_bound {
    //         Err(HwDeviceError::DeviceNotBound {}.into())
    //     } else {
    //         match id {
    //             0x0f => {
    //                 let mut buf: [u8; 256] = [0; 256];
    //                 buf[0] = id;

    //                 let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
    //                 let ctrl_dev = ctrl_dev.as_ref().unwrap();

    //                 match ctrl_dev.get_feature_report(&mut buf) {
    //                     Ok(_result) => {
    //                         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

    //                         Ok(())
    //                     }

    //                     Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
    //                 }
    //             }

    //             _ => Err(HwDeviceError::InvalidStatusCode {}.into()),
    //         }
    //     }
    // }

    fn send_ctrl_report(&self, id: u8) -> Result<()> {
        trace!("Sending control device feature report");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            match id {
                0x15 => {
                    let buf: [u8; 3] = [0x15, 0x00, 0x01];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x05 => {
                    let buf: [u8; 4] = [0x05, 0x04, 0x00, 0x04];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x07 => {
                    let buf: [u8; 143] = [
                        0x07, 0x8f, 0x00, 0x3a, 0x00, 0x00, 0x3b, 0x00, 0x00, 0x3c, 0x00, 0x00,
                        0x3d, 0x00, 0x00, 0x3e, 0x00, 0x00, 0x3f, 0x00, 0x00, 0x40, 0x00, 0x00,
                        0x41, 0x00, 0x00, 0x42, 0x00, 0x00, 0x43, 0x00, 0x00, 0x44, 0x00, 0x00,
                        0x45, 0x00, 0x00, 0x49, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x4b, 0x00, 0x00,
                        0x4c, 0x00, 0x00, 0x4d, 0x00, 0x00, 0x4e, 0x00, 0x00, 0x52, 0x00, 0x00,
                        0x50, 0x00, 0x00, 0x51, 0x00, 0x00, 0x4f, 0x00, 0x00, 0xe4, 0x00, 0x00,
                        0x3a, 0x00, 0x00, 0x3b, 0x00, 0x00, 0x3c, 0x00, 0x00, 0x3d, 0x00, 0x00,
                        0x3e, 0x00, 0x00, 0x3f, 0x00, 0x00, 0x40, 0x00, 0x00, 0x41, 0x00, 0x00,
                        0xce, 0x00, 0x00, 0xcf, 0x00, 0x00, 0xcc, 0x00, 0x00, 0xcd, 0x00, 0x00,
                        0x46, 0x00, 0x00, 0x47, 0x00, 0x00, 0x48, 0x00, 0x00, 0x9c, 0x00, 0x00,
                        0x9e, 0x00, 0x00, 0xfc, 0x00, 0x00, 0xa4, 0x00, 0x00, 0xfe, 0x00, 0x00,
                        0x8e, 0x00, 0x00, 0xfd, 0x00, 0x00, 0x9a, 0x00, 0x00, 0x6f, 0x13,
                    ];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x0a => {
                    let buf: [u8; 8] = [0x0a, 0x08, 0x00, 0xff, 0xf1, 0x00, 0x02, 0x02];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x0b => {
                    let buf: [u8; 65] = [
                        0x0b, 0x41, 0x00, 0x1e, 0x00, 0x00, 0x1f, 0x00, 0x00, 0x20, 0x00, 0x00,
                        0x21, 0x00, 0x00, 0x22, 0x00, 0x00, 0x14, 0x00, 0x00, 0x1a, 0x00, 0x00,
                        0x08, 0x00, 0x00, 0x15, 0x00, 0x00, 0x17, 0x00, 0x00, 0x04, 0x00, 0x00,
                        0x16, 0x00, 0x00, 0x07, 0x00, 0x00, 0x09, 0x00, 0x00, 0x0a, 0x00, 0x00,
                        0x1d, 0x00, 0x00, 0x1b, 0x00, 0x00, 0x06, 0x00, 0x00, 0x19, 0x00, 0x00,
                        0x05, 0x00, 0x00, 0xde, 0x01,
                    ];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x06 => {
                    let buf: [u8; 133] = [
                        0x06, 0x85, 0x00, 0x3a, 0x29, 0x35, 0x1e, 0x2b, 0x39, 0xe1, 0xe0, 0x3b,
                        0x1f, 0x14, 0x1a, 0x04, 0x64, 0x00, 0x00, 0x3d, 0x3c, 0x20, 0x21, 0x08,
                        0x16, 0x1d, 0xe2, 0x3e, 0x23, 0x22, 0x15, 0x07, 0x1b, 0x06, 0x8b, 0x3f,
                        0x24, 0x00, 0x17, 0x0a, 0x09, 0x19, 0x91, 0x40, 0x41, 0x00, 0x1c, 0x18,
                        0x0b, 0x05, 0x2c, 0x42, 0x26, 0x25, 0x0c, 0x0d, 0x0e, 0x10, 0x11, 0x43,
                        0x2a, 0x27, 0x2d, 0x12, 0x0f, 0x36, 0x8a, 0x44, 0x45, 0x89, 0x2e, 0x13,
                        0x33, 0x37, 0x90, 0x46, 0x49, 0x4c, 0x2f, 0x30, 0x34, 0x38, 0x88, 0x47,
                        0x4a, 0x4d, 0x31, 0x32, 0x00, 0x87, 0xe6, 0x48, 0x4b, 0x4e, 0x28, 0x52,
                        0x50, 0xe5, 0xe7, 0xd2, 0x53, 0x5f, 0x5c, 0x59, 0x51, 0x00, 0xf1, 0xd1,
                        0x54, 0x60, 0x5d, 0x5a, 0x4f, 0x8e, 0x65, 0xd0, 0x55, 0x61, 0x5e, 0x5b,
                        0x62, 0xa4, 0xe4, 0xfc, 0x56, 0x57, 0x85, 0x58, 0x63, 0x00, 0x00, 0xc2,
                        0x24,
                    ];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x09 => {
                    let buf: [u8; 43] = [
                        0x09, 0x2b, 0x00, 0x49, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x4b, 0x00, 0x00,
                        0x4c, 0x00, 0x00, 0x4d, 0x00, 0x00, 0x4e, 0x00, 0x00, 0xa4, 0x00, 0x00,
                        0x8e, 0x00, 0x00, 0xd0, 0x00, 0x00, 0xd1, 0x00, 0x00, 0x3a, 0x00, 0x00,
                        0x3b, 0x00, 0x00, 0x00, 0x00, 0x41, 0x05,
                    ];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x0d => {
                    // custom effects
                    let buf: [u8; 443] = [
                        0x0d, 0xbb, 0x01, 0x00, 0x09, 0x06, 0x05, 0x45, 0x80, 0xff, 0xff, 0xff,
                        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0a, 0x0a, 0x0a,
                        0x0a, 0x0a, 0x0a, 0x11, 0x11, 0x11, 0x11, 0x17, 0x17, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff,
                        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x17, 0x17, 0x17,
                        0x17, 0x1e, 0x1e, 0x1e, 0x1e, 0x1e, 0x1e, 0x1e, 0x25, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff,
                        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x25, 0x25, 0x25,
                        0x25, 0x2b, 0x2b, 0x2b, 0x2b, 0x32, 0x32, 0x39, 0x39, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff,
                        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xfe, 0xfc, 0xff, 0x32, 0x39, 0x39,
                        0x3f, 0x39, 0x39, 0x3f, 0x3f, 0x46, 0x46, 0x46, 0x3f, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff,
                        0xff, 0xfa, 0xff, 0xff, 0xff, 0xff, 0xfe, 0xfe, 0xff, 0x3f, 0x46, 0x46,
                        0x4d, 0x46, 0x46, 0x46, 0x4d, 0x4d, 0x53, 0x53, 0x4d, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xfe, 0xfe, 0xfc,
                        0xfc, 0xfc, 0xfc, 0xfc, 0xfc, 0xfa, 0xf1, 0xfa, 0xfa, 0x53, 0x53, 0x57,
                        0x57, 0x57, 0x57, 0x57, 0x57, 0x5c, 0x71, 0x5c, 0x5c, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xfa, 0xfa, 0xf8,
                        0xf6, 0xf6, 0xf8, 0xed, 0xf6, 0xf6, 0xeb, 0xf6, 0xeb, 0x5c, 0x5c, 0x62,
                        0x66, 0x66, 0x62, 0x7a, 0x66, 0x66, 0x7f, 0x66, 0x7f, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf4, 0xf4, 0xf4,
                        0xeb, 0xf1, 0xf1, 0xf1, 0xf1, 0xf4, 0xef, 0xef, 0xef, 0x6b, 0x6b, 0x6b,
                        0x7f, 0x71, 0x71, 0x71, 0x71, 0x6b, 0x75, 0x75, 0x75, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xc6, 0x78,
                    ];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x13 => {
                    // custom effects
                    let buf: [u8; 8] = [0x13, 0x08, 0x01, 0x00, 0x00, 0x45, 0x00, 0x00];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                _ => Err(HwDeviceError::InvalidStatusCode {}.into()),
            }
        }
    }

    fn wait_for_ctrl_dev(&self) -> Result<()> {
        trace!("Waiting for control device to respond...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            // loop {
            thread::sleep(Duration::from_millis(constants::DEVICE_SETTLE_MILLIS));

            // let mut buf: [u8; 4] = [0; 4];
            // buf[0] = 0x01;

            // let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            // let ctrl_dev = ctrl_dev.as_ref().unwrap();

            // match ctrl_dev.get_feature_report(&mut buf) {
            //     Ok(_result) => {
            //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

            //         return Ok(());
            //     }

            //     Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
            // }
            // }

            Ok(())
        }
    }
}

impl DeviceTrait for RoccatVulcanTKL {
    fn send_init_sequence(&self) -> Result<()> {
        trace!("Sending device init sequence...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            println!("Step 1");
            // self.query_ctrl_report(0x0f)
            //     .unwrap_or_else(|e| eprintln!("Step 1: {}", e));

            println!("Step 2");
            self.send_ctrl_report(0x15)
                .unwrap_or_else(|e| eprintln!("Step 2: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| eprintln!("Step 2: {}", e));

            println!("Step 3");
            self.send_ctrl_report(0x05)
                .unwrap_or_else(|e| eprintln!("Step 3: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| eprintln!("Step 3: {}", e));

            println!("Step 4");
            self.send_ctrl_report(0x0a)
                .unwrap_or_else(|e| eprintln!("Step 4: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| eprintln!("Step 4: {}", e));

            println!("Step 5");
            self.send_ctrl_report(0x0b)
                .unwrap_or_else(|e| eprintln!("Step 5: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| eprintln!("Step 5: {}", e));

            println!("Step 6");
            self.send_ctrl_report(0x06)
                .unwrap_or_else(|e| eprintln!("Step 6: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| eprintln!("Step 6: {}", e));

            println!("Step 7");
            self.send_ctrl_report(0x09)
                .unwrap_or_else(|e| eprintln!("Step 7: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| eprintln!("Step 7: {}", e));

            println!("Step 8");
            self.send_ctrl_report(0x0d)
                .unwrap_or_else(|e| eprintln!("Step 8: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| eprintln!("Step 8: {}", e));

            println!("Step 9");
            self.send_ctrl_report(0x07)
                .unwrap_or_else(|e| eprintln!("Step 9: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| eprintln!("Step 9: {}", e));

            println!("Step 10");
            self.send_ctrl_report(0x13)
                .unwrap_or_else(|e| eprintln!("Step 10: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| eprintln!("Step 10: {}", e));

            Ok(())
        }
    }

    fn write_data_raw(&self, buf: &[u8]) -> Result<()> {
        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            match ctrl_dev.write(&buf) {
                Ok(_result) => {
                    hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                    Ok(())
                }

                Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
            }
        }
    }

    fn read_data_raw(&self, size: usize) -> Result<Vec<u8>> {
        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            let mut buf = Vec::new();
            buf.resize(size, 0);

            match ctrl_dev.read(buf.as_mut_slice()) {
                Ok(_result) => {
                    hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                    Ok(buf)
                }

                Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
            }
        }
    }

    fn send_led_map(&self, led_map: &[RGBA]) -> Result<()> {
        trace!("Setting LEDs from supplied map...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            match *self.led_hiddev.lock() {
                Some(ref led_dev) => {
                    if led_map.len() < NUM_KEYS {
                        error!(
                            "Received a short LED map: Got {} elements, but should be {}",
                            led_map.len(),
                            NUM_KEYS
                        );

                        Err(HwDeviceError::LedMapError {}.into())
                    } else {
                        // Colors are in blocks of 12 keys (2 columns). Color parts are sorted by color e.g. the red
                        // values for all 12 keys are first then come the green values etc.

                        let mut buffer: [u8; 448] = [0; 448];
                        buffer[0..4].copy_from_slice(&[0xa1, 0x01, 0x01, 0xb4]);

                        for i in 0..NUM_KEYS {
                            let color = led_map[i];
                            let offset = ((i / 12) * 36) + (i % 12);

                            buffer[offset + 4] = color.r;
                            buffer[offset + 4 + 12] = color.g;
                            buffer[offset + 4 + 24] = color.b;
                        }

                        for bytes in buffer.chunks(64) {
                            let mut tmp: [u8; 65] = [0; 65];
                            tmp[1..65].copy_from_slice(&bytes);

                            match led_dev.write(&tmp) {
                                Ok(len) => {
                                    if len < 65 {
                                        return Err(HwDeviceError::WriteError {}.into());
                                    }
                                }

                                Err(_) => return Err(HwDeviceError::WriteError {}.into()),
                            }
                        }

                        Ok(())
                    }
                }

                None => Err(HwDeviceError::DeviceNotOpened {}.into()),
            }
        }
    }

    fn send_test_pattern(&self) -> Result<()> {
        self.send_led_map(
            &[RGBA {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            }; 144],
        )?;

        thread::sleep(Duration::from_millis(500));

        self.send_led_map(
            &[RGBA {
                r: 0,
                g: 0,
                b: 255,
                a: 255,
            }; 144],
        )?;

        Ok(())
    }
}
