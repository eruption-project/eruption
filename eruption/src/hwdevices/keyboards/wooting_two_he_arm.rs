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

use bitfield_struct::bitfield;
#[cfg(not(target_os = "windows"))]
use evdev_rs::enums::EV_KEY;
use flume::Receiver;
use hidapi::HidApi;
use libc::wchar_t;
use ndarray::{s, ArrayView2};
use resize::Pixel::RGB8;
use resize::Type;
use rgb::RGB8;
use std::any::Any;
use std::collections::HashMap;
use std::time::Duration;
use std::{sync::Arc, thread};
use tracing::*;
use tracing_mutex::stdsync::Mutex;

use crate::constants;

use crate::hwdevices::{
    self, Capability, DeviceCapabilities, DeviceClass, DeviceExt, DeviceInfoExt, DeviceQuirks,
    DeviceStatus, DeviceZoneAllocationExt, HwDeviceError, KeyboardDeviceExt, KeyboardHidEvent,
    KeyboardHidEventCode, LedKind, MouseDeviceExt, Result, Zone, RGBA,
};

pub const CTRL_INTERFACE: i32 = 1; // Control USB sub device
pub const LED_INTERFACE: i32 = 2; // LED USB sub device

pub const NUM_ROWS: usize = 6;
pub const NUM_COLS: usize = 21;
#[allow(unused)]
pub const NUM_LEDS: usize = NUM_ROWS * NUM_COLS;
pub const NUM_KEYS: usize = 135;
// pub const NUM_RGB: usize = 196;
pub const LED_INDICES: usize = 135;

// Wooting protocol v2 constants
// pub const COMMAND_SIZE: usize = 8;
// pub const REPORT_SIZE: usize = 256 + 1;
pub const SMALL_PACKET_SIZE: usize = 64;
pub const SMALL_PACKET_COUNT: usize = 4;
pub const RESPONSE_SIZE: usize = 256;
pub const READ_RESPONSE_TIMEOUT: i32 = 150; // Default: 1000;

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

#[bitfield(u8)]
struct KeyCoordinates {
    #[bits(5)]
    pub column: u8,

    #[bits(3)]
    pub row: u8,
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
        Ok(Box::new(WootingTwoHeArm::bind(
            ctrl_dev.unwrap(),
            led_dev.unwrap(),
        )))
    }
}

/// Wooting Two HE keyboard device info
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
/// Device specific code for the Wooting Two HE (ARM) series keyboards
pub struct WootingTwoHeArm {
    #[cfg(not(target_os = "windows"))]
    pub evdev_rx: Option<Receiver<Option<evdev_rs::InputEvent>>>,

    pub is_initialized: bool,

    // keyboard
    pub is_bound: bool,
    pub is_opened: bool,
    pub has_failed: bool,

    pub ctrl_hiddev_info: Option<hidapi::DeviceInfo>,
    pub led_hiddev_info: Option<hidapi::DeviceInfo>,

    pub ctrl_hiddev: Arc<Mutex<Option<hidapi::HidDevice>>>,
    pub led_hiddev: Arc<Mutex<Option<hidapi::HidDevice>>>,

    pub allocated_zone: Zone,

    // device specific configuration options
    pub brightness: i32,
}

