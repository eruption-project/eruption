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
use crate::{constants, eprintln_v, interact, println_v};

use super::{DeviceTrait, HwDeviceError, RGBA};

pub type Result<T> = super::Result<T>;

pub const NUM_KEYS: usize = 144;

// pub const CTRL_INTERFACE: i32 = 1; // Control USB sub device
pub const LED_INTERFACE: i32 = 3; // LED USB sub device

#[derive(Clone)]
/// Device specific code for the ROCCAT Magma series keyboards
pub struct RoccatMagma {
    pub is_bound: bool,
    pub ctrl_hiddev: Arc<Mutex<Option<hidapi::HidDevice>>>,
    pub led_hiddev: Arc<Mutex<Option<hidapi::HidDevice>>>,
}

impl RoccatMagma {
    /// Binds the driver to the supplied HID devices
    pub fn bind(ctrl_dev: hidapi::HidDevice, led_dev: hidapi::HidDevice) -> Self {
        println_v!(1, "Bound driver: ROCCAT Magma");

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
                0x00 => {
                    /* let mut buf: [u8; 256] = [0; 256];
                    buf[0] = id;

                    let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
                    let ctrl_dev = ctrl_dev.as_ref().unwrap();

                    match ctrl_dev.get_feature_report(&mut buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    } */

                    thread::sleep(Duration::from_millis(constants::DEVICE_SETTLE_MILLIS));

                    Ok(())
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

                0x05 => {
                    let buf: [u8; 4] = [0x05, 0x04, 0x00, 0x05];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x07 => {
                    let buf: [u8; 11] = [
                        0x07, 0x0b, 0x00, 0x00, 0x00, 0x0b, 0x0a, 0x00, 0x00, 0x27, 0x00,
                    ];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x0c => {
                    let buf: [u8; 85] = [
                        0x0c, 0x55, 0x00, 0x00, 0x00, 0x1e, 0x0c, 0x00, 0x00, 0x1f, 0x0c, 0x00,
                        0x00, 0x20, 0x0c, 0x00, 0x00, 0x21, 0x0c, 0x00, 0x00, 0x22, 0x0c, 0x00,
                        0x00, 0x14, 0x0c, 0x00, 0x00, 0x1a, 0x0c, 0x00, 0x00, 0x08, 0x0c, 0x00,
                        0x00, 0x15, 0x0c, 0x00, 0x00, 0x17, 0x0c, 0x00, 0x00, 0x04, 0x0c, 0x00,
                        0x00, 0x16, 0x0c, 0x00, 0x00, 0x07, 0x0c, 0x00, 0x00, 0x09, 0x0c, 0x00,
                        0x00, 0x0a, 0x0c, 0x00, 0x00, 0x1d, 0x0c, 0x00, 0x00, 0x1b, 0x0c, 0x00,
                        0x00, 0x06, 0x0c, 0x00, 0x00, 0x19, 0x0c, 0x00, 0x00, 0x05, 0x0c, 0xe3,
                        0x02,
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
                    let buf: [u8; 157] = [
                        0x0a, 0x9d, 0x00, 0x00, 0x00, 0x3a, 0x0c, 0x00, 0x00, 0x3b, 0x0c, 0x00,
                        0x00, 0x3c, 0x0c, 0x00, 0x00, 0x3d, 0x0c, 0x00, 0x00, 0x3e, 0x0c, 0x00,
                        0x00, 0x3f, 0x0c, 0x00, 0x00, 0x40, 0x0c, 0x00, 0x00, 0x41, 0x0c, 0x00,
                        0x00, 0x42, 0x0c, 0x00, 0x00, 0x43, 0x0c, 0x00, 0x00, 0x44, 0x0c, 0x00,
                        0x00, 0x45, 0x0c, 0x00, 0x00, 0x46, 0x0c, 0x00, 0x00, 0x47, 0x0c, 0x00,
                        0x00, 0x48, 0x0c, 0x00, 0x00, 0x52, 0x0c, 0x00, 0x00, 0x50, 0x0c, 0x00,
                        0x00, 0x51, 0x0c, 0x00, 0x00, 0x4f, 0x0c, 0x00, 0x00, 0x3a, 0x0c, 0x00,
                        0x00, 0x3b, 0x0c, 0x00, 0x00, 0x3c, 0x0c, 0x00, 0x00, 0x3d, 0x0c, 0x00,
                        0x00, 0x08, 0x03, 0x00, 0x00, 0x07, 0x03, 0x00, 0x00, 0x40, 0x0c, 0x00,
                        0x00, 0x06, 0x03, 0x00, 0x00, 0x02, 0x03, 0x00, 0x00, 0x05, 0x03, 0x00,
                        0x00, 0x04, 0x03, 0x00, 0x00, 0x03, 0x03, 0x00, 0x00, 0x46, 0x0c, 0x00,
                        0x00, 0x0b, 0x08, 0x00, 0x00, 0x48, 0x0c, 0x00, 0x00, 0x09, 0x08, 0x00,
                        0x00, 0x50, 0x0c, 0x00, 0x00, 0x0a, 0x08, 0x00, 0x00, 0x4f, 0x0c, 0xd1,
                        0x09,
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
                    let buf: [u8; 150] = [
                        0x06, 0x96, 0x00, 0x01, 0x90, 0x62, 0x1a, 0x26, 0x2c, 0x00, 0x00, 0x91,
                        0x00, 0x5d, 0xe5, 0x00, 0xe0, 0x00, 0x00, 0x24, 0x47, 0x00, 0x00, 0x59,
                        0xe1, 0xe6, 0x42, 0x1f, 0x57, 0x5c, 0x58, 0x00, 0x55, 0x23, 0x4a, 0x4d,
                        0x61, 0x00, 0x32, 0x63, 0x00, 0x56, 0x4b, 0x22, 0x12, 0x40, 0x04, 0x34,
                        0x37, 0x65, 0x41, 0x00, 0x60, 0x44, 0x16, 0x00, 0x54, 0x21, 0x49, 0x00,
                        0x18, 0x1c, 0x07, 0x0b, 0x10, 0x11, 0x50, 0x00, 0x31, 0x2a, 0x89, 0x5a,
                        0x28, 0x45, 0x00, 0x20, 0x39, 0x00, 0x0e, 0x64, 0x1b, 0x8a, 0x3a, 0x46,
                        0x13, 0x2f, 0x33, 0x00, 0x5b, 0x38, 0x2d, 0x27, 0x00, 0x5e, 0x00, 0x00,
                        0x00, 0x00, 0x25, 0x4f, 0x0c, 0x30, 0x00, 0x3f, 0x36, 0x87, 0x2e, 0x52,
                        0x15, 0x17, 0x09, 0x0a, 0x19, 0x05, 0x4e, 0x00, 0x08, 0x3c, 0x0f, 0x3d,
                        0x06, 0x88, 0x3b, 0x43, 0x14, 0x2b, 0x0d, 0x29, 0x1d, 0x8b, 0x35, 0x3e,
                        0x5f, 0x85, 0xe2, 0x00, 0x53, 0x51, 0x4c, 0x00, 0x48, 0x00, 0x00, 0x00,
                        0xf1, 0x00, 0xe4, 0x1e, 0x4c, 0x1f,
                    ];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x11 => {
                    let buf: [u8; 26] = [
                        0x11, 0x1a, 0x00, 0x09, 0x06, 0x45, 0x00, 0x00, 0x80, 0xff, 0xff, 0xff,
                        0xf5, 0xeb, 0x10, 0x30, 0x50, 0x68, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x54, 0x07,
                    ];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| println_v!(2, "  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
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

                _ => Err(HwDeviceError::InvalidStatusCode {}.into()),
            }
        }
    }

    fn wait_for_ctrl_dev(&self) -> Result<()> {
        println_v!(1, "Waiting for control device to respond...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            /* loop {
                let mut buf: [u8; 4] = [0; 4];
                buf[0] = 0x04;

                let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
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
            } */

            thread::sleep(Duration::from_millis(constants::DEVICE_SETTLE_MILLIS));

            Ok(())
        }
    }
}

impl DeviceTrait for RoccatMagma {
    fn send_init_sequence(&self) -> Result<()> {
        interact::prompt("Press any key to send initialization sequence.");
        println_v!(1, "Sending device init sequence...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            println_v!(1, "Step 1");
            self.query_ctrl_report(0x00)
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 1: {}", e));

            println_v!(1, "Step 2");
            self.send_ctrl_report(0x0d)
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
            self.send_ctrl_report(0x0c)
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 5: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 5: {}", e));

            println_v!(1, "Step 6");
            self.send_ctrl_report(0x0a)
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 6: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 6: {}", e));

            println_v!(1, "Step 7");
            self.send_ctrl_report(0x06)
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 7: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 7: {}", e));

            println_v!(1, "Step 8");
            self.send_ctrl_report(0x11)
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 8: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 8: {}", e));

            println_v!(1, "Step 9");
            self.send_ctrl_report(0x0e)
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 9: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| crate::eprintln_v!(2, "Step 9: {}", e));

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
            let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
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
            let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
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
            let ctrl_dev = self.ctrl_hiddev.lock();
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
                        fn interpolate_colors(colors: &[RGBA]) -> RGBA {
                            type ColorRep = u64;

                            let (mut red, mut green, mut blue, mut alpha): (
                                ColorRep,
                                ColorRep,
                                ColorRep,
                                ColorRep,
                            ) = (0, 0, 0, 0);
                            let len = colors.len() as ColorRep;

                            for color in colors {
                                red = red.saturating_add(color.r as ColorRep);
                                green = green.saturating_add(color.g as ColorRep);
                                blue = blue.saturating_add(color.b as ColorRep);
                                alpha = alpha.saturating_add(color.a as ColorRep);
                            }

                            RGBA {
                                r: (red / len) as u8,
                                g: (green / len) as u8,
                                b: (blue / len) as u8,
                                a: (alpha / len) as u8,
                            }
                        }

                        let zone1 = interpolate_colors(&led_map[0..25]);
                        let zone2 = interpolate_colors(&led_map[25..50]);
                        let zone3 = interpolate_colors(&led_map[50..75]);
                        let zone4 = interpolate_colors(&led_map[75..100]);
                        let zone5 = interpolate_colors(&led_map[100..125]);

                        #[rustfmt::skip]
                        let tmp = [
                            0xa1, 0x01, 0x40,

                            // red
                            zone1.r, zone2.r, zone3.r, zone4.r, zone5.r,

                            // green
                            zone1.g, zone2.g, zone3.g, zone4.g, zone5.g,

                            // blue
                            zone1.b, zone2.b, zone3.b, zone4.b, zone5.b,

                            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                            0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        ];

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
