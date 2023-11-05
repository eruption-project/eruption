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
use byteorder::{BigEndian, ByteOrder};
#[cfg(not(target_os = "windows"))]
use evdev_rs::enums::EV_KEY;
use flume::Receiver;
use hidapi::HidApi;
use libc::wchar_t;
use tracing::*;
use tracing_mutex::stdsync::Mutex;
// use std::sync::atomic::Ordering;
use lazy_static::lazy_static;
use std::any::Any;
use std::collections::HashMap;
use std::thread;
use std::time::Duration;
use std::{mem::size_of, sync::Arc};

use crate::{constants, hwdevices, hwdevices::DeviceStatus};

use crate::hwdevices::{
    Capability, DeviceCapabilities, DeviceClass, DeviceExt, DeviceInfoExt, DeviceZoneAllocationExt,
    HwDeviceError, MouseDeviceExt, MouseHidEvent, Result, Zone, RGBA,
};

pub const CTRL_INTERFACE: i32 = 0; // Control USB sub device
pub const LED_INTERFACE: i32 = 2; // LED USB sub device

// pub const NUM_BUTTONS: usize = 9;

// canvas to LED index mapping
pub const LED_0: usize = constants::CANVAS_SIZE - 36;
pub const LED_1: usize = constants::CANVAS_SIZE - 1;
pub const NUM_LEDS: usize = 2;

lazy_static! {
    static ref CRC8: Arc<Mutex<crc8::Crc8>> = Arc::new(Mutex::new(crc8::Crc8::create_msb(0x01)));
}

/// Binds the driver to a device
pub fn bind_hiddev(
    hidapi: &HidApi,
    usb_vid: u16,
    usb_pid: u16,
    serial: &[wchar_t],
) -> Result<Box<dyn DeviceExt + Sync + Send>> {
    let ctrl_dev = hidapi.device_list().find(|&device| {
        device.vendor_id() == usb_vid
            && device.product_id() == usb_pid
            && device.serial_number_raw().unwrap_or(&[]) == serial
            && device.interface_number() == CTRL_INTERFACE
    });

    let led_dev = hidapi.device_list().find(|&device| {
        device.vendor_id() == usb_vid
            && device.product_id() == usb_pid
            && device.serial_number_raw().unwrap_or(&[]) == serial
            && device.interface_number() == LED_INTERFACE
    });

    if ctrl_dev.is_none() || led_dev.is_none() {
        Err(HwDeviceError::EnumerationError {}.into())
    } else {
        Ok(Box::new(RoccatKain2xx::bind(
            ctrl_dev.unwrap(),
            led_dev.unwrap(),
        )))
    }
}

/// ROCCAT Kain 2xx info struct (sent as HID report)
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
/// Device specific code for the ROCCAT Kain 2xx mouse
pub struct RoccatKain2xx {
    pub evdev_rx: Option<Receiver<Option<evdev_rs::InputEvent>>>,

    pub is_initialized: bool,

    pub is_bound: bool,
    pub ctrl_hiddev_info: Option<hidapi::DeviceInfo>,
    pub led_hiddev_info: Option<hidapi::DeviceInfo>,

    pub is_opened: bool,
    pub ctrl_hiddev: Arc<Mutex<Option<hidapi::HidDevice>>>,
    pub led_hiddev: Arc<Mutex<Option<hidapi::HidDevice>>>,

    pub button_states: Arc<Mutex<BitVec>>,

    pub has_failed: bool,

    pub allocated_zone: Zone,

    // device specific configuration options
    pub brightness: i32,
}

impl RoccatKain2xx {
    /// Binds the driver to the supplied HID device
    pub fn bind(ctrl_dev: &hidapi::DeviceInfo, led_dev: &hidapi::DeviceInfo) -> Self {
        debug!("Bound driver: ROCCAT Kain 2xx AIMO");

        Self {
            evdev_rx: None,

            is_initialized: false,

            is_bound: true,
            ctrl_hiddev_info: Some(ctrl_dev.clone()),
            led_hiddev_info: Some(led_dev.clone()),

            is_opened: false,
            ctrl_hiddev: Arc::new(Mutex::new(None)),
            led_hiddev: Arc::new(Mutex::new(None)),

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

    //                 let ctrl_dev = self.ctrl_hiddev.as_ref().lock().unwrap();
    //                 let ctrl_dev = ctrl_dev.as_ref().unwrap();

    //                 match ctrl_dev.get_feature_report(&mut buf) {
    //                     Ok(_result) => {
    //         #[cfg(debug_assertions)]
    //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

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
    //         let ctrl_dev = self.ctrl_hiddev.as_ref().lock().unwrap();
    //         let ctrl_dev = ctrl_dev.as_ref().unwrap();

    //         Ok(())
    //     }
    // }

    // fn wait_for_ctrl_dev(&mut self) -> Result<()> {
    //     trace!("Waiting for control device to respond...");

    //     if !self.is_bound {
    //         Err(HwDeviceError::DeviceNotBound {}.into())
    //     } else if !self.is_opened {
    //         Err(HwDeviceError::DeviceNotOpened {}.into())
    //     } else {
    //         loop {
    //             let mut buf: [u8; 2] = [0; 2];
    //             buf[0] = 0x00;

    //             let ctrl_dev = self.ctrl_hiddev.as_ref().lock().unwrap();
    //             let ctrl_dev = ctrl_dev.as_ref().unwrap();

    //             match ctrl_dev.get_feature_report(&mut buf) {
    //                 Ok(_result) => {
    //         #[cfg(debug_assertions)]
    //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));
    //                     if buf[1] == 0x01 {
    //                         return Ok(());
    //                     }
    //                 }

    //                 Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
    //             }
    //         }
    //     }
    // }

    fn write_feature_report(&self, buffer: &[u8]) -> Result<()> {
        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            // we have to use the led_hiddev here, this is intentional
            let ctrl_dev = self.led_hiddev.as_ref().lock().unwrap();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            match ctrl_dev.send_feature_report(buffer) {
                Ok(_result) => {
                    hexdump::hexdump_iter(buffer).for_each(|s| trace!("  {}", s));

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
            // we have to use the led_hiddev here, this is intentional
            let ctrl_dev = self.led_hiddev.as_ref().lock().unwrap();
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
                            #[cfg(debug_assertions)]
                            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                            break Ok(buf);
                        }
                    }

                    Err(_) => break Err(HwDeviceError::InvalidResult {}.into()),
                }
            }
        }
    }
}

