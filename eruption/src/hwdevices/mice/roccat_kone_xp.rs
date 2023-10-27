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

use bitvec::prelude::*;
#[cfg(not(target_os = "windows"))]
use evdev_rs::enums::EV_KEY;
use hidapi::HidApi;
use parking_lot::Mutex;
use tracing::*;
// use std::sync::atomic::Ordering;
use std::any::Any;
use std::collections::HashMap;
use std::sync::Arc;

use crate::{constants, hwdevices};

use crate::hwdevices::{
    Capability, DeviceCapabilities, DeviceClass, DeviceExt, DeviceInfoExt, DeviceStatus,
    DeviceZoneAllocationExt, HwDeviceError, MouseDeviceExt, MouseHidEvent, Result,
    Zone, RGBA,
};

pub const SUB_DEVICE: i32 = 2; // USB HID sub-device to bind to

// pub const NUM_BUTTONS: usize = 9;

// canvas to LED index mapping
pub const LED_0: usize = constants::CANVAS_SIZE - 36;
pub const LED_1: usize = constants::CANVAS_SIZE - 35;
pub const LED_2: usize = constants::CANVAS_SIZE - 34;
pub const LED_3: usize = constants::CANVAS_SIZE - 33;
pub const LED_4: usize = constants::CANVAS_SIZE - 32;
pub const LED_5: usize = constants::CANVAS_SIZE - 31;
pub const LED_6: usize = constants::CANVAS_SIZE - 30;
pub const LED_7: usize = constants::CANVAS_SIZE - 29;
pub const LED_8: usize = constants::CANVAS_SIZE - 28;
pub const LED_9: usize = constants::CANVAS_SIZE - 27;
pub const LED_10: usize = constants::CANVAS_SIZE - 26;
pub const LED_11: usize = constants::CANVAS_SIZE - 25;
pub const LED_12: usize = constants::CANVAS_SIZE - 24;
pub const LED_13: usize = constants::CANVAS_SIZE - 23;
pub const LED_14: usize = constants::CANVAS_SIZE - 22;
pub const LED_15: usize = constants::CANVAS_SIZE - 21;
pub const LED_16: usize = constants::CANVAS_SIZE - 20;
pub const LED_17: usize = constants::CANVAS_SIZE - 19;
pub const LED_18: usize = constants::CANVAS_SIZE - 18;
pub const LED_19: usize = constants::CANVAS_SIZE - 17;
pub const LED_20: usize = constants::CANVAS_SIZE - 16;
pub const LED_21: usize = constants::CANVAS_SIZE - 15;
pub const LED_22: usize = constants::CANVAS_SIZE - 14;

// stripes
pub const LED_23: usize = constants::CANVAS_SIZE - 36;
pub const LED_24: usize = constants::CANVAS_SIZE - 1;

pub const LED_25: usize = constants::CANVAS_SIZE - 35;
pub const LED_26: usize = constants::CANVAS_SIZE - 1;

pub const LED_27: usize = constants::CANVAS_SIZE - 34;
pub const LED_28: usize = constants::CANVAS_SIZE - 2;

pub const LED_29: usize = constants::CANVAS_SIZE - 33;
pub const LED_30: usize = constants::CANVAS_SIZE - 3;

pub const LED_31: usize = constants::CANVAS_SIZE - 33;
pub const LED_32: usize = constants::CANVAS_SIZE - 3;

pub const LED_33: usize = constants::CANVAS_SIZE - 34;
pub const LED_34: usize = constants::CANVAS_SIZE - 2;

pub const LED_35: usize = constants::CANVAS_SIZE - 35;
pub const LED_36: usize = constants::CANVAS_SIZE - 1;

pub const LED_37: usize = constants::CANVAS_SIZE - 36;
pub const LED_38: usize = constants::CANVAS_SIZE - 1;

pub const NUM_LEDS: usize = 39;

/// Binds the driver to a device
pub fn bind_hiddev(
    hidapi: &HidApi,
    usb_vid: u16,
    usb_pid: u16,
    serial: &str,
) -> Result<Box<dyn DeviceExt + Sync + Send>> {
    let ctrl_dev = hidapi.device_list().find(|&device| {
        device.vendor_id() == usb_vid
            && device.product_id() == usb_pid
            && device.serial_number().unwrap_or("") == serial
            && device.interface_number() == SUB_DEVICE
    });

    if ctrl_dev.is_none() {
        Err(HwDeviceError::EnumerationError {}.into())
    } else {
        Ok(Box::new(RoccatKoneXp::bind(ctrl_dev.unwrap())))
    }
}

