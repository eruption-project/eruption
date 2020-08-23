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
use std::any::Any;
use std::cell::RefCell;
use std::fs::File;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Instant;

use crate::macros;
use crate::util;

use crate::plugins::{self, Plugin};

pub const MAX_MOUSE_BUTTONS: usize = 16;

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum MousePluginError {
    #[error("Could not peek evdev event")]
    EvdevEventError {},

    #[error("Evdev device is not available")]
    EvdevNoDevError {},

    // #[error("Could not get the name of the evdev device from udev")]
    // UdevError {},
    #[error("Could not open the evdev device")]
    EvdevError {},

    #[error("Could not create a libevdev device handle")]
    EvdevHandleError {},
}

lazy_static! {
    static ref BUTTON_STATES: Arc<RwLock<Vec<bool>>> =
        Arc::new(RwLock::new(vec![false; MAX_MOUSE_BUTTONS]));

    // cached value
    static ref GRAB_MOUSE: AtomicBool = {
        let config = &*crate::CONFIG.lock();
        let grab_mouse = config
            .as_ref()
            .unwrap()
            .get::<bool>("global.grab_mouse")
            .unwrap_or_else(|_| true);

        AtomicBool::from(grab_mouse)
    };
}

thread_local! {
    static DEVICE: RefCell<Option<Device>> = RefCell::new(None);
}

/// A plugin that listens for mouse events
/// Registered events can be subsequently processed by Lua scripts
pub struct MousePlugin {}

impl MousePlugin {
    pub fn new() -> Self {
        MousePlugin {}
    }

    pub fn initialize_thread_locals(&mut self) -> Result<()> {
        let filename =
            util::get_evdev_mouse_from_udev().or_else(|_e| util::get_mouse_dev_from_udev())?;

        match File::open(&filename) {
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

                    if GRAB_MOUSE.load(Ordering::SeqCst) {
                        info!("Grabbing the mouse device exclusively");
                        device
                            .grab(GrabMode::Grab)
                            .expect("Could not grab the device, terminating now.");
                    }

                    DEVICE.with(|dev| *dev.borrow_mut() = Some(device));

                    Ok(())
                }

                Err(_e) => Err(MousePluginError::EvdevHandleError {}.into()),
            },

            Err(_e) => Err(MousePluginError::EvdevError {}.into()),
        }
    }

    pub fn get_next_event(&self) -> Result<Option<evdev_rs::InputEvent>> {
        let result = DEVICE.with(
            |dev| -> Result<(evdev_rs::ReadStatus, evdev_rs::InputEvent)> {
                if let Some(dev) = dev.borrow().as_ref() {
                    let result =
                        dev.next_event(evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING);

                    match result {
                        Ok(k) => {
                            trace!("Mouse event: {:?}", k.1);

                            // update AFK timer
                            *crate::LAST_INPUT_TIME.lock() = Instant::now();

                            // reset "to be dropped" flag
                            macros::DROP_CURRENT_MOUSE_INPUT.store(false, Ordering::SeqCst);

                            // update our internal representation of the mouse state
                            let event_code = k.1.event_code.clone();
                            if let evdev_rs::enums::EventCode::EV_KEY(code) = event_code {
                                let is_pressed = k.1.value > 0;
                                let index = util::ev_key_to_button_index(code).unwrap() as usize;

                                BUTTON_STATES.write().unwrap()[index] = is_pressed;
                            } else if let evdev_rs::enums::EventCode::EV_REL(code) = event_code {
                                if code != evdev_rs::enums::EV_REL::REL_WHEEL
                                    && code != evdev_rs::enums::EV_REL::REL_HWHEEL
                                    && code != evdev_rs::enums::EV_REL::REL_WHEEL_HI_RES
                                    && code != evdev_rs::enums::EV_REL::REL_HWHEEL_HI_RES
                                {
                                    // directly mirror pointer motion events, to reduce input lag.
                                    // This currently prohibits further manipulation of pointer motion events
                                    if GRAB_MOUSE.load(Ordering::SeqCst) {
                                        macros::UINPUT_TX
                                            .lock()
                                            .as_ref()
                                            .unwrap()
                                            .send(macros::Message::MirrorMouseEventImmediate(
                                                k.1.clone(),
                                            ))
                                            .unwrap_or_else(|e| {
                                                error!(
                                                    "Could not send a pending mouse event: {}",
                                                    e
                                                )
                                            });
                                    }
                                }
                            } else {
                                // error!("Invalid event code received")
                            }

                            Ok(k)
                        }

                        Err(_e) => {
                            // if e.raw_os_error().unwrap() == libc::ENODEV {
                            //     error!("Mouse device went away: {}", e);

                            //     crate::QUIT.store(true, Ordering::SeqCst);
                            //     Err(MousePluginError::EvdevEventError {}.into())
                            // } else {
                            //     error!("Could not peek evdev event: {}", e);

                            //     crate::QUIT.store(true, Ordering::SeqCst);
                            //     Err(MousePluginError::EvdevEventError {}.into())
                            // }

                            error!("Could not get mouse events");
                            Err(MousePluginError::EvdevEventError {}.into())
                        }
                    }
                } else {
                    Err(MousePluginError::EvdevNoDevError {}.into())
                }
            },
        )?;

        match result.0 {
            evdev_rs::ReadStatus::Success => Ok(Some(result.1)),

            _ => Ok(None),
        }
    }

    pub(crate) fn get_button_state(button_index: usize) -> bool {
        BUTTON_STATES.read().unwrap()[button_index]
    }
}

#[async_trait::async_trait]
impl Plugin for MousePlugin {
    fn get_name(&self) -> String {
        "Mouse".to_string()
    }

    fn get_description(&self) -> String {
        "Process mouse events".to_string()
    }

    fn initialize(&mut self) -> plugins::Result<()> {
        Ok(())
    }

    fn register_lua_funcs(&self, lua_ctx: &Lua) -> mlua::Result<()> {
        let globals = lua_ctx.globals();

        let get_button_state = lua_ctx.create_function(|_, button_index: usize| {
            Ok(MousePlugin::get_button_state(button_index))
        })?;
        globals.set("get_button_state", get_button_state)?;

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