impl DeviceInfoExt for RoccatKain2xx {
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
            let mut buf = [0; size_of::<DeviceInfo>()];
            buf[0] = 0x09; // Query device info (HID report 0x09)

            let ctrl_dev = self.ctrl_hiddev.as_ref().lock().unwrap();
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
            }
        }
    }

    fn get_firmware_revision(&self) -> String {
        // if let Ok(device_info) = self.get_device_info() {
        //     format!(
        //         "{}.{:02}",
        //         device_info.firmware_version / 100,
        //         device_info.firmware_version % 100
        //     )
        // } else {
        "<unknown>".to_string()
        // }
    }
}

impl DeviceExt for RoccatKain2xx {
    fn get_dev_paths(&self) -> Vec<String> {
        vec![
            self.ctrl_hiddev_info
                .clone()
                .unwrap()
                .path()
                .to_str()
                .unwrap()
                .to_string(),
            self.led_hiddev_info
                .clone()
                .unwrap()
                .path()
                .to_str()
                .unwrap()
                .to_string(),
        ]
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
        "mice/roccat_kain_2xx".to_string()
    }

    fn open(&mut self, api: &hidapi::HidApi) -> Result<()> {
        trace!("Opening HID devices now...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            trace!("Opening control device...");

            match self.ctrl_hiddev_info.as_ref().unwrap().open_device(api) {
                Ok(dev) => *self.ctrl_hiddev.lock().unwrap() = Some(dev),
                Err(_) => return Err(HwDeviceError::DeviceOpenError {}.into()),
            };

            trace!("Opening LED device...");

            match self.led_hiddev_info.as_ref().unwrap().open_device(api) {
                Ok(dev) => *self.led_hiddev.lock().unwrap() = Some(dev),
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
            *self.ctrl_hiddev.lock().unwrap() = None;

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
            //         if device_info.firmware_version < 110 {
            //             warn!(
            //                 "Outdated firmware version: {}, should be: >= 1.10",
            //                 format!(
            //                     "{}.{:02}",
            //                     device_info.firmware_version / 100,
            //                     device_info.firmware_version % 100
            //                 )
            //             );
            //         }
            //     }

            //     Err(e) => {
            //         error!("Could not get firmware version: {}", e);
            //     }
            // }

            // self.send_ctrl_report(0x04)
            //     .unwrap_or_else(|e| error!("Step 1: {}", e));
            // self.wait_for_ctrl_dev()
            //     .unwrap_or_else(|e| error!("Wait 1: {}", e));

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
            let ctrl_dev = self.ctrl_hiddev.as_ref().lock().unwrap();
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
            let ctrl_dev = self.ctrl_hiddev.as_ref().lock().unwrap();
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
        let read_results = || -> Result<DeviceStatus> {
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
                                format!("{battery_status}"),
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
                                format!("{transceiver_enabled}"),
                            );

                            // signal strength
                            table.insert(
                                "signal-strength-percent".to_string(),
                                format!("{:.0}", (signal as f32 / 100.0).clamp(0.0, 100.0)),
                            );

                            table.insert("signal-strength-raw".to_string(), format!("{signal}"));
                        }
                    }

                    _ => { /* do nothing */ }
                }
            }

            Ok(DeviceStatus(table))
        };

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else if !self.is_initialized {
            Err(HwDeviceError::DeviceNotInitialized {}.into())
        } else {
            // TODO: Further investigate the meaning of the fields

            let buf: [u8; 22] = [
                0x08, 0x03, 0x53, 0x00, 0x58, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
            ];

            self.write_feature_report(&buf)?;

            let result = read_results()?;

            thread::sleep(Duration::from_millis(constants::DEVICE_SHORT_DELAY));

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

    fn set_brightness(&mut self, brightness: i32) -> Result<()> {
        trace!("Setting device specific brightness");

        self.brightness = brightness;

        Ok(())
    }

    fn get_brightness(&self) -> Result<i32> {
        trace!("Querying device specific brightness");

        Ok(self.brightness)
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
            let led_dev = self.led_hiddev.as_ref().lock().unwrap();
            let led_dev = led_dev.as_ref().unwrap();

            let mut buf: [u8; 22] = [
                0x08,
                0x09,
                0x33,
                0x00,
                (led_map[LED_0].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_0].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_0].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_1].r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_1].g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                (led_map[LED_1].b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
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

            buf[10] = CRC8.lock().unwrap().calc(&buf[4..10], 6, 0x32);

            match led_dev.send_feature_report(&buf) {
                Ok(_result) => {
                    #[cfg(debug_assertions)]
                    hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));
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

    fn get_evdev_input_rx(&self) -> &Option<flume::Receiver<Option<evdev_rs::InputEvent>>> {
        &self.evdev_rx
    }

    fn set_evdev_input_rx(&mut self, rx: Option<flume::Receiver<Option<evdev_rs::InputEvent>>>) {
        self.evdev_rx = rx;
    }
}

