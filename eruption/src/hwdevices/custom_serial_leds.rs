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

use std::{any::Any, collections::HashMap, path::PathBuf, sync::Arc};

use log::*;
use parking_lot::Mutex;
use serialport::SerialPort;
use std::time::Duration;

use crate::hwdevices::DeviceStatus;

use super::{
    DeviceCapabilities, DeviceInfoTrait, DeviceTrait, HwDeviceError, MiscDeviceTrait,
    MouseDeviceTrait, RGBA,
};

const BAUD_RATE: u32 = 460800;
const NUM_LEDS: usize = 80;

pub type Result<T> = super::Result<T>;

#[derive(Clone)]
pub struct CustomSerialLeds {
    device_file: PathBuf,
    port: Arc<Mutex<Option<Box<dyn SerialPort>>>>,

    // device specific configuration options
    pub brightness: i32,
}

impl CustomSerialLeds {
    /// Binds the driver to the supplied device
    pub fn bind(serial_port: PathBuf) -> Self {
        info!("Bound driver: Adalight Custom Serial LEDs");

        Self {
            device_file: serial_port,
            port: Arc::new(Mutex::new(None)),

            brightness: 100,
        }
    }
}

impl DeviceInfoTrait for CustomSerialLeds {
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

impl DeviceTrait for CustomSerialLeds {
    fn get_usb_path(&self) -> String {
        "<unsupported>".to_string()
    }

    fn get_usb_vid(&self) -> u16 {
        0
    }

    fn get_usb_pid(&self) -> u16 {
        0
    }

    fn get_serial(&self) -> Option<&str> {
        None
    }

    fn get_support_script_file(&self) -> String {
        "misc/custom_serial_leds".to_string()
    }

    fn open(&mut self, _api: &hidapi::HidApi) -> Result<()> {
        trace!("Opening devices now...");

        let port = serialport::new(self.device_file.to_string_lossy(), BAUD_RATE)
            .timeout(Duration::from_millis(1000))
            // .data_bits(DataBits::Eight)
            // .stop_bits(StopBits::One)
            // .parity(Parity::Even)
            .open();

        match port {
            Ok(port) => *self.port.lock() = Some(port),

            Err(_e) => return Err(HwDeviceError::DeviceOpenError {}.into()),
        }

        Ok(())
    }

    fn close_all(&mut self) -> Result<()> {
        trace!("Closing devices now...");

        Ok(())
    }

    fn has_failed(&self) -> Result<bool> {
        Ok(false)
    }

    fn send_init_sequence(&mut self) -> Result<()> {
        trace!("Sending device init sequence...");

        // some devices need many iterations to sync, so we need to try multiple times
        for _ in 0..8 {
            let led_map = [RGBA {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            }; NUM_LEDS];

            self.send_led_map(&led_map)?;
        }

        Ok(())
    }

    fn is_initialized(&self) -> Result<bool> {
        Ok(true)
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
        None
    }

    fn as_mouse_device_mut(&mut self) -> Option<&mut dyn MouseDeviceTrait> {
        None
    }
}

impl MiscDeviceTrait for CustomSerialLeds {
    fn has_input_device(&self) -> bool {
        false
    }

    fn set_local_brightness(&mut self, brightness: i32) -> Result<()> {
        trace!("Setting device specific brightness");

        self.brightness = brightness;

        Ok(())
    }

    fn get_local_brightness(&self) -> Result<i32> {
        trace!("Querying device specific brightness");

        Ok(self.brightness)
    }

    fn send_led_map(&mut self, led_map: &[RGBA]) -> Result<()> {
        trace!("Setting LEDs from supplied map...");

        match *self.port.lock() {
            Some(ref mut port) => {
                const HEADER_OFFSET: usize = 6;

                let mut buffer: [u8; HEADER_OFFSET + (NUM_LEDS * 3)] =
                    [0x00; HEADER_OFFSET + (NUM_LEDS * 3)];

                buffer[0..HEADER_OFFSET].clone_from_slice(&[
                    b'A',
                    b'd',
                    b'a',
                    0x00,
                    NUM_LEDS as u8,
                    0x00 ^ NUM_LEDS as u8 ^ 0x55,
                ]);

                let mut cntr = 0;
                for e in led_map[0..NUM_LEDS].iter() {
                    buffer[HEADER_OFFSET + cntr + 0] =
                        (e.r as f32 * (self.brightness as f32 / 100.0)).round() as u8;
                    buffer[HEADER_OFFSET + cntr + 1] =
                        (e.g as f32 * (self.brightness as f32 / 100.0)).round() as u8;
                    buffer[HEADER_OFFSET + cntr + 2] =
                        (e.b as f32 * (self.brightness as f32 / 100.0)).round() as u8;

                    cntr += 3;
                }

                port.write_all(&buffer)?;

                Ok(())
            }

            None => Err(HwDeviceError::DeviceNotOpened {}.into()),
        }
    }

    fn set_led_init_pattern(&mut self) -> Result<()> {
        trace!("Setting LED init pattern...");

        // some devices need many iterations to sync, so we need to try multiple times
        for _ in 0..8 {
            let led_map = [RGBA {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            }; NUM_LEDS];

            self.send_led_map(&led_map)?;
        }

        Ok(())
    }

    fn set_led_off_pattern(&mut self) -> Result<()> {
        trace!("Setting LED off pattern...");

        // some devices need many iterations to sync, so we need to try multiple times
        for _ in 0..8 {
            let led_map = [RGBA {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            }; NUM_LEDS];

            self.send_led_map(&led_map)?;
        }

        Ok(())
    }
}
