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

use std::{any::Any, collections::HashMap, sync::Arc};

use openrgb::*;
use rgb::RGB8;
use tokio::net::TcpStream;
use tokio::runtime::{Builder, Runtime};
use tracing::{debug, error, trace};
use tracing_mutex::stdsync::Mutex;

use crate::constants;
use crate::hwdevices::{
    self, Capability, DeviceClass, DeviceQuirks, DeviceStatus, DeviceZoneAllocationExt, Zone,
};

use crate::hwdevices::{
    DeviceCapabilities, DeviceExt, DeviceInfoExt, MiscDeviceExt, MouseDeviceExt, Result, RGBA,
};
use crate::util::ratelimited;

const NUM_LEDS: usize = constants::CANVAS_SIZE;

#[derive(Clone)]
pub struct OpenRgbBridge {
    rt: Arc<Runtime>,

    connection: Arc<Mutex<Option<OpenRGB<TcpStream>>>>,
    resource: String,

    // device specific configuration options
    pub brightness: i32,

    pub allocated_zone: Zone,

    pub is_opened: bool,
    pub has_failed: bool,
}

impl OpenRgbBridge {
    /// Binds the driver to the supplied device
    pub fn bind(resource: String) -> Self {
        debug!("Bound driver: OpenRGB virtual bridge device");

        let runtime = Builder::new_multi_thread()
            // .worker_threads(8)
            .thread_name("OpenRGB bridge".to_string())
            .enable_io()
            .build()
            .unwrap();

        Self {
            rt: Arc::new(runtime),

            connection: Arc::new(Mutex::new(None)),
            resource,

            brightness: 100,
            allocated_zone: Zone::new(
                0,
                0,
                constants::CANVAS_WIDTH as i32,
                constants::CANVAS_HEIGHT as i32,
                true,
            ),
            is_opened: false,
            has_failed: false,
        }
    }
}

impl DeviceInfoExt for OpenRgbBridge {
    fn get_device_capabilities(&self) -> DeviceCapabilities {
        DeviceCapabilities::from([Capability::Virtual, Capability::RgbLighting])
    }

    fn get_device_quirks(&self) -> hwdevices::DeviceQuirks {
        DeviceQuirks::from([])
    }

    fn get_device_info(&self) -> Result<hwdevices::DeviceInfo> {
        trace!("Querying the device for information...");

        let result = hwdevices::DeviceInfo::new(0);
        Ok(result)
    }

    fn get_firmware_revision(&self) -> String {
        "0".to_string()
    }
}

impl DeviceExt for OpenRgbBridge {
    fn get_dev_paths(&self) -> Vec<String> {
        vec!["openrgb_bridge".to_string()]
    }

    fn get_usb_vid(&self) -> u16 {
        0x0E00
    }

    fn get_usb_pid(&self) -> u16 {
        0x0001
    }

    fn get_serial(&self) -> Option<&str> {
        Some("00000000")
    }

    fn get_support_script_file(&self) -> String {
        "misc/openrgb_bridge".to_string()
    }

    fn open(&mut self, _api: &hidapi::HidApi) -> Result<()> {
        trace!("Opening devices now...");

        self.rt.block_on(async {
            // let split = self.resource.rsplitn(1, ':').collect_vec();
            //
            // let result = match split.len() {
            //     0 => Ok(("localhost", "6742")),
            //     1 => Ok((split[0].trim(), "6742")),
            //     2 => Ok((split[1].trim(), split[0].trim())),
            //
            //     _ => Err(HwDeviceError::InvalidDevice {}),
            // };

            match OpenRGB::connect_to(&self.resource).await {
                Ok(connection) => *self.connection.lock().unwrap() = Some(connection),
                Err(e) => error!("Could not connect to OpenRGB server: {e}"),
            }
        });

        self.rt.block_on(async {
            if let Some(ref connection) = *self.connection.lock().unwrap() {
                let _ = connection
                    .set_name("Eruption to OpenRGB bridge")
                    .await
                    .map_err(|e| error!("Could not set the name of the client connection: {e}"));

                let _ = connection
                    .set_custom_mode(0)
                    .await
                    .map_err(|e| error!("Could not set 'custom mode' on a controller: {e}"));
            }
        });

        self.is_opened = true;

        Ok(())
    }

    fn close_all(&mut self) -> Result<()> {
        trace!("Closing devices now...");

        *self.connection.lock().unwrap() = None;

        Ok(())
    }

    fn has_failed(&self) -> Result<bool> {
        Ok(self.has_failed)
    }

    fn fail(&mut self) -> Result<()> {
        self.has_failed = true;
        Ok(())
    }

    fn send_init_sequence(&mut self) -> Result<()> {
        trace!("Sending device init sequence...");

        // some devices need many iterations to sync, so we need to try multiple times
        // for _ in 0..8 {
        //     let led_map = [RGBA {
        //         r: 0,
        //         g: 0,
        //         b: 0,
        //         a: 0,
        //     }; NUM_LEDS];
        //
        //     self.send_led_map(&led_map)?;
        // }

        Ok(())
    }

    fn send_shutdown_sequence(&mut self) -> Result<()> {
        trace!("Sending device shutdown sequence...");

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

        if self.is_opened {
            match *self.connection.lock().unwrap() {
                Some(ref connection) => {
                    if self.allocated_zone.enabled {
                        let colors = led_map
                            .iter()
                            .map(|e| RGB8 {
                                r: (e.r as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                                g: (e.g as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                                b: (e.b as f32 * (self.brightness as f32 / 100.0)).floor() as u8,
                            })
                            .collect::<Vec<RGB8>>();

                        self.rt.block_on(async {
                            let _ = connection
                                .update_zone_leds(0, 0, colors)
                                .await
                                .map_err(|e| {
                                    ratelimited::error!(
                                        "Could not send the LED map to OpenRGB: {e}"
                                    )
                                });
                        });

                        Ok(())
                    } else {
                        Ok(())
                    }
                }

                None => Ok(()), //Err(HwDeviceError::DeviceNotOpened {}.into()),
            }
        } else {
            Ok(())
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
        DeviceClass::Misc
    }

    fn as_keyboard_device(&self) -> Option<&(dyn hwdevices::KeyboardDeviceExt + Sync + Send)> {
        None
    }

    fn as_keyboard_device_mut(
        &mut self,
    ) -> Option<&mut (dyn hwdevices::KeyboardDeviceExt + Sync + Send)> {
        None
    }

    fn as_misc_device(&self) -> Option<&(dyn MiscDeviceExt + Sync + Send)> {
        Some(self)
    }

    fn as_misc_device_mut(&mut self) -> Option<&mut (dyn MiscDeviceExt + Sync + Send)> {
        Some(self)
    }

    #[cfg(not(target_os = "windows"))]
    fn get_evdev_input_rx(&self) -> &Option<flume::Receiver<Option<evdev_rs::InputEvent>>> {
        &None
    }

    #[cfg(not(target_os = "windows"))]
    fn set_evdev_input_rx(&mut self, _rx: Option<flume::Receiver<Option<evdev_rs::InputEvent>>>) {
        // do nothing
    }
}

impl DeviceZoneAllocationExt for OpenRgbBridge {
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

impl MiscDeviceExt for OpenRgbBridge {
    fn has_input_device(&self) -> bool {
        false
    }
}
