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

pub const NUM_KEYS: usize = 144;

// pub const CTRL_INTERFACE: i32 = 0; // Control USB sub device
// pub const LED_INTERFACE: i32 = 1; // LED USB sub device

#[derive(Clone)]
/// Device specific code for the Corsair Strafe series keyboards
pub struct CorsairStrafe {
    pub is_bound: bool,
    pub ctrl_hiddev: Arc<Mutex<Option<hidapi::HidDevice>>>,
}

impl CorsairStrafe {
    /// Binds the driver to the supplied HID devices
    pub fn bind(ctrl_dev: hidapi::HidDevice) -> Self {
        println!("Bound driver: Corsair STRAFE Gaming Keyboard");

        Self {
            is_bound: true,
            ctrl_hiddev: Arc::new(Mutex::new(Some(ctrl_dev))),
        }
    }

    // pub(self) fn query_ctrl_report(&self, id: u8) -> Result<()> {
    //     trace!("Querying control device feature report");

    //     if !self.is_bound {
    //         Err(HwDeviceError::DeviceNotBound {}.into())
    //     } else {
    //         // match id {
    //         //     0x0f => {
    //         //         let mut buf: [u8; 256] = [0; 256];
    //         //         buf[0] = id;

    //         //         let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
    //         //         let ctrl_dev = ctrl_dev.as_ref().unwrap();

    //         //         match ctrl_dev.get_feature_report(&mut buf) {
    //         //             Ok(_result) => {
    //         //                 hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

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

    // fn send_ctrl_report(&self, id: u8) -> Result<()> {
    //     trace!("Sending control device feature report");

    //     if !self.is_bound {
    //         Err(HwDeviceError::DeviceNotBound {}.into())
    //     } else {
    //         let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
    //         let ctrl_dev = ctrl_dev.as_ref().unwrap();

    //         match id {
    //             0x01 => {
    //                 let buf: [u8; 2] = [0x01, 0x00];

    //                 match ctrl_dev.send_feature_report(&buf) {
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

    // fn wait_for_ctrl_dev(&self) -> Result<()> {
    //     trace!("Waiting for control device to respond...");

    //     if !self.is_bound {
    //         Err(HwDeviceError::DeviceNotBound {}.into())
    //     } else {
    //         // loop {
    //         thread::sleep(Duration::from_millis(constants::DEVICE_SETTLE_MILLIS));
    //         //     let mut buf: [u8; 4] = [0; 4];
    //         //     buf[0] = 0x04;

    //         //     let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
    //         //     let ctrl_dev = ctrl_dev.as_ref().unwrap();

    //         //     match ctrl_dev.get_feature_report(&mut buf) {
    //         //         Ok(_result) => {
    //         //             hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

    //         //             if buf[1] == 0x01 {
    //         //                 return Ok(());
    //         //             }
    //         //         }

    //         //         Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
    //         //     }

    //         //     thread::sleep(Duration::from_millis(constants::DEVICE_SETTLE_MILLIS));
    //         // }

