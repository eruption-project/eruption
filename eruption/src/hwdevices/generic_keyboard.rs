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

use evdev_rs::enums::EV_KEY;
use hidapi::HidApi;
use log::*;
use parking_lot::RwLock;
use std::any::Any;
use std::sync::Arc;

use super::{
    DeviceCapabilities, DeviceInfoTrait, DeviceTrait, HwDeviceError, KeyboardDevice,
    KeyboardDeviceTrait, KeyboardHidEvent, KeyboardHidEventCode, LedKind, RGBA,
};

pub type Result<T> = super::Result<T>;

/// Binds the driver to a device
pub fn bind_hiddev(
    _hidapi: &HidApi,
    usb_vid: u16,
    usb_pid: u16,
    _serial: &str,
) -> super::Result<KeyboardDevice> {
    Ok(Arc::new(RwLock::new(Box::new(GenericKeyboard::bind(
        usb_vid, usb_pid,
    )))))
}

#[derive(Clone)]
/// Device specific code for a generic keyboard device
pub struct GenericKeyboard {
    usb_vid: u16,
    usb_pid: u16,
}

impl GenericKeyboard {
    /// Binds the driver to the supplied HID devices
    pub fn bind(usb_vid: u16, usb_pid: u16) -> Self {
        info!("Bound driver: Generic Keyboard Device");

        Self { usb_vid, usb_pid }
    }

    // pub(self) fn query_ctrl_report(&mut self, id: u8) -> Result<()> {
    //     trace!("Querying control device feature report");

    //     Ok(())
    // }

    // fn send_ctrl_report(&mut self, id: u8) -> Result<()> {
    //     trace!("Sending control device feature report");

    //     Ok(())
    // }

    // fn wait_for_ctrl_dev(&mut self) -> Result<()> {
    //     trace!("Waiting for control device to respond...");

    //     return Ok(());
    // }
}

impl DeviceInfoTrait for GenericKeyboard {
    fn get_device_capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities {}
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

impl DeviceTrait for GenericKeyboard {
    fn get_usb_path(&self) -> String {
        "<unsupported>".to_string()
    }

    fn get_usb_vid(&self) -> u16 {
        self.usb_vid
    }

    fn get_usb_pid(&self) -> u16 {
        self.usb_pid
    }

    fn get_support_script_file(&self) -> String {
        "keyboards/generic_keyboard".to_string()
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

    fn write_data_raw(&self, _buf: &[u8]) -> Result<()> {
        Ok(())
    }

    fn read_data_raw(&self, size: usize) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        buf.resize(size, 0);

        Ok(buf)
    }

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl KeyboardDeviceTrait for GenericKeyboard {
    fn set_status_led(&self, _led_kind: LedKind, _on: bool) -> Result<()> {
        trace!("Setting status LED state");

        Ok(())
    }

    #[inline]
    fn get_next_event(&self) -> Result<KeyboardHidEvent> {
        self.get_next_event_timeout(-1)
    }

    fn get_next_event_timeout(&self, _millis: i32) -> Result<KeyboardHidEvent> {
        trace!("Querying control device for next event");

        Err(HwDeviceError::InvalidResult {}.into())
    }

    fn ev_key_to_key_index(&self, _key: EV_KEY) -> u8 {
        0
    }

    fn hid_event_code_to_key_index(&self, _code: &KeyboardHidEventCode) -> u8 {
        0
    }

    fn hid_event_code_to_report(&self, _code: &KeyboardHidEventCode) -> u8 {
        0
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

    /// Returns the number of keys
    fn get_num_keys(&self) -> usize {
        0
    }

    /// Returns the number of rows (vertical number of keys)
    fn get_num_rows(&self) -> usize {
        0
    }

    /// Returns the number of columns (horizontal number of keys)
    fn get_num_cols(&self) -> usize {
        0
    }

    /// Returns the indices of the keys in row `row`
    fn get_row_topology(&self, _row: usize) -> &'static [u8] {
        &NIL
    }

    /// Returns the indices of the keys in column `col`
    fn get_col_topology(&self, _col: usize) -> &'static [u8] {
        &NIL
    }
}

pub const NIL: [u8; 0] = [];
