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

use bitvec::{order::Lsb0, prelude::BitField, view::BitView};
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
                        // build and send data buffer chunks
                        let mut buffer: [u8; NUM_KEYS * 3] = [0xff; NUM_KEYS * 3];

                        for i in 0..NUM_KEYS {
                            let color = led_map[i];

                            // convert RGB color to monochromatic value
                            // let color = (((led_map[i].r as f64 * 0.29)
                            //     + (led_map[i].g as f64 * 0.59)
                            //     + (led_map[i].b as f64 * 0.114))
                            //     .round() as u8)
                            //     .clamp(0, 255);

                            let bitvec = buffer.view_bits_mut::<Lsb0>();

                            let offset = (i * 3) + 1;

                            bitvec[(offset + 0)..(offset + 3)].store(color.r.to_le() >> 5);
                            bitvec[(offset + 3)..(offset + 6)].store(color.g.to_le() >> 5);
                            bitvec[(offset + 6)..(offset + 9)].store(color.b.to_le() >> 5);
                        }

                        for (cntr, bytes) in buffer.chunks(60).take(4).enumerate() {
                            let mut tmp: [u8; 64] = [0; 64];

                            if cntr < 3 {
                                tmp[0..4].copy_from_slice(&[0x7f, cntr as u8 + 1, 0x3c, 00]);
                            } else {
                                tmp[0..4].copy_from_slice(&[0x7f, cntr as u8 + 1, 0x30, 00]);
                            }

                            tmp[4..64].copy_from_slice(&bytes);

                            hexdump::hexdump_iter(&tmp).for_each(|s| trace!("  {}", s));

                            match led_dev.write(&tmp) {
                                Ok(len) => {
                                    if len < 64 {
                                        return Err(HwDeviceError::WriteError {}.into());
                                    }
                                }

                                Err(_) => return Err(HwDeviceError::WriteError {}.into()),
                            }
                        }

                        // commit the LED map to the keyboard
                        let tmp: [u8; 64] = [
                            0x07, 0x27, 0x00, 0x00, 0xd8, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                            0x00, 0x00, 0x00, 0x00,
                        ];

                        hexdump::hexdump_iter(&tmp).for_each(|s| trace!("  {}", s));

                        match led_dev.write(&tmp) {
                            Ok(len) => {
                                if len < 64 {
                                    return Err(HwDeviceError::WriteError {}.into());
                                }
                            }

                            Err(_) => return Err(HwDeviceError::WriteError {}.into()),
                        }

                        Ok(())
                    }
                }

                None => Err(HwDeviceError::DeviceNotOpened {}.into()),
            }
        }
    }

    fn send_test_pattern(&self) -> Result<()> {
        // init to LEDs off
        self.send_led_map(
            &[RGBA {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            }; NUM_KEYS],
        )?;

        // test each LED

        for i in (0..NUM_KEYS).into_iter() {
            let mut led_map = [RGBA {
                r: 0,
                g: 0,
                b: 0,
                a: 255,
            }; NUM_KEYS];

            led_map[i].r = 255;
            led_map[i].g = 0;
            led_map[i].b = 0;
            led_map[i].a = 255;

            self.send_led_map(&led_map)?;

            thread::sleep(Duration::from_millis(100));
        }

        Ok(())
    }
}
