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

use evdev_rs::{Device, GrabMode};
use lazy_static::lazy_static;
use log::*;
use mlua::prelude::*;
use parking_lot::RwLock;
use std::any::Any;
use std::cell::RefCell;
use std::fs::File;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use std::time::Instant;

use crate::hwdevices;
use crate::util;

use crate::plugins::macros;

use crate::plugins::{self, Plugin};

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum KeyboardPluginError {
    #[error("Could not peek evdev event")]
    EvdevEventError {},

    #[error("Could not get the name of the evdev device from udev")]
    UdevError {},

    #[error("Could not open the evdev device")]
    EvdevError {},

    #[error("Could not create a libevdev device handle")]
    EvdevHandleError {},
}

lazy_static! {
    pub static ref KEY_STATES: Arc<RwLock<Vec<bool>>> =
        Arc::new(RwLock::new(vec![false; hwdevices::NUM_KEYS]));
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
                    Ok(mut device) => {
                        info!("Now listening on: {}", filename);

                        info!(
                            "Input device name: \"{}\"",
                            device.name().unwrap_or("<n/a>")
                        );

                        info!(
                            "Input device ID: bus 0x{:x} vendor 0x{:x} product 0x{:x}",
                            device.bustype(),
                            device.vendor_id(),
                            device.product_id()
                        );

                        // info!("Driver version: {:x}", device.driver_version());

                        info!("Physical location: {}", device.phys().unwrap_or("<n/a>"));

                        // info!("Unique identifier: {}", device.uniq().unwrap_or("<n/a>"));

                        info!("Grabbing the keyboard device exclusively");
                        device
                            .grab(GrabMode::Grab)
                            .expect("Could not grab the device, terminating now.");

                        DEVICE.with(|dev| *dev.borrow_mut() = Some(device));

                        Ok(())
                    }

                    Err(_e) => Err(KeyboardPluginError::EvdevHandleError {}.into()),
                },

                Err(_e) => Err(KeyboardPluginError::EvdevError {}.into()),
            },

            Err(_e) => Err(KeyboardPluginError::UdevError {}.into()),
        }
    }

    pub fn get_next_event(&self) -> Result<Option<evdev_rs::InputEvent>> {
        let result = DEVICE.with(
            |dev| -> Result<(evdev_rs::ReadStatus, evdev_rs::InputEvent)> {
                let result = dev
                    .borrow()
                    .as_ref()
                    .unwrap()
                    .next_event(evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING);

                match result {
                    Ok(k) => {
                        trace!("Key event: {:?}", k.1);

                        // update AFK timer
                        *crate::LAST_INPUT_TIME.lock() = Instant::now();

                        // update our internal representation of the keyboard state
                        if let evdev_rs::enums::EventCode::EV_KEY(ref code) = k.1.event_code {
                            let is_pressed = k.1.value > 0;
                            let index = util::ev_key_to_key_index(code.clone()) as usize;

                            KEY_STATES.write()[index] = is_pressed;

                            // reset "to be dropped" flag
                            macros::DROP_CURRENT_KEY.store(false, Ordering::SeqCst);
                        } else {
                            // error!("Invalid event code received")
                        }

                        Ok(k)
                    }

                    Err(e) => {
                        if e.raw_os_error().unwrap() == libc::ENODEV {
                            error!("Fatal: Keyboard device went away: {}", e);

                            crate::QUIT.store(true, Ordering::SeqCst);
                            Err(KeyboardPluginError::EvdevEventError {}.into())
                        } else {
                            error!("Fatal: Could not peek evdev event: {}", e);

                            crate::QUIT.store(true, Ordering::SeqCst);
                            Err(KeyboardPluginError::EvdevEventError {}.into())
                        }
                    }
                }
            },
        )?;

        match result.0 {
            evdev_rs::ReadStatus::Success => Ok(Some(result.1)),

            _ => Ok(None),
        }
    }

    pub(crate) fn get_key_state(key_index: usize) -> bool {
        KEY_STATES.read()[key_index]
    }
}

#[async_trait::async_trait]
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

    fn register_lua_funcs(&self, lua_ctx: &Lua) -> mlua::Result<()> {
        let globals = lua_ctx.globals();

        let get_key_state = lua_ctx
            .create_function(|_, key_index: usize| Ok(KeyboardPlugin::get_key_state(key_index)))?;
        globals.set("get_key_state", get_key_state)?;

        Ok(())
    }

    async fn main_loop_hook(&self, _ticks: u64) {}

    fn sync_main_loop_hook(&self, _ticks: u64) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
