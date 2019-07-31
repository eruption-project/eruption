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
use failure::Fail;
use log::*;
use rlua;
use rlua::Context;
use std::any::Any;
use std::cell::RefCell;
use std::fs::File;

use crate::plugins::{self, Plugin};
use crate::util;

pub type Result<T> = std::result::Result<T, KeyboardPluginError>;

#[derive(Debug, Fail)]
pub enum KeyboardPluginError {
    #[fail(display = "Could not peek evdev event")]
    EvdevEventError {},

    #[fail(display = "Could not convert key code")]
    KeyCodeConversionError {},

    #[fail(display = "Not a key code")]
    InvalidKeyCode {},

    #[fail(display = "Could not get the name of the evdev device from udev")]
    UdevError {},

    #[fail(display = "Could not open the evdev device")]
    EvdevError {},

    #[fail(display = "Could not create a libevdev device handle")]
    EvdevHandleError {},
    // #[fail(display = "Unknown error: {}", description)]
    // UnknownError { description: String },
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
                        // info!("Evdev version: {:x}", device.driver_version());
                        info!(
                            "Input device name: \"{}\"",
                            device.name().unwrap_or("<n/a>")
                        );
                        info!("Physical location: {}", device.phys().unwrap_or("<n/a>"));
                        // info!("Unique identifier: {}", device.uniq().unwrap_or("<n/a>"));

                        DEVICE.with(|dev| *dev.borrow_mut() = Some(device));

                        Ok(())
                    }

                    Err(_e) => Err(KeyboardPluginError::EvdevHandleError {}),
                },

                Err(_e) => Err(KeyboardPluginError::EvdevError {}),
            },

            Err(_e) => Err(KeyboardPluginError::UdevError {}),
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
                        Err(KeyboardPluginError::EvdevEventError {})
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
                        Err(KeyboardPluginError::KeyCodeConversionError {})
                    }
                }

                _ => Err(KeyboardPluginError::InvalidKeyCode {}),
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

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