impl WootingTwoHeArm {
    /// Binds the driver to the supplied HID devices
    pub fn bind(ctrl_dev: &hidapi::DeviceInfo, led_dev: &hidapi::DeviceInfo) -> Self {
        debug!("Bound driver: Wooting Two HE (ARM)");

        Self {
            #[cfg(not(target_os = "windows"))]
            evdev_rx: None,

            is_initialized: false,

            is_bound: true,
            is_opened: false,
            has_failed: false,

            ctrl_hiddev_info: Some(ctrl_dev.clone()),
            led_hiddev_info: Some(led_dev.clone()),

            ctrl_hiddev: Arc::new(Mutex::new(None)),
            led_hiddev: Arc::new(Mutex::new(None)),

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

    //                 let ctrl_dev = self.ctrl_hiddev.as_ref().lock().unwrap();
    //                 let ctrl_dev = ctrl_dev.as_ref().unwrap();

    //                 match ctrl_dev.get_feature_report(&mut buf) {
    //                     Ok(_result) => {
    //                         #[cfg(debug_assertions)]
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

    // fn v2_set_led_xy(&self, x: usize, y: usize, color: &RGBA) -> Result<()> {
    //     let id = TOPOLOGY.get(y * NUM_COLS + x).cloned().unwrap_or(0xff);

    //     self.v2_send_feature_report(
    //         Command::SINGLE_COLOR_COMMAND as u8,
    //         &[id, color.r, color.g, color.b],
    //     )?;

    //     Ok(())
    // }

    fn v2_send_feature_report(&self, id: u8, params: &[u8; 4]) -> Result<()> {
        trace!("Sending control device feature report [Wooting v2]");

        let mut report_buffer = [0x0; SMALL_PACKET_SIZE + 1];

        report_buffer[0] = 0x00;
        report_buffer[1] = 0xd0;
        report_buffer[2] = 0xda;
        report_buffer[3] = id;
        report_buffer[4] = params[3];
        report_buffer[5] = params[2];
        report_buffer[6] = params[1];
        report_buffer[7] = params[0];

        let ctrl_dev = self.ctrl_hiddev.as_ref().lock().unwrap();
        let ctrl_dev = ctrl_dev.as_ref().unwrap();

        let result = ctrl_dev.write(&report_buffer);

        match result {
            Ok(_result) => {
                hexdump::hexdump_iter(&report_buffer).for_each(|s| debug!("  {}", s));

                thread::sleep(Duration::from_millis(constants::DEVICE_MICRO_DELAY));

                Ok(())
            }

            Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
        }
    }

    #[allow(dead_code)]
    fn v2_send_feature_report_with_response(&self, id: u8, params: &[u8; 4]) -> Result<()> {
        trace!("Sending control device feature report [Wooting v2]");

        let mut report_buffer = [0x0; SMALL_PACKET_SIZE + 1];

        report_buffer[0] = 0x00;
        report_buffer[1] = 0xd0;
        report_buffer[2] = 0xda;
        report_buffer[3] = id;
        report_buffer[4] = params[3];
        report_buffer[5] = params[2];
        report_buffer[6] = params[1];
        report_buffer[7] = params[0];

        let ctrl_dev = self.ctrl_hiddev.as_ref().lock().unwrap();
        let ctrl_dev = ctrl_dev.as_ref().unwrap();

        let result = ctrl_dev.write(&report_buffer);

        match result {
            Ok(_result) => {
                hexdump::hexdump_iter(&report_buffer).for_each(|s| debug!("  {}", s));

                let mut buf = Vec::with_capacity(RESPONSE_SIZE);
                match ctrl_dev.read_timeout(&mut buf, READ_RESPONSE_TIMEOUT) {
                    Ok(_result) => {
                        hexdump::hexdump_iter(&buf).for_each(|s| debug!("  {}", s));

                        Ok(())
                    }

                    Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                }
            }

            Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
        }
    }

    #[allow(dead_code)]
    fn send_ctrl_report(&mut self, _id: u8) -> Result<()> {
        trace!("Sending control device feature report");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            // let ctrl_dev = self.ctrl_hiddev.as_ref().lock().unwrap();
            // let ctrl_dev = ctrl_dev.as_ref().unwrap();

            // match id {
            //     0x00 => {

            //     }

            //     _ => Err(HwDeviceError::InvalidStatusCode {}.into()),
            // }

            Ok(())
        }
    }

    #[allow(dead_code)]
    fn wait_for_ctrl_dev(&mut self) -> Result<()> {
        trace!("Waiting for control device to respond...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            let mut buf: [u8; RESPONSE_SIZE] = [0x00; RESPONSE_SIZE];

            let ctrl_dev = self.ctrl_hiddev.as_ref().lock().unwrap();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            match ctrl_dev.read_timeout(&mut buf, READ_RESPONSE_TIMEOUT) {
                Ok(_result) => {
                    #[cfg(debug_assertions)]
                    hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                    Ok(())
                }

                Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
            }
        }
    }

    #[allow(dead_code)]
    fn wait_for_led_dev(&mut self) -> Result<()> {
        trace!("Waiting for LED device to respond...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            let mut buf: [u8; RESPONSE_SIZE] = [0x00; RESPONSE_SIZE];

            let led_dev = self.led_hiddev.as_ref().lock().unwrap();
            let led_dev = led_dev.as_ref().unwrap();

            match led_dev.read_timeout(&mut buf, READ_RESPONSE_TIMEOUT) {
                Ok(_result) => {
                    #[cfg(debug_assertions)]
                    hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                    Ok(())
                }

                Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
            }
        }
    }
}

impl DeviceInfoExt for WootingTwoHeArm {
    fn get_device_capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities::from([Capability::Keyboard, Capability::RgbLighting])
    }

