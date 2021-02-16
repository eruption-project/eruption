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

use bitvec::prelude::*;
use evdev_rs::enums::EV_KEY;
use hidapi::HidApi;
use log::*;
use parking_lot::{Mutex, RwLock};
// use std::sync::atomic::Ordering;
use std::time::Duration;
use std::{any::Any, thread};
use std::{mem::size_of, sync::Arc};

use crate::constants;

use super::{
    DeviceCapabilities, DeviceInfoTrait, DeviceTrait, HwDeviceError, MouseDevice, MouseDeviceTrait,
    MouseHidEvent, RGBA,
};

pub type Result<T> = super::Result<T>;

pub const SUB_DEVICE: i32 = 2; // USB HID sub-device to bind to

/// Binds the driver to a device
pub fn bind_hiddev(
    hidapi: &HidApi,
    usb_vid: u16,
    usb_pid: u16,
    serial: &str,
) -> super::Result<MouseDevice> {
    let ctrl_dev = hidapi.device_list().find(|&device| {
        device.vendor_id() == usb_vid
            && device.product_id() == usb_pid
            && device.serial_number().unwrap_or_else(|| "") == serial
            && device.interface_number() == SUB_DEVICE
    });

    if ctrl_dev.is_none() {
        Err(HwDeviceError::EnumerationError {}.into())
    } else {
        Ok(Arc::new(RwLock::new(Box::new(RoccatNyth::bind(
            &ctrl_dev.unwrap(),
        )))))
    }
}

/// ROCCAT Nyth info struct (sent as HID report)
#[derive(Debug, Copy, Clone)]
#[repr(C, packed)]
pub struct DeviceInfo {
    pub report_id: u8,
    pub size: u8, // always 0x06
    pub firmware_version: u8,
    pub reserved1: u8,
    pub reserved2: u8,
    pub reserved3: u8,
}

#[derive(Clone)]
/// Device specific code for the ROCCAT Nyth mouse
pub struct RoccatNyth {
    pub is_initialized: bool,

    pub is_bound: bool,
    pub ctrl_hiddev_info: Option<hidapi::DeviceInfo>,

    pub is_opened: bool,
    pub ctrl_hiddev: Arc<Mutex<Option<hidapi::HidDevice>>>,

    pub button_states: Arc<Mutex<BitVec>>,
}

impl RoccatNyth {
    /// Binds the driver to the supplied HID device
    pub fn bind(ctrl_dev: &hidapi::DeviceInfo) -> Self {
        info!("Bound driver: ROCCAT Nyth");

        Self {
            is_initialized: false,

            is_bound: true,
            ctrl_hiddev_info: Some(ctrl_dev.clone()),

            is_opened: false,
            ctrl_hiddev: Arc::new(Mutex::new(None)),

            button_states: Arc::new(Mutex::new(bitvec![0; constants::MAX_MOUSE_BUTTONS])),
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
    //             0x15 => {
    //                 let buf: [u8; 3] = [0x15, 0x00, 0x01];

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

impl DeviceInfoTrait for RoccatNyth {
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
            buf[0] = 0x09; // Query device info (HID report 0x09)

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

impl DeviceTrait for RoccatNyth {
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

    fn get_support_script_file(&self) -> String {
        "mice/roccat_nyth".to_string()
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
            self.query_ctrl_report(0x0f)
                .unwrap_or_else(|e| error!("Step 1: {}", e));
            self.wait_for_ctrl_dev()
                .unwrap_or_else(|e| error!("Wait 1: {}", e));

            // self.send_ctrl_report(0x15)
            //     .unwrap_or_else(|e| error!("Step 2: {}", e));
            // self.wait_for_ctrl_dev()
            //     .unwrap_or_else(|e| error!("Wait 2: {}", e));

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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl MouseDeviceTrait for RoccatNyth {
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
                    // hexdump::hexdump_iter(&buf).for_each(|s| trace!("  {}", s));
                    hexdump::hexdump_iter(&buf).for_each(|s| debug!("  {}", s));

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

    fn send_led_map(&mut self, _led_map: &[RGBA]) -> Result<()> {
        trace!("Setting LEDs from supplied map...");

        if !self.is_bound {
            Err(HwDeviceError::DeviceNotBound {}.into())
        } else if !self.is_opened {
            Err(HwDeviceError::DeviceNotOpened {}.into())
        } else if !self.is_initialized {
            Err(HwDeviceError::DeviceNotInitialized {}.into())
        } else {
            // match *self.led_hiddev.lock() {
            //     Some(ref led_dev) => {
            //         // TODO: Implement this
            //         Ok(())
            //     }

            //     None => Err(HwDeviceError::DeviceNotOpened {}.into()),
            // }

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
            // TODO: Implement this
            thread::sleep(Duration::from_millis(constants::DEVICE_SETTLE_MILLIS));

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
            // TODO: Implement this
            thread::sleep(Duration::from_millis(constants::DEVICE_SETTLE_MILLIS));

            Ok(())
        }
    }

    fn has_secondary_device(&self) -> bool {
        true
    }
}
