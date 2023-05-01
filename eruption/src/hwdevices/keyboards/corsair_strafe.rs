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

use bitvec::{field::BitField, order::Lsb0, view::BitView};
use evdev_rs::enums::EV_KEY;
use hidapi::HidApi;
use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;
use std::{any::Any, time::Duration};
use std::{sync::Arc, thread};
use tracing::*;

use crate::{constants, hwdevices};

use crate::hwdevices::{
    Capability, DeviceCapabilities, DeviceClass, DeviceInfoTrait, DeviceStatus, DeviceTrait,
    DeviceZoneAllocationTrait, HwDeviceError, KeyboardDevice, KeyboardDeviceTrait,
    KeyboardHidEvent, KeyboardHidEventCode, LedKind, MouseDeviceTrait, Result, Zone, RGBA,
};

pub const NUM_KEYS: usize = 143;

pub const NUM_ROWS: usize = 6;
pub const NUM_COLS: usize = 21;
#[allow(unused)]
pub const NUM_LEDS: usize = NUM_ROWS * NUM_COLS;

// pub const CTRL_INTERFACE: i32 = 2; // Control USB sub device
pub const LED_INTERFACE: i32 = 1; // LED USB sub device

/// Binds the driver to a device
pub fn bind_hiddev(
    hidapi: &HidApi,
    usb_vid: u16,
    usb_pid: u16,
    serial: &str,
) -> Result<KeyboardDevice> {
    // let ctrl_dev = hidapi.device_list().find(|&device| {
    //     device.vendor_id() == usb_vid
    //         && device.product_id() == usb_pid
    //         && device.serial_number().unwrap_or_else(|| "") == serial
    //         && device.interface_number() == CTRL_INTERFACE
    // });

    let led_dev = hidapi.device_list().find(|&device| {
        device.vendor_id() == usb_vid
            && device.product_id() == usb_pid
            && device.serial_number().unwrap_or("") == serial
            && device.interface_number() == LED_INTERFACE
    });

    if
    /* ctrl_dev.is_none() || */
    led_dev.is_none() {
        Err(HwDeviceError::EnumerationError {}.into())
    } else {
        Ok(Arc::new(RwLock::new(Box::new(CorsairStrafe::bind(
            // &ctrl_dev.unwrap(),
            led_dev.unwrap(),
        )))))
    }
}

/// Corsair STRAFE device info struct (sent as HID report)
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

#[derive(Debug, PartialEq, Eq)]
pub enum DialMode {
    // Volume,
    // Brightness,
}

#[derive(Clone)]
/// Device specific code for the Corsair STRAFE series keyboards
pub struct CorsairStrafe {
    pub is_initialized: bool,

    // keyboard
    pub is_bound: bool,
    // pub ctrl_hiddev_info: Option<hidapi::DeviceInfo>,
    pub led_hiddev_info: Option<hidapi::DeviceInfo>,

    pub is_opened: bool,
    // pub ctrl_hiddev: Arc<Mutex<Option<hidapi::HidDevice>>>,
    pub led_hiddev: Arc<Mutex<Option<hidapi::HidDevice>>>,

    pub has_failed: bool,

    pub allocated_zone: Zone,

    // device specific configuration options
    pub brightness: i32,
}