    fn get_device_quirks(&self) -> hwdevices::DeviceQuirks {
        DeviceQuirks::from([])
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

            // let ctrl_dev = self.ctrl_hiddev.as_ref().lock().unwrap();
            // let ctrl_dev = ctrl_dev.as_ref().unwrap();

            // match ctrl_dev.get_feature_report(&mut buf) {
            //     Ok(_result) => {
            //         #[cfg(debug_assertions)]
            //         hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));
            //         let tmp: DeviceInfo =
            //             unsafe { std::ptr::read_unaligned(buf.as_ptr() as *const _) };

            //         let result = hwdevices::DeviceInfo::new(tmp.firmware_version as i32);
            //         Ok(result)
            //     }

            //     Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
            // }

            Ok(hwdevices::DeviceInfo::new(0))
        }
    }

    fn get_firmware_revision(&self) -> String {
        if let Ok(device_info) = self.get_device_info() {
            format!(
                "{}.{:2}",
                device_info.firmware_version / 100,
                device_info.firmware_version % 100
            )
        } else {
            "<unknown>".to_string()
        }
    }
}

impl DeviceZoneAllocationExt for WootingTwoHeArm {
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

impl DeviceExt for WootingTwoHeArm {
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
        "keyboards/wooting_two_he_arm".to_string()
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

            trace!("Closing LED device...");
            *self.led_hiddev.lock().unwrap() = None;

            self.is_opened = false;

