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

use evdev_rs::enums::EventCode;
use evdev_rs::Device;
use log::*;
use rlua;
use rlua::Context;
use std::any::Any;
use std::cell::RefCell;
use std::error;
use std::error::Error;
use std::fmt;
use std::fs::File;

use crate::plugins;
use crate::plugins::Plugin;
use crate::util;

pub type Result<T> = std::result::Result<T, KeyboardPluginError>;

#[derive(Debug, Clone)]
pub struct KeyboardPluginError {
    code: u32,
}

impl error::Error for KeyboardPluginError {
    fn description(&self) -> &str {
        match self.code {
            0 => "Could not peek evdev event",
            1 => "Could not convert key code",
            2 => "Not a key code",
            3 => "Could not get the name of the evdev device from udev",
            4 => "Could not open the evdev device",
            5 => "Could not create a libevdev device handle",
            _ => "Unknown error",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}

impl fmt::Display for KeyboardPluginError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

thread_local! {
    static DEVICE: RefCell<Option<Device>> = RefCell::new(None);
}

/// A plugin that listens for key events
/// Registered events can be subsequently processed by Lua scripts
pub struct KeyboardPlugin {}

impl KeyboardPlugin {
    pub fn new() -> Self {
        KeyboardPlugin {}
    }

    pub fn initialize_thread_locals(&mut self) -> Result<()> {
        match crate::util::get_evdev_from_udev() {
            Ok(filename) => match File::open(filename.clone()) {
                Ok(devfile) => match Device::new_from_fd(devfile) {
                    Ok(device) => {
                        info!("Now listening on: {}", filename);

                        info!(
                            "Input device ID: bus 0x{:x} vendor 0x{:x} product 0x{:x}",
                            device.bustype(),
                            device.vendor_id(),
                            device.product_id()
                        );
                        info!("Evdev version: {:x}", device.driver_version());
                        info!(
                            "Input device name: \"{}\"",
                            device.name().unwrap_or("<n/a>")
                        );
                        info!("Physical location: {}", device.phys().unwrap_or("<n/a>"));
                        info!("Unique identifier: {}", device.uniq().unwrap_or("<n/a>"));

                        DEVICE.with(|dev| *dev.borrow_mut() = Some(device));

                        Ok(())
                    }

                    Err(_e) => Err(KeyboardPluginError { code: 5 }),
                },

                Err(_e) => Err(KeyboardPluginError { code: 4 }),
            },

            Err(_e) => Err(KeyboardPluginError { code: 3 }),
        }
    }

    pub fn get_next_event(&self) -> Result<Option<u8>> {
        let result = DEVICE.with(|dev| {
            let result = dev
                .borrow()
                .as_ref()
                .unwrap()
                .next_event(evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING);

            match result {
                Ok(k) => {
                    trace!("Key event: {}", k.1.event_code);
                    Ok(k)
                }

                Err(e) => {
                    if e as i32 == libc::ENODEV {
                        error!("Keyboard device went away: {}", e);
                        panic!();
                    } else {
                        error!("Could not peek evdev event: {}", e);
                        Err(KeyboardPluginError { code: 0 })
                    }
                }
            }
        })?;

        match result.0 {
            evdev_rs::ReadStatus::Success => match result.1.event_code {
                EventCode::EV_KEY(code) => {
                    let result = util::ev_key_to_key_index(code);

                    if result != 0xff {
                        Ok(Some(result))
                    } else {
                        Err(KeyboardPluginError { code: 1 })
                    }
                }

                _ => Err(KeyboardPluginError { code: 2 }),
            },

            _ => Ok(None),
        }
    }
}

impl Plugin for KeyboardPlugin {
    fn get_name(&self) -> String {
        "Keyboard".to_string()
    }

    fn get_description(&self) -> String {
        "Process keyboard events".to_string()
    }

    fn initialize(&mut self) -> plugins::Result<()> {
        Ok(())
    }

    fn register_lua_funcs(&self, _lua_ctx: Context) -> rlua::Result<()> {
        Ok(())
    }

    fn main_loop_hook(&self, _ticks: u64) {}

    fn as_any(&self) -> &Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self
    }
}