impl DeviceZoneAllocationExt for RoccatKain2xx {
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

impl MouseDeviceExt for RoccatKain2xx {
    fn get_profile(&self) -> Result<i32> {
        trace!("Querying device profile config");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            // let ctrl_dev = self.ctrl_hiddev.as_ref().lock().unwrap();
            // let ctrl_dev = ctrl_dev.as_ref().unwrap();

            // let mut buf: [u8; 64] = [0x00 as u8; 64];
            // buf[0] = 0x06;

            // match ctrl_dev.get_feature_report(&mut buf) {
            //     Ok(_result) => {
            //         #[cfg(debug_assertions)]
            //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

            //         Ok(())
            //     }

            //     Err(_) => Err(HwDeviceError::InvalidResult {}),
            // }?;

            // Ok(buf[6] as i32)

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
            // let ctrl_dev = self.ctrl_hiddev.as_ref().lock().unwrap();
            // let ctrl_dev = ctrl_dev.as_ref().unwrap();

            // let mut buf: [u8; 64] = [0x00 as u8; 64];
            // buf[0] = 0x06;

            // match ctrl_dev.get_feature_report(&mut buf) {
            //     Ok(_result) => {
            //         #[cfg(debug_assertions)]
            //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

            //         Ok(())
            //     }

            //     Err(_) => Err(HwDeviceError::InvalidResult {}),
            // }?;

            // buf[6] = profile as u8;

            // match ctrl_dev.send_feature_report(&buf) {
            //     Ok(_result) => {
            //         #[cfg(debug_assertions)]
            //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

            //         Ok(())
            //     }

            //     Err(_) => Err(HwDeviceError::InvalidResult {}),
            // }?;

            // Ok(())

            Err(HwDeviceError::OpNotSupported {}.into())
        }
    }

    fn get_dpi(&self) -> Result<i32> {
        trace!("Querying device DPI config");

        Err(HwDeviceError::OpNotSupported {}.into())
    }

    fn set_dpi(&mut self, _dpi: i32) -> Result<()> {
        trace!("Setting device DPI config");

        Err(HwDeviceError::OpNotSupported {}.into())
    }

    fn get_rate(&self) -> Result<i32> {
        trace!("Querying device poll rate config");

        Err(HwDeviceError::OpNotSupported {}.into())
    }

    fn set_rate(&mut self, _rate: i32) -> Result<()> {
        trace!("Setting device poll rate config");

        Err(HwDeviceError::OpNotSupported {}.into())
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

        Err(HwDeviceError::OpNotSupported {}.into())
    }

    fn set_angle_snapping(&mut self, _angle_snapping: bool) -> Result<()> {
        trace!("Setting device angle-snapping config");

        Err(HwDeviceError::OpNotSupported {}.into())
    }

    fn get_debounce(&self) -> Result<bool> {
        trace!("Querying device debounce config");

        Err(HwDeviceError::OpNotSupported {}.into())
    }

    fn set_debounce(&mut self, _debounce: bool) -> Result<()> {
        trace!("Setting device debounce config");

        Err(HwDeviceError::OpNotSupported {}.into())
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
            // led_hiddev has to be used to query HID events, this is intentional
            let ctrl_dev = self.led_hiddev.as_ref().lock().unwrap();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            let mut buf = [0; 8];

            match ctrl_dev.read_timeout(&mut buf, millis) {
                Ok(size) => {
                    #[cfg(debug_assertions)]
                    hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                    let event = match buf[0..6] {
                        // Button reports (DPI)
                        [0x07, 0x04, 0x17, 0x01, level, _] => MouseHidEvent::DpiChange(level),

                        // Button reports
                        [button_mask, 0x00, button_mask2, 0x00, _] if size > 0 => {
                            let mut result = vec![];

                            let button_mask = button_mask.view_bits::<Lsb0>();
                            let button_mask2 = button_mask2.view_bits::<Lsb0>();

                            let mut button_states = self.button_states.lock().unwrap();

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