    //         Ok(())
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
                0x01 => {
                    let buf: [u8; 64] = [
                        0x0e, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00,
                    ];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x02 => {
                    let buf: [u8; 64] = [
                        0x07, 0x04, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00,
                    ];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x03 => {
                    let buf: [u8; 64] = [
                        0x0e, 0x48, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00,
                    ];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x04 => {
                    let buf: [u8; 64] = [
                        0x07, 0x05, 0x02, 0x00, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00,
                    ];

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
            thread::sleep(Duration::from_millis(constants::DEVICE_SETTLE_MILLIS));

            Ok(())
        }
    }
}

impl DeviceTrait for CorsairStrafe {
    fn send_init_sequence(&self) -> Result<()> {
        trace!("Sending device init sequence...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            println!("Step 1");
            self.send_ctrl_report(0x01)
                .unwrap_or_else(|e| eprintln!("Step 1: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| eprintln!("Step 1: {}", e));

            println!("Step 2");
            self.send_ctrl_report(0x02)
                .unwrap_or_else(|e| eprintln!("Step 2: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| eprintln!("Step 2: {}", e));

            println!("Step 3");
            self.send_ctrl_report(0x03)
                .unwrap_or_else(|e| eprintln!("Step 3: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| eprintln!("Step 3: {}", e));

            println!("Step 4");
            self.send_ctrl_report(0x04)
                .unwrap_or_else(|e| eprintln!("Step 3: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| eprintln!("Step 3: {}", e));

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
            match *self.ctrl_hiddev.lock() {
                Some(ref led_dev) => {
                    if led_map.len() < NUM_KEYS {
                        error!(
                            "Received a short LED map: Got {} elements, but should be {}",
                            led_map.len(),
                            NUM_KEYS
                        );

                        Err(HwDeviceError::LedMapError {}.into())
                    } else {
                        println!("write 1");

                        let tmp = [
                            0x07, 0x27, 0x00, 0x00, 0xd8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                            0x00, 0x00, 0x00, 0x00,
                        ];

                        match led_dev.write(&tmp) {
                            Ok(len) => {
                                if len < 64 {
                                    println!("short write: {}", len);

                                    return Err(HwDeviceError::WriteError {}.into());
                                }
                            }

                            Err(_) => return Err(HwDeviceError::WriteError {}.into()),
                        }

                        // println!("write 2");

                        // let tmp = [
                        //     0x7f, 0x01, 0x3c, 0x00, 0x00, 0x00, 0x00, 0x11, 0x07, 0x77, 0x00, 0x00,
                        //     0x70, 0x70, 0x07, 0x77, 0x00, 0x70, 0x66, 0x00, 0x07, 0x77, 0x00, 0x66,
                        //     0x76, 0x00, 0x77, 0x77, 0x70, 0x56, 0x35, 0x00, 0x07, 0x77, 0x66, 0x55,
                        //     0x74, 0x00, 0x07, 0x77, 0x55, 0x44, 0x74, 0x00, 0x00, 0x77, 0x54, 0x34,
                        //     0x13, 0x00, 0x00, 0x77, 0x44, 0x33, 0x02, 0x07, 0x00, 0x77, 0x33, 0x22,
                        //     0x02, 0x00, 0x00, 0x77,
                        // ];

                        // match led_dev.write(&tmp) {
                        //     Ok(len) => {
                        //         if len < 64 {
                        //             return Err(HwDeviceError::WriteError {}.into());
                        //         }
                        //     }

                        //     Err(_) => return Err(HwDeviceError::WriteError {}.into()),
                        // }

                        // println!("write 3");

                        // let tmp = [
                        //     0x7f, 0x02, 0x3c, 0x00, 0x32, 0x12, 0x71, 0x00, 0x00, 0x77, 0x21, 0x11,
                        //     0x00, 0x07, 0x03, 0x77, 0x00, 0x00, 0x00, 0x11, 0x07, 0x77, 0x00, 0x00,
                        //     0x70, 0x70, 0x07, 0x77, 0x00, 0x70, 0x66, 0x00, 0x07, 0x77, 0x00, 0x66,
                        //     0x76, 0x00, 0x77, 0x77, 0x70, 0x56, 0x35, 0x00, 0x07, 0x77, 0x66, 0x55,
                        //     0x74, 0x00, 0x07, 0x77, 0x55, 0x44, 0x74, 0x00, 0x00, 0x77, 0x54, 0x34,
                        //     0x13, 0x00, 0x00, 0x77,
                        // ];

                        // match led_dev.write(&tmp) {
                        //     Ok(len) => {
                        //         if len < 64 {
                        //             return Err(HwDeviceError::WriteError {}.into());
                        //         }
                        //     }

                        //     Err(_) => return Err(HwDeviceError::WriteError {}.into()),
                        // }

                        // println!("write 4");

                        // let tmp = [
                        //     0x7f, 0x03, 0x3c, 0x00, 0x44, 0x33, 0x02, 0x07, 0x00, 0x77, 0x33, 0x22,
                        //     0x02, 0x00, 0x00, 0x77, 0x32, 0x12, 0x71, 0x00, 0x00, 0x77, 0x21, 0x11,
                        //     0x00, 0x07, 0x03, 0x77, 0x00, 0x00, 0x00, 0x11, 0x07, 0x77, 0x00, 0x00,
                        //     0x70, 0x70, 0x07, 0x77, 0x00, 0x70, 0x66, 0x00, 0x07, 0x77, 0x00, 0x66,
                        //     0x76, 0x00, 0x77, 0x77, 0x70, 0x56, 0x35, 0x00, 0x07, 0x77, 0x66, 0x55,
                        //     0x74, 0x00, 0x07, 0x77,
                        // ];

                        // match led_dev.write(&tmp) {
                        //     Ok(len) => {
                        //         if len < 64 {
                        //             return Err(HwDeviceError::WriteError {}.into());
                        //         }
                        //     }

                        //     Err(_) => return Err(HwDeviceError::WriteError {}.into()),
                        // }

                        // println!("write 5");

                        // let tmp = [
                        //     0x7f, 0x04, 0x24, 0x00, 0x55, 0x44, 0x74, 0x00, 0x00, 0x77, 0x54, 0x34,
                        //     0x13, 0x00, 0x00, 0x77, 0x44, 0x33, 0x02, 0x07, 0x00, 0x77, 0x33, 0x22,
                        //     0x02, 0x00, 0x00, 0x77, 0x32, 0x12, 0x71, 0x00, 0x00, 0x77, 0x21, 0x11,
                        //     0x00, 0x07, 0x03, 0x77, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        //     0x00, 0x00, 0x00, 0x00,
                        // ];

                        // match led_dev.write(&tmp) {
                        //     Ok(len) => {
                        //         if len < 64 {
                        //             return Err(HwDeviceError::WriteError {}.into());
                        //         }
                        //     }

                        //     Err(_) => return Err(HwDeviceError::WriteError {}.into()),
                        // }

                        // for i in (1..=4).into_iter() {
                        //     let tmp = [
                        //         0x7f, i, 0x3c, 0x00, 0x06, 0x00, 0x66, 0x00, 0x07, 0x77, 0x00,
                        //         0x63, 0x57, 0x70, 0x07, 0x77, 0x00, 0x56, 0x55, 0x00, 0x07, 0x77,
                        //         0x66, 0x55, 0x75, 0x00, 0x77, 0x77, 0x66, 0x45, 0x24, 0x00, 0x07,
                        //         0x77, 0x55, 0x34, 0x73, 0x00, 0x07, 0x77, 0x44, 0x33, 0x73, 0x00,
                        //         0x00, 0x77, 0x43, 0x23, 0x02, 0x00, 0x00, 0x77, 0x33, 0x12, 0x01,
                        //         0x07, 0x00, 0x77, 0x21, 0x11, 0x01, 0x07, 0x70, 0x77,
                        //     ];

                        //     match led_dev.write(&tmp) {
                        //         Ok(len) => {
                        //             if len < 65 {
                        //                 return Err(HwDeviceError::WriteError {}.into());
                        //             }
                        //         }

                        //         Err(_) => return Err(HwDeviceError::WriteError {}.into()),
                        //     }
                        // }

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
}
