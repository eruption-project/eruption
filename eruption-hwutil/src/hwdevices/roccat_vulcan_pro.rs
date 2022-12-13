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

use parking_lot::Mutex;
use std::time::Duration;
use std::{sync::Arc, thread};
use tracing::*;

#[allow(unused)]
use crate::{constants, eprintln_v, println_v};

use super::{DeviceTrait, HwDeviceError, RGBA};

pub type Result<T> = super::Result<T>;

pub const NUM_KEYS: usize = 144;

// pub const CTRL_INTERFACE: i32 = 1; // Control USB sub device
pub const LED_INTERFACE: i32 = 3; // LED USB sub device

#[derive(Clone)]
/// Device specific code for the ROCCAT Vulcan Pro series keyboards
pub struct RoccatVulcanPro {
    pub is_bound: bool,
    pub ctrl_hiddev: Arc<Mutex<Option<hidapi::HidDevice>>>,
    pub led_hiddev: Arc<Mutex<Option<hidapi::HidDevice>>>,
}

impl RoccatVulcanPro {
    /// Binds the driver to the supplied HID devices
    pub fn bind(ctrl_dev: hidapi::HidDevice, led_dev: hidapi::HidDevice) -> Self {
        println_v!(1, "Bound driver: ROCCAT Vulcan Pro");

        Self {
            is_bound: true,
            ctrl_hiddev: Arc::new(Mutex::new(Some(ctrl_dev))),
            led_hiddev: Arc::new(Mutex::new(Some(led_dev))),
        }
    }

