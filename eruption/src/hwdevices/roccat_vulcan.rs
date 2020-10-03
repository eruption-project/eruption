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
use std::{mem::size_of, time::Duration};
use std::{sync::Arc, thread};

use crate::constants;
use crate::plugins::keyboard;

use super::{
    DeviceCapabilities, DeviceInfoTrait, DeviceTrait, HwDeviceError, KeyboardDeviceTrait,
    KeyboardHidEvent, LedKind, RGBA,
};

pub type Result<T> = super::Result<T>;

pub const NUM_KEYS: usize = 144;

pub const CTRL_INTERFACE: i32 = 1; // Control USB sub device
pub const LED_INTERFACE: i32 = 3; // LED USB sub device

/// ROCCAT Vulcan 100/12x device info struct (sent as HID report)
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

/// Event code of a device HID message
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum KeyboardHidEventCode {
    Unknown(u8),

    KEY_F1,
    KEY_F2,
    KEY_F3,
    KEY_F4,

    KEY_F5,
    KEY_F6,
    KEY_F7,
    KEY_F8,

    KEY_ESC,
    KEY_CAPS_LOCK,
    KEY_FN,
    KEY_EASY_SHIFT,
}

impl KeyboardHidEventCode {
    /// Instantiate a HidEventCode from raw HID report data
    pub fn from_report(report: u8, code: u8) -> Self {
        match report {
            0xfb => match code {
                16 => Self::KEY_F1,
                24 => Self::KEY_F2,
                33 => Self::KEY_F3,
                32 => Self::KEY_F4,

                40 => Self::KEY_F5,
                48 => Self::KEY_F6,
                56 => Self::KEY_F7,
                57 => Self::KEY_F8,

                17 => Self::KEY_ESC,
                119 => Self::KEY_FN,

                _ => Self::Unknown(code),
            },

            0x0a => match code {
                57 => Self::KEY_CAPS_LOCK,
                255 => Self::KEY_EASY_SHIFT,

                _ => Self::Unknown(code),
            },

            _ => Self::Unknown(code),
        }
    }
}

/// Convert a HidEventCode to an integer code value
impl Into<u8> for KeyboardHidEventCode {
    fn into(self) -> u8 {
        match self {
            Self::KEY_F1 => 16,
            Self::KEY_F2 => 24,
            Self::KEY_F3 => 33,
            Self::KEY_F4 => 32,

            Self::KEY_F5 => 40,
            Self::KEY_F6 => 48,
            Self::KEY_F7 => 56,
            Self::KEY_F8 => 57,

            Self::KEY_ESC => 17,
            Self::KEY_CAPS_LOCK => 57,
            Self::KEY_FN => 119,
            Self::KEY_EASY_SHIFT => 255,

            KeyboardHidEventCode::Unknown(code) => code,
        }
    }
}

#[derive(Clone)]
/// Device specific code for the ROCCAT Vulcan 100/12x series keyboards
pub struct RoccatVulcan1xx {
    pub is_initialized: bool,

    // keyboard
    pub is_bound: bool,
    pub ctrl_hiddev_info: Option<hidapi::DeviceInfo>,
    pub led_hiddev_info: Option<hidapi::DeviceInfo>,

    pub is_opened: bool,
    pub ctrl_hiddev: Arc<Mutex<Option<hidapi::HidDevice>>>,
    pub led_hiddev: Arc<Mutex<Option<hidapi::HidDevice>>>,
}

impl RoccatVulcan1xx {
    /// Binds the driver to the supplied HID devices
    pub fn bind(ctrl_dev: &hidapi::DeviceInfo, led_dev: &hidapi::DeviceInfo) -> Self {
        info!("Bound driver: ROCCAT Vulcan");

        Self {
            is_initialized: false,

            is_bound: true,
            ctrl_hiddev_info: Some(ctrl_dev.clone()),
            led_hiddev_info: Some(led_dev.clone()),

            is_opened: false,
            ctrl_hiddev: Arc::new(Mutex::new(None)),
            led_hiddev: Arc::new(Mutex::new(None)),
        }
    }

