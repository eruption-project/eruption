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

    Copyright (c) 2019-2022, The Eruption Development Team
*/

use std::{any::Any, collections::HashMap, sync::Arc};

use evdev_rs::enums::EV_KEY;
use hidapi::HidApi;
use log::*;
use parking_lot::RwLock;

use crate::hwdevices::DeviceStatus;

use super::{
    Capability, DeviceCapabilities, DeviceInfoTrait, DeviceTrait, HwDeviceError, MouseDevice,
    MouseDeviceTrait, MouseHidEvent, RGBA,
};

pub type Result<T> = super::Result<T>;

/// Binds the driver to a device
pub fn bind_hiddev(
    _hidapi: &HidApi,
    usb_vid: u16,
    usb_pid: u16,
    _serial: &str,
) -> super::Result<MouseDevice> {
    Ok(Arc::new(RwLock::new(Box::new(GenericMouse::bind(
        usb_vid, usb_pid,
    )))))
}

#[derive(Clone)]
/// Device specific code for a generic mouse device
pub struct GenericMouse {
    usb_vid: u16,
    usb_pid: u16,

    pub has_failed: bool,
}

impl GenericMouse {
    /// Binds the driver to the supplied HID devices
    pub fn bind(usb_vid: u16, usb_pid: u16) -> Self {
        info!("Bound driver: Generic Mouse Device");

        Self {
            usb_vid,
            usb_pid,
            has_failed: false,
        }
    }

    //     fn send_ctrl_report(&mut self, _id: u8) -> Result<()> {
    //         trace!("Sending control device feature report");

    //         Ok(())
    //     }

    //     fn wait_for_ctrl_dev(&mut self) -> Result<()> {
    //         trace!("Waiting for control device to respond...");

    //         Ok(())
    //     }
}

impl DeviceInfoTrait for GenericMouse {
    fn get_device_capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities::from([Capability::Mouse])
    }

    fn get_device_info(&self) -> Result<super::DeviceInfo> {
        trace!("Querying the device for information...");

        let result = super::DeviceInfo::new(0);
        Ok(result)
    }

    fn get_firmware_revision(&self) -> String {
        "<not supported>".to_string()
    }
}

impl DeviceTrait for GenericMouse {
    fn get_usb_path(&self) -> String {
        "<unsupported>".to_string()
    }

    fn get_usb_vid(&self) -> u16 {
        self.usb_vid
    }

    fn get_usb_pid(&self) -> u16 {
        self.usb_pid
    }

    fn get_serial(&self) -> Option<&str> {
        None
    }

    fn get_support_script_file(&self) -> String {
        "mice/generic_mouse".to_string()
    }

    fn open(&mut self, _api: &hidapi::HidApi) -> Result<()> {
        trace!("Opening HID devices now...");

        Ok(())
    }

    fn close_all(&mut self) -> Result<()> {
        trace!("Closing HID devices now...");

        Ok(())
    }

    fn send_init_sequence(&mut self) -> Result<()> {
        trace!("Sending device init sequence...");

        Ok(())
    }

    fn is_initialized(&self) -> Result<bool> {
        Ok(true)
    }

    fn has_failed(&self) -> Result<bool> {
        Ok(false)
    }

    fn fail(&mut self) -> Result<()> {
        self.has_failed = true;
        Ok(())
    }

    fn write_data_raw(&self, _buf: &[u8]) -> Result<()> {
        Ok(())
    }

    fn read_data_raw(&self, size: usize) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.resize(size, 0);

        Ok(buf)
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
        Some(self as &dyn MouseDeviceTrait)
    }

    fn as_mouse_device_mut(&mut self) -> Option<&mut dyn MouseDeviceTrait> {
        Some(self as &mut dyn MouseDeviceTrait)
    }
}

impl MouseDeviceTrait for GenericMouse {
    fn get_profile(&self) -> Result<i32> {
        trace!("Querying device profile config");

        Err(HwDeviceError::OpNotSupported {}.into())
    }

    fn set_profile(&mut self, _profile: i32) -> Result<()> {
        trace!("Setting device profile config");

        Err(HwDeviceError::OpNotSupported {}.into())
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

    fn get_local_brightness(&self) -> Result<i32> {
        trace!("Querying device specific brightness");

        Err(HwDeviceError::OpNotSupported {}.into())
    }

    fn set_local_brightness(&mut self, _brightness: i32) -> Result<()> {
        trace!("Setting device specific brightness");

        Err(HwDeviceError::OpNotSupported {}.into())
    }

    #[inline]
    fn get_next_event(&self) -> Result<MouseHidEvent> {
        self.get_next_event_timeout(-1)
    }

    fn get_next_event_timeout(&self, _millis: i32) -> Result<MouseHidEvent> {
        trace!("Querying control device for next event");

        Err(HwDeviceError::InvalidResult {}.into())
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

        Ok(())
    }

    fn set_led_init_pattern(&mut self) -> Result<()> {
        trace!("Setting LED init pattern...");

        Ok(())
    }

    fn set_led_off_pattern(&mut self) -> Result<()> {
        trace!("Setting LED off pattern...");

        Ok(())
    }

    fn has_secondary_device(&self) -> bool {
        false
    }
}