    // pub(self) fn query_ctrl_report(&self, id: u8) -> Result<()> {
    //     println_v!(0, "Querying control device feature report");

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
    //                         hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

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
        println_v!(1, "Sending control device feature report");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            match id {
                0x00 => {
                    let buf: [u8; 1] = [0x00];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x0d => {
                    let buf: [u8; 16] = [
                        0x0d, 0x10, 0x00, 0x00, 0x02, 0x0f, 0x45, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00,
                    ];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x04 => {
                    for j in &[0x00, 0x01, 0x02, 0x03, 0x04] {
                        for i in &[0xa0, 0xa1, 0xa2, 0xa3, 0xa4, 0xa5, 0xb0] {
                            let buf: [u8; 4] = [0x04, *j, *i, 0x00];

                            match ctrl_dev.send_feature_report(&buf) {
                                Ok(_result) => {
                                    hexdump::hexdump_iter(&buf)
                                        .for_each(|s| println_v!(2, "  {}", s));

                                    Ok(())
                                }

                                Err(_) => Err(HwDeviceError::InvalidResult {}),
                            }?;

                            // thread::sleep(Duration::from_millis(constants::DEVICE_SETTLE_MILLIS));
                        }
                    }

                    Ok(())
                }

                0x0e => {
                    let buf: [u8; 5] = [0x0e, 0x05, 0x01, 0x00, 0x00];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x11 => {
                    let buf: [u8; 371] = [
                        0x11, 0x73, 0x01, 0x00, 0x06, 0x00, 0x45, 0x80, 0x00, 0xff, 0xff, 0xff,
                        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0a, 0x0a, 0x0a,
                        0x0a, 0x0a, 0x0a, 0x11, 0x11, 0x11, 0x11, 0x17, 0x17, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff,
                        0xff, 0xff, 0xff, 0x1c, 0x1c, 0xff, 0xff, 0xff, 0xff, 0x17, 0x17, 0x17,
                        0x17, 0x1e, 0x1e, 0x1c, 0x1c, 0x1e, 0x1e, 0x1e, 0x25, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0xff, 0xff, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff,
                        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x25, 0x25, 0x25,
                        0x25, 0x2b, 0x2b, 0x2b, 0x2b, 0x32, 0x32, 0x39, 0x39, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0xff,
                        0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x42, 0x42, 0xff, 0x32, 0x39, 0x39,
                        0x3f, 0x39, 0x39, 0x3f, 0x3f, 0x46, 0x42, 0x42, 0x3f, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xff, 0x00, 0xff, 0xff, 0xff,
                        0xff, 0x48, 0xff, 0xff, 0xff, 0xff, 0xfe, 0xfe, 0xff, 0x3f, 0x46, 0x46,
                        0x4d, 0x48, 0x46, 0x46, 0x4d, 0x4d, 0x53, 0x53, 0x4d, 0x00, 0x00, 0x00,
                        0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0xfe, 0xfc,
                        0xfc, 0xfc, 0xfc, 0xfc, 0xfc, 0xfa, 0x5b, 0xfa, 0xfa, 0x4d, 0x53, 0x57,
                        0x57, 0x57, 0x57, 0x57, 0x57, 0x5c, 0x5b, 0x5c, 0x5c, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xff, 0x00, 0x00, 0xfa, 0xfa, 0xf8,
                        0xf6, 0xf6, 0xf8, 0x61, 0xf6, 0xf6, 0x67, 0xf6, 0xf4, 0x5c, 0x5c, 0x62,
                        0x66, 0x66, 0x62, 0x61, 0x66, 0x66, 0x67, 0x66, 0x6b, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0xff, 0x00, 0x00, 0xff, 0x00, 0x00, 0xf4, 0xf4, 0xf4,
                        0xf1, 0xf1, 0xf1, 0xf1, 0xf1, 0xef, 0xef, 0xef, 0xef, 0x6b, 0x6b, 0x6b,
                        0x71, 0x71, 0x71, 0x71, 0x71, 0x75, 0x75, 0x75, 0x75, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x80, 0xed, 0xed,
                        0xed, 0xed, 0xed, 0x86, 0xeb, 0xeb, 0xeb, 0xeb, 0x8d, 0x80, 0x7a, 0x7a,
                        0x7a, 0x7a, 0x7a, 0x86, 0x7f, 0x7f, 0x7f, 0x7f, 0x8d, 0xff, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0xff, 0xe9, 0xe9, 0xe9,
                        0xe9, 0xe9, 0x93, 0xe7, 0xe7, 0x93, 0xe7, 0x00, 0x00, 0x84, 0x84, 0x84,
                        0x84, 0x84, 0x93, 0x89, 0x89, 0x93, 0x89, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0xff, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00, 0xfd, 0x9c,
                    ];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                _ => Err(HwDeviceError::InvalidStatusCode {}.into()),
            }
        }
    }

    fn send_led_data(&self, id: u8) -> Result<()> {
        println_v!(0, "Sending data to LED device");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            let led_dev = self.led_hiddev.as_ref().lock();
            let led_dev = led_dev.as_ref().unwrap();

            match id {
                0xa1 => {
                    let buf: [u8; 64] = [
                        0xa1, 0x01, 0x80, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00,
                    ];

                    match led_dev.write(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));
                        }

                        Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
                    }

                    let buf: [u8; 64] = [
                        0xa1, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00,
                    ];

                    match led_dev.write(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));
                        }

                        Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
                    }

                    let buf: [u8; 64] = [
                        0xa1, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00,
                    ];

                    match led_dev.write(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));
                        }

                        Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
                    }

                    let buf: [u8; 64] = [
                        0xa1, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00,
                    ];

                    match led_dev.write(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));
                        }

                        Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
                    }

                    let buf: [u8; 64] = [
                        0xa1, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00,
                    ];

                    match led_dev.write(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));
                        }

                        Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
                    }

                    let buf: [u8; 64] = [
                        0xa1, 0x06, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00,
                    ];

                    match led_dev.write(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));
                        }

                        Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
                    }

                    Ok(())
                }

                _ => Err(HwDeviceError::InvalidStatusCode {}.into()),
            }
        }
    }

    fn wait_for_ctrl_dev(&self) -> Result<()> {
        println_v!(1, "Waiting for control device to respond...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            // loop {
            thread::sleep(Duration::from_millis(constants::DEVICE_SETTLE_MILLIS));

            //     let mut buf: [u8; 4] = [0; 4];
            //     buf[0] = 0x04;

            //     let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            //     let ctrl_dev = ctrl_dev.as_ref().unwrap();

            //     match ctrl_dev.get_feature_report(&mut buf) {
            //         Ok(_result) => {
            //             hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

            //             if buf[1] == 0x01 {
            //                 return Ok(());
            //             }
            //         }

            //         Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
            //     }

            //     thread::sleep(Duration::from_millis(constants::DEVICE_SETTLE_MILLIS));
            // }

            Ok(())
        }
    }
}

