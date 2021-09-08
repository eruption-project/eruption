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

use std::time::Duration;
use std::{cell::RefCell, thread};

use crate::constants;

use super::{DeviceTrait, HwDeviceError, Result, RGBA};

/// Device specific code for the ROCCAT Kone XTD mouse
pub struct RoccatKoneXtd {
    pub is_bound: bool,
    pub ctrl_hiddev: RefCell<Option<hidapi::HidDevice>>,
}

impl RoccatKoneXtd {
    /// Binds the driver to the supplied HID device
    pub fn bind(ctrl_dev: hidapi::HidDevice) -> Self {
        println!("Bound driver: ROCCAT Kone XTD Mouse");

        Self {
            is_bound: true,
            ctrl_hiddev: RefCell::new(Some(ctrl_dev)),
        }
    }

    fn send_ctrl_report(&self, id: u8) -> Result<()> {
        println!("Sending control device feature report");

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
                                    hexdump::hexdump_iter(&buf).for_each(|s| println!("  {}", s));

                                    Ok(())
                                }

                                Err(_) => Err(HwDeviceError::InvalidResult {}),
                            }?;

                            let mut buf: [u8; 5] = [0xa1, 0x00, 0x00, 0x00, 0x00];
                            match ctrl_dev.get_feature_report(&mut buf) {
                                Ok(_result) => {
                                    hexdump::hexdump_iter(&buf).for_each(|s| println!("  {}", s));

                                    Ok(())
                                }

                                Err(_) => Err(HwDeviceError::InvalidResult {}),
                            }?;
                        }
                    }

                    Ok(())
                }

                0x0e => {
                    let buf: [u8; 3] = [0x0e, 0x03, 0x01];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println!("  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x06 => {
                    let buf: [u8; 43] = [
                        0x06, 0x2b, 0x00, 0x00, 0x06, 0x06, 0x1f, 0x10, 0x20, 0x40, 0x80, 0xa4,
                        0x00, 0x10, 0x20, 0x40, 0x80, 0xa4, 0x00, 0x03, 0x01, 0x00, 0x00, 0x01,
                        0x03, 0x18, 0x00, 0x00, 0x7d, 0x13, 0x00, 0x64, 0xfa, 0x13, 0x00, 0x64,
                        0xfa, 0x13, 0x00, 0x64, 0xfa, 0x74, 0x08,
                    ];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println!("  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x07 => {
                    let buf: [u8; 77] = [
                        0x07, 0x4d, 0x00, 0x01, 0x00, 0x00, 0x02, 0x00, 0x00, 0x03, 0x00, 0x00,
                        0x07, 0x00, 0x00, 0x08, 0x00, 0x00, 0x09, 0x00, 0x00, 0x0a, 0x00, 0x00,
                        0x0d, 0x00, 0x00, 0x0e, 0x00, 0x00, 0x15, 0x00, 0x00, 0x16, 0x00, 0x00,
                        0x1a, 0x00, 0x00, 0x08, 0x00, 0x00, 0x07, 0x00, 0x00, 0x25, 0x00, 0x00,
                        0x06, 0x00, 0x00, 0x06, 0x00, 0x00, 0x21, 0x00, 0x00, 0x22, 0x00, 0x00,
                        0x26, 0x00, 0x00, 0x27, 0x00, 0x00, 0x11, 0x00, 0x00, 0x12, 0x00, 0x00,
                        0x1b, 0x00, 0x00, 0xea, 0x01,
                    ];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println!("  {}", s));

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
        println!("Waiting for control device to respond...");

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
                        hexdump::hexdump_iter(&buf).for_each(|s| println!("  {}", s));

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

impl DeviceTrait for RoccatKoneXtd {
    fn send_init_sequence(&self) -> Result<()> {
        println!("Sending device init sequence...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            // println!("Step 1");
            // self.send_ctrl_report(0x04)
            //     .unwrap_or_else(|e| eprintln!("Step 1: {}", e));
            // self.wait_for_ctrl_dev()
            //     .unwrap_or_else(|e| eprintln!("Step 1: {}", e));

            println!("Step 2");
            self.send_ctrl_report(0x0e)
                .unwrap_or_else(|e| eprintln!("Step 2: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| eprintln!("Step 2: {}", e));

            println!("Step 3");
            self.send_ctrl_report(0x06)
                .unwrap_or_else(|e| eprintln!("Step 3: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| eprintln!("Step 3: {}", e));

            println!("Step 4");
            self.send_ctrl_report(0x07)
                .unwrap_or_else(|e| eprintln!("Step 4: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| eprintln!("Step 4: {}", e));

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
                    hexdump::hexdump_iter(buf).for_each(|s| println!("  {}", s));

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
                    hexdump::hexdump_iter(&buf).for_each(|s| println!("  {}", s));

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

            match ctrl_dev.send_feature_report(&buffer) {
                Ok(_result) => {
                    hexdump::hexdump_iter(&buffer).for_each(|s| println!("  {}", s));

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
                    hexdump::hexdump_iter(&buf).for_each(|s| println!("  {}", s));

                    Ok(buf)
                }

                Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
            }
        }
    }

    fn send_led_map(&self, led_map: &[RGBA]) -> Result<()> {
        println!("Setting LEDs from supplied map...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            let ctrl_dev = self.ctrl_hiddev.borrow_mut();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            let buf: [u8; 43] = [
                0x06,
                0x2b,
                0x00,
                0x00,
                0x06,
                0x06,
                0x1f,
                0x10,
                0x20,
                0x40,
                0x80,
                0xa4,
                0x00,
                0x10,
                0x20,
                0x40,
                0x80,
                0xa4,
                0x00,
                0x03,
                0x01,
                0x00,
                0x00,
                0x01,
                0x03,
                0x01,
                0xfa,
                led_map[0].g,
                led_map[0].b,
                led_map[0].r,
                0x00,
                0x64,
                0xfa,
                0x13,
                0x00,
                0x64,
                0xfa,
                0x13,
                0x00,
                0x64,
                0xfa,
                0xda,
                0x08,
            ];

            match ctrl_dev.send_feature_report(&buf) {
                Ok(_result) => {
                    hexdump::hexdump_iter(&buf).for_each(|s| println!("  {}", s));
                    Ok(())
                }

                Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
            }
        }
    }

    fn send_test_pattern(&self) -> Result<()> {
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
