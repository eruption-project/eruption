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

/// Device specific code for the ROCCAT Kone XP mouse
pub struct RoccatKoneXp {
    pub is_bound: bool,
    pub ctrl_hiddev: RefCell<Option<hidapi::HidDevice>>,
}

impl RoccatKoneXp {
    /// Binds the driver to the supplied HID device
    pub fn bind(ctrl_dev: hidapi::HidDevice) -> Self {
        println_v!(1, "Bound driver: ROCCAT Kone XP");

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
                    for j in [0x80, 0x90] {
                        for i in 0..=4 {
                            let buf: [u8; 4] = [0x04, i, j, 0x00];

                            match ctrl_dev.send_feature_report(&buf) {
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

                0x0d => {
                    let buf: [u8; 122] = [
                        0x0d, 0x7a, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00,
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

    fn wait_for_ctrl_dev(&self) -> Result<()> {
        println_v!(1, "Waiting for control device to respond...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            loop {
                let mut buf: [u8; 4] = [0; 4];
                buf[0] = 0x01;

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

impl DeviceTrait for RoccatKoneXp {
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
            self.send_ctrl_report(0x0d)
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

            let buf: [u8; 122] = [
                0x0d,
                0x7a,
                led_map[0].r,
                led_map[0].g,
                led_map[0].b,
                led_map[1].r,
                led_map[1].g,
                led_map[1].b,
                led_map[2].r,
                led_map[2].g,
                led_map[2].b,
                led_map[3].r,
                led_map[3].g,
                led_map[3].b,
                led_map[4].r,
                led_map[4].g,
                led_map[4].b,
                led_map[5].r,
                led_map[5].g,
                led_map[5].b,
                led_map[6].r,
                led_map[6].g,
                led_map[6].b,
                led_map[7].r,
                led_map[7].g,
                led_map[7].b,
                led_map[8].r,
                led_map[8].g,
                led_map[8].b,
                led_map[9].r,
                led_map[9].g,
                led_map[9].b,
                led_map[10].r,
                led_map[10].g,
                led_map[10].b,
                led_map[11].r,
                led_map[11].g,
                led_map[11].b,
                led_map[12].r,
                led_map[12].g,
                led_map[12].b,
                led_map[13].r,
                led_map[13].g,
                led_map[13].b,
                led_map[14].r,
                led_map[14].g,
                led_map[14].b,
                led_map[15].r,
                led_map[15].g,
                led_map[15].b,
                led_map[16].r,
                led_map[16].g,
                led_map[16].b,
                led_map[17].r,
                led_map[17].g,
                led_map[17].b,
                led_map[18].r,
                led_map[18].g,
                led_map[18].b,
                led_map[19].r,
                led_map[19].g,
                led_map[19].b,
                led_map[20].r,
                led_map[20].g,
                led_map[20].b,
                led_map[21].r,
                led_map[21].g,
                led_map[21].b,
                led_map[22].r,
                led_map[22].g,
                led_map[22].b,
                //
                led_map[23].r,
                led_map[23].g,
                led_map[23].b,
                led_map[24].r,
                led_map[24].g,
                led_map[24].b,
                led_map[25].r,
                led_map[25].g,
                led_map[25].b,
                led_map[26].r,
                led_map[26].g,
                led_map[26].b,
                led_map[27].r,
                led_map[27].g,
                led_map[27].b,
                led_map[28].r,
                led_map[28].g,
                led_map[28].b,
                led_map[29].r,
                led_map[29].g,
                led_map[29].b,
                led_map[30].r,
                led_map[30].g,
                led_map[30].b,
                led_map[31].r,
                led_map[31].g,
                led_map[31].b,
                led_map[32].r,
                led_map[32].g,
                led_map[32].b,
                led_map[33].r,
                led_map[33].g,
                led_map[33].b,
                led_map[34].r,
                led_map[34].g,
                led_map[34].b,
                led_map[35].r,
                led_map[35].g,
                led_map[35].b,
                led_map[36].r,
                led_map[36].g,
                led_map[36].b,
                led_map[37].r,
                led_map[37].g,
                led_map[37].b,
                led_map[38].r,
                led_map[38].g,
                led_map[38].b,
                0xff,
                0xff,
                0xff,
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
