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

use std::time::Duration;
use std::{sync::Arc, thread};
use tracing::*;
use tracing_mutex::stdsync::Mutex;

#[allow(unused)]
use crate::{constants, eprintln_v, interact, println_v};

use super::{DeviceTrait, HwDeviceError, RGBA};

pub type Result<T> = super::Result<T>;

pub const NUM_KEYS: usize = 144;

// pub const CTRL_INTERFACE: i32 = 1; // Control USB sub device
pub const LED_INTERFACE: i32 = 3; // LED USB sub device

#[derive(Clone)]
/// Device specific code for the ROCCAT Vulcan 100/12x series keyboards
pub struct RoccatVulcan1xx {
    pub is_bound: bool,
    pub ctrl_hiddev: Arc<Mutex<Option<hidapi::HidDevice>>>,
    pub led_hiddev: Arc<Mutex<Option<hidapi::HidDevice>>>,
}

impl RoccatVulcan1xx {
    /// Binds the driver to the supplied HID devices
    pub fn bind(ctrl_dev: hidapi::HidDevice, led_dev: hidapi::HidDevice) -> Self {
        println_v!(1, "Bound driver: ROCCAT Vulcan 100/12x");

        Self {
            is_bound: true,
            ctrl_hiddev: Arc::new(Mutex::new(Some(ctrl_dev))),
            led_hiddev: Arc::new(Mutex::new(Some(led_dev))),
        }
    }