impl CorsairStrafe {
    /// Binds the driver to the supplied HID devices
    pub fn bind(/* ctrl_dev: &hidapi::DeviceInfo, */ led_dev: &hidapi::DeviceInfo) -> Self {
        info!("Bound driver: Corsair STRAFE Gaming Keyboard");

        Self {
            is_initialized: false,

            is_bound: true,
            // ctrl_hiddev_info: Some(ctrl_dev.clone()),
            led_hiddev_info: Some(led_dev.clone()),

            is_opened: false,
            // ctrl_hiddev: Arc::new(Mutex::new(None)),
            led_hiddev: Arc::new(Mutex::new(None)),

            has_failed: false,

            allocated_zone: Zone::defaults_for(DeviceClass::Keyboard),

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

    // fn send_ctrl_report(&mut self, id: u8) -> Result<()> {
    //     trace!("Sending control device feature report");

    //     if !self.is_bound {
    //         Err(HwDeviceError::DeviceNotBound {}.into())
    //     } else if !self.is_opened {
    //         Err(HwDeviceError::DeviceNotOpened {}.into())
    //     } else {
    //         let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
    //         let ctrl_dev = ctrl_dev.as_ref().unwrap();

    //         match id {
    //             0x01 => {
    //                 let buf: [u8; 0] = [];

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

    fn send_led_report(&mut self, id: u8) -> Result<()> {
        trace!("Sending LED device feature report");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            let led_dev = self.led_hiddev.as_ref().lock();
            let led_dev = led_dev.as_ref().unwrap();

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

                    match led_dev.send_feature_report(&buf) {
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

                    match led_dev.send_feature_report(&buf) {
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

                    match led_dev.send_feature_report(&buf) {
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

                    match led_dev.send_feature_report(&buf) {
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

    // fn wait_for_ctrl_dev(&mut self) -> Result<()> {
    //     trace!("Waiting for control device to respond...");

    //     if !self.is_bound {
    //         Err(HwDeviceError::DeviceNotBound {}.into())
    //     } else if !self.is_opened {
    //         Err(HwDeviceError::DeviceNotOpened {}.into())
    //     } else {
    //         thread::sleep(Duration::from_millis(70));

    //         Ok(())
    //     }
    // }

    fn wait_for_led_dev(&mut self) -> Result<()> {
        trace!("Waiting for LED device to respond...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            thread::sleep(Duration::from_millis(20));

            Ok(())
        }
    }
}

impl DeviceInfoTrait for CorsairStrafe {
    fn get_device_capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities::from([Capability::Keyboard])
    }

    fn get_device_info(&self) -> Result<hwdevices::DeviceInfo> {
        trace!("Querying the device for information...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            // let mut buf = [0; size_of::<DeviceInfo>()];
            // buf[0] = 0x0f; // Query device info (HID report 0x0f)

            // let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            // let ctrl_dev = ctrl_dev.as_ref().unwrap();

            // match ctrl_dev.get_feature_report(&mut buf) {
            //     Ok(_result) => {
            //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));
            //         let tmp: DeviceInfo =
            //             unsafe { std::ptr::read_unaligned(buf.as_ptr() as *const _) };

            //         let result = hwdevices::DeviceInfo::new(tmp.firmware_version as i32);
            //         Ok(result)
            //     }

            //     Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
            // }

            let result = hwdevices::DeviceInfo::new(0x00);
            Ok(result)
        }
    }

    fn get_firmware_revision(&self) -> String {
        if let Ok(device_info) = self.get_device_info() {
            format!("{}", device_info.firmware_version)
        } else {
            "<unknown>".to_string()
        }
    }
}

impl DeviceZoneAllocationTrait for CorsairStrafe {
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

impl DeviceTrait for CorsairStrafe {
    fn get_usb_path(&self) -> String {
        self.led_hiddev_info
            .clone()
            .unwrap()
            .path()
            .to_str()
            .unwrap()
            .to_string()
    }

    fn get_usb_vid(&self) -> u16 {
        // self.ctrl_hiddev_info.as_ref().unwrap().vendor_id()

        self.led_hiddev_info.as_ref().unwrap().vendor_id()
    }

    fn get_usb_pid(&self) -> u16 {
        // self.ctrl_hiddev_info.as_ref().unwrap().product_id()

        self.led_hiddev_info.as_ref().unwrap().product_id()
    }

    fn get_serial(&self) -> Option<&str> {
        self.led_hiddev_info.as_ref().unwrap().serial_number()
    }

    fn get_support_script_file(&self) -> String {
        "keyboards/corsair_strafe".to_string()
    }

    fn open(&mut self, api: &hidapi::HidApi) -> Result<()> {
        trace!("Opening HID devices now...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            // trace!("Opening control device...");

            // match self.ctrl_hiddev_info.as_ref().unwrap().open_device(&api) {
            //     Ok(dev) => *self.ctrl_hiddev.lock() = Some(dev),
            //     Err(_) => return Err(HwDeviceError::DeviceOpenError {}.into()),
            // };

            trace!("Opening LED device...");

            match self.led_hiddev_info.as_ref().unwrap().open_device(api) {
                Ok(dev) => *self.led_hiddev.lock() = Some(dev),
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
            // trace!("Closing control device...");
            // *self.ctrl_hiddev.lock() = None;

            trace!("Closing LED device...");
            *self.led_hiddev.lock() = None;

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
            self.send_led_report(0x01)
                .unwrap_or_else(|e| error!("Step 1: {}", e));
            self.wait_for_led_dev()
                .unwrap_or_else(|e| error!("Wait 1: {}", e));

            self.send_led_report(0x02)
                .unwrap_or_else(|e| error!("Step 2: {}", e));
            self.wait_for_led_dev()
                .unwrap_or_else(|e| error!("Wait 2: {}", e));

            self.send_led_report(0x03)
                .unwrap_or_else(|e| error!("Step 3: {}", e));
            self.wait_for_led_dev()
                .unwrap_or_else(|e| error!("Wait 3: {}", e));

            self.send_led_report(0x04)
                .unwrap_or_else(|e| error!("Step 4: {}", e));
            self.wait_for_led_dev()
                .unwrap_or_else(|e| error!("Wait 4: {}", e));

            self.is_initialized = true;

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

    fn write_data_raw(&self, _buf: &[u8]) -> Result<()> {
        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else if !self.is_initialized {
            Err(HwDeviceError::DeviceNotInitialized {}.into())
        } else {
            // let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            // let ctrl_dev = ctrl_dev.as_ref().unwrap();

            // match ctrl_dev.write(&buf) {
            //     Ok(_result) => {
            //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

            //         Ok(())
            //     }

            //     Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
            // }

            Err(HwDeviceError::InvalidResult {}.into())
        }
    }

    fn read_data_raw(&self, _size: usize) -> Result<Vec<u8>> {
        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else if !self.is_initialized {
            Err(HwDeviceError::DeviceNotInitialized {}.into())
        } else {
            // let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            // let ctrl_dev = ctrl_dev.as_ref().unwrap();

            // let mut buf = Vec::new();
            // buf.resize(size, 0);

            // match ctrl_dev.read(buf.as_mut_slice()) {
            //     Ok(_result) => {
            //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

            //         Ok(buf)
            //     }

            //     Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
            // }

            Err(HwDeviceError::InvalidResult {}.into())
        }
    }

    fn device_status(&self) -> Result<DeviceStatus> {
        let mut table = HashMap::new();

        table.insert("connected".to_owned(), format!("{}", true));

        Ok(DeviceStatus(table))
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }

    fn as_device(&self) -> &dyn DeviceTrait {
        self
    }

    fn as_device_mut(&mut self) -> &mut dyn DeviceTrait {
        self
    }

    fn as_mouse_device(&self) -> Option<&dyn MouseDeviceTrait> {
        None
    }

    fn as_mouse_device_mut(&mut self) -> Option<&mut dyn MouseDeviceTrait> {
        None
    }
}

impl KeyboardDeviceTrait for CorsairStrafe {
    fn set_status_led(&self, led_kind: LedKind, _on: bool) -> Result<()> {
        trace!("Setting status LED state");

        match led_kind {
            LedKind::Unknown => warn!("No LEDs have been set, request was a no-op"),
            LedKind::AudioMute => {
                // self.write_data_raw(&[0x00, 0x09, 0x22, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])?;
            }
            LedKind::Fx => {}
            LedKind::Volume => {}
            LedKind::NumLock => {
                // self.write_data_raw(&[0x21, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])?;
            }
            LedKind::CapsLock => {
                // self.write_data_raw(&[0x22, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])?;
            }
            LedKind::ScrollLock => {
                // self.write_data_raw(&[0x23, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])?;
            }
            LedKind::GameMode => {
                // self.write_data_raw(&[0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])?;
            }
        }

        Ok(())
    }

    fn set_local_brightness(&mut self, _brightness: i32) -> Result<()> {
        trace!("Setting device specific brightness");

        Err(HwDeviceError::OpNotSupported {}.into())
    }

    fn get_local_brightness(&self) -> Result<i32> {
        trace!("Querying device specific brightness");

        Err(HwDeviceError::OpNotSupported {}.into())
    }

    #[inline]
    fn get_next_event(&self) -> Result<KeyboardHidEvent> {
        self.get_next_event_timeout(-1)
    }

    fn get_next_event_timeout(&self, millis: i32) -> Result<KeyboardHidEvent> {
        trace!("Querying control device for next event");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else if !self.is_initialized {
            Err(HwDeviceError::DeviceNotInitialized {}.into())
        } else {
            // let ctrl_dev = self.ctrl_hiddev.as_ref().lock();

            // TODO: Implement this
            let ctrl_dev = self.led_hiddev.as_ref().lock();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            let mut buf = [0; 8];

            match ctrl_dev.read_timeout(&mut buf, millis) {
                Ok(_size) => {
                    if buf.iter().any(|e| *e != 0) {
                        hexdump::hexdump_iter(&buf).for_each(|s| debug!("  {}", s));
                    }

                    // let event = match buf[0..5] {
                    //     // Key reports, incl. KEY_FN, ..
                    //     [0x03, 0x00, 0xfb, code, status] => match status {
                    //         0x00 => KeyboardHidEvent::KeyUp {
                    //             code: keyboard_hid_event_code_from_report(0xfb, code),
                    //         },

                    //         0x01 => KeyboardHidEvent::KeyDown {
                    //             code: keyboard_hid_event_code_from_report(0xfb, code),
                    //         },

                    //         _ => KeyboardHidEvent::Unknown,
                    //     },

                    //     // CAPS LOCK, Easy Shift+, ..
                    //     [0x03, 0x00, 0x0a, code, status] => match code {
                    //         0x39 | 0xff => match status {
                    //             0x00 => KeyboardHidEvent::KeyDown {
                    //                 code: keyboard_hid_event_code_from_report(0x0a, code),
                    //             },

                    //             0x01 => KeyboardHidEvent::KeyUp {
                    //                 code: keyboard_hid_event_code_from_report(0x0a, code),
                    //             },

                    //             _ => KeyboardHidEvent::Unknown,
                    //         },

                    //         _ => KeyboardHidEvent::Unknown,
                    //     },

                    //     // volume up/down adjustment is initiated by the following sequence
                    //     [0x03, 0x00, 0x0b, 0x26, _] => {
                    //         *self.dial_mode.lock() = DialMode::Volume;
                    //         KeyboardHidEvent::Unknown
                    //     }
                    //     [0x03, 0x00, 0x0b, 0x27, _] => {
                    //         *self.dial_mode.lock() = DialMode::Volume;
                    //         KeyboardHidEvent::Unknown
                    //     }

                    //     [0x03, 0x00, 0xcc, code, _] => {
                    //         let result = if *self.dial_mode.lock() == DialMode::Volume {
                    //             match code {
                    //                 0x01 => KeyboardHidEvent::VolumeUp,
                    //                 0xff => KeyboardHidEvent::VolumeDown,

                    //                 _ => KeyboardHidEvent::Unknown,
                    //             }
                    //         } else {
                    //             match code {
                    //                 0x01 => KeyboardHidEvent::BrightnessUp,
                    //                 0xff => KeyboardHidEvent::BrightnessDown,

                    //                 _ => KeyboardHidEvent::Unknown,
                    //             }
                    //         };

                    //         // default to brightness
                    //         *self.dial_mode.lock() = DialMode::Brightness;

                    //         result
                    //     }

                    //     [0x03, 0x00, 0x0c, val, _] => KeyboardHidEvent::SetBrightness(val),

                    //     [0x02, 0xe2, 0x00, 0x00, _] => KeyboardHidEvent::MuteDown,
                    //     [0x02, 0x00, 0x00, 0x00, _] => KeyboardHidEvent::MuteUp,

                    //     _ => KeyboardHidEvent::Unknown,
                    // };

                    /* match event {
                        KeyboardHidEvent::KeyDown { code } => {
                            // update our internal representation of the keyboard state
                            let index = self.hid_event_code_to_key_index(&code) as usize;
                            crate::KEY_STATES.write()[index] = true;
                        }

                        KeyboardHidEvent::KeyUp { code } => {
                            // update our internal representation of the keyboard state
                            let index = self.hid_event_code_to_key_index(&code) as usize;
                            crate::KEY_STATES.write()[index] = false;
                        }

                        _ => { /* ignore other events */ }
                    }

                    Ok(event) */

                    Ok(KeyboardHidEvent::Unknown)
                }

                Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
            }
        }
    }

    fn ev_key_to_key_index(&self, key: EV_KEY) -> u8 {
        EV_TO_INDEX_ISO[(key as u8) as usize].saturating_add(1)
    }

    fn hid_event_code_to_key_index(&self, code: &KeyboardHidEventCode) -> u8 {
        match code {
            KeyboardHidEventCode::KEY_FN => 77,

            KeyboardHidEventCode::KEY_CAPS_LOCK => 4,
            KeyboardHidEventCode::KEY_EASY_SHIFT => 4,

            // We don't need all the other key codes, for now
            _ => 0,
        }
    }

    fn hid_event_code_to_report(&self, code: &KeyboardHidEventCode) -> u8 {
        match code {
            KeyboardHidEventCode::KEY_F1 => 16,
            KeyboardHidEventCode::KEY_F2 => 24,
            KeyboardHidEventCode::KEY_F3 => 33,
            KeyboardHidEventCode::KEY_F4 => 32,

            KeyboardHidEventCode::KEY_F5 => 40,
            KeyboardHidEventCode::KEY_F6 => 48,
            KeyboardHidEventCode::KEY_F7 => 56,
            KeyboardHidEventCode::KEY_F8 => 57,

            KeyboardHidEventCode::KEY_ESC => 17,
            KeyboardHidEventCode::KEY_FN => 119,

            KeyboardHidEventCode::KEY_CAPS_LOCK => 57,
            KeyboardHidEventCode::KEY_EASY_SHIFT => 57,

            KeyboardHidEventCode::Unknown(code) => *code,
        }
    }

    fn send_led_map(&mut self, led_map: &[RGBA]) -> Result<()> {
        trace!("Setting LEDs from supplied map...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else if !self.is_initialized {
            Err(HwDeviceError::DeviceNotInitialized {}.into())
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
                        #[inline]
                        fn index_to_canvas(index: usize) -> usize {
                            let index = ROWS_TOPOLOGY
                                .iter()
                                .position(|e| *e as usize == index)
                                .unwrap_or(0);

                            let x = index % NUM_COLS;
                            let y = index / NUM_COLS;

                            let scale_x = 1; // constants::CANVAS_WIDTH / NUM_COLS;
                            let scale_y = 1; // constants::CANVAS_HEIGHT / NUM_ROWS;

                            // let x = index % NUM_COLS + (NUM_COLS / 2);
                            // let y = index / NUM_COLS + (NUM_ROWS / 2);

                            let result = (constants::CANVAS_WIDTH * y * scale_y) + (x * scale_x);

                            result.clamp(0, constants::CANVAS_SIZE - 1)
                        }

                        // build and send data buffer chunks
                        let mut buffer: [u8; NUM_KEYS * 3] = [0x00; NUM_KEYS * 3];

                        // convert RGB color to monochromatic value
                        // let color = 255
                        //     - (((led_map[i].r as f32 * 0.29)
                        //         + (led_map[i].g as f32 * 0.59)
                        //         + (led_map[i].b as f32 * 0.114))
                        //         .round() as u8)
                        //         .clamp(0, 255);

                        let bitvec = buffer.view_bits_mut::<Lsb0>();

                        for i in 0..NUM_KEYS {
                            let offset = i * 3;
                            let color = led_map[index_to_canvas(i)];
                            bitvec[offset..(offset + 3)].store(255_u8 - color.r);
                        }

                        for i in 0..NUM_KEYS {
                            let offset = i * 3 + (NUM_KEYS * 3);
                            let color = led_map[index_to_canvas(i)];
                            bitvec[offset..(offset + 3)].store(255_u8 - color.g);
                        }

                        for i in 0..NUM_KEYS {
                            let offset = i * 3 + (NUM_KEYS * 6);
                            let color = led_map[index_to_canvas(i)];
                            bitvec[offset..(offset + 3)].store(255_u8 - color.b);
                        }

                        for (cntr, bytes) in buffer.chunks(60).take(4).enumerate() {
                            let mut tmp: [u8; 64] = [0; 64];

                            if cntr < 3 {
                                tmp[0..4].copy_from_slice(&[0x7f, cntr as u8 + 1, 0x3c, 00]);
                            } else {
                                tmp[0..4].copy_from_slice(&[0x7f, cntr as u8 + 1, 0x30, 00]);
                            }

                            tmp[4..64].copy_from_slice(bytes);

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

                            Err(_) => {
                                // the device has failed or has been disconnected
                                self.is_initialized = false;
                                self.is_opened = false;
                                self.has_failed = true;

                                return Err(HwDeviceError::InvalidResult {}.into());
                            }
                        }

                        Ok(())
                    }
                }

                None => Err(HwDeviceError::DeviceNotOpened {}.into()),
            }
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

    /// Returns the number of keys
    fn get_num_keys(&self) -> usize {
        NUM_KEYS
    }

    /// Returns the number of rows (vertical number of keys)
    fn get_num_rows(&self) -> usize {
        NUM_ROWS
    }

    /// Returns the number of columns (horizontal number of keys)
    fn get_num_cols(&self) -> usize {
        NUM_COLS
    }

    /// Returns the indices of the keys in row `row`
    fn get_row_topology(&self, row: usize) -> &'static [u8] {
        let idx = row * NUM_COLS;
        &ROWS_TOPOLOGY[idx..(idx + NUM_COLS + 1)]
    }

    /// Returns the indices of the keys in column `col`
    fn get_col_topology(&self, col: usize) -> &'static [u8] {
        let idx = col * NUM_ROWS;
        &COLS_TOPOLOGY[idx..(idx + NUM_ROWS + 1)]
    }
}

// fn keyboard_hid_event_code_from_report(report: u8, code: u8) -> KeyboardHidEventCode {
//     match report {
//         0xfb => match code {
//             16 => KeyboardHidEventCode::KEY_F1,
//             24 => KeyboardHidEventCode::KEY_F2,
//             33 => KeyboardHidEventCode::KEY_F3,
//             32 => KeyboardHidEventCode::KEY_F4,

//             40 => KeyboardHidEventCode::KEY_F5,
//             48 => KeyboardHidEventCode::KEY_F6,
//             56 => KeyboardHidEventCode::KEY_F7,
//             57 => KeyboardHidEventCode::KEY_F8,

//             17 => KeyboardHidEventCode::KEY_ESC,
//             119 => KeyboardHidEventCode::KEY_FN,

//             _ => KeyboardHidEventCode::Unknown(code),
//         },

//         0x0a => match code {
//             57 => KeyboardHidEventCode::KEY_CAPS_LOCK,
//             255 => KeyboardHidEventCode::KEY_EASY_SHIFT,

//             _ => KeyboardHidEventCode::Unknown(code),
//         },

//         _ => KeyboardHidEventCode::Unknown(code),
//     }
// }

/// Map evdev event codes to key indices, for ISO variant
#[rustfmt::skip]
const EV_TO_INDEX_ISO: [u8; 0x2ff + 1] = [
    0xff, 0x00, 0x06, 0x0c, 0x12, 0x18, 0x1d, 0x21, 0x31, 0x36, 0x3c, 0x42, 0x48, 0x4f, 0x57,
    0x02, // 0x000
    0x07, 0x0d, 0x13, 0x19, 0x1e, 0x22, 0x32, 0x37, 0x3d, 0x43, 0x49, 0x50, 0x58, 0x05, 0x08,
    0x0e, // 0x010
    0x14, 0x1a, 0x1f, 0x23, 0x33, 0x38, 0x3e, 0x44, 0x4a, 0x01, 0x04, 0x60, 0x0f, 0x15, 0x1b,
    0x20, // 0x020
    0x24, 0x34, 0x39, 0x3f, 0x45, 0x4b, 0x52, 0x7c, 0x10, 0x25, 0x03, 0x0b, 0x11, 0x17, 0x1c,
    0x30, // 0x030
    0x35, 0x3b, 0x41, 0x4e, 0x54, 0x71, 0x67, 0x72, 0x78, 0x7d, 0x81, 0x73, 0x79, 0x7e, 0x82,
    0x74, // 0x040
    0x7a, 0x7f, 0x75, 0x80, 0xff, 0xff, 0x09, 0x55, 0x56, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x050
    0x83, 0x59, 0x77, 0x63, 0x46, 0xff, 0x68, 0x6a, 0x6d, 0x66, 0x6f, 0x69, 0x6b, 0x6e, 0x64,
    0x65, // 0x060
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x6c, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0a, 0xff,
    0x53, // 0x070
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x080
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x090
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x0a0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x0b0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x0c0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x0d0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x0e0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x0f0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x100
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x110
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x120
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x130
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x140
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x150
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x160
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x170
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x180
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x190
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x1a0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x1b0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x1c0
    0x4c, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x1d0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x1e0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x1f0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x200
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x210
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x220
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x230
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x240
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x250
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x260
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x270
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x280
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x290
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x2a0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x2b0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x2c0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x2d0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x2e0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x2f0
];

/// Map evdev event codes to key indices, for ANSI variant
#[rustfmt::skip]
const _EV_TO_INDEX_ANSI: [u8; 0x2ff + 1] = [
    0xff, 0x00, 0x06, 0x0c, 0x12, 0x18, 0x1d, 0x21, 0x31, 0x36, 0x3c, 0x42, 0x48, 0x4f, 0x57,
    0x02, // 0x000
    0x07, 0x0d, 0x13, 0x19, 0x1e, 0x22, 0x32, 0x37, 0x3d, 0x43, 0x49, 0x50, 0x58, 0x05, 0x08,
    0x0e, // 0x010
    0x14, 0x1a, 0x1f, 0x23, 0x33, 0x38, 0x3e, 0x44, 0x4a, 0x01, 0x04, 0x51, 0x0f, 0x15, 0x1b,
    0x20, // 0x020
    0x24, 0x34, 0x39, 0x3f, 0x45, 0x4b, 0x52, 0x7c, 0x10, 0x25, 0x03, 0x0b, 0x11, 0x17, 0x1c,
    0x30, // 0x030
    0x35, 0x3b, 0x41, 0x4e, 0x54, 0x71, 0x67, 0x72, 0x78, 0x7d, 0x81, 0x73, 0x79, 0x7e, 0x82,
    0x74, // 0x040
    0x7a, 0x7f, 0x75, 0x80, 0xff, 0xff, 0xff, 0x55, 0x56, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x050
    0x83, 0x59, 0x77, 0x63, 0x46, 0xff, 0x68, 0x6a, 0x6d, 0x66, 0x6f, 0x69, 0x6b, 0x6e, 0x64,
    0x65, // 0x060
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x6c, 0xff, 0xff, 0xff, 0xff, 0xff, 0x0a, 0xff,
    0x53, // 0x070
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x080
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x090
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x0a0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x0b0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x0c0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x0d0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x0e0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x0f0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x100
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x110
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x120
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x130
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x140
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x150
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x160
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x170
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x180
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x190
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x1a0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x1b0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x1c0
    0x4c, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x1d0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x1e0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x1f0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x200
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x210
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x220
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x230
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x240
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x250
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x260
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x270
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x280
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x290
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x2a0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x2b0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x2c0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x2d0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x2e0
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, // 0x2f0
];

#[rustfmt::skip]
pub const ROWS_TOPOLOGY: [u8; 264] = [

	// ISO model
	0x00, 0x0b, 0x11, 0x17, 0x1c, 0x30, 0x35, 0x3b, 0x41, 0x4e, 0x54, 0x55, 0x56, 0x63, 0x67, 0x6c, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
	0x01, 0x06, 0x0c, 0x12, 0x18, 0x1d, 0x21, 0x31, 0x36, 0x3c, 0x42, 0x48, 0x4f, 0x57, 0x64, 0x68, 0x6d, 0x71, 0x77, 0x7c, 0x81, 0xff,
	0x02, 0x07, 0x0d, 0x13, 0x19, 0x1e, 0x22, 0x32, 0x37, 0x3d, 0x43, 0x49, 0x50, 0x58, 0x65, 0x69, 0x6e, 0x72, 0x78, 0x7d, 0x82, 0xff,
	0x03, 0x08, 0x0e, 0x14, 0x1a, 0x1f, 0x23, 0x33, 0x38, 0x3e, 0x44, 0x4a, 0x60, 0x73, 0x79, 0x7e, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
	0x04, 0x09, 0x0f, 0x15, 0x1b, 0x20, 0x24, 0x34, 0x39, 0x3f, 0x45, 0x4b, 0x52, 0x6a, 0x74, 0x7a, 0x7f, 0x83, 0xff, 0xff, 0xff, 0xff,
	0x05, 0x0a, 0x10, 0x25, 0x46, 0x4c, 0x53, 0x59, 0x66, 0x6b, 0x6f, 0x75, 0x80, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,

	// ANSI model
	0x00, 0x0b, 0x11, 0x17, 0x1c, 0x30, 0x35, 0x3b, 0x41, 0x4e, 0x54, 0x55, 0x56, 0x63, 0x67, 0x6c, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
	0x01, 0x06, 0x0c, 0x12, 0x18, 0x1d, 0x21, 0x31, 0x36, 0x3c, 0x42, 0x48, 0x4f, 0x57, 0x64, 0x68, 0x6d, 0x71, 0x77, 0x7c, 0x81, 0xff,
	0x02, 0x07, 0x0d, 0x13, 0x19, 0x1e, 0x22, 0x32, 0x37, 0x3d, 0x43, 0x49, 0x50, 0x51, 0x65, 0x69, 0x6e, 0x72, 0x78, 0x7d, 0x82, 0xff,
	0x03, 0x08, 0x0e, 0x14, 0x1a, 0x1f, 0x23, 0x33, 0x38, 0x3e, 0x44, 0x4a, 0x58, 0x73, 0x79, 0x7e, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
	0x04, 0x0f, 0x15, 0x1b, 0x20, 0x24, 0x34, 0x39, 0x3f, 0x45, 0x4b, 0x52, 0x6a, 0x74, 0x7a, 0x7f, 0x83, 0xff, 0xff, 0xff, 0xff, 0xff,
	0x05, 0x0a, 0x10, 0x25, 0x46, 0x4c, 0x53, 0x59, 0x66, 0x6b, 0x6f, 0x75, 0x80, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff

];

#[rustfmt::skip]
pub const COLS_TOPOLOGY: [u8; 252] = [

	// ISO model
	0x00, 0x01, 0x02, 0x03, 0x04, 0x05,
	0x06, 0x07, 0x08, 0x09, 0x0a, 0xff,
	0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
	0x11, 0x12, 0x13, 0x14, 0x15, 0xff,
	0x17, 0x18, 0x19, 0x1a, 0x1b, 0xff,
	0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0xff,
	0x21, 0x22, 0x23, 0x24, 0x25, 0x30,
	0x31, 0x32, 0x33, 0x34, 0x35, 0xff,
	0x36, 0x37, 0x38, 0x39, 0x3b, 0xff,
	0x3c, 0x3d, 0x3e, 0x3f, 0x41, 0xff,
	0x42, 0x43, 0x44, 0x45, 0x46, 0xff,
	0x48, 0x49, 0x4a, 0x4b, 0x4e, 0x4c,
	0x4f, 0x50, 0x53, 0x54, 0x60, 0xff,
	0x52, 0x55, 0x57, 0x58, 0x59, 0xff,
	0x56, 0x63, 0x64, 0x65, 0x66, 0xff,
	0x67, 0x68, 0x69, 0x6a, 0x6b, 0xff,
	0x6c, 0x6d, 0x6e, 0x6f, 0xff, 0xff,
	0x71, 0x72, 0x73, 0x74, 0x75, 0xff,
	0x77, 0x78, 0x79, 0x7a, 0xff, 0xff,
	0x7c, 0x7d, 0x7e, 0x7f, 0x80, 0xff,
	0x81, 0x82, 0x83, 0xff, 0xff, 0xff,

	// ANSI model
	0x00, 0x01, 0x02, 0x03, 0x04, 0x05,
	0x06, 0x07, 0x08, 0x0a, 0xff, 0xff,
	0x0b, 0x0c, 0x0d, 0x0e, 0x0f, 0x10,
	0x11, 0x12, 0x13, 0x14, 0x15, 0xff,
	0x17, 0x18, 0x19, 0x1a, 0x1b, 0xff,
	0x1c, 0x1d, 0x1e, 0x1f, 0x20, 0xff,
	0x21, 0x22, 0x23, 0x24, 0x25, 0x30,
	0x31, 0x32, 0x33, 0x34, 0x35, 0xff,
	0x36, 0x37, 0x38, 0x39, 0x3b, 0xff,
	0x3c, 0x3d, 0x3e, 0x3f, 0x41, 0xff,
	0x42, 0x43, 0x44, 0x45, 0x46, 0xff,
	0x48, 0x49, 0x4a, 0x4b, 0x4e, 0x4c,
	0x4f, 0x50, 0x53, 0x54, 0xff, 0xff,
	0x51, 0x52, 0x55, 0x57, 0x58, 0x59,
	0x56, 0x63, 0x64, 0x65, 0x66, 0xff,
	0x67, 0x68, 0x69, 0x6a, 0x6b, 0xff,
	0x6c, 0x6d, 0x6e, 0x6f, 0xff, 0xff,
	0x71, 0x72, 0x73, 0x74, 0x75, 0xff,
	0x77, 0x78, 0x79, 0x7a, 0xff, 0xff,
	0x7c, 0x7d, 0x7e, 0x7f, 0x80, 0xff,
	0x81, 0x82, 0x83, 0xff, 0xff, 0xff
];