/// ROCCAT Kone XP info struct (sent as HID report)
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct DeviceInfo {
    pub report_id: u8,
    pub size: u8,
    pub firmware_version: u8,
    pub reserved1: u8,
    pub reserved2: u8,
    pub reserved3: u8,
}

#[derive(Clone)]
/// Device specific code for the ROCCAT Kone XP mouse
pub struct RoccatKoneXp {
    pub is_initialized: bool,

    pub is_bound: bool,
    pub ctrl_hiddev_info: Option<hidapi::DeviceInfo>,

    pub is_opened: bool,
    pub ctrl_hiddev: Arc<Mutex<Option<hidapi::HidDevice>>>,

    pub button_states: Arc<Mutex<BitVec>>,

    pub has_failed: bool,

    pub allocated_zone: Zone,

    // device specific configuration options
    pub brightness: i32,
}

impl RoccatKoneXp {
    /// Binds the driver to the supplied HID device
    pub fn bind(ctrl_dev: &hidapi::DeviceInfo) -> Self {
        info!("Bound driver: ROCCAT Kone XP");

        Self {
            is_initialized: false,

            is_bound: true,
            ctrl_hiddev_info: Some(ctrl_dev.clone()),

            is_opened: false,
            ctrl_hiddev: Arc::new(Mutex::new(None)),

            button_states: Arc::new(Mutex::new(bitvec![0; constants::MAX_MOUSE_BUTTONS])),

            has_failed: false,

            allocated_zone: Zone::defaults_for(DeviceClass::Mouse),

            brightness: 100,
        }
    }

    // pub(self) fn query_ctrl_report(&mut self, id: u8) -> Result<()> {
    //     trace!("Querying control device feature report");

    //     if !self.is_bound {
    //         Err(HwDeviceError::DeviceNotBound {}.into())
    //     } else if !self.is_opened {
    //         Err(HwDeviceError::DeviceNotOpened {}.into())
    //     } else {
    //         match id {
    //             0x0f => {
    //                 let mut buf: [u8; 256] = [0; 256];
    //                 buf[0] = id;

    //                 let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
    //                 let ctrl_dev = ctrl_dev.as_ref().unwrap();

    //                 match ctrl_dev.get_feature_report(&mut buf) {
    //                     Ok(_result) => {
    //                         #[cfg(debug_assertions)]
    // hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

    //                         Ok(())
    //                     }

    //                     Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
    //                 }
    //             }

    //             _ => Err(HwDeviceError::InvalidStatusCode {}.into()),
    //         }
    //     }
    // }