            Ok(())
        }
    }

    fn device_status(&self) -> Result<DeviceStatus> {
        let mut table = HashMap::new();

        table.insert("connected".to_owned(), format!("{}", true));

        Ok(DeviceStatus(table))
    }

    fn send_init_sequence(&mut self) -> Result<()> {
        trace!("Sending device init sequence...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            // TODO: Implement firmware version check

            // This helps slow USB HUBs and KVM switches to not fail to init the device
            thread::sleep(Duration::from_millis(constants::DEVICE_SETTLE_DELAY));

            self.v2_send_feature_report(Command::RESET_ALL_COMMAND as u8, &[0, 0, 0, 0])
                .unwrap_or_else(|e| error!("Step 1: {}", e));

            thread::sleep(Duration::from_millis(constants::DEVICE_SETTLE_DELAY));

            self.v2_send_feature_report(Command::COLOR_INIT_COMMAND as u8, &[0, 0, 0, 0])
                .unwrap_or_else(|e| error!("Step 2: {}", e));

            thread::sleep(Duration::from_millis(constants::DEVICE_SHORT_DELAY));

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
        } else {
            match *self.led_hiddev.lock().unwrap() {
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
                        #[rustfmt::skip]
                        fn encode_color(color: &RGB8, brightness: i32) -> u16 {
                            let mut encoded_color: u16 = 0x0000;

                            encoded_color |= ((color.b as f32 * (brightness as f32 / 100.0)).floor() as u16 & 0xf8) >> 3;
                            encoded_color |= ((color.g as f32 * (brightness as f32 / 100.0)).floor() as u16 & 0xfc) << 3;
                            encoded_color |= ((color.r as f32 * (brightness as f32 / 100.0)).floor() as u16 & 0xf8) << 8;

                            encoded_color
                        }

                        fn submit_packet(led_dev: &hidapi::HidDevice, buffer: &[u8]) -> Result<()> {
                            hexdump::hexdump_iter(buffer).for_each(|s| trace!("  {}", s));

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
                                    //             .for_each(|s| trace!("  {}", s));
                                    //     }

                                    //     Err(_) => {
                                    //         return Err(HwDeviceError::InvalidResult {}.into())
                                    //     }
                                    // }

                                    // thread::sleep(Duration::from_millis(
                                    //     constants::DEVICE_MICRO_DELAY,
                                    // ));
                                }

                                Err(_) => return Err(HwDeviceError::WriteError {}.into()),
                            }

                            Ok(())
                        }

                        // this device uses a row major layout, starting at the top row, going down
                        // to the bottom row; so we have to transpose the color map to the right layout

                        if self.allocated_zone.enabled {
                            let canvas = ArrayView2::from_shape(
                                (constants::CANVAS_HEIGHT, constants::CANVAS_WIDTH),
                                led_map,
                            )?;

                            let canvas = canvas.slice(s![
                                self.allocated_zone.y..self.allocated_zone.y2(),
                                self.allocated_zone.x..self.allocated_zone.x2(),
                            ]);

                            // resize
                            let (w1, h1) = (
                                self.allocated_zone.width as usize,
                                self.allocated_zone.height as usize,
                            );
                            let (w2, h2) = (NUM_COLS, NUM_ROWS);

                            let canvas = canvas.map(|v| RGB8::new(v.r, v.g, v.b));
                            let mut led_map = vec![RGB8::new(0, 0, 0); w2 * h2];

                            let mut resizer = resize::new(w1, h1, w2, h2, RGB8, Type::Point)?;
                            resizer.resize(canvas.as_slice().unwrap(), &mut led_map)?;

                            const BUFFER_SIZE: usize =
                                4 + (SMALL_PACKET_COUNT * (SMALL_PACKET_SIZE + 1));
                            let mut buffer = [0x00_u8; BUFFER_SIZE];

                            // init sequence
                            buffer[0..4].copy_from_slice(&[
                                0x00,
                                0xd0,
                                0xda,
                                Command::RAW_COLORS_REPORT as u8,
                            ]);

                            // encode color sequence and submit a packet on every 64th byte to the device
                            let mut cntr = 0;

                            for i in (4..BUFFER_SIZE).step_by(2) {
                                let encoded_color = encode_color(
                                    led_map.get(cntr).unwrap_or(&RGB8 { r: 0, g: 0, b: 0 }),
                                    self.brightness,
                                );

                                buffer[i..i + 2].copy_from_slice(&encoded_color.to_le_bytes());

                                cntr += 1;

                                if i % 64 == 0 {
                                    submit_packet(led_dev, &buffer[(i - 64)..=i])?;
                                    buffer[i] = 0x0;
                                }
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
        None
    }

    fn as_mouse_device_mut(&mut self) -> Option<&mut (dyn MouseDeviceExt + Sync + Send)> {
        None
    }

    fn get_device_class(&self) -> DeviceClass {
        DeviceClass::Keyboard
    }

    fn as_keyboard_device(&self) -> Option<&(dyn KeyboardDeviceExt + Sync + Send)> {
        Some(self)
    }

    fn as_keyboard_device_mut(&mut self) -> Option<&mut (dyn KeyboardDeviceExt + Sync + Send)> {
        Some(self)
    }

    fn as_misc_device(&self) -> Option<&(dyn hwdevices::MiscDeviceExt + Sync + Send)> {
        None
    }

    fn as_misc_device_mut(&mut self) -> Option<&mut (dyn hwdevices::MiscDeviceExt + Sync + Send)> {
        None
    }

    #[cfg(not(target_os = "windows"))]
    fn get_evdev_input_rx(&self) -> &Option<flume::Receiver<Option<evdev_rs::InputEvent>>> {
        &self.evdev_rx
    }

    #[cfg(not(target_os = "windows"))]
    fn set_evdev_input_rx(&mut self, rx: Option<flume::Receiver<Option<evdev_rs::InputEvent>>>) {
        self.evdev_rx = rx;
    }
}

impl KeyboardDeviceExt for WootingTwoHeArm {
    fn set_status_led(&self, _led_kind: LedKind, _on: bool) -> Result<()> {
        trace!("Setting status LED state");

        // match led_kind {
        //     LedKind::Unknown => warn!("No LEDs have been set, request was a no-op"),
        //     LedKind::AudioMute => {
        //         // self.write_data_raw(&[0x00, 0x09, 0x22, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])?;
        //     }
        //     LedKind::Fx => {}
        //     LedKind::Volume => {}
        //     LedKind::NumLock => {
        //         self.write_data_raw(&[0x21, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])?;
        //     }
        //     LedKind::CapsLock => {
        //         self.write_data_raw(&[0x22, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])?;
        //     }
        //     LedKind::ScrollLock => {
        //         self.write_data_raw(&[0x23, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])?;
        //     }
        //     LedKind::GameMode => {
        //         self.write_data_raw(&[0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])?;
        //     }
        // }

        Ok(())
    }

    #[inline]
    fn get_next_event(&self) -> Result<KeyboardHidEvent> {
        self.get_next_event_timeout(-1)
    }

    fn get_next_event_timeout(&self, _millis: i32) -> Result<KeyboardHidEvent> {
        trace!("Querying control device for next event");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else if !self.is_initialized {
            Err(HwDeviceError::DeviceNotInitialized {}.into())
        } else {
            Ok(KeyboardHidEvent::Unknown)
        }
    }

    #[cfg(not(target_os = "windows"))]
    fn ev_key_to_key_index(&self, key: EV_KEY) -> u8 {
        EV_TO_INDEX_ISO[(key as u8) as usize].saturating_add(1)
    }

    fn hid_event_code_to_key_index(&self, code: &KeyboardHidEventCode) -> u8 {
        match code {
            KeyboardHidEventCode::KEY_FN => 65,

            KeyboardHidEventCode::KEY_CAPS_LOCK => 6,
            KeyboardHidEventCode::KEY_EASY_SHIFT => 6,

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

#[allow(dead_code)]
fn keyboard_hid_event_code_from_report(report: u8, code: u8) -> KeyboardHidEventCode {
    match report {
        0xfb => match code {
            16 => KeyboardHidEventCode::KEY_F1,
            24 => KeyboardHidEventCode::KEY_F2,
            33 => KeyboardHidEventCode::KEY_F3,
            32 => KeyboardHidEventCode::KEY_F4,

            40 => KeyboardHidEventCode::KEY_F5,
            48 => KeyboardHidEventCode::KEY_F6,
            56 => KeyboardHidEventCode::KEY_F7,
            57 => KeyboardHidEventCode::KEY_F8,

            17 => KeyboardHidEventCode::KEY_ESC,
            119 => KeyboardHidEventCode::KEY_FN,

            _ => KeyboardHidEventCode::Unknown(code),
        },

        0x0a => match code {
            57 => KeyboardHidEventCode::KEY_CAPS_LOCK,
            255 => KeyboardHidEventCode::KEY_EASY_SHIFT,

            _ => KeyboardHidEventCode::Unknown(code),
        },

        _ => KeyboardHidEventCode::Unknown(code),
    }
}

/// Map evdev event codes to key indices, for ISO variant
#[rustfmt::skip]
const EV_TO_INDEX_ISO: [u8; 0x2ff + 1] = [
    0xff, 0x02, 0x08, 0x0e, 0x15, 0x1a, 0x1f, 0x24, 0x29, 0x30, 0x36, 0x3c, 0x42, 0x48, 0x50, 0x04,
    0x09, 0x0f, 0x16, 0x1b, 0x20, 0x25, 0x2a, 0x31, 0x37, 0x3d, 0x43, 0x49, 0x52, 0x01, 0x0a, 0x10,
    0x17, 0x1c, 0x21, 0x26, 0x2b, 0x32, 0x38, 0x3e, 0x44, 0x03, 0x00, 0x4a, 0x0b, 0x11, 0x18, 0x1d,
    0x22, 0x27, 0x2c, 0x33, 0x39, 0x3f, 0x4b, 0xff, 0x0c, 0x23, 0x05, 0x0d, 0x14, 0x19, 0x1e, 0x28,
    0x2f, 0x35, 0x3b, 0x41, 0x47, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x06, 0x4d, 0x4f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0x4c, 0xff, 0xff, 0x3a, 0xff, 0x58, 0x5a, 0x5d, 0x56, 0x5f, 0x59, 0x5b, 0x5e, 0x54, 0x55,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x07, 0xff, 0x46,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
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

#[rustfmt::skip]
pub static ROWS_TOPOLOGY: [u8; 127] = [
    0x02, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x03, 0x08, 0x0e, 0x15, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x04, 0x09, 0x0f, 0x16,
    0x1b, 0x20, 0x25, 0x2a, 0x31, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
    0xff, 0xff, 0x05, 0x0a, 0x10, 0x17, 0x1c, 0x21, 0x26, 0x2b, 0x32, 0x38, 0x3e, 0x44, 0x4a, 0xff,
    0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x00, 0x06, 0x0b, 0x11, 0x18, 0x1d, 0x22, 0x27,
    0x2c, 0x33, 0x39, 0x3f, 0x4b, 0xff, 0x5a, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01, 0x07,
    0x0c, 0x23, 0x3a, 0xff, 0x4c, 0x56, 0x5b, 0x5f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
];

#[rustfmt::skip]
pub static COLS_TOPOLOGY: [u8; 127] = [
    0x02, 0x03, 0x04, 0x05, 0x00, 0x01, 0xff, 0x08, 0x09, 0x0a, 0x06, 0x07, 0x0d, 0x0e, 0x0f, 0x10,
    0x0b, 0x0c, 0x14, 0x15, 0x16, 0x17, 0x11, 0x11, 0x1e, 0x1f, 0x1b, 0x1c, 0x1d, 0xff, 0xff, 0x24,
    0x20, 0x21, 0x22, 0x23, 0x28, 0x29, 0x25, 0x26, 0x27, 0xff, 0x2f, 0x30, 0x2a, 0x2b, 0x2c, 0xff,
    0x3b, 0x36, 0x37, 0x38, 0x33, 0xff, 0xff, 0xff, 0x3c, 0x3d, 0x3e, 0x39, 0x41, 0x42, 0x43, 0x44,
    0x3f, 0xfe, 0x47, 0x48, 0x49, 0x4a, 0x4b, 0xff, 0x4d, 0x50, 0x52, 0x4b, 0x4c, 0xff, 0xfe, 0x54,
    0x55, 0xff, 0xff, 0x56, 0xfe, 0x58, 0x59, 0xff, 0x5a, 0x5b, 0xfe, 0x5d, 0x5e, 0xff, 0xff, 0x5f,
    0xff, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xfe, 0xff, 0xff, 0xff, 0xfe, 0xfe, 0xfe,
    0xfe, 0xfe, 0xff, 0xfe, 0xfe, 0xfe, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff,
];
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
