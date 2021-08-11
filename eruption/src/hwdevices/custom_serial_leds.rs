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

use std::{any::Any, path::PathBuf, sync::Arc};

use log::*;
use parking_lot::Mutex;
use serialport::SerialPort;
use std::time::Duration;

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
}

impl CustomSerialLeds {
    /// Binds the driver to the supplied device
    pub fn bind(serial_port: PathBuf) -> Self {
        info!("Bound driver: Adalight Custom Serial LEDs");

        Self {
            device_file: serial_port,
            port: Arc::new(Mutex::new(None)),
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
    fn send_led_map(&mut self, led_map: &[RGBA]) -> Result<()> {
        trace!("Setting LEDs from supplied map...");

        match *self.port.lock() {
            Some(ref mut port) => {
                const HEADER_OFFSET: usize = 6;

                let mut buffer: [u8; HEADER_OFFSET + (NUM_LEDS * 3)] =
                    [0x00; HEADER_OFFSET + (NUM_LEDS * 3)];

                buffer[0..HEADER_OFFSET].clone_from_slice(&[
                    'A' as u8,
                    'd' as u8,
                    'a' as u8,
                    0x00,
                    NUM_LEDS as u8,
                    0x00 ^ NUM_LEDS as u8 ^ 0x55,
                ]);

                let mut cntr = 0;
                for e in led_map[0..NUM_LEDS].iter() {
                    buffer[HEADER_OFFSET + cntr + 0] = e.r;
                    buffer[HEADER_OFFSET + cntr + 1] = e.g;
                    buffer[HEADER_OFFSET + cntr + 2] = e.b;

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