    pub(self) fn query_ctrl_report(&self, id: u8) -> Result<()> {
        println_v!(0, "Querying control device feature report");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            match id {
                0x0f => {
                    let mut buf: [u8; 256] = [0; 256];
                    buf[0] = id;

                    let ctrl_dev = self.ctrl_hiddev.as_ref().lock().unwrap();
                    let ctrl_dev = ctrl_dev.as_ref().unwrap();

                    match ctrl_dev.get_feature_report(&mut buf) {
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

    fn send_ctrl_report(&self, id: u8) -> Result<()> {
        println_v!(1, "Sending control device feature report");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            let ctrl_dev = self.ctrl_hiddev.as_ref().lock().unwrap();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            match id {
                0x15 => {
                    let buf: [u8; 3] = [0x15, 0x00, 0x01];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x05 => {
                    let buf: [u8; 4] = [0x05, 0x04, 0x00, 0x04];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x07 => {
                    let buf: [u8; 95] = [
                        0x07, 0x5f, 0x00, 0x3a, 0x00, 0x00, 0x3b, 0x00, 0x00, 0x3c, 0x00, 0x00,
                        0x3d, 0x00, 0x00, 0x3e, 0x00, 0x00, 0x3f, 0x00, 0x00, 0x40, 0x00, 0x00,
                        0x41, 0x00, 0x00, 0x42, 0x00, 0x00, 0x43, 0x00, 0x00, 0x44, 0x00, 0x00,
                        0x45, 0x00, 0x00, 0x46, 0x00, 0x00, 0x47, 0x00, 0x00, 0x48, 0x00, 0x00,
                        0xb3, 0x00, 0x00, 0xb4, 0x00, 0x00, 0xb5, 0x00, 0x00, 0xb6, 0x00, 0x00,
                        0xc2, 0x00, 0x00, 0xc3, 0x00, 0x00, 0xc0, 0x00, 0x00, 0xc1, 0x00, 0x00,
                        0xce, 0x00, 0x00, 0xcf, 0x00, 0x00, 0xcc, 0x00, 0x00, 0xcd, 0x00, 0x00,
                        0x46, 0x00, 0x00, 0xfc, 0x00, 0x00, 0x48, 0x00, 0x00, 0xcd, 0x0e,
                    ];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x0a => {
                    let buf: [u8; 8] = [0x0a, 0x08, 0x00, 0xff, 0xf1, 0x00, 0x02, 0x02];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

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
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

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
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x09 => {
                    let buf: [u8; 43] = [
                        0x09, 0x2b, 0x00, 0x49, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x4b, 0x00, 0x00,
                        0x4c, 0x00, 0x00, 0x4d, 0x00, 0x00, 0x4e, 0x00, 0x00, 0xa4, 0x00, 0x00,
                        0x8e, 0x00, 0x00, 0xd0, 0x00, 0x00, 0xd1, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x01, 0x00, 0x00, 0x00, 0x00, 0xcd, 0x04,
                    ];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x0d => {
                    // hardware wave effect
                    /* let mut buf: [u8; 443] = [
                        0x0d, 0xbb, 0x01, 0x00, 0x0a, 0x04, 0x05, 0x45, 0x83, 0xca, 0xca, 0xca,
                        0xca, 0xca, 0xca, 0xce, 0xce, 0xd2, 0xce, 0xce, 0xd2, 0x19, 0x19, 0x19,
                        0x19, 0x19, 0x19, 0x23, 0x23, 0x2d, 0x23, 0x23, 0x2d, 0xe0, 0xe0, 0xe0,
                        0xe0, 0xe0, 0xe0, 0xe3, 0xe3, 0xe6, 0xe3, 0xe3, 0xe6, 0xd2, 0xd2, 0xd5,
                        0xd2, 0xd2, 0xd5, 0xd5, 0xd5, 0xd9, 0xd5, 0x00, 0xd9, 0x2d, 0x2d, 0x36,
                        0x2d, 0x2d, 0x36, 0x36, 0x36, 0x40, 0x36, 0x00, 0x40, 0xe6, 0xe6, 0xe9,
                        0xe6, 0xe6, 0xe9, 0xe9, 0xe9, 0xec, 0xe9, 0x00, 0xec, 0xd9, 0xd9, 0xdd,
                        0xd9, 0xdd, 0xdd, 0xe0, 0xe0, 0xdd, 0xe0, 0xe4, 0xe4, 0x40, 0x40, 0x4a,
                        0x40, 0x4a, 0x4a, 0x53, 0x53, 0x4a, 0x53, 0x5d, 0x5d, 0xec, 0xec, 0xef,
                        0xec, 0xef, 0xef, 0xf2, 0xf2, 0xef, 0xf2, 0xf5, 0xf5, 0xe4, 0xe4, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5d, 0x5d, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf5, 0xf5, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe4, 0xe4, 0xe8,
                        0xe8, 0xe8, 0xe8, 0xe8, 0xeb, 0xeb, 0xeb, 0x00, 0xeb, 0x5d, 0x5d, 0x67,
                        0x67, 0x67, 0x67, 0x67, 0x70, 0x70, 0x70, 0x00, 0x70, 0xf5, 0xf5, 0xf8,
                        0xf8, 0xf8, 0xf8, 0xf8, 0xfb, 0xfb, 0xfb, 0x00, 0xfb, 0xeb, 0xef, 0xef,
                        0xef, 0x00, 0xef, 0xf0, 0xf0, 0xed, 0xf0, 0xf0, 0x00, 0x70, 0x7a, 0x7a,
                        0x7a, 0x00, 0x7a, 0x7a, 0x7a, 0x6f, 0x7a, 0x7a, 0x00, 0xfb, 0xfd, 0xfd,
                        0xfd, 0x00, 0xfd, 0xf8, 0xf8, 0xea, 0xf8, 0xf8, 0x00, 0xed, 0xed, 0xea,
                        0xed, 0xed, 0x00, 0xed, 0xea, 0xea, 0xf6, 0xe7, 0xea, 0x6f, 0x6f, 0x65,
                        0x6f, 0x6f, 0x00, 0x6f, 0x65, 0x65, 0x66, 0x5a, 0x65, 0xea, 0xea, 0xdc,
                        0xea, 0xea, 0x00, 0xea, 0xdc, 0xdc, 0x00, 0xce, 0xdc, 0xea, 0xe7, 0xe5,
                        0xe7, 0xe5, 0xe5, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x65, 0x5a, 0x50,
                        0x5a, 0x50, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xdc, 0xce, 0xc0,
                        0xce, 0xc0, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe7, 0x00, 0x00,
                        0xe2, 0xe2, 0xe2, 0xe2, 0xdf, 0xdf, 0xdf, 0xdf, 0xdf, 0x5a, 0x00, 0x00,
                        0x45, 0x45, 0x45, 0x45, 0x3b, 0x3b, 0x3b, 0x3b, 0x3b, 0xce, 0x00, 0x00,
                        0xb2, 0xb2, 0xb2, 0xb2, 0xa4, 0xa4, 0xa4, 0xa4, 0xa4, 0xdc, 0xdc, 0xdc,
                        0xdc, 0x00, 0xda, 0xda, 0xda, 0xda, 0xda, 0x00, 0xd7, 0x30, 0x30, 0x30,
                        0x30, 0x00, 0x26, 0x26, 0x26, 0x26, 0x26, 0x00, 0x1c, 0x96, 0x96, 0x96,
                        0x96, 0x00, 0x88, 0x88, 0x88, 0x88, 0x88, 0x00, 0x7a, 0xd7, 0xd7, 0xd7,
                        0x00, 0xd4, 0xd4, 0xd4, 0xd4, 0xd4, 0xd1, 0xd1, 0xd1, 0x1c, 0x1c, 0x1c,
                        0x00, 0x11, 0x11, 0x11, 0x11, 0x11, 0x06, 0x06, 0x06, 0x7a, 0x7a, 0x7a,
                        0x00, 0x6c, 0x6c, 0x6c, 0x6c, 0x6c, 0x5e, 0x5e, 0x5e, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21, 0xcf,
                    ];

                    // byte 5   == 01-slow 06-med 0b-fast
                    // byte 441 == 1e-slow 23-med 28-fast

                    buf[5] = 0x06;
                    buf[441] = 0x23;

                    */

                    // custom effects
                    let buf: [u8; 443] = [
                        0x0d, 0xbb, 0x01, 0x00, 0x06, 0x0b, 0x05, 0x45, 0x83, 0xca, 0xca, 0xca,
                        0xca, 0xca, 0xca, 0xce, 0xce, 0xd2, 0xce, 0xce, 0xd2, 0x19, 0x19, 0x19,
                        0x19, 0x19, 0x19, 0x23, 0x23, 0x2d, 0x23, 0x23, 0x2d, 0xe0, 0xe0, 0xe0,
                        0xe0, 0xe0, 0xe0, 0xe3, 0xe3, 0xe6, 0xe3, 0xe3, 0xe6, 0xd2, 0xd2, 0xd5,
                        0xd2, 0xd2, 0xd5, 0xd5, 0xd5, 0xd9, 0xd5, 0x00, 0xd9, 0x2d, 0x2d, 0x36,
                        0x2d, 0x2d, 0x36, 0x36, 0x36, 0x40, 0x36, 0x00, 0x40, 0xe6, 0xe6, 0xe9,
                        0xe6, 0xe6, 0xe9, 0xe9, 0xe9, 0xec, 0xe9, 0x00, 0xec, 0xd9, 0xd9, 0xdd,
                        0xd9, 0xdd, 0xdd, 0xe0, 0xe0, 0xdd, 0xe0, 0xe4, 0xe4, 0x40, 0x40, 0x4a,
                        0x40, 0x4a, 0x4a, 0x53, 0x53, 0x4a, 0x53, 0x5d, 0x5d, 0xec, 0xec, 0xef,
                        0xec, 0xef, 0xef, 0xf2, 0xf2, 0xef, 0xf2, 0xf5, 0xf5, 0xe4, 0xe4, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5d, 0x5d, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf5, 0xf5, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe4, 0xe4, 0xe8,
                        0xe8, 0xe8, 0xe8, 0xe8, 0xeb, 0xeb, 0xeb, 0x00, 0xeb, 0x5d, 0x5d, 0x67,
                        0x67, 0x67, 0x67, 0x67, 0x70, 0x70, 0x70, 0x00, 0x70, 0xf5, 0xf5, 0xf8,
                        0xf8, 0xf8, 0xf8, 0xf8, 0xfb, 0xfb, 0xfb, 0x00, 0xfb, 0xeb, 0xef, 0xef,
                        0xef, 0x00, 0xef, 0xf0, 0xf0, 0xed, 0xf0, 0xf0, 0x00, 0x70, 0x7a, 0x7a,
                        0x7a, 0x00, 0x7a, 0x7a, 0x7a, 0x6f, 0x7a, 0x7a, 0x00, 0xfb, 0xfd, 0xfd,
                        0xfd, 0x00, 0xfd, 0xf8, 0xf8, 0xea, 0xf8, 0xf8, 0x00, 0xed, 0xed, 0xea,
                        0xed, 0xed, 0x00, 0xed, 0xea, 0xea, 0xf6, 0xe7, 0xea, 0x6f, 0x6f, 0x65,
                        0x6f, 0x6f, 0x00, 0x6f, 0x65, 0x65, 0x66, 0x5a, 0x65, 0xea, 0xea, 0xdc,
                        0xea, 0xea, 0x00, 0xea, 0xdc, 0xdc, 0x00, 0xce, 0xdc, 0xea, 0xe7, 0xe5,
                        0xe7, 0xe5, 0xe5, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x65, 0x5a, 0x50,
                        0x5a, 0x50, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xdc, 0xce, 0xc0,
                        0xce, 0xc0, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe7, 0x00, 0x00,
                        0xe2, 0xe2, 0xe2, 0xe2, 0xdf, 0xdf, 0xdf, 0xdf, 0xdf, 0x5a, 0x00, 0x00,
                        0x45, 0x45, 0x45, 0x45, 0x3b, 0x3b, 0x3b, 0x3b, 0x3b, 0xce, 0x00, 0x00,
                        0xb2, 0xb2, 0xb2, 0xb2, 0xa4, 0xa4, 0xa4, 0xa4, 0xa4, 0xdc, 0xdc, 0xdc,
                        0xdc, 0x00, 0xda, 0xda, 0xda, 0xda, 0xda, 0x00, 0xd7, 0x30, 0x30, 0x30,
                        0x30, 0x00, 0x26, 0x26, 0x26, 0x26, 0x26, 0x00, 0x1c, 0x96, 0x96, 0x96,
                        0x96, 0x00, 0x88, 0x88, 0x88, 0x88, 0x88, 0x00, 0x7a, 0xd7, 0xd7, 0xd7,
                        0x00, 0xd4, 0xd4, 0xd4, 0xd4, 0xd4, 0xd1, 0xd1, 0xd1, 0x1c, 0x1c, 0x1c,
                        0x00, 0x11, 0x11, 0x11, 0x11, 0x11, 0x06, 0x06, 0x06, 0x7a, 0x7a, 0x7a,
                        0x00, 0x6c, 0x6c, 0x6c, 0x6c, 0x6c, 0x5e, 0x5e, 0x5e, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x24, 0xcf,
                    ];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x13 => {
                    // hardware wave effect
                    // let buf: [u8; 8] = [0x13, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

                    // custom effects
                    let buf: [u8; 8] = [0x13, 0x08, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];

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

    fn wait_for_ctrl_dev(&self) -> Result<()> {
        println_v!(1, "Waiting for control device to respond...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            loop {
                let mut buf: [u8; 4] = [0; 4];
                buf[0] = 0x04;

                let ctrl_dev = self.ctrl_hiddev.as_ref().lock().unwrap();
                let ctrl_dev = ctrl_dev.as_ref().unwrap();

                match ctrl_dev.get_feature_report(&mut buf) {
                    Ok(_result) => {
                        hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

                        if buf[1] == 0x01 {
                            return Ok(());
                        }
                    }

                    Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
                }

                thread::sleep(Duration::from_millis(constants::DEVICE_SETTLE_MILLIS));
            }
        }
    }
}

impl DeviceTrait for RoccatVulcan1xx {
    fn send_init_sequence(&self) -> Result<()> {
        interact::prompt("Press any key to send initialization sequence.");
        println_v!(1, "Sending device init sequence...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            println_v!(1, "Step 1");
            self.query_ctrl_report(0x0f)
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 1: {}", e));

            println_v!(1, "Step 2");
            self.send_ctrl_report(0x15)
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 2: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 2: {}", e));

            println_v!(1, "Step 3");
            self.send_ctrl_report(0x05)
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 3: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 3: {}", e));

            println_v!(1, "Step 4");
            self.send_ctrl_report(0x07)
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 4: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 4: {}", e));

            println_v!(1, "Step 5");
            self.send_ctrl_report(0x0a)
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 5: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 5: {}", e));

            println_v!(1, "Step 6");
            self.send_ctrl_report(0x0b)
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 6: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 6: {}", e));

            println_v!(1, "Step 7");
            self.send_ctrl_report(0x06)
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 7: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 7: {}", e));

            println_v!(1, "Step 8");
            self.send_ctrl_report(0x09)
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 8: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 8: {}", e));

            println_v!(1, "Step 9");
            self.send_ctrl_report(0x0d)
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 9: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 9: {}", e));

            println_v!(1, "Step 10");
            self.send_ctrl_report(0x13)
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 10: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 10: {}", e));

            Ok(())
        }
    }

    fn write_data_raw(&self, buf: &[u8]) -> Result<()> {
        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            let ctrl_dev = self.ctrl_hiddev.as_ref().lock().unwrap();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            match ctrl_dev.write(buf) {
                Ok(_result) => {
                    hexdump::hexdump_iter(buf).for_each(|s| println_v!(2, "  {}", s));

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
            let ctrl_dev = self.ctrl_hiddev.as_ref().lock().unwrap();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            let mut buf = Vec::new();
            buf.resize(size, 0);

            match ctrl_dev.read(buf.as_mut_slice()) {
                Ok(_result) => {
                    hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

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
            let ctrl_dev = self.ctrl_hiddev.as_ref().lock().unwrap();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            match ctrl_dev.send_feature_report(buffer) {
                Ok(_result) => {
                    hexdump::hexdump_iter(buffer).for_each(|s| println_v!(2, "  {}", s));

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
            let ctrl_dev = self.ctrl_hiddev.lock().unwrap();
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
            match *self.led_hiddev.lock().unwrap() {
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

                        // TODO: The #' key (on QWERTZ layout) seems to be out of order!?
                        //       This is an ugly hack, find a better way to fix this
                        // let mut led_map = led_map.to_vec();
                        // led_map.swap(81, 96);

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
                            tmp[1..65].copy_from_slice(bytes);

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

        interact::prompt_or_wait(
            "Press any key to change colors.",
            Duration::from_millis(500),
        );

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
