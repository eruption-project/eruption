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
// use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use crate::constants;

use super::{
    DeviceCapabilities, DeviceInfoTrait, DeviceTrait, HwDeviceError, MouseDeviceTrait,
    MouseHidEvent, NUM_KEYS, RGBA,
};

pub type Result<T> = super::Result<T>;

// pub const NUM_KEYS: usize = 9;
pub const KEYBOARD_SUB_DEVICE: usize = 2;

/// ROCCAT Kone Pure Ultra info struct (sent as HID report)
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct DeviceInfo {
    pub report_id: u8,
    pub size: u8,
    pub reserved1: u16,
    pub firmware_version: i32,
    pub reserved2: u16,
}

/// Event code of a device HID message
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MouseHidEventCode {
    #[allow(dead_code)]
    Unknown(u8),

    KEY_BTN1,
}

impl MouseHidEventCode {
    // Instantiate a HidEventCode from raw HID report data
    // pub fn from_report(report: u8, code: u8) -> Self {
    //     match report {
    //         0xfb => match code {
    //             16 => Self::KEY_BTN1,

    //             _ => Self::Unknown(code),
    //         },

    //         // 0x0a => match code {
    //         //     57 => Self::KEY_CAPS_LOCK,
    //         //     255 => Self::KEY_EASY_SHIFT,

    //         //     _ => Self::Unknown(code),
    //         // },
    //         _ => Self::Unknown(code),
    //     }
    // }
}

/// Convert a HidEventCode to an integer code value
impl Into<u8> for MouseHidEventCode {
    fn into(self) -> u8 {
        match self {
            Self::KEY_BTN1 => 16,

            MouseHidEventCode::Unknown(code) => code,
        }
    }
}

#[derive(Clone)]
/// Device specific code for the ROCCAT Kone Pure Ultra mouse
pub struct RoccatKonePureUltra {
    pub is_initialized: bool,

    pub is_bound: bool,
    pub ctrl_hiddev_info: Option<hidapi::DeviceInfo>,

    pub is_opened: bool,
    pub ctrl_hiddev: Arc<Mutex<Option<hidapi::HidDevice>>>,
}

impl RoccatKonePureUltra {
    /// Binds the driver to the supplied HID device
    pub fn bind(ctrl_dev: &hidapi::DeviceInfo) -> Self {
        info!("Bound driver: ROCCAT Kone Pure Ultra");

        Self {
            is_initialized: false,

            is_bound: true,
            ctrl_hiddev_info: Some(ctrl_dev.clone()),

            is_opened: false,
            ctrl_hiddev: Arc::new(Mutex::new(None)),
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
                    for j in 0..=1 {
                        for i in 0..=4 {
                            let buf: [u8; 4] = [0x04, i, j, 0x00];

                            match ctrl_dev.send_feature_report(&buf) {
                                Ok(_result) => {
                                    hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                                    Ok(())
                                }

                                Err(_) => Err(HwDeviceError::InvalidResult {}).into(),
                            }?;

                            let mut buf: [u8; 5] = [0xa1, 0x00, 0x00, 0x00, 0x00];
                            match ctrl_dev.get_feature_report(&mut buf) {
                                Ok(_result) => {
                                    hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                                    Ok(())
                                }

                                Err(_) => Err(HwDeviceError::InvalidResult {}).into(),
                            }?;
                        }
                    }

                    Ok(())
                }

                0x0e => {
                    let buf: [u8; 6] = [0x0e, 0x06, 0x01, 0x01, 0x00, 0xff];

                    match ctrl_dev.send_feature_report(&buf) {
                        Ok(_result) => {
                            hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                            Ok(())
                        }

                        Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
                    }
                }

                0x0d => {
                    let buf: [u8; 11] = [
                        0x0d, 0x0b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
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

impl DeviceInfoTrait for RoccatKonePureUltra {
    type NativeDeviceInfo = self::DeviceInfo;

    fn get_device_capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities {}
    }

    fn get_device_info(&self) -> Result<self::DeviceInfo> {
        trace!("Querying the device for information...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else if !self.is_initialized {
            Err(HwDeviceError::DeviceNotInitialized {}.into())
        } else {
            let mut buf = [0; 64];
            buf[0] = 0x0f; // Query device info (HID report 0x0f)

            let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            match ctrl_dev.get_feature_report(&mut buf) {
                Ok(_result) => {
                    hexdump::hexdump_iter(&buf).for_each(|s| debug!("  {}", s));
                    let result: DeviceInfo =
                        unsafe { std::ptr::read_unaligned(buf.as_ptr() as *const _) };

                    Ok(result)
                }

                Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
            }
        }
    }
}

impl DeviceTrait for RoccatKonePureUltra {
    fn get_usb_path(&self) -> String {
        self.ctrl_hiddev_info
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
            self.send_ctrl_report(0x04)
                .unwrap_or_else(|e| error!("{}", e));
            self.wait_for_ctrl_dev().unwrap_or_else(|e| error!("{}", e));

            self.send_ctrl_report(0x0e)
                .unwrap_or_else(|e| error!("{}", e));
            self.wait_for_ctrl_dev().unwrap_or_else(|e| error!("{}", e));

            self.send_ctrl_report(0x0d)
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

impl MouseDeviceTrait for RoccatKonePureUltra {
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
                Ok(_size) => {
                    hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                    let event = match buf[0..5] {
                        // Key reports, incl. KEY_FN, ..
                        [0x03, 0x00, 0xb0, level, _] => MouseHidEvent::DpiChange(level),

                        _ => MouseHidEvent::Unknown,
                    };

                    // match event {
                    //     HidEvent::KeyDown { code } => {
                    //         // reset "to be dropped" flag
                    //         macros::DROP_CURRENT_KEY.store(false, Ordering::SeqCst);

                    //         // update our internal representation of the keyboard state
                    //         let index = util::hid_code_to_key_index(code) as usize;
                    //         keyboard::KEY_STATES.write()[index] = true;
                    //     }

                    //     HidEvent::KeyUp { code } => {
                    //         // reset "to be dropped" flag
                    //         macros::DROP_CURRENT_KEY.store(false, Ordering::SeqCst);

                    //         // update our internal representation of the keyboard state
                    //         let index = util::hid_code_to_key_index(code) as usize;
                    //         keyboard::KEY_STATES.write()[index] = false;
                    //     }

                    //     _ => { /* ignore other events */ }
                    // }

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
            let ctrl_dev = self.ctrl_hiddev.as_ref().lock();
            let ctrl_dev = ctrl_dev.as_ref().unwrap();

            let buf: [u8; 11] = [
                0x0d,
                0x0b,
                led_map[1].r,
                led_map[1].g,
                led_map[1].b,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
            ];

            match ctrl_dev.send_feature_report(&buf) {
                Ok(_result) => {
                    hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));

                    Ok(())
                }

                Err(_) => Err(HwDeviceError::InvalidResult {}.into()),
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