impl DeviceTrait for RoccatVulcanPro {
    fn send_init_sequence(&self) -> Result<()> {
        println_v!(1, "Sending device init sequence...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            println_v!(1, "Step 1");
            self.send_ctrl_report(0x0d)
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 1: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 1: {}", e));

            println_v!(1, "Step 2 skipped");
            // println_v!(1, "Step 2");
            // self.send_ctrl_report(0x04)
            //     .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 2: {}", e));
            // self.wait_for_ctrl_dev()
            //     .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 2: {}", e));

            println_v!(1, "Step 3");
            self.send_ctrl_report(0x0e)
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 3: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 3: {}", e));

            println_v!(1, "Step 4");
            self.send_ctrl_report(0x11)
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 4: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 4: {}", e));

            // init colors
            println_v!(1, "Step 5");
            self.send_led_data(0xa1)
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 5: {}", e));
            thread::sleep(Duration::from_millis(constants::DEVICE_SETTLE_MILLIS));

            Ok(())
        }
    }

    fn write_data_raw(&self, buf: &[u8]) -> Result<()> {
        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            match ctrl_dev.write(buf) {
                Ok(_result) => {
                    hexdump::hexdump_iter(buf).for_each(|s| crate::eprintln_v!(2, "  {}", s));

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
                    hexdump::hexdump_iter(&buf).for_each(|s| crate::eprintln_v!(2, "  {}", s));

                    Ok(buf)
                }

                Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
            }
        }
    }

    fn write_feature_report(&self, buffer: &[u8]) -> Result<()> {
        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            match ctrl_dev.send_feature_report(buffer) {
                Ok(_result) => {
                    hexdump::hexdump_iter(buffer).for_each(|s| println_v!(1, "  {}", s));

                    Ok(())
                }

                Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
            }
        }
    }

    fn read_feature_report(&self, id: u8, size: usize) -> Result<Vec<u8>> {
        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            let mut buf = Vec::new();
            buf.resize(size, 0);
            buf[0] = id;

            match ctrl_dev.get_feature_report(buf.as_mut_slice()) {
                Ok(_result) => {
                    hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

                    Ok(buf)
                }

                Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
            }
        }
    }

    fn send_led_map(&self, led_map: &[RGBA]) -> Result<()> {
        println_v!(1, "Setting LEDs from supplied map...");

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
                        for i in 0..NUM_KEYS {
                            let color = led_map[i];
                            let offset = ((i / 12) * 36) + (i % 12);

                            buffer[offset] = color.r;
                            buffer[offset + 12] = color.g;
                            buffer[offset + 24] = color.b;
                        }

                        for (cntr, bytes) in buffer.chunks(60).take(6).enumerate() {
                            let mut tmp: [u8; 64] = [0; 64];

                            if cntr < 1 {
                                tmp[0..4].copy_from_slice(&[0xa1, 0x01, 0x80, 0x01]);
                            } else {
                                tmp[0..4].copy_from_slice(&[0xa1, cntr as u8 + 1, 0x00, 0x00]);
                            }

                            tmp[4..64].copy_from_slice(bytes);

                            hexdump::hexdump_iter(&tmp).for_each(|s| println_v!(2, "  {}", s));

                            match led_dev.write(&tmp) {
                                Ok(len) => {
                                    if len < 64 {
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

    fn device_status(&self) -> super::Result<super::DeviceStatus> {
        Err(HwDeviceError::OpNotSupported {}.into())
    }
}