    fn send_ctrl_report(&mut self, id: u8) -> Result<()> {
        trace!("Sending control device feature report");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            match id {
                0x04 => {
                    for j in [0x80, 0x90] {
                        for i in 0..=4 {
                            let buf: [u8; 4] = [0x04, i, j, 0x00];

                            match ctrl_dev.send_feature_report(&buf) {
                                Ok(_result) => {
                                    #[cfg(debug_assertions)]
                                    hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

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
                            #[cfg(debug_assertions)]
                            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

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
                            #[cfg(debug_assertions)]
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

    fn wait_for_ctrl_dev(&mut self) -> Result<()> {
        trace!("Waiting for control device to respond...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            loop {
                let mut buf: [u8; 4] = [0; 4];
                buf[0] = 0x01;

                let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
                let ctrl_dev = ctrl_dev.as_ref().unwrap();

                match ctrl_dev.get_feature_report(&mut buf) {
                    Ok(_result) => {
                        #[cfg(debug_assertions)]
                        hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                        if buf[1] == 0x01 {
                            return Ok(());
                        }
                    }

                    Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
                }
            }
        }
    }
}

impl DeviceInfoExt for RoccatKoneXp {
    fn get_device_capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities::from([Capability::Mouse, Capability::RgbLighting])
    }

    fn get_device_info(&self) -> Result<hwdevices::DeviceInfo> {
        trace!("Querying the device for information...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            /* let mut buf = [0; size_of::<DeviceInfo>()];
                        buf[0] = 0x09; // Query device info (HID report 0x09)

                        let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
                        let ctrl_dev = ctrl_dev.as_ref().unwrap();

                        match ctrl_dev.get_feature_report(&mut buf) {
                            Ok(_result) => {
                                #[cfg(debug_assertions)]
            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));
                                let tmp: DeviceInfo =
                                    unsafe { std::ptr::read_unaligned(buf.as_ptr() as *const _) };

                                let result = hwdevices::DeviceInfo::new(tmp.firmware_version as i32);
                                Ok(result)
                            }

                            Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                        } */

            let result = hwdevices::DeviceInfo::new(0_i32);
            Ok(result)
        }
    }

    fn get_firmware_revision(&self) -> String {
        if let Ok(device_info) = self.get_device_info() {
            format!(
                "{}.{:02}",
                device_info.firmware_version / 100,
                device_info.firmware_version % 100
            )
        } else {
            "<unknown>".to_string()
        }
    }
}

impl DeviceExt for RoccatKoneXp {
    fn get_usb_path(&self) -> String {
        self.ctrl_hiddev_info
            .clone()
            .unwrap()
            .path()
            .to_str()
            .unwrap()
            .to_string()
    }

    fn get_usb_vid(&self) -> u16 {
        self.ctrl_hiddev_info.as_ref().unwrap().vendor_id()
    }

    fn get_usb_pid(&self) -> u16 {
        self.ctrl_hiddev_info.as_ref().unwrap().product_id()
    }

    fn get_serial(&self) -> Option<&str> {
        self.ctrl_hiddev_info.as_ref().unwrap().serial_number()
    }

    fn get_support_script_file(&self) -> String {
        "mice/roccat_kone_xp".to_string()
    }

    fn open(&mut self, api: &hidapi::HidApi) -> Result<()> {
        trace!("Opening HID devices now...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            trace!("Opening control device...");

            match self.ctrl_hiddev_info.as_ref().unwrap().open_device(api) {
                Ok(dev) => *self.ctrl_hiddev.lock() = Some(dev),
                Err(_) => return Err(HwDeviceError::DeviceOpenError {}.into()),
            };

            self.is_opened = true;

            Ok(())
        }
    }

    fn close_all(&mut self) -> Result<()> {
        trace!("Closing HID devices now...");

        // close keyboard device
        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            trace!("Closing control device...");
            *self.ctrl_hiddev.lock() = None;

            self.is_opened = false;

            Ok(())
        }
    }

    fn send_init_sequence(&mut self) -> Result<()> {
        trace!("Sending device init sequence...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            // match self.get_device_info() {
            //     Ok(device_info) => {
            //         if device_info.firmware_version < 106 {
            //             warn!(
            //                 "Outdated firmware version: {}, should be: >= 1.06",
            //                 format!(
            //                     "{}.{:02}",
            //                     device_info.firmware_version / 100,
            //                     device_info.firmware_version % 100
            //                 )
            //             );
            //         }
            //     }
            //
            //     Err(e) => {
            //         error!("Could not get firmware version: {}", e);
            //     }
            // }

            self.send_ctrl_report(0x04)
                .unwrap_or_else(|e| error!("Step 1: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| error!("Wait 1: {}", e));

            self.send_ctrl_report(0x0e)
                .unwrap_or_else(|e| error!("Step 2: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| error!("Wait 2: {}", e));

            self.send_ctrl_report(0x0d)
                .unwrap_or_else(|e| error!("Step 3: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| error!("Wait 3: {}", e));

            self.is_initialized = true;

            Ok(())
        }
    }

    fn send_shutdown_sequence(&mut self) -> Result<()> {
        trace!("Sending device shutdown sequence...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            // self.send_ctrl_report(0xa1)
            //     .unwrap_or_else(|e| error!("Step 1: {}", e));
            // self.wait_for_ctrl_dev()
            //     .unwrap_or_else(|e| error!("Wait 1: {}", e));

            self.is_initialized = false;

            Ok(())
        }
    }

    fn is_initialized(&self) -> Result<bool> {
        Ok(self.is_initialized)
    }

    fn has_failed(&self) -> Result<bool> {
        Ok(self.has_failed)
    }

    fn fail(&mut self) -> Result<()> {
        self.has_failed = true;
        Ok(())
    }

    fn write_data_raw(&self, buf: &[u8]) -> Result<()> {
        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else if !self.is_initialized {
            Err(HwDeviceError::DeviceNotInitialized {}.into())
        } else {
            let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            match ctrl_dev.write(buf) {
                Ok(_result) => {
                    hexdump::hexdump_iter(buf).for_each(|s| trace!("  {}", s));

                    Ok(())
                }

                Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
            }
        }
    }

    fn read_data_raw(&self, size: usize) -> Result<Vec<u8>> {
        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else if !self.is_initialized {
            Err(HwDeviceError::DeviceNotInitialized {}.into())
        } else {
            let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            let mut buf = Vec::new();
            buf.resize(size, 0);

            match ctrl_dev.read(buf.as_mut_slice()) {
                Ok(_result) => {
                    #[cfg(debug_assertions)]
                    hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                    Ok(buf)
                }

                Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
            }
        }
    }

    fn device_status(&self) -> Result<DeviceStatus> {
        let mut table = HashMap::new();

        table.insert("connected".to_owned(), format!("{}", true));

        Ok(DeviceStatus(table))
    }

    fn get_brightness(&self) -> Result<i32> {
        trace!("Querying device specific brightness");

        Ok(self.brightness)
    }

    fn set_brightness(&mut self, brightness: i32) -> Result<()> {
        trace!("Setting device specific brightness");

        self.brightness = brightness;

        Ok(())
    }

    fn send_led_map(&mut self, led_map: &[RGBA]) -> Result<()> {
        trace!("Setting LEDs from supplied map...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else if !self.is_initialized {
            Err(HwDeviceError::DeviceNotInitialized {}.into())
        } else if self.allocated_zone.enabled {
            let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            let buf: [u8; 122] = [
                0x0d,
                0x7a,
                (led_map[LED_0].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_0].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_0].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_1].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_1].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_1].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_2].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_2].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_2].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_3].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_3].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_3].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_4].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_4].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_4].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_5].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_5].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_5].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_6].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_6].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_6].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_7].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_7].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_7].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_8].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_8].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_8].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_9].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_9].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_9].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_10].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_10].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_10].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_11].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_11].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_11].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_12].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_12].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_12].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_13].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_13].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_13].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_14].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_14].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_14].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_15].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_15].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_15].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_16].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_16].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_16].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_17].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_17].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_17].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_18].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_18].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_18].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_19].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_19].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_19].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_20].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_20].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_20].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_21].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_21].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_21].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_22].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_22].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_22].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_23].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_23].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_23].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_24].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_24].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_24].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_25].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_25].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_25].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_26].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_26].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_26].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_27].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_27].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_27].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_28].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_28].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_28].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_29].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_29].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_29].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_30].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_30].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_30].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_31].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_31].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_31].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_32].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_32].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_32].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_33].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_33].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_33].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_34].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_34].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_34].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_35].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_35].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_35].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_36].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_36].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_36].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_37].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_37].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_37].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_38].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_38].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_38].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                0xff,
                0xff,
                0xff,
            ];

            match ctrl_dev.send_feature_report(&buf) {
                Ok(_result) => {
                    #[cfg(debug_assertions)]
                    hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                    Ok(())
                }

                Err(_) => {
                    // the device has failed or has been disconnected
                    self.is_initialized = false;
                    self.is_opened = false;
                    self.has_failed = true;

                    Err(HwDeviceError::InvalidResult {}.into())
                }
            }
        } else {
            Ok(())
        }
    }

    fn set_led_init_pattern(&mut self) -> Result<()> {
        trace!("Setting LED init pattern...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else if !self.is_initialized {
            Err(HwDeviceError::DeviceNotInitialized {}.into())
        } else {
            let led_map: [RGBA; constants::CANVAS_SIZE] = [RGBA {
                r: 0x00,
                g: 0x00,
                b: 0x00,
                a: 0x00,
            }; constants::CANVAS_SIZE];

            self.send_led_map(&led_map)?;

            Ok(())
        }
    }

    fn set_led_off_pattern(&mut self) -> Result<()> {
        trace!("Setting LED off pattern...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else if !self.is_initialized {
            Err(HwDeviceError::DeviceNotInitialized {}.into())
        } else {
            let led_map: [RGBA; constants::CANVAS_SIZE] = [RGBA {
                r: 0x00,
                g: 0x00,
                b: 0x00,
                a: 0x00,
            }; constants::CANVAS_SIZE];

            self.send_led_map(&led_map)?;

            Ok(())
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_device(&self) -> &(dyn DeviceExt + Sync + Send) {
        self
    }

    fn as_device_mut(&mut self) -> &mut (dyn DeviceExt + Sync + Send) {
        self
    }

    fn as_mouse_device(&self) -> Option<&(dyn MouseDeviceExt + Sync + Send)> {
        Some(self)
    }

    fn as_mouse_device_mut(&mut self) -> Option<&mut (dyn MouseDeviceExt + Sync + Send)> {
        Some(self)
    }

    fn get_device_class(&self) -> DeviceClass {
        DeviceClass::Mouse
    }

    fn as_keyboard_device(&self) -> Option<&(dyn hwdevices::KeyboardDeviceExt + Send + Sync)> {
        None
    }

    fn as_keyboard_device_mut(
        &mut self,
    ) -> Option<&mut (dyn hwdevices::KeyboardDeviceExt + Send + Sync)> {
        None
    }

    fn as_misc_device(&self) -> Option<&(dyn hwdevices::MiscDeviceExt + Send + Sync)> {
        None
    }

    fn as_misc_device_mut(&mut self) -> Option<&mut (dyn hwdevices::MiscDeviceExt + Send + Sync)> {
        None
    }
}

impl DeviceZoneAllocationExt for RoccatKoneXp {
    fn get_zone_size_hint(&self) -> usize {
        NUM_LEDS
    }

    fn get_allocated_zone(&self) -> Zone {
        self.allocated_zone
    }

    fn set_zone_allocation(&mut self, zone: Zone) {
        self.allocated_zone = zone;
    }
}

impl MouseDeviceExt for RoccatKoneXp {
    fn get_profile(&self) -> Result<i32> {
        trace!("Querying device profile config");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            // let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            // let ctrl_dev = ctrl_dev.as_ref().unwrap();
            //
            // let mut buf: [u8; 64] = [0x00_u8; 64];
            // buf[0] = 0x06;
            //
            // match ctrl_dev.get_feature_report(&mut buf) {
            //     Ok(_result) => {
            //         #[cfg(debug_assertions)]
            //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));
            //
            //         Ok(())
            //     }
            //
            //     Err(_) => Err(HwDeviceError::InvalidResult {}),
            // }?;
            //
            // Ok(buf[3] as i32)

            Err(HwDeviceError::OpNotSupported {}.into())
        }
    }

    fn set_profile(&mut self, _profile: i32) -> Result<()> {
        trace!("Setting device profile config");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            // let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            // let ctrl_dev = ctrl_dev.as_ref().unwrap();
            //
            // let mut buf: [u8; 64] = [0x00_u8; 64];
            // buf[0] = 0x06;
            //
            // match ctrl_dev.get_feature_report(&mut buf) {
            //     Ok(_result) => {
            //         #[cfg(debug_assertions)]
            //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));
            //
            //         Ok(())
            //     }
            //
            //     Err(_) => Err(HwDeviceError::InvalidResult {}),
            // }?;
            //
            // buf[3] = profile as u8;
            //
            // match ctrl_dev.send_feature_report(&buf) {
            //     Ok(_result) => {
            //         #[cfg(debug_assertions)]
            //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));
            //
            //         Ok(())
            //     }
            //
            //     Err(_) => Err(HwDeviceError::InvalidResult {}),
            // }?;
            //
            // Ok(())

            Err(HwDeviceError::OpNotSupported {}.into())
        }
    }

    fn get_dpi(&self) -> Result<i32> {
        trace!("Querying device DPI config");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            // let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            // let ctrl_dev = ctrl_dev.as_ref().unwrap();
            //
            // let mut buf: [u8; 64] = [0x00_u8; 64];
            // buf[0] = 0x06;
            //
            // match ctrl_dev.get_feature_report(&mut buf) {
            //     Ok(_result) => {
            //         #[cfg(debug_assertions)]
            //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));
            //
            //         Ok(())
            //     }
            //
            //     Err(_) => Err(HwDeviceError::InvalidResult {}),
            // }?;
            //
            // Ok(buf[6] as i32)

            Err(HwDeviceError::OpNotSupported {}.into())
        }
    }

    fn set_dpi(&mut self, _dpi: i32) -> Result<()> {
        trace!("Setting device DPI config");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            // let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            // let ctrl_dev = ctrl_dev.as_ref().unwrap();
            //
            // let mut buf: [u8; 64] = [0x00_u8; 64];
            // buf[0] = 0x06;
            //
            // match ctrl_dev.get_feature_report(&mut buf) {
            //     Ok(_result) => {
            //         #[cfg(debug_assertions)]
            //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));
            //
            //         Ok(())
            //     }
            //
            //     Err(_) => Err(HwDeviceError::InvalidResult {}),
            // }?;
            //
            // buf[6] = dpi as u8;
            //
            // match ctrl_dev.send_feature_report(&buf) {
            //     Ok(_result) => {
            //         #[cfg(debug_assertions)]
            //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));
            //
            //         Ok(())
            //     }
            //
            //     Err(_) => Err(HwDeviceError::InvalidResult {}),
            // }?;
            //
            // Ok(())

            Err(HwDeviceError::OpNotSupported {}.into())
        }
    }

    fn get_rate(&self) -> Result<i32> {
        trace!("Querying device poll rate config");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            // let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            // let ctrl_dev = ctrl_dev.as_ref().unwrap();
            //
            // let mut buf: [u8; 64] = [0x00_u8; 64];
            // buf[0] = 0x11;
            //
            // match ctrl_dev.get_feature_report(&mut buf) {
            //     Ok(_result) => {
            //         #[cfg(debug_assertions)]
            //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));
            //
            //         Ok(())
            //     }
            //
            //     Err(_) => Err(HwDeviceError::InvalidResult {}),
            // }?;
            //
            // match buf[29] {
            //     0 => Ok(125),
            //
            //     1 => Ok(250),
            //
            //     2 => Ok(500),
            //
            //     3 => Ok(1000),
            //
            //     _ => Err(HwDeviceError::InvalidResult {}.into()),
            // }

            Err(HwDeviceError::OpNotSupported {}.into())
        }
    }

    fn set_rate(&mut self, _rate: i32) -> Result<()> {
        trace!("Setting device poll rate config");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            // let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            // let ctrl_dev = ctrl_dev.as_ref().unwrap();
            //
            // let mut buf: [u8; 64] = [0x00_u8; 64];
            // buf[0] = 0x11;
            //
            // match ctrl_dev.get_feature_report(&mut buf) {
            //     Ok(_result) => {
            //         #[cfg(debug_assertions)]
            //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));
            //
            //         Ok(())
            //     }
            //
            //     Err(_) => Err(HwDeviceError::InvalidResult {}),
            // }?;
            //
            // buf[29] = rate as u8;
            //
            // match ctrl_dev.send_feature_report(&buf) {
            //     Ok(_result) => {
            //         #[cfg(debug_assertions)]
            //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));
            //
            //         Ok(())
            //     }
            //
            //     Err(_) => Err(HwDeviceError::InvalidResult {}),
            // }?;
            //
            // Ok(())

            Err(HwDeviceError::OpNotSupported {}.into())
        }
    }

    fn get_dcu_config(&self) -> Result<i32> {
        trace!("Querying device DCU config");

        Err(HwDeviceError::OpNotSupported {}.into())
    }

    fn set_dcu_config(&mut self, _dcu: i32) -> Result<()> {
        trace!("Setting device DCU config");

        Err(HwDeviceError::OpNotSupported {}.into())
    }

    fn get_angle_snapping(&self) -> Result<bool> {
        trace!("Querying device angle-snapping config");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            // let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            // let ctrl_dev = ctrl_dev.as_ref().unwrap();
            //
            // let mut buf: [u8; 64] = [0x00_u8; 64];
            // buf[0] = 0x11;
            //
            // match ctrl_dev.get_feature_report(&mut buf) {
            //     Ok(_result) => {
            //         #[cfg(debug_assertions)]
            //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));
            //
            //         Ok(())
            //     }
            //
            //     Err(_) => Err(HwDeviceError::InvalidResult {}),
            // }?;
            //
            // if buf[18] == 0x00 {
            //     Ok(false)
            // } else {
            //     Ok(true)
            // }

            Err(HwDeviceError::OpNotSupported {}.into())
        }
    }

    fn set_angle_snapping(&mut self, _angle_snapping: bool) -> Result<()> {
        trace!("Setting device angle-snapping config");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            // let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            // let ctrl_dev = ctrl_dev.as_ref().unwrap();
            //
            // let mut buf: [u8; 64] = [0x00_u8; 64];
            // buf[0] = 0x11;
            //
            // match ctrl_dev.get_feature_report(&mut buf) {
            //     Ok(_result) => {
            //         #[cfg(debug_assertions)]
            //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));
            //
            //         Ok(())
            //     }
            //
            //     Err(_) => Err(HwDeviceError::InvalidResult {}),
            // }?;
            //
            // buf[18] = if angle_snapping { 0x01 } else { 0x00 };
            //
            // match ctrl_dev.send_feature_report(&buf) {
            //     Ok(_result) => {
            //         #[cfg(debug_assertions)]
            //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));
            //
            //         Ok(())
            //     }
            //
            //     Err(_) => Err(HwDeviceError::InvalidResult {}),
            // }?;
            //
            // Ok(())

            Err(HwDeviceError::OpNotSupported {}.into())
        }
    }

    fn get_debounce(&self) -> Result<bool> {
        trace!("Querying device debounce config");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            // let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            // let ctrl_dev = ctrl_dev.as_ref().unwrap();
            //
            // let mut buf: [u8; 64] = [0x00_u8; 64];
            // buf[0] = 0x11;
            //
            // match ctrl_dev.get_feature_report(&mut buf) {
            //     Ok(_result) => {
            //         #[cfg(debug_assertions)]
            //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));
            //
            //         Ok(())
            //     }
            //
            //     Err(_) => Err(HwDeviceError::InvalidResult {}),
            // }?;
            //
            // // inverted logic here, if the zero-debounce feature is disabled then debounce is on
            // if buf[2] == 0x00 {
            //     Ok(true)
            // } else {
            //     Ok(false)
            // }

            Err(HwDeviceError::OpNotSupported {}.into())
        }
    }

    fn set_debounce(&mut self, _debounce: bool) -> Result<()> {
        trace!("Setting device debounce config");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            // let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            // let ctrl_dev = ctrl_dev.as_ref().unwrap();
            //
            // let mut buf: [u8; 64] = [0x00_u8; 64];
            // buf[0] = 0x11;
            //
            // match ctrl_dev.get_feature_report(&mut buf) {
            //     Ok(_result) => {
            //         #[cfg(debug_assertions)]
            //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));
            //
            //         Ok(())
            //     }
            //
            //     Err(_) => Err(HwDeviceError::InvalidResult {}),
            // }?;
            //
            // // inverted logic here, if debounce is true then we have to disable the zero-debounce feature
            // buf[2] = if debounce { 0x00 } else { 0x01 };
            //
            // match ctrl_dev.send_feature_report(&buf) {
            //     Ok(_result) => {
            //         #[cfg(debug_assertions)]
            //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));
            //
            //         Ok(())
            //     }
            //
            //     Err(_) => Err(HwDeviceError::InvalidResult {}),
            // }?;
            //
            // Ok(())

            Err(HwDeviceError::OpNotSupported {}.into())
        }
    }

    #[inline]
    fn get_next_event(&self) -> Result<MouseHidEvent> {
        self.get_next_event_timeout(-1)
    }

    fn get_next_event_timeout(&self, millis: i32) -> Result<MouseHidEvent> {
        trace!("Querying control device for next event");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else if !self.is_initialized {
            Err(HwDeviceError::DeviceNotInitialized {}.into())
        } else {
            let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            let mut buf = [0; 8];

            match ctrl_dev.read_timeout(&mut buf, millis) {
                Ok(size) => {
                    #[cfg(debug_assertions)]
                    hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                    let event = match buf[0..5] {
                        // Button reports (DPI)
                        [0x03, 0x00, 0xb0, level, _] => MouseHidEvent::DpiChange(level),

                        // Button reports
                        [button_mask, 0x00, button_mask2, 0x00, _] if size > 0 => {
                            let mut result = vec![];

                            let button_mask = button_mask.view_bits::<Lsb0>();
                            let button_mask2 = button_mask2.view_bits::<Lsb0>();

                            let mut button_states = self.button_states.lock();

                            // notify button press events for the buttons 0..7
                            for (index, down) in button_mask.iter().enumerate() {
                                if *down && !*button_states.get(index).unwrap() {
                                    result.push(MouseHidEvent::ButtonDown(index as u8));
                                    button_states.set(index, *down);

                                    break;
                                }
                            }

                            // notify button press events for the buttons 8..15
                            for (index, down) in button_mask2.iter().enumerate() {
                                let index = index + 8; // offset by 8

                                if *down && !*button_states.get(index).unwrap() {
                                    result.push(MouseHidEvent::ButtonDown(index as u8));
                                    button_states.set(index, *down);

                                    break;
                                }
                            }

                            // notify button release events for the buttons 0..7
                            for (index, down) in button_mask.iter().enumerate() {
                                if !*down && *button_states.get(index).unwrap() {
                                    result.push(MouseHidEvent::ButtonUp(index as u8));
                                    button_states.set(index, *down);

                                    break;
                                }
                            }

                            // notify button release events for the buttons 8..15
                            for (index, down) in button_mask2.iter().enumerate() {
                                let index = index + 8; // offset by 8

                                if !*down && *button_states.get(index).unwrap() {
                                    result.push(MouseHidEvent::ButtonUp(index as u8));
                                    button_states.set(index, *down);

                                    break;
                                }
                            }

                            if result.len() > 1 {
                                error!(
                                    "We missed a HID event, mouse button states will be inconsistent"
                                );
                            }

                            if result.is_empty() {
                                MouseHidEvent::Unknown
                            } else {
                                debug!("{:?}", result[0]);
                                result[0]
                            }
                        }

                        _ => MouseHidEvent::Unknown,
                    };

                    Ok(event)
                }

                Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
            }
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn ev_key_to_button_index(&self, code: EV_KEY) -> Result<u8> {
        match code {
            EV_KEY::KEY_RESERVED => Ok(0),

            EV_KEY::BTN_LEFT => Ok(1),
            EV_KEY::BTN_MIDDLE => Ok(2),
            EV_KEY::BTN_RIGHT => Ok(3),

            EV_KEY::BTN_0 => Ok(4),
            EV_KEY::BTN_1 => Ok(5),
            EV_KEY::BTN_2 => Ok(6),
            EV_KEY::BTN_3 => Ok(7),
            EV_KEY::BTN_4 => Ok(8),
            EV_KEY::BTN_5 => Ok(9),
            EV_KEY::BTN_6 => Ok(10),
            EV_KEY::BTN_7 => Ok(11),
            EV_KEY::BTN_8 => Ok(12),
            EV_KEY::BTN_9 => Ok(13),

            EV_KEY::BTN_EXTRA => Ok(14),
            EV_KEY::BTN_SIDE => Ok(15),
            EV_KEY::BTN_FORWARD => Ok(16),
            EV_KEY::BTN_BACK => Ok(17),
            EV_KEY::BTN_TASK => Ok(18),

            EV_KEY::KEY_0 => Ok(19),
            EV_KEY::KEY_1 => Ok(20),
            EV_KEY::KEY_2 => Ok(21),
            EV_KEY::KEY_3 => Ok(22),
            EV_KEY::KEY_4 => Ok(23),
            EV_KEY::KEY_5 => Ok(24),
            EV_KEY::KEY_6 => Ok(25),
            EV_KEY::KEY_7 => Ok(26),
            EV_KEY::KEY_8 => Ok(27),
            EV_KEY::KEY_9 => Ok(28),

            EV_KEY::KEY_MINUS => Ok(29),
            EV_KEY::KEY_EQUAL => Ok(30),

            _ => Err(HwDeviceError::MappingError {}.into()),
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn button_index_to_ev_key(&self, index: u32) -> Result<EV_KEY> {
        match index {
            0 => Ok(EV_KEY::KEY_RESERVED),

            1 => Ok(EV_KEY::BTN_LEFT),
            2 => Ok(EV_KEY::BTN_MIDDLE),
            3 => Ok(EV_KEY::BTN_RIGHT),

            4 => Ok(EV_KEY::BTN_0),
            5 => Ok(EV_KEY::BTN_1),
            6 => Ok(EV_KEY::BTN_2),
            7 => Ok(EV_KEY::BTN_3),
            8 => Ok(EV_KEY::BTN_4),
            9 => Ok(EV_KEY::BTN_5),
            10 => Ok(EV_KEY::BTN_6),
            11 => Ok(EV_KEY::BTN_7),
            12 => Ok(EV_KEY::BTN_8),
            13 => Ok(EV_KEY::BTN_9),

            14 => Ok(EV_KEY::BTN_EXTRA),
            15 => Ok(EV_KEY::BTN_SIDE),
            16 => Ok(EV_KEY::BTN_FORWARD),
            17 => Ok(EV_KEY::BTN_BACK),
            18 => Ok(EV_KEY::BTN_TASK),

            19 => Ok(EV_KEY::KEY_0),
            20 => Ok(EV_KEY::KEY_1),
            21 => Ok(EV_KEY::KEY_2),
            22 => Ok(EV_KEY::KEY_3),
            23 => Ok(EV_KEY::KEY_4),
            24 => Ok(EV_KEY::KEY_5),
            25 => Ok(EV_KEY::KEY_6),
            26 => Ok(EV_KEY::KEY_7),
            27 => Ok(EV_KEY::KEY_8),
            28 => Ok(EV_KEY::KEY_9),

            29 => Ok(EV_KEY::KEY_MINUS),
            30 => Ok(EV_KEY::KEY_EQUAL),

            _ => Err(HwDeviceError::MappingError {}.into()),
        }
    }
}
