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

use std::mem::size_of;
use std::time::Duration;
use std::{sync::Arc, thread};
use tracing::*;
use tracing_mutex::stdsync::RwLock;

#[allow(unused)]
use crate::{constants, eprintln_v, println_v};

use super::{DeviceTrait, HwDeviceError, RGBA};

pub type Result<T> = super::Result<T>;

#[allow(unused)]
pub const CTRL_INTERFACE: i32 = 1; // Control USB sub device
pub const LED_INTERFACE: i32 = 2; // LED USB sub device

#[allow(unused)]
pub const NUM_ROWS: usize = 6;

#[allow(unused)]
pub const NUM_COLS: usize = 21;

#[allow(unused)]
pub const NUM_KEYS: usize = 127;
// pub const NUM_RGB: usize = 196;
pub const LED_INDICES: usize = 127;

// Wooting protocol v2 constants
// pub const COMMAND_SIZE: usize = 8;
// pub const REPORT_SIZE: usize = 256 + 1;
pub const SMALL_PACKET_SIZE: usize = 64;
pub const SMALL_PACKET_COUNT: usize = 4;
pub const RESPONSE_SIZE: usize = 256;
pub const READ_RESPONSE_TIMEOUT: i32 = 1000;

/// Wooting protocol v2 commands
#[allow(non_camel_case_types)]
#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum Command {
    RAW_COLORS_REPORT = 11,
    // DEVICE_CONFIG_COMMAND = 19,
    // SINGLE_COLOR_COMMAND = 30,
    // SINGLE_RESET_COMMAND = 31,
    RESET_ALL_COMMAND = 32,
    COLOR_INIT_COMMAND = 33,
}

#[derive(Clone)]
/// Device specific code for the Wooting Two HE series keyboards
pub struct WootingTwoHeArm {
    pub is_bound: bool,
    pub ctrl_hiddev: Arc<RwLock<Option<hidapi::HidDevice>>>,
    pub led_hiddev: Arc<RwLock<Option<hidapi::HidDevice>>>,
}

impl WootingTwoHeArm {
    /// Binds the driver to the supplied HID devices
    pub fn bind(ctrl_dev: hidapi::HidDevice, led_dev: hidapi::HidDevice) -> Self {
        println_v!(1, "Bound driver: Wooting Two HE (ARM)");

        Self {
            is_bound: true,
            ctrl_hiddev: Arc::new(RwLock::new(Some(ctrl_dev))),
            led_hiddev: Arc::new(RwLock::new(Some(led_dev))),
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

    //                 let ctrl_dev = self.ctrl_hiddev.as_ref().read().unwrap();
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

    fn v2_send_feature_report(&self, id: u8, params: &[u8; 4]) -> Result<()> {
        println_v!(2, "Sending control device feature report [Wooting v2]");

        let mut report_buffer = [0x0; SMALL_PACKET_SIZE + 1];

        report_buffer[0] = 0x00;
        report_buffer[1] = 0xd0;
        report_buffer[2] = 0xda;
        report_buffer[3] = id;
        report_buffer[4] = params[3];
        report_buffer[5] = params[2];
        report_buffer[6] = params[1];
        report_buffer[7] = params[0];

        let ctrl_dev = self.ctrl_hiddev.as_ref().read().unwrap();
        let ctrl_dev = ctrl_dev.as_ref().unwrap();

        let result = ctrl_dev.write(&report_buffer);

        match result {
            Ok(_result) => {
                hexdump::hexdump_iter(&report_buffer).for_each(|s| println_v!(2, "  {}", s));

                let mut buf = Vec::with_capacity(RESPONSE_SIZE);
                match ctrl_dev.read_timeout(&mut buf, READ_RESPONSE_TIMEOUT) {
                    Ok(_result) => {
                        hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

                        Ok(())
                    }

                    Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                }
            }

            Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
        }
    }

    #[allow(dead_code)]
    fn send_ctrl_report(&self, _id: u8) -> Result<()> {
        println_v!(1, "Sending control device feature report");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            // let ctrl_dev = self.ctrl_hiddev.as_ref().read().unwrap();
            // let ctrl_dev = ctrl_dev.as_ref().unwrap();

            // match id {
            //     0x00 => {
            //         let buf: [u8; 1] = [0x00];

            //         match ctrl_dev.send_feature_report(&buf) {
            //             Ok(_result) => {
            //                 hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

            //                 Ok(())
            //             }

            //             Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
            //         }
            //     }

            //     _ => Err(HwDeviceError::InvalidStatusCode {}.into()),
            // }

            Ok(())
        }
    }

    #[allow(dead_code)]
    fn send_led_data(&self, id: u8) -> Result<()> {
        println_v!(0, "Sending data to LED device");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            // let led_dev = self.ctrl_hiddev.as_ref().lock().unwrap();
            // let led_dev = led_dev.as_ref().unwrap();

            match id {
                0xa1 => {
                    // let buf: [u8; 64] = [
                    //     0xa1, 0x01, 0x34, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00,
                    // ];

                    // match led_dev.write(&buf) {
                    //     Ok(_result) => {
                    //         hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));
                    //     }

                    //     Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
                    // }

                    // let buf: [u8; 64] = [
                    //     0xa1, 0x02, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00,
                    // ];

                    // match led_dev.write(&buf) {
                    //     Ok(_result) => {
                    //         hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));
                    //     }

                    //     Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
                    // }

                    // let buf: [u8; 64] = [
                    //     0xa1, 0x03, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00,
                    // ];

                    // match led_dev.write(&buf) {
                    //     Ok(_result) => {
                    //         hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));
                    //     }

                    //     Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
                    // }

                    // let buf: [u8; 64] = [
                    //     0xa1, 0x04, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00,
                    // ];

                    // match led_dev.write(&buf) {
                    //     Ok(_result) => {
                    //         hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));
                    //     }

                    //     Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
                    // }

                    // let buf: [u8; 64] = [
                    //     0xa1, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                    //     0x00, 0x00, 0x00, 0x00,
                    // ];

                    // match led_dev.write(&buf) {
                    //     Ok(_result) => {
                    //         hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));
                    //     }

                    //     Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
                    // }

                    Ok(())
                }

                _ => Err(HwDeviceError::InvalidStatusCode {}.into()),
            }
        }
    }

