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

use super::{
    DeviceCapabilities, DeviceInfoTrait, DeviceTrait, HwDeviceError, MouseDeviceTrait,
    MouseHidEvent, RGBA,
};

pub type Result<T> = super::Result<T>;

// pub const NUM_KEYS: usize = 0;
// pub const KEYBOARD_SUB_DEVICE: usize = 0;

#[derive(Clone)]
/// Device specific code for a generic mouse device
pub struct GenericMouse {
    device_path: String,
}

impl GenericMouse {
    /// Binds the driver to the supplied HID device
    pub fn bind(device_path: &str) -> Self {
        info!("Bound driver: Generic Mouse Device");

        Self {
            device_path: device_path.to_string(),
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

impl DeviceTrait for GenericMouse {
    fn get_usb_path(&self) -> String {
        self.device_path.clone()
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
}

impl MouseDeviceTrait for GenericMouse {
    #[inline]
    fn get_next_event(&self) -> Result<MouseHidEvent> {
        self.get_next_event_timeout(-1)
    }

    fn get_next_event_timeout(&self, _millis: i32) -> Result<MouseHidEvent> {
        trace!("Querying control device for next event");

        Err(HwDeviceError::InvalidResult {}.into())
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
