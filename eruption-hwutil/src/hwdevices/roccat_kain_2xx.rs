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

use std::collections::HashMap;
use std::time::Duration;
use std::{cell::RefCell, thread};

// use crate::constants;

use super::{DeviceStatus, DeviceTrait, HwDeviceError, Result, RGBA};

pub const CTRL_INTERFACE: i32 = 2; // Control USB sub device

/// Device specific code for the ROCCAT Kain 2xx AIMO mouse
pub struct RoccatKain2xx {
    pub is_bound: bool,
    pub ctrl_hiddev: RefCell<Option<hidapi::HidDevice>>,
}

impl RoccatKain2xx {
    /// Binds the driver to the supplied HID device
    pub fn bind(ctrl_dev: hidapi::HidDevice) -> Self {
        crate::println_v!(0, "Bound driver: ROCCAT Kain 2xx AIMO");

        Self {
            is_bound: true,
            ctrl_hiddev: RefCell::new(Some(ctrl_dev)),
        }
    }

    // fn send_ctrl_report(&self, id: u8) -> Result<()> {
    //     crate::println_v!(0, "Sending control device feature report");

    //     if !self.is_bound {
    //         Err(HwDeviceError::DeviceNotBound.into())
    //     } else {
    //         // let ctrl_dev = self.ctrl_hiddev.borrow_mut();
    //         // let ctrl_dev = ctrl_dev.as_ref().unwrap();

    //         // match id {
    //         //     0x08 => {
    //         //         let buf: [u8; 22] = [
    //         //             0x08, 0x03, 0x53, 0x00, 0x58, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         //             0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    //         //         ];

    //         //         match ctrl_dev.send_feature_report(&buf) {
    //         //             Ok(_result) => {
    //         //                 hexdump::hexdump_iter(&buf).for_each(|s| crate::println_v!(1, "  {}", s));

    //         //                 Ok(())
    //         //             }

    //         //             Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
    //         //         }
    //         //     }

    //         //     _ => Err(HwDeviceError::InvalidStatusCode {}.into()),
    //         // }

    //         Ok(())
    //     }
    // }

    // fn wait_for_ctrl_dev(&self) -> Result<()> {
    //     crate::println_v!(0, "Waiting for control device to respond...");

    //     if !self.is_bound {
    //         Err(HwDeviceError::DeviceNotBound {}.into())
    //     } else {
    //         // loop {
    //         //     let mut buf: [u8; 4] = [0; 4];
    //         //     buf[0] = 0x04;

    //         //     let ctrl_dev = self.ctrl_hiddev.borrow_mut();
    //         //     let ctrl_dev = ctrl_dev.as_ref().unwrap();

    //         //     match ctrl_dev.get_feature_report(&mut buf) {
    //         //         Ok(_result) => {
    //         //             hexdump::hexdump_iter(&buf).for_each(|s| crate::println_v!(1, "  {}", s));

    //         //             if buf[1] == 0x01 {
    //         //                 return Ok(());
    //         //             }
    //         //         }

    //         //         Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
    //         //     }

    //         //     thread::sleep(Duration::from_millis(constants::DEVICE_SETTLE_MILLIS));
    //         // }

    //         thread::sleep(Duration::from_millis(constants::DEVICE_SETTLE_MILLIS));

    //         Ok(())
    //     }
    // }
}