    fn wait_for_ctrl_dev(&self) -> Result<()> {
        println_v!(2, "Waiting for control device to respond...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            let mut buf: [u8; RESPONSE_SIZE] = [0x00; RESPONSE_SIZE];

            let ctrl_dev = self.ctrl_hiddev.as_ref().read().unwrap();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            match ctrl_dev.read_timeout(&mut buf, READ_RESPONSE_TIMEOUT) {
                Ok(_result) => {
                    hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

                    Ok(())
                }

                Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
            }
        }
    }

    // #[allow(dead_code)]
    // fn wait_for_led_dev(&mut self) -> Result<()> {
    //     println_v!(2, "Waiting for LED device to respond...");

    //     if !self.is_bound {
    //         Err(HwDeviceError::DeviceNotBound {}.into())
    //     } else {
    //         let mut buf: [u8; RESPONSE_SIZE] = [0x00; RESPONSE_SIZE];

    //         let led_dev = self.led_hiddev.as_ref().lock().unwrap();
    //         let led_dev = led_dev.as_ref().unwrap();

    //         match led_dev.read_timeout(&mut buf, READ_RESPONSE_TIMEOUT) {
    //             Ok(_result) => {
    //                 hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

    //                 Ok(())
    //             }

    //             Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
    //         }
    //     }
    // }
}

impl DeviceTrait for WootingTwoHeArm {
    fn send_init_sequence(&self) -> Result<()> {
        println_v!(1, "Sending device init sequence...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            println_v!(1, "Step 1");
            self.v2_send_feature_report(Command::RESET_ALL_COMMAND as u8, &[0, 0, 0, 0])
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 1: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 1: {}", e));

            println_v!(1, "Step 2");
            self.v2_send_feature_report(Command::COLOR_INIT_COMMAND as u8, &[0, 0, 0, 0])
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 1: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 1: {}", e));

            Ok(())
        }
    }

