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
use std::{cell::RefCell, thread};

#[allow(unused)]
use crate::{constants, eprintln_v, interact, println_v};

use super::{DeviceTrait, HwDeviceError, Result, RGBA};

/// Device specific code for the ROCCAT Kova AIMO mouse
pub struct RoccatKovaAimo {
    pub is_bound: bool,
    pub ctrl_hiddev: RefCell<Option<hidapi::HidDevice>>,
}

impl RoccatKovaAimo {
    /// Binds the driver to the supplied HID device
    pub fn bind(ctrl_dev: hidapi::HidDevice) -> Self {
        println_v!(1, "Bound driver: ROCCAT Kova AIMO");

        Self {
            is_bound: true,
            ctrl_hiddev: RefCell::new(Some(ctrl_dev)),
        }
    }

    fn send_ctrl_report(&self, id: u8) -> Result<()> {
        println_v!(1, "Sending control device feature report");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound.into())
        } else {
            let ctrl_dev = self.ctrl_hiddev.borrow_mut();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            match id {
                0x04 => {
                    for j in &[0x80, 0x90] {
                        for i in 0..=4 {
                            let buf: [u8; 3] = [0x04, i, *j];

                            match ctrl_dev.send_feature_report(&buf) {
                                Ok(_result) => {
                                    hexdump::hexdump_iter(&buf)
                                        .for_each(|s| println_v!(2, "  {}", s));

                                    Ok(())
                                }

                                Err(_) => Err(HwDeviceError::InvalidResult {}),
                            }?;

                            let mut buf: [u8; 5] = [0xa1, 0x00, 0x00, 0x00, 0x00];
                            match ctrl_dev.get_feature_report(&mut buf) {
                                Ok(_result) => {
                                    hexdump::hexdump_iter(&buf)
                                        .for_each(|s| println_v!(2, "  {}", s));

                                    Ok(())
                                }

                                Err(_) => Err(HwDeviceError::InvalidResult {}),
                            }?;
                        }
                    }

                    Ok(())
                }

                0x0e => {
                    let buf: [u8; 6] = [0x0e, 0x06, 0x01, 0x01, 0x00, 0xff];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x09 => {
                    let buf: [u8; 8] = [0x09, 0x08, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];

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

                let ctrl_dev = self.ctrl_hiddev.borrow_mut();
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

impl DeviceTrait for RoccatKovaAimo {
    fn send_init_sequence(&self) -> Result<()> {
        interact::prompt("Press any key to send initialization sequence.");
        println_v!(1, "Sending device init sequence...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            println_v!(1, "Step 1");
            self.send_ctrl_report(0x04)
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 1: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 1: {}", e));

            println_v!(1, "Step 2");
            self.send_ctrl_report(0x0e)
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 2: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 2: {}", e));

            println_v!(1, "Step 3");
            self.send_ctrl_report(0x09)
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 3: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 3: {}", e));

            Ok(())
        }
    }

    fn write_data_raw(&self, buf: &[u8]) -> Result<()> {
        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            let ctrl_dev = self.ctrl_hiddev.borrow_mut();
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
            let ctrl_dev = self.ctrl_hiddev.borrow_mut();
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
            let ctrl_dev = self.ctrl_hiddev.borrow_mut();
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
            let ctrl_dev = self.ctrl_hiddev.borrow_mut();
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
            let ctrl_dev = self.ctrl_hiddev.borrow_mut();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            let buf: [u8; 8] = [
                0x0a,
                0x08,
                led_map[0].r,
                led_map[0].g,
                led_map[0].b,
                led_map[0].r,
                led_map[0].g,
                led_map[0].b,
            ];

            match ctrl_dev.send_feature_report(&buf) {
                Ok(_result) => {
                    hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

                    Ok(())
                }

                Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
            }
        }
    }

    fn send_test_pattern(&self) -> Result<()> {
        self.send_led_map(&[RGBA {
            r: 255,
            g: 0,
            b: 0,
            a: 255,
        }])?;

        interact::prompt_or_wait(
            "Press any key to change colors.",
            Duration::from_millis(500),
        );

        self.send_led_map(&[RGBA {
            r: 0,
            g: 0,
            b: 255,
            a: 255,
        }])?;

        Ok(())
    }

    fn device_status(&self) -> super::Result<super::DeviceStatus> {
        Err(HwDeviceError::OpNotSupported {}.into())
    }
}
