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

    Copyright (c) 2019-2022, The Eruption Development Team
*/

use byteorder::{BigEndian, ByteOrder};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::{cell::RefCell, thread};

#[allow(unused)]
use crate::{constants, eprintln_v, println_v};

use super::{DeviceStatus, DeviceTrait, HwDeviceError, Result, RGBA};

pub const CTRL_INTERFACE: i32 = 2; // Control USB sub device

lazy_static! {
    static ref CRC8: Arc<Mutex<crc8::Crc8>> = Arc::new(Mutex::new(crc8::Crc8::create_msb(0x01)));
}

/// Device specific code for the ROCCAT Kain 2xx AIMO mouse
pub struct RoccatKain2xx {
    pub is_bound: bool,
    pub ctrl_hiddev: RefCell<Option<hidapi::HidDevice>>,
}

impl RoccatKain2xx {
    /// Binds the driver to the supplied HID device
    pub fn bind(ctrl_dev: hidapi::HidDevice) -> Self {
        println_v!(1, "Bound driver: ROCCAT Kain 2xx AIMO");

        Self {
            is_bound: true,
            ctrl_hiddev: RefCell::new(Some(ctrl_dev)),
        }
    }

    // fn send_ctrl_report(&self, id: u8) -> Result<()> {
    //     println_v!(1, "Sending control device feature report");

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
    //         //                 hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

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
    //     println_v!(1, "Waiting for control device to respond...");

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
    //         //             hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

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
        println_v!(1, "Sending device init sequence...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            // println_v!(1, "Step 1");
            // self.send_ctrl_report(0x08)
            //     .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 1: {}", e));
            // self.wait_for_ctrl_dev()
            //     .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 1: {}", e));

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

            loop {
                let mut buf = Vec::new();
                buf.resize(size, 0);
                buf[0] = id;

                match ctrl_dev.read_timeout(buf.as_mut_slice(), 10) {
                    Ok(_result) => {
                        if buf[0] == 0x01 || buf[0..2] == [0x07, 0x14] {
                            continue;
                        } else {
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

                            break Ok(buf);
                        }
                    }

                    Err(_) => break Err(HwDeviceError::InvalidResult {}.into()),
                }
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

            let mut buf: [u8; 22] = [
                0x08,
                0x09,
                0x33,
                0x00,
                led_map[0].r,
                led_map[0].g,
                led_map[0].b,
                led_map[1].r,
                led_map[1].g,
                led_map[1].b,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
            ];

            buf[10] = CRC8.lock().calc(&buf[4..10], 6, 0x32);

            match ctrl_dev.send_feature_report(&buf) {
                Ok(_result) => {
                    hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));
                }

                Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
            }

            thread::sleep(Duration::from_millis(70));

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
                r: 0x59,
                g: 0xa5,
                b: 0xff,
                a: 0xff,
            },
            RGBA {
                r: 0,
                g: 0,
                b: 0,
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
        let read_results = || -> Result<super::DeviceStatus> {
            let mut table = HashMap::new();

            for _ in 0..=2 {
                // query results
                let buf = self.read_feature_report(0x07, 22)?;

                match buf[1] {
                    0x04 => {
                        if buf[2] == 0x40 {
                            let battery_status = buf[5];

                            let battery_level = match battery_status {
                                71 => "100",
                                64 => "80",
                                65 => "60",
                                66 => "40",
                                67 => "20",
                                68 => "0",
                                _ => "unknown",
                            };

                            table.insert(
                                "battery-level-percent".to_string(),
                                battery_level.to_string(),
                            );

                            table.insert(
                                "battery-level-raw".to_string(),
                                format!("{}", battery_status),
                            );
                        }
                    }

                    0x07 => {
                        if buf[2] == 0x53 {
                            let transceiver_enabled = buf[6] != 0x00;
                            let signal = BigEndian::read_u16(&buf[7..9]);

                            // radio
                            table.insert(
                                "transceiver-enabled".to_string(),
                                format!("{}", transceiver_enabled),
                            );

                            // signal strength
                            table.insert(
                                "signal-strength-percent".to_string(),
                                format!("{:.0}", (signal as f32 / 100.0).clamp(0.0, 100.0)),
                            );

                            table.insert("signal-strength-raw".to_string(), format!("{}", signal));
                        }
                    }

                    _ => { /* do nothing */ }
                }

                thread::sleep(Duration::from_millis(15));
            }

            Ok(DeviceStatus(table))
        };

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            // TODO: Further investigate the meaning of the fields

            let buf: [u8; 22] = [
                0x08, 0x03, 0x53, 0x00, 0x58, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ];

            self.write_feature_report(&buf)?;

            let result = read_results()?;

            let buf: [u8; 22] = [
                0x08, 0x03, 0x40, 0x00, 0x4b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ];

            self.write_feature_report(&buf)?;

            let result2 = read_results()?;

            // let buf: [u8; 22] = [
            //     0x08, 0x05, 0x12, 0x01, 0x04, 0x01, 0x1b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // ];

            // self.write_feature_report(&buf)?;

            // let result3 = read_results()?;

            // let buf: [u8; 22] = [
            //     0x08, 0x05, 0x12, 0x01, 0x04, 0x02, 0x18, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // ];

            // self.write_feature_report(&buf)?;

            // let result4 = read_results()?;

            // let buf: [u8; 22] = [
            //     0x08, 0x04, 0x33, 0x85, 0x04, 0xbe, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // ];

            // self.write_feature_report(&buf)?;

            // let result5 = read_results()?;

            // let buf: [u8; 22] = [
            //     0x08, 0x04, 0x34, 0x01, 0x00, 0x39, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            // ];

            // self.write_feature_report(&buf)?;

            // let result6 = read_results()?;

            Ok(DeviceStatus(
                result
                    .0
                    .into_iter()
                    .chain(result2.0)
                    // .chain(result3.0)
                    // .chain(result4.0)
                    // .chain(result5.0)
                    // .chain(result6.0)
                    .collect(),
            ))
        }
    }
}