impl DeviceTrait for RoccatKain2xx {
    fn send_init_sequence(&self) -> Result<()> {
        crate::println_v!(0, "Sending device init sequence...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            // crate::println_v!(0, "Step 1");
            // self.send_ctrl_report(0x08)
            //     .unwrap_or_else(|e| crate::eprintln_v!(0, "Step 1: {}", e));
            // self.wait_for_ctrl_dev()
            //     .unwrap_or_else(|e| crate::eprintln_v!(0, "Step 1: {}", e));

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
                    hexdump::hexdump_iter(buf).for_each(|s| crate::println_v!(0, "  {}", s));

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
                    hexdump::hexdump_iter(&buf).for_each(|s| crate::println_v!(1, "  {}", s));

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
                    hexdump::hexdump_iter(buffer).for_each(|s| crate::println_v!(1, "  {}", s));

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

            loop {
                thread::sleep(Duration::from_millis(75));

                let mut buf = Vec::new();
                buf.resize(size, 0);
                buf[0] = id;

                match ctrl_dev.read_timeout(buf.as_mut_slice(), 5000) {
                    Ok(_result) => {
                        if buf[0] == 0x01 {
                            continue;
                        } else {
                            hexdump::hexdump_iter(&buf)
                                .for_each(|s| crate::println_v!(1, "  {}", s));

                            if buf[0..2] != [0x07, 0x14] && buf[0..2] != [0x07, 0x04] {
                                break Ok(buf);
                            }
                        }
                    }

                    Err(_) => break Err(HwDeviceError::InvalidResult {}.into()),
                }
            }
        }
    }

    fn send_led_map(&self, _led_map: &[RGBA]) -> Result<()> {
        crate::println_v!(0, "Setting LEDs from supplied map...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            let ctrl_dev = self.ctrl_hiddev.borrow_mut();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            let buf: [u8; 22] = [
                0x08, 0x09, 0x33, 0x00, 0xfd, 0x01, 0x46, 0xf5, 0xff, 0x00, 0x82, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ];

            match ctrl_dev.send_feature_report(&buf) {
                Ok(_result) => {
                    hexdump::hexdump_iter(&buf).for_each(|s| crate::println_v!(1, "  {}", s));
                }

                Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
            }

            thread::sleep(Duration::from_millis(500));

            let buf: [u8; 22] = [
                0x08, 0x09, 0x33, 0x00, 0xf5, 0x00, 0xff, 0x12, 0x00, 0xff, 0xd5, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ];

            match ctrl_dev.send_feature_report(&buf) {
                Ok(_result) => {
                    hexdump::hexdump_iter(&buf).for_each(|s| crate::println_v!(1, "  {}", s));
                }

                Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
            }

            Ok(())
        }
    }

    fn send_test_pattern(&self) -> Result<()> {
        self.send_led_map(&[
            RGBA {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
            RGBA {
                r: 0,
                g: 0,
                b: 255,
                a: 255,
            },
        ])?;

        thread::sleep(Duration::from_millis(500));

        self.send_led_map(&[
            RGBA {
                r: 0,
                g: 0,
                b: 255,
                a: 255,
            },
            RGBA {
                r: 255,
                g: 0,
                b: 0,
                a: 255,
            },
        ])?;

        Ok(())
    }

    fn device_status(&self) -> super::Result<super::DeviceStatus> {
        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            let mut table = HashMap::new();

            // TODO: Further investigate the meaning of the fields
            let buf: [u8; 22] = [
                0x08, 0x03, 0x53, 0x00, 0x58, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ];

            self.write_feature_report(&buf)?;

            // query results
            let buf = self.read_feature_report(0x07, 22)?;

            let battery_level = (buf[7] as f64 / 256.0) * 100.0;
            let snr = 100.0 - ((buf[8] as f64) / 256.0) * 100.0;

            // icecream::ice!(buf);

            table.insert("battery-level".to_string(), format!("{}", battery_level));
            table.insert("signal-strength".to_string(), format!("{}", snr));

            // TODO: Further investigate the meaning of the fields
            let buf: [u8; 22] = [
                0x08, 0x03, 0x40, 0x00, 0x4b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ];

            self.write_feature_report(&buf)?;

            let battery_level = (buf[9] as f64 / 256.0) * 100.0;
            let snr = 100.0 - ((buf[8] as f64) / 256.0) * 100.0;

            // icecream::ice!(buf);

            table.insert("battery-level2".to_string(), format!("{}", battery_level));
            table.insert("signal-strength2".to_string(), format!("{}", snr));

            let result = DeviceStatus(table);

            Ok(result)
        }
    }
}