    fn write_data_raw(&self, buf: &[u8]) -> Result<()> {
        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            let ctrl_dev = self.ctrl_hiddev.as_ref().read().unwrap();
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
            let ctrl_dev = self.ctrl_hiddev.as_ref().read().unwrap();
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
            let ctrl_dev = self.ctrl_hiddev.as_ref().read().unwrap();
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
            let ctrl_dev = self.ctrl_hiddev.as_ref().read().unwrap();
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
            match *self.led_hiddev.read().unwrap() {
                Some(ref led_dev) => {
                    if led_map.len() < LED_INDICES {
                        error!(
                            "Received a short LED map: Got {} elements, but should be {}",
                            led_map.len(),
                            LED_INDICES
                        );

                        Err(HwDeviceError::LedMapError {}.into())
                    } else {
                        #[inline]
                        fn encode_color(color: &RGBA) -> u16 {
                            let mut encoded_color: u16 = 0x0000;

                            encoded_color |= (color.b as u16 & 0xf8) >> 3;
                            encoded_color |= (color.g as u16 & 0xfc) << 3;
                            encoded_color |= (color.r as u16 & 0xf8) << 8;

                            encoded_color
                        }

                        #[inline]
                        #[allow(dead_code)]
                        fn index_of(cntr: usize) -> Option<usize> {
                            // let x = cntr / NUM_COLS;
                            // let y = cntr % NUM_COLS;

                            TOPOLOGY.get(cntr).cloned().and_then(|v| {
                                if v == 0xff {
                                    None
                                } else {
                                    Some(v as usize)
                                }
                            })
                        }

                        #[inline]
                        fn submit_packet(led_dev: &hidapi::HidDevice, buffer: &[u8]) -> Result<()> {
                            hexdump::hexdump_iter(buffer).for_each(|s| println_v!(2, "  {}", s));

                            assert_eq!(buffer.len(), SMALL_PACKET_SIZE + 1);

                            match led_dev.write(buffer) {
                                Ok(len) => {
                                    if len < SMALL_PACKET_SIZE + 1 {
                                        return Err(HwDeviceError::WriteError {}.into());
                                    }

                                    // let mut buf: [u8; RESPONSE_SIZE] = [0x00; RESPONSE_SIZE];
                                    // match led_dev.read_timeout(&mut buf, 50) {
                                    //     Ok(_result) => {
                                    //         hexdump::hexdump_iter(&buf)
                                    //             .for_each(|s| println_v2!("  {}", s));
                                    //     }

                                    //     Err(_) => {
                                    //         return Err(HwDeviceError::InvalidResult {}.into())
                                    //     }
                                    // }

                                    thread::sleep(Duration::from_millis(10));
                                }

                                Err(_) => return Err(HwDeviceError::WriteError {}.into()),
                            }

                            Ok(())
                        }

                        const BUFFER_SIZE: usize =
                            4 + (SMALL_PACKET_COUNT * (SMALL_PACKET_SIZE + 1)) + 2;
                        let mut buffer = [0x0_u8; BUFFER_SIZE];
                        let mut cntr = 0;

                        // let led_map = led_map
                        //     .iter()
                        //     .enumerate()
                        //     .map(|(idx, _c)| led_map[index_of(idx)])
                        //     .collect::<Vec<_>>();

                        // init sequence
                        buffer[0..4].copy_from_slice(&[
                            0x00,
                            0xd0,
                            0xda,
                            Command::RAW_COLORS_REPORT as u8,
                        ]);

                        // encoded color sequence and submit a packet on every 64th byte to the device
                        for i in (4..BUFFER_SIZE).step_by(2) {
                            if i % 64 == 0 {
                                buffer[i] = 0x0;

                                submit_packet(led_dev, &buffer[(i - 64)..=i])?;
                            } else {
                                let index = cntr / size_of::<RGBA>();
                                let encoded_color =
                                            // encode_color(&led_map[index_of(cntr).unwrap_or(0x0)]);
                                            encode_color(led_map.get(index).unwrap_or(&RGBA {
                                                r: 0x00,
                                                g: 0x00,
                                                b: 0x00,
                                                a: 0x00,
                                            }));

                                buffer[i..i + 2].copy_from_slice(&encoded_color.to_le_bytes());

                                cntr += 1;
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
            }; 127],
        )?;

        thread::sleep(Duration::from_millis(500));

        self.send_led_map(
            &[RGBA {
                r: 0,
                g: 0,
                b: 255,
                a: 255,
            }; 127],
        )?;

        Ok(())
    }

    fn device_status(&self) -> super::Result<super::DeviceStatus> {
        Err(HwDeviceError::OpNotSupported {}.into())
    }
}

/// Utility functions
mod util {
    /// Implementation of CRC16_CCITT
    /// TODO: Do we need to use persistent state?
    #[inline]
    #[allow(dead_code)]
    fn crc16_ccitt(data: &[u8]) -> u16 {
        let mut state = crc16::State::<crc16::AUG_CCITT>::new();
        state.update(data);
        state.get()
    }
}

const NOKEY: u8 = 0xff;

#[rustfmt::skip]
#[allow(dead_code)]
pub const TOPOLOGY: [u8; 126] = [
    0, NOKEY, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 107, 108, 109, 110,
    16, 17, 18, 19, 20, 21, 22, 23, 24, 25, 26, 27, 28, 29, 30, 31, 61, 106, 105, 104, 103,
    32, 33, 34, 35, 36, 37, 38, 39, 40, 41, 42, 43, 44, 45, 46, 47, 62, 102, 101, 100, 99,
    48, 49, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 45, 60, NOKEY, NOKEY, NOKEY, 98, 97, 96, NOKEY,
    64, 87, 65, 66, 67, 68, 69, 70, 71, 72, 73, 74, NOKEY, 75, NOKEY, 63, NOKEY, 90, 91, 92, 93,
    80, 81, 82, NOKEY, NOKEY, NOKEY, 83, NOKEY, NOKEY, NOKEY, 84, 85, 86, 79, 76, 77, 78, NOKEY, 95, 94, NOKEY
];