    pub(self) fn query_ctrl_report(&mut self, id: u8) -> Result<()> {
        trace!("Querying control device feature report");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            match id {
                0x0f => {
                    let mut buf: [u8; 256] = [0; 256];
                    buf[0] = id;

                    let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
                    let ctrl_dev = ctrl_dev.as_ref().unwrap();

                    match ctrl_dev.get_feature_report(&mut buf) {
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
                0x15 => {
                    let buf: [u8; 3] = [0x15, 0x00, 0x01];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x05 => {
                    let buf: [u8; 4] = [0x05, 0x04, 0x00, 0x04];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x07 => {
                    let buf: [u8; 95] = [
                        0x07, 0x5f, 0x00, 0x3a, 0x00, 0x00, 0x3b, 0x00, 0x00, 0x3c, 0x00, 0x00,
                        0x3d, 0x00, 0x00, 0x3e, 0x00, 0x00, 0x3f, 0x00, 0x00, 0x40, 0x00, 0x00,
                        0x41, 0x00, 0x00, 0x42, 0x00, 0x00, 0x43, 0x00, 0x00, 0x44, 0x00, 0x00,
                        0x45, 0x00, 0x00, 0x46, 0x00, 0x00, 0x47, 0x00, 0x00, 0x48, 0x00, 0x00,
                        0xb3, 0x00, 0x00, 0xb4, 0x00, 0x00, 0xb5, 0x00, 0x00, 0xb6, 0x00, 0x00,
                        0xc2, 0x00, 0x00, 0xc3, 0x00, 0x00, 0xc0, 0x00, 0x00, 0xc1, 0x00, 0x00,
                        0xce, 0x00, 0x00, 0xcf, 0x00, 0x00, 0xcc, 0x00, 0x00, 0xcd, 0x00, 0x00,
                        0x46, 0x00, 0x00, 0xfc, 0x00, 0x00, 0x48, 0x00, 0x00, 0xcd, 0x0e,
                    ];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x0a => {
                    let buf: [u8; 8] = [0x0a, 0x08, 0x00, 0xff, 0xf1, 0x00, 0x02, 0x02];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x0b => {
                    let buf: [u8; 65] = [
                        0x0b, 0x41, 0x00, 0x1e, 0x00, 0x00, 0x1f, 0x00, 0x00, 0x20, 0x00, 0x00,
                        0x21, 0x00, 0x00, 0x22, 0x00, 0x00, 0x14, 0x00, 0x00, 0x1a, 0x00, 0x00,
                        0x08, 0x00, 0x00, 0x15, 0x00, 0x00, 0x17, 0x00, 0x00, 0x04, 0x00, 0x00,
                        0x16, 0x00, 0x00, 0x07, 0x00, 0x00, 0x09, 0x00, 0x00, 0x0a, 0x00, 0x00,
                        0x1d, 0x00, 0x00, 0x1b, 0x00, 0x00, 0x06, 0x00, 0x00, 0x19, 0x00, 0x00,
                        0x05, 0x00, 0x00, 0xde, 0x01,
                    ];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x06 => {
                    let buf: [u8; 133] = [
                        0x06, 0x85, 0x00, 0x3a, 0x29, 0x35, 0x1e, 0x2b, 0x39, 0xe1, 0xe0, 0x3b,
                        0x1f, 0x14, 0x1a, 0x04, 0x64, 0x00, 0x00, 0x3d, 0x3c, 0x20, 0x21, 0x08,
                        0x16, 0x1d, 0xe2, 0x3e, 0x23, 0x22, 0x15, 0x07, 0x1b, 0x06, 0x8b, 0x3f,
                        0x24, 0x00, 0x17, 0x0a, 0x09, 0x19, 0x91, 0x40, 0x41, 0x00, 0x1c, 0x18,
                        0x0b, 0x05, 0x2c, 0x42, 0x26, 0x25, 0x0c, 0x0d, 0x0e, 0x10, 0x11, 0x43,
                        0x2a, 0x27, 0x2d, 0x12, 0x0f, 0x36, 0x8a, 0x44, 0x45, 0x89, 0x2e, 0x13,
                        0x33, 0x37, 0x90, 0x46, 0x49, 0x4c, 0x2f, 0x30, 0x34, 0x38, 0x88, 0x47,
                        0x4a, 0x4d, 0x31, 0x32, 0x00, 0x87, 0xe6, 0x48, 0x4b, 0x4e, 0x28, 0x52,
                        0x50, 0xe5, 0xe7, 0xd2, 0x53, 0x5f, 0x5c, 0x59, 0x51, 0x00, 0xf1, 0xd1,
                        0x54, 0x60, 0x5d, 0x5a, 0x4f, 0x8e, 0x65, 0xd0, 0x55, 0x61, 0x5e, 0x5b,
                        0x62, 0xa4, 0xe4, 0xfc, 0x56, 0x57, 0x85, 0x58, 0x63, 0x00, 0x00, 0xc2,
                        0x24,
                    ];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x09 => {
                    let buf: [u8; 43] = [
                        0x09, 0x2b, 0x00, 0x49, 0x00, 0x00, 0x4a, 0x00, 0x00, 0x4b, 0x00, 0x00,
                        0x4c, 0x00, 0x00, 0x4d, 0x00, 0x00, 0x4e, 0x00, 0x00, 0xa4, 0x00, 0x00,
                        0x8e, 0x00, 0x00, 0xd0, 0x00, 0x00, 0xd1, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x01, 0x00, 0x00, 0x00, 0x00, 0xcd, 0x04,
                    ];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x0d => {
                    // hardware wave effect
                    /* let mut buf: [u8; 443] = [
                        0x0d, 0xbb, 0x01, 0x00, 0x0a, 0x04, 0x05, 0x45, 0x83, 0xca, 0xca, 0xca,
                        0xca, 0xca, 0xca, 0xce, 0xce, 0xd2, 0xce, 0xce, 0xd2, 0x19, 0x19, 0x19,
                        0x19, 0x19, 0x19, 0x23, 0x23, 0x2d, 0x23, 0x23, 0x2d, 0xe0, 0xe0, 0xe0,
                        0xe0, 0xe0, 0xe0, 0xe3, 0xe3, 0xe6, 0xe3, 0xe3, 0xe6, 0xd2, 0xd2, 0xd5,
                        0xd2, 0xd2, 0xd5, 0xd5, 0xd5, 0xd9, 0xd5, 0x00, 0xd9, 0x2d, 0x2d, 0x36,
                        0x2d, 0x2d, 0x36, 0x36, 0x36, 0x40, 0x36, 0x00, 0x40, 0xe6, 0xe6, 0xe9,
                        0xe6, 0xe6, 0xe9, 0xe9, 0xe9, 0xec, 0xe9, 0x00, 0xec, 0xd9, 0xd9, 0xdd,
                        0xd9, 0xdd, 0xdd, 0xe0, 0xe0, 0xdd, 0xe0, 0xe4, 0xe4, 0x40, 0x40, 0x4a,
                        0x40, 0x4a, 0x4a, 0x53, 0x53, 0x4a, 0x53, 0x5d, 0x5d, 0xec, 0xec, 0xef,
                        0xec, 0xef, 0xef, 0xf2, 0xf2, 0xef, 0xf2, 0xf5, 0xf5, 0xe4, 0xe4, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5d, 0x5d, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf5, 0xf5, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe4, 0xe4, 0xe8,
                        0xe8, 0xe8, 0xe8, 0xe8, 0xeb, 0xeb, 0xeb, 0x00, 0xeb, 0x5d, 0x5d, 0x67,
                        0x67, 0x67, 0x67, 0x67, 0x70, 0x70, 0x70, 0x00, 0x70, 0xf5, 0xf5, 0xf8,
                        0xf8, 0xf8, 0xf8, 0xf8, 0xfb, 0xfb, 0xfb, 0x00, 0xfb, 0xeb, 0xef, 0xef,
                        0xef, 0x00, 0xef, 0xf0, 0xf0, 0xed, 0xf0, 0xf0, 0x00, 0x70, 0x7a, 0x7a,
                        0x7a, 0x00, 0x7a, 0x7a, 0x7a, 0x6f, 0x7a, 0x7a, 0x00, 0xfb, 0xfd, 0xfd,
                        0xfd, 0x00, 0xfd, 0xf8, 0xf8, 0xea, 0xf8, 0xf8, 0x00, 0xed, 0xed, 0xea,
                        0xed, 0xed, 0x00, 0xed, 0xea, 0xea, 0xf6, 0xe7, 0xea, 0x6f, 0x6f, 0x65,
                        0x6f, 0x6f, 0x00, 0x6f, 0x65, 0x65, 0x66, 0x5a, 0x65, 0xea, 0xea, 0xdc,
                        0xea, 0xea, 0x00, 0xea, 0xdc, 0xdc, 0x00, 0xce, 0xdc, 0xea, 0xe7, 0xe5,
                        0xe7, 0xe5, 0xe5, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x65, 0x5a, 0x50,
                        0x5a, 0x50, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xdc, 0xce, 0xc0,
                        0xce, 0xc0, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe7, 0x00, 0x00,
                        0xe2, 0xe2, 0xe2, 0xe2, 0xdf, 0xdf, 0xdf, 0xdf, 0xdf, 0x5a, 0x00, 0x00,
                        0x45, 0x45, 0x45, 0x45, 0x3b, 0x3b, 0x3b, 0x3b, 0x3b, 0xce, 0x00, 0x00,
                        0xb2, 0xb2, 0xb2, 0xb2, 0xa4, 0xa4, 0xa4, 0xa4, 0xa4, 0xdc, 0xdc, 0xdc,
                        0xdc, 0x00, 0xda, 0xda, 0xda, 0xda, 0xda, 0x00, 0xd7, 0x30, 0x30, 0x30,
                        0x30, 0x00, 0x26, 0x26, 0x26, 0x26, 0x26, 0x00, 0x1c, 0x96, 0x96, 0x96,
                        0x96, 0x00, 0x88, 0x88, 0x88, 0x88, 0x88, 0x00, 0x7a, 0xd7, 0xd7, 0xd7,
                        0x00, 0xd4, 0xd4, 0xd4, 0xd4, 0xd4, 0xd1, 0xd1, 0xd1, 0x1c, 0x1c, 0x1c,
                        0x00, 0x11, 0x11, 0x11, 0x11, 0x11, 0x06, 0x06, 0x06, 0x7a, 0x7a, 0x7a,
                        0x00, 0x6c, 0x6c, 0x6c, 0x6c, 0x6c, 0x5e, 0x5e, 0x5e, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x21, 0xcf,
                    ];

                    // byte 5   == 01-slow 06-med 0b-fast
                    // byte 441 == 1e-slow 23-med 28-fast

                    buf[5] = 0x06;
                    buf[441] = 0x23;

                    */

                    // custom effects
                    let buf: [u8; 443] = [
                        0x0d, 0xbb, 0x01, 0x00, 0x06, 0x0b, 0x05, 0x45, 0x83, 0xca, 0xca, 0xca,
                        0xca, 0xca, 0xca, 0xce, 0xce, 0xd2, 0xce, 0xce, 0xd2, 0x19, 0x19, 0x19,
                        0x19, 0x19, 0x19, 0x23, 0x23, 0x2d, 0x23, 0x23, 0x2d, 0xe0, 0xe0, 0xe0,
                        0xe0, 0xe0, 0xe0, 0xe3, 0xe3, 0xe6, 0xe3, 0xe3, 0xe6, 0xd2, 0xd2, 0xd5,
                        0xd2, 0xd2, 0xd5, 0xd5, 0xd5, 0xd9, 0xd5, 0x00, 0xd9, 0x2d, 0x2d, 0x36,
                        0x2d, 0x2d, 0x36, 0x36, 0x36, 0x40, 0x36, 0x00, 0x40, 0xe6, 0xe6, 0xe9,
                        0xe6, 0xe6, 0xe9, 0xe9, 0xe9, 0xec, 0xe9, 0x00, 0xec, 0xd9, 0xd9, 0xdd,
                        0xd9, 0xdd, 0xdd, 0xe0, 0xe0, 0xdd, 0xe0, 0xe4, 0xe4, 0x40, 0x40, 0x4a,
                        0x40, 0x4a, 0x4a, 0x53, 0x53, 0x4a, 0x53, 0x5d, 0x5d, 0xec, 0xec, 0xef,
                        0xec, 0xef, 0xef, 0xf2, 0xf2, 0xef, 0xf2, 0xf5, 0xf5, 0xe4, 0xe4, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x5d, 0x5d, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf5, 0xf5, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe4, 0xe4, 0xe8,
                        0xe8, 0xe8, 0xe8, 0xe8, 0xeb, 0xeb, 0xeb, 0x00, 0xeb, 0x5d, 0x5d, 0x67,
                        0x67, 0x67, 0x67, 0x67, 0x70, 0x70, 0x70, 0x00, 0x70, 0xf5, 0xf5, 0xf8,
                        0xf8, 0xf8, 0xf8, 0xf8, 0xfb, 0xfb, 0xfb, 0x00, 0xfb, 0xeb, 0xef, 0xef,
                        0xef, 0x00, 0xef, 0xf0, 0xf0, 0xed, 0xf0, 0xf0, 0x00, 0x70, 0x7a, 0x7a,
                        0x7a, 0x00, 0x7a, 0x7a, 0x7a, 0x6f, 0x7a, 0x7a, 0x00, 0xfb, 0xfd, 0xfd,
                        0xfd, 0x00, 0xfd, 0xf8, 0xf8, 0xea, 0xf8, 0xf8, 0x00, 0xed, 0xed, 0xea,
                        0xed, 0xed, 0x00, 0xed, 0xea, 0xea, 0xf6, 0xe7, 0xea, 0x6f, 0x6f, 0x65,
                        0x6f, 0x6f, 0x00, 0x6f, 0x65, 0x65, 0x66, 0x5a, 0x65, 0xea, 0xea, 0xdc,
                        0xea, 0xea, 0x00, 0xea, 0xdc, 0xdc, 0x00, 0xce, 0xdc, 0xea, 0xe7, 0xe5,
                        0xe7, 0xe5, 0xe5, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x65, 0x5a, 0x50,
                        0x5a, 0x50, 0x50, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xdc, 0xce, 0xc0,
                        0xce, 0xc0, 0xc0, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xe7, 0x00, 0x00,
                        0xe2, 0xe2, 0xe2, 0xe2, 0xdf, 0xdf, 0xdf, 0xdf, 0xdf, 0x5a, 0x00, 0x00,
                        0x45, 0x45, 0x45, 0x45, 0x3b, 0x3b, 0x3b, 0x3b, 0x3b, 0xce, 0x00, 0x00,
                        0xb2, 0xb2, 0xb2, 0xb2, 0xa4, 0xa4, 0xa4, 0xa4, 0xa4, 0xdc, 0xdc, 0xdc,
                        0xdc, 0x00, 0xda, 0xda, 0xda, 0xda, 0xda, 0x00, 0xd7, 0x30, 0x30, 0x30,
                        0x30, 0x00, 0x26, 0x26, 0x26, 0x26, 0x26, 0x00, 0x1c, 0x96, 0x96, 0x96,
                        0x96, 0x00, 0x88, 0x88, 0x88, 0x88, 0x88, 0x00, 0x7a, 0xd7, 0xd7, 0xd7,
                        0x00, 0xd4, 0xd4, 0xd4, 0xd4, 0xd4, 0xd1, 0xd1, 0xd1, 0x1c, 0x1c, 0x1c,
                        0x00, 0x11, 0x11, 0x11, 0x11, 0x11, 0x06, 0x06, 0x06, 0x7a, 0x7a, 0x7a,
                        0x00, 0x6c, 0x6c, 0x6c, 0x6c, 0x6c, 0x5e, 0x5e, 0x5e, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
                        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x24, 0xcf,
                    ];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x13 => {
                    // hardware wave effect
                    // let buf: [u8; 8] = [0x13, 0x08, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];

                    // custom effects
                    let buf: [u8; 8] = [0x13, 0x08, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];

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

    fn wait_for_ctrl_dev(&mut self) -> Result<()> {
        trace!("Waiting for control device to respond...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else {
            loop {
                let mut buf: [u8; 4] = [0; 4];
                buf[0] = 0x04;

                let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
                let ctrl_dev = ctrl_dev.as_ref().unwrap();

                match ctrl_dev.get_feature_report(&mut buf) {
                    Ok(_result) => {
                        hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                        if buf[1] == 0x01 {
                            return Ok(());
                        }
                    }

                    Err(_) => return Err(HwDeviceError::InvalidResult {}.into()),
                }

                thread::sleep(Duration::from_millis(constants::DEVICE_SETTLE_MILLIS));
            }
        }
    }
}

impl DeviceInfoTrait for RoccatVulcan1xx {
    fn get_device_capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities {}
    }

    fn get_device_info(&self) -> Result<super::DeviceInfo> {
        trace!("Querying the device for information...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else if !self.is_initialized {
            Err(HwDeviceError::DeviceNotInitialized {}.into())
        } else {
            let mut buf = [0; size_of::<DeviceInfo>()];
            buf[0] = 0x0f; // Query device info (HID report 0x0f)

            let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            match ctrl_dev.get_feature_report(&mut buf) {
                Ok(_result) => {
                    hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));
                    let tmp: DeviceInfo =
                        unsafe { std::ptr::read_unaligned(buf.as_ptr() as *const _) };

                    let result = super::DeviceInfo::new(tmp.firmware_version as i32);
                    Ok(result)
                }

                Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
            }
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

impl DeviceTrait for RoccatVulcan1xx {
    fn get_usb_path(&self) -> String {
        self.led_hiddev_info
            .clone()
            .unwrap()
            .path()
            .to_str()
            .unwrap()
            .to_string()
    }

    fn open(&mut self, api: &hidapi::HidApi) -> Result<()> {
        trace!("Opening HID devices now...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else {
            trace!("Opening control device...");

            match self.ctrl_hiddev_info.as_ref().unwrap().open_device(&api) {
                Ok(dev) => *self.ctrl_hiddev.lock() = Some(dev),
                Err(_) => return Err(HwDeviceError::DeviceOpenError {}.into()),
            };

            trace!("Opening LED device...");

            match self.led_hiddev_info.as_ref().unwrap().open_device(&api) {
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
            trace!("Closing control device...");
            *self.ctrl_hiddev.lock() = None;

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
            self.query_ctrl_report(0x0f)
                .unwrap_or_else(|e| error!("{}", e));
            self.send_ctrl_report(0x15)
                .unwrap_or_else(|e| error!("{}", e));
            self.wait_for_ctrl_dev().unwrap_or_else(|e| error!("{}", e));

            self.send_ctrl_report(0x05)
                .unwrap_or_else(|e| error!("{}", e));
            self.wait_for_ctrl_dev().unwrap_or_else(|e| error!("{}", e));

            self.send_ctrl_report(0x07)
                .unwrap_or_else(|e| error!("{}", e));
            self.wait_for_ctrl_dev().unwrap_or_else(|e| error!("{}", e));

            self.send_ctrl_report(0x0a)
                .unwrap_or_else(|e| error!("{}", e));
            self.wait_for_ctrl_dev().unwrap_or_else(|e| error!("{}", e));

            self.send_ctrl_report(0x0b)
                .unwrap_or_else(|e| error!("{}", e));
            self.wait_for_ctrl_dev().unwrap_or_else(|e| error!("{}", e));

            self.send_ctrl_report(0x06)
                .unwrap_or_else(|e| error!("{}", e));
            self.wait_for_ctrl_dev().unwrap_or_else(|e| error!("{}", e));

            self.send_ctrl_report(0x09)
                .unwrap_or_else(|e| error!("{}", e));
            self.wait_for_ctrl_dev().unwrap_or_else(|e| error!("{}", e));

            self.send_ctrl_report(0x0d)
                .unwrap_or_else(|e| error!("{}", e));
            self.wait_for_ctrl_dev().unwrap_or_else(|e| error!("{}", e));

            self.send_ctrl_report(0x13)
                .unwrap_or_else(|e| error!("{}", e));
            self.wait_for_ctrl_dev().unwrap_or_else(|e| error!("{}", e));

            self.is_initialized = true;

            Ok(())
        }
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
                    hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                    Ok(buf)
                }

                Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
            }
        }
    }
}

impl KeyboardDeviceTrait for RoccatVulcan1xx {
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
                self.write_data_raw(&[0x21, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])?;
            }
            LedKind::CapsLock => {
                self.write_data_raw(&[0x22, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])?;
            }
            LedKind::ScrollLock => {
                self.write_data_raw(&[0x23, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])?;
            }
            LedKind::GameMode => {
                self.write_data_raw(&[0x24, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])?;
            }
        }

        Ok(())
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
            let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            let mut buf = [0; 8];

            match ctrl_dev.read_timeout(&mut buf, millis) {
                Ok(_size) => {
                    hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                    let event = match buf[0..5] {
                        // Key reports, incl. KEY_FN, ..
                        [0x03, 0x00, 0xfb, code, status] => match status {
                            0x00 => KeyboardHidEvent::KeyUp {
                                code: KeyboardHidEventCode::from_report(0xfb, code),
                            },

                            0x01 => KeyboardHidEvent::KeyDown {
                                code: KeyboardHidEventCode::from_report(0xfb, code),
                            },

                            _ => KeyboardHidEvent::Unknown,
                        },

                        // CAPS LOCK, Easy Shift+, ..
                        [0x03, 0x00, 0x0a, code, status] => match code {
                            0x39 | 0xff => match status {
                                0x00 => KeyboardHidEvent::KeyDown {
                                    code: KeyboardHidEventCode::from_report(0x0a, code),
                                },

                                0x01 => KeyboardHidEvent::KeyUp {
                                    code: KeyboardHidEventCode::from_report(0x0a, code),
                                },

                                _ => KeyboardHidEvent::Unknown,
                            },

                            _ => KeyboardHidEvent::Unknown,
                        },

                        [0x03, 0x00, 0xcc, code, _] => match code {
                            0x01 => KeyboardHidEvent::VolumeUp,
                            0xff => KeyboardHidEvent::VolumeDown,

                            _ => KeyboardHidEvent::Unknown,
                        },

                        [0x02, 0xe2, 0x00, 0x00, _] => KeyboardHidEvent::MuteDown,
                        [0x02, 0x00, 0x00, 0x00, _] => KeyboardHidEvent::MuteUp,

                        _ => KeyboardHidEvent::Unknown,
                    };

                    match event {
                        KeyboardHidEvent::KeyDown { code } => {
                            // update our internal representation of the keyboard state
                            let index = hid_code_to_key_index(code) as usize;
                            keyboard::KEY_STATES.write()[index] = true;
                        }

                        KeyboardHidEvent::KeyUp { code } => {
                            // update our internal representation of the keyboard state
                            let index = hid_code_to_key_index(code) as usize;
                            keyboard::KEY_STATES.write()[index] = false;
                        }

                        _ => { /* ignore other events */ }
                    }

                    Ok(event)
                }

                Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
            }
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
                    // TODO: The #' key (on QWERTZ layout) seems to be out of order!?
                    //       This is an ugly hack, find a better way to fix this
                    // let mut led_map = led_map.to_vec();
                    // led_map.swap(81, 96);

                    let mut hwmap: [u8; 444] = [0; 444];

                    // Colors are in blocks of 12 keys (2 columns). Color parts are sorted by color e.g. the red
                    // values for all 12 keys are first then come the green values etc.
                    for (i, color) in led_map.iter().enumerate() {
                        let offset = ((i / 12) * 36) + (i % 12);

                        hwmap[offset] = color.r;
                        hwmap[offset + 12] = color.g;
                        hwmap[offset + 24] = color.b;
                    }

                    let (slice, hwmap) = hwmap.split_at(60);

                    let mut buf: [u8; 65] = [0; 65];
                    buf[1..5].copy_from_slice(&[0xa1, 0x01, 0x01, 0xb4]);
                    buf[5..65].copy_from_slice(&slice);

                    hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                    match led_dev.write(&buf) {
                        Ok(len) => {
                            trace!("Wrote: {} bytes", len);
                            if len < 65 {
                                return Err(HwDeviceError::WriteError {}.into());
                            }
                        }

                        Err(_) => return Err(HwDeviceError::WriteError {}.into()),
                    }

                    for bytes in hwmap.chunks(64) {
                        buf[1..65].copy_from_slice(bytes);

                        hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                        match led_dev.write(&buf) {
                            Ok(len) => {
                                trace!("Wrote: {} bytes", len);
                                if len < 65 {
                                    return Err(HwDeviceError::WriteError {}.into());
                                }
                            }

                            Err(_) => return Err(HwDeviceError::WriteError {}.into()),
                        }
                    }

                    Ok(())
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
            let led_map: [RGBA; NUM_KEYS] = [RGBA {
                r: 0x00,
                g: 0x00,
                b: 0x00,
                a: 0x00,
            }; NUM_KEYS];

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
            let led_map: [RGBA; NUM_KEYS] = [RGBA {
                r: 0x00,
                g: 0x00,
                b: 0x00,
                a: 0x00,
            }; NUM_KEYS];

            self.send_led_map(&led_map)?;

            Ok(())
        }
    }
}

pub fn hid_code_to_key_index(code: KeyboardHidEventCode) -> u8 {
    match code {
        KeyboardHidEventCode::KEY_FN => 77,

        KeyboardHidEventCode::KEY_CAPS_LOCK => 4,
        KeyboardHidEventCode::KEY_EASY_SHIFT => 4,

        // We don't need all the other key codes, for now
        _ => 0,
    }
}
