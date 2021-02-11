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

// use async_macros::join;
use clap::{App, Arg};
use crossbeam::channel::{unbounded, Receiver, Select, Sender};
use evdev_rs::{Device, GrabMode};
use futures::future::join_all;
use hotwatch::{
    blocking::{Flow, Hotwatch},
    Event,
};
use lazy_static::lazy_static;
use log::*;
use parking_lot::{Condvar, Mutex};
use std::env;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::u64;
use std::{collections::HashSet, thread};

mod util;

mod hwdevices;
use hwdevices::{KeyboardDevice, KeyboardHidEvent, MouseDevice, MouseHidEvent};

mod constants;
mod dbus_interface;
mod events;
mod plugin_manager;
mod plugins;
mod profiles;
mod scripting;
mod state;

use plugins::macros;
use profiles::Profile;
use scripting::manifest::Manifest;
use scripting::script;

lazy_static! {
    /// Managed keyboard devices
    pub static ref KEYBOARD_DEVICES: Arc<Mutex<Vec<hwdevices::KeyboardDevice>>> = Arc::new(Mutex::new(Vec::new()));

    /// Managed mouse devices
    pub static ref MOUSE_DEVICES: Arc<Mutex<Vec<hwdevices::MouseDevice>>> = Arc::new(Mutex::new(Vec::new()));

    /// The currently active slot (1-4)
    pub static ref ACTIVE_SLOT: AtomicUsize = AtomicUsize::new(0);

    /// The custom names of each slot
    pub static ref SLOT_NAMES: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));

    /// The slot to profile associations
    pub static ref SLOT_PROFILES: Arc<Mutex<Option<Vec<PathBuf>>>> = Arc::new(Mutex::new(None));

    /// The currently active profile
    pub static ref ACTIVE_PROFILE: Arc<Mutex<Option<Profile>>> = Arc::new(Mutex::new(None));

    /// Contains the file name part of the active profile;
    /// may be used to switch profiles at runtime
    pub static ref ACTIVE_PROFILE_NAME: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    /// The profile that was active before we entered AFK mode
    pub static ref ACTIVE_PROFILE_NAME_BEFORE_AFK: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    /// The current "pipeline" of scripts
    pub static ref ACTIVE_SCRIPTS: Arc<Mutex<Vec<Manifest>>> = Arc::new(Mutex::new(vec![]));

    /// Global configuration
    pub static ref CONFIG: Arc<Mutex<Option<config::Config>>> = Arc::new(Mutex::new(None));

    // Flags

    /// Global "quit" status flag
    pub static ref QUIT: AtomicBool = AtomicBool::new(false);

    /// Global "is AFK" status flag
    pub static ref AFK: AtomicBool = AtomicBool::new(false);

    /// Global "enable experimental features" flag
    pub static ref EXPERIMENTAL_FEATURES: AtomicBool = AtomicBool::new(false);

    // Other state

    /// Global "keyboard brightness" modifier
    pub static ref BRIGHTNESS: AtomicIsize = AtomicIsize::new(100);

    /// AFK timer
    pub static ref LAST_INPUT_TIME: Arc<Mutex<Instant>> = Arc::new(Mutex::new(Instant::now()));

    /// Channels to the Lua VMs
    static ref LUA_TXS: Arc<Mutex<Vec<Sender<script::Message>>>> = Arc::new(Mutex::new(vec![]));

    /// Key states
    pub static ref KEY_STATES: Arc<Mutex<Vec<bool>>> =
        Arc::new(Mutex::new(vec![false; constants::MAX_KEYS]));

    pub static ref BUTTON_STATES: Arc<Mutex<Vec<bool>>> =
        Arc::new(Mutex::new(vec![false; constants::MAX_MOUSE_BUTTONS]));

    // cached value
    static ref GRAB_MOUSE: AtomicBool = {
        let config = &*crate::CONFIG.lock();
        let grab_mouse = config
            .as_ref()
            .unwrap()
            .get::<bool>("global.grab_mouse")
            .unwrap_or(true);

        AtomicBool::from(grab_mouse)
    };
}

lazy_static! {
    // Color maps of Lua VMs ready?
    pub static ref COLOR_MAPS_READY_CONDITION: Arc<(Mutex<usize>, Condvar)> =
        Arc::new((Mutex::new(0), Condvar::new()));

    // All upcalls (event handlers) in Lua VM completed?
    pub static ref UPCALL_COMPLETED_ON_KEY_DOWN: Arc<(Mutex<usize>, Condvar)> =
        Arc::new((Mutex::new(0), Condvar::new()));
    pub static ref UPCALL_COMPLETED_ON_KEY_UP: Arc<(Mutex<usize>, Condvar)> =
        Arc::new((Mutex::new(0), Condvar::new()));

    pub static ref UPCALL_COMPLETED_ON_MOUSE_BUTTON_DOWN: Arc<(Mutex<usize>, Condvar)> =
        Arc::new((Mutex::new(0), Condvar::new()));
    pub static ref UPCALL_COMPLETED_ON_MOUSE_BUTTON_UP: Arc<(Mutex<usize>, Condvar)> =
        Arc::new((Mutex::new(0), Condvar::new()));

    pub static ref UPCALL_COMPLETED_ON_MOUSE_MOVE: Arc<(Mutex<usize>, Condvar)> =
        Arc::new((Mutex::new(0), Condvar::new()));

    pub static ref UPCALL_COMPLETED_ON_MOUSE_EVENT: Arc<(Mutex<usize>, Condvar)> =
        Arc::new((Mutex::new(0), Condvar::new()));

    pub static ref UPCALL_COMPLETED_ON_KEYBOARD_HID_EVENT: Arc<(Mutex<usize>, Condvar)> =
        Arc::new((Mutex::new(0), Condvar::new()));

        pub static ref UPCALL_COMPLETED_ON_MOUSE_HID_EVENT: Arc<(Mutex<usize>, Condvar)> =
        Arc::new((Mutex::new(0), Condvar::new()));

    pub static ref UPCALL_COMPLETED_ON_SYSTEM_EVENT: Arc<(Mutex<usize>, Condvar)> =
        Arc::new((Mutex::new(0), Condvar::new()));

    pub static ref UPCALL_COMPLETED_ON_QUIT: Arc<(Mutex<usize>, Condvar)> =
        Arc::new((Mutex::new(0), Condvar::new()));
}

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum MainError {
    #[error("Could not access storage: {description}")]
    StorageError { description: String },

    #[error("Lost connection to device")]
    DeviceDisconnected {},

    #[error("Could not switch profiles")]
    SwitchProfileError {},

    #[error("Could not execute Lua script")]
    ScriptExecError {},
}

#[derive(Debug, thiserror::Error)]
pub enum EvdevError {
    #[error("Could not peek evdev event")]
    EvdevEventError {},

    #[error("Could not get the name of the evdev device from udev")]
    UdevError {},

    #[error("Could not open the evdev device")]
    EvdevError {},

    #[error("Could not create a libevdev device handle")]
    EvdevHandleError {},
}

#[derive(Debug, Clone)]
pub enum FileSystemEvent {
    ProfilesChanged,
    ScriptsChanged,
}

fn print_header() {
    println!(
        r#"
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
"#
    );
}

/// Process commandline options
fn parse_commandline() -> clap::ArgMatches {
    App::new("Eruption")
        .version(env!("CARGO_PKG_VERSION"))
        .author("X3n0m0rph59 <x3n0m0rph59@gmail.com>")
        .about("A Linux user-mode input and LED driver for keyboards, mice and other devices")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .about("Sets the configuration file to use")
                .takes_value(true),
        )
        // .arg(
        //     Arg::new("completions")
        //         .long("completions")
        //         .value_name("SHELL")
        //         .about("Generate shell completions")
        //         .takes_value(true),
        // )
        .get_matches()
}

#[derive(Debug, Clone)]
pub enum DbusApiEvent {
    ProfilesChanged,
    ActiveProfileChanged,
    ActiveSlotChanged,
    BrightnessChanged,
}

/// Spawns the D-Bus API thread and executes it's main loop
fn spawn_dbus_api_thread(
    dbus_tx: Sender<dbus_interface::Message>,
) -> plugins::Result<Sender<DbusApiEvent>> {
    let (dbus_api_tx, dbus_api_rx) = unbounded();

    thread::Builder::new()
        .name("dbus_interface".into())
        .spawn(move || -> Result<()> {
            let dbus = dbus_interface::initialize(dbus_tx)?;

            // will be set to true if we received a dbus event in the current iteration of the loop
            let mut event_received = false;

            loop {
                let timeout = if event_received { 0 } else { 15 };

                // process events, destined for the dbus api
                match dbus_api_rx.recv_timeout(Duration::from_millis(timeout)) {
                    Ok(result) => match result {
                        DbusApiEvent::ProfilesChanged => dbus.notify_profiles_changed(),

                        DbusApiEvent::ActiveProfileChanged => dbus.notify_active_profile_changed(),

                        DbusApiEvent::ActiveSlotChanged => dbus.notify_active_slot_changed(),

                        DbusApiEvent::BrightnessChanged => dbus.notify_brightness_changed(),
                    },

                    Err(_e) => {
                        event_received = dbus.get_next_event_timeout(0).unwrap_or_else(|e| {
                            error!("Could not get the next D-Bus event: {}", e);

                            false
                        });
                    }
                }
            }
        })?;

    Ok(dbus_api_tx)
}

/// Spawns the keyboard events thread and executes it's main loop
fn spawn_keyboard_input_thread(
    kbd_tx: Sender<Option<evdev_rs::InputEvent>>,
    keyboard_device: KeyboardDevice,
    device_index: usize,
    usb_vid: u16,
    usb_pid: u16,
) -> plugins::Result<()> {
    thread::Builder::new()
        .name(format!("events/kbd:{}", device_index))
        .spawn(move || -> Result<()> {
            let device = match hwdevices::get_input_dev_from_udev(usb_vid, usb_pid) {
                Ok(filename) => match File::open(filename.clone()) {
                    Ok(devfile) => match Device::new_from_fd(devfile) {
                        Ok(mut device) => {
                            info!("Now listening on keyboard: {}", filename);

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

                            device
                        }

                        Err(_e) => return Err(EvdevError::EvdevHandleError {}.into()),
                    },

                    Err(_e) => return Err(EvdevError::EvdevError {}.into()),
                },

                Err(_e) => return Err(EvdevError::UdevError {}.into()),
            };

            loop {
                // check if we shall terminate the input thread, before we poll the keyboard
                if QUIT.load(Ordering::SeqCst) {
                    break Ok(());
                }

                match device.next_event(evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING) {
                    Ok(k) => {
                        trace!("Key event: {:?}", k.1);

                        // reset "to be dropped" flag
                        macros::DROP_CURRENT_KEY.store(false, Ordering::SeqCst);

                        // update our internal representation of the keyboard state
                        if let evdev_rs::enums::EventCode::EV_KEY(ref code) = k.1.event_code {
                            let is_pressed = k.1.value > 0;
                            let index =
                                keyboard_device.read().ev_key_to_key_index(code.clone()) as usize;

                            KEY_STATES.lock()[index] = is_pressed;
                        }

                        kbd_tx.send(Some(k.1)).unwrap_or_else(|e| {
                            error!("Could not send a keyboard event to the main thread: {}", e)
                        });

                        // update AFK timer
                        *crate::LAST_INPUT_TIME.lock() = Instant::now();
                    }

                    Err(e) => {
                        if e.raw_os_error().unwrap() == libc::ENODEV {
                            error!("Fatal: Keyboard device went away: {}", e);

                            QUIT.store(true, Ordering::SeqCst);

                            return Err(EvdevError::EvdevEventError {}.into());
                        } else {
                            error!("Fatal: Could not peek evdev event: {}", e);

                            QUIT.store(true, Ordering::SeqCst);

                            return Err(EvdevError::EvdevEventError {}.into());
                        }
                    }
                };
            }
        })
        .unwrap_or_else(|e| {
            error!("Could not spawn a thread: {}", e);
            panic!()
        });

    Ok(())
}

/// Spawns the mouse events thread and executes it's main loop
fn spawn_mouse_input_thread(
    mouse_tx: Sender<Option<evdev_rs::InputEvent>>,
    mouse_device: MouseDevice,
    device_index: usize,
    usb_vid: u16,
    usb_pid: u16,
) -> plugins::Result<()> {
    thread::Builder::new()
        .name(format!("events/mouse:{}", device_index))
        .spawn(move || -> Result<()> {
            let device = match hwdevices::get_input_dev_from_udev(usb_vid, usb_pid) {
                Ok(filename) => match File::open(filename.clone()) {
                    Ok(devfile) => match Device::new_from_fd(devfile) {
                        Ok(mut device) => {
                            info!("Now listening on mouse: {}", filename);

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

                            info!("Grabbing the mouse device exclusively");
                            device
                                .grab(GrabMode::Grab)
                                .expect("Could not grab the device, terminating now.");

                            device
                        }

                        Err(_e) => return Err(EvdevError::EvdevHandleError {}.into()),
                    },

                    Err(_e) => return Err(EvdevError::EvdevError {}.into()),
                },

                Err(_e) => return Err(EvdevError::UdevError {}.into()),
            };

            loop {
                // check if we shall terminate the input thread, before we poll the keyboard
                if QUIT.load(Ordering::SeqCst) {
                    break Ok(());
                }

                match device.next_event(evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING) {
                    Ok(k) => {
                        trace!("Mouse event: {:?}", k.1);

                        // reset "to be dropped" flag
                        macros::DROP_CURRENT_MOUSE_INPUT.store(false, Ordering::SeqCst);

                        // update our internal representation of the device state
                        if let evdev_rs::enums::EventCode::EV_KEY(code) = k.1.clone().event_code {
                            let is_pressed = k.1.value > 0;
                            let index =
                                mouse_device.read().ev_key_to_button_index(code).unwrap() as usize;

                            BUTTON_STATES.lock()[index] = is_pressed;
                        } else if let evdev_rs::enums::EventCode::EV_REL(code) =
                            k.1.clone().event_code
                        {
                            if code != evdev_rs::enums::EV_REL::REL_WHEEL
                                && code != evdev_rs::enums::EV_REL::REL_HWHEEL
                                && code != evdev_rs::enums::EV_REL::REL_WHEEL_HI_RES
                                && code != evdev_rs::enums::EV_REL::REL_HWHEEL_HI_RES
                            {
                                // directly mirror pointer motion events to reduce input lag.
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
                                            error!("Could not send a pending mouse event: {}", e)
                                        });
                                }
                            }
                        }

                        mouse_tx.send(Some(k.1)).unwrap_or_else(|e| {
                            error!("Could not send a mouse event to the main thread: {}", e)
                        });

                        // update AFK timer
                        *crate::LAST_INPUT_TIME.lock() = Instant::now();
                    }

                    Err(e) => {
                        if e.raw_os_error().unwrap() == libc::ENODEV {
                            error!("Fatal: Mouse device went away: {}", e);

                            QUIT.store(true, Ordering::SeqCst);

                            return Err(EvdevError::EvdevEventError {}.into());
                        } else {
                            error!("Fatal: Could not peek evdev event: {}", e);

                            QUIT.store(true, Ordering::SeqCst);

                            return Err(EvdevError::EvdevEventError {}.into());
                        }
                    }
                };
            }
        })
        .unwrap_or_else(|e| {
            error!("Could not spawn a thread: {}", e);
            panic!()
        });

    Ok(())
}

/// Spawns the mouse events thread for an additional sub-device on the mouse and executes the thread's main loop
fn spawn_mouse_input_thread_secondary(
    mouse_tx: Sender<Option<evdev_rs::InputEvent>>,
    mouse_device: MouseDevice,
    device_index: usize,
    usb_vid: u16,
    usb_pid: u16,
) -> plugins::Result<()> {
    thread::Builder::new()
        .name(format!("events/mouse-sub:{}", device_index))
        .spawn(move || -> Result<()> {
            let device = match hwdevices::get_input_sub_dev_from_udev(usb_vid, usb_pid, 2) {
                Ok(filename) => match File::open(filename.clone()) {
                    Ok(devfile) => match Device::new_from_fd(devfile) {
                        Ok(mut device) => {
                            info!("Now listening on mouse sub-dev: {}", filename);

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

                            info!("Grabbing the sub-device exclusively");
                            device
                                .grab(GrabMode::Grab)
                                .expect("Could not grab the device, terminating now.");

                            device
                        }

                        Err(_e) => return Err(EvdevError::EvdevHandleError {}.into()),
                    },

                    Err(_e) => return Err(EvdevError::EvdevError {}.into()),
                },

                Err(_e) => return Err(EvdevError::UdevError {}.into()),
            };

            loop {
                // check if we shall terminate the input thread, before we poll the keyboard
                if QUIT.load(Ordering::SeqCst) {
                    break Ok(());
                }

                match device.next_event(evdev_rs::ReadFlag::NORMAL | evdev_rs::ReadFlag::BLOCKING) {
                    Ok(k) => {
                        trace!("Mouse sub-device event: {:?}", k.1);

                        // reset "to be dropped" flag
                        macros::DROP_CURRENT_MOUSE_INPUT.store(false, Ordering::SeqCst);

                        // update our internal representation of the device state
                        if let evdev_rs::enums::EventCode::EV_KEY(code) = k.1.clone().event_code {
                            let is_pressed = k.1.value > 0;
                            let index = mouse_device.read().ev_key_to_button_index(code).unwrap() as usize;

                            BUTTON_STATES.lock()[index] = is_pressed;
                        } else if let evdev_rs::enums::EventCode::EV_REL(code) =
                            k.1.clone().event_code
                        {
                            if code != evdev_rs::enums::EV_REL::REL_WHEEL
                                && code != evdev_rs::enums::EV_REL::REL_HWHEEL
                                && code != evdev_rs::enums::EV_REL::REL_WHEEL_HI_RES
                                && code != evdev_rs::enums::EV_REL::REL_HWHEEL_HI_RES
                            {
                                // directly mirror pointer motion events to reduce input lag.
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
                                            error!("Could not send a pending mouse sub-device event: {}", e)
                                        });
                                }
                            }
                        }

                        mouse_tx.send(Some(k.1)).unwrap_or_else(|e| {
                            error!("Could not send a mouse sub-device event to the main thread: {}", e)
                        });

                        // update AFK timer
                        *crate::LAST_INPUT_TIME.lock() = Instant::now();
                    }

                    Err(e) => {
                        if e.raw_os_error().unwrap() == libc::ENODEV {
                            error!("Fatal: Mouse sub-device went away: {}", e);

                            QUIT.store(true, Ordering::SeqCst);

                            return Err(EvdevError::EvdevEventError {}.into());
                        } else {
                            error!("Fatal: Could not peek evdev event: {}", e);

                            QUIT.store(true, Ordering::SeqCst);

                            return Err(EvdevError::EvdevEventError {}.into());
                        }
                    }
                };
            }
        })
        .unwrap_or_else(|e| {
            error!("Could not spawn a thread: {}", e);
            panic!()
        });

    Ok(())
}

fn spawn_lua_thread(
    thread_idx: usize,
    lua_rx: Receiver<script::Message>,
    script_path: PathBuf,
    keyboard_devices: Vec<KeyboardDevice>,
    mouse_devices: Vec<MouseDevice>,
) -> Result<()> {
    let result = util::is_file_accessible(&script_path);
    if let Err(result) = result {
        error!(
            "Script file '{}' is not accessible: {}",
            script_path.display(),
            result
        );

        return Err(MainError::ScriptExecError {}.into());
    }

    let result = util::is_file_accessible(util::get_manifest_for(&script_path));
    if let Err(result) = result {
        error!(
            "Manifest file for script '{}' is not accessible: {}",
            script_path.display(),
            result
        );

        return Err(MainError::ScriptExecError {}.into());
    }

    let builder = thread::Builder::new().name(format!(
        "{}:{}",
        thread_idx,
        script_path.file_name().unwrap().to_string_lossy(),
    ));

    builder.spawn(move || -> Result<()> {
        #[allow(clippy::never_loop)]
        loop {
            let result = script::run_script(
                script_path.clone(),
                &lua_rx,
                &keyboard_devices.clone(),
                &mouse_devices.clone(),
            )?;

            match result {
                //script::RunScriptResult::ReExecuteOtherScript(script_file) => {
                //script_path = script_file;
                //continue;
                //}
                script::RunScriptResult::TerminatedGracefully => break,

                script::RunScriptResult::TerminatedWithErrors => {
                    error!("Script execution failed");

                    // TODO: Try to get rid of this! We currently need it here since
                    //       otherwise, we may deadlock on error sometimes.
                    std::process::abort();

                    // return Err(MainError::ScriptExecError {}.into());
                }
            }
        }

        Ok(())
    })?;

    Ok(())
}

/// Switches the currently active profile to the profile file `profile_path`
fn switch_profile<P: AsRef<Path>>(
    profile_file: P,
    dbus_api_tx: &Sender<DbusApiEvent>,
    keyboard_devices: &[KeyboardDevice],
    mouse_devices: &[MouseDevice],
) -> Result<()> {
    info!("Switching to profile: {}", &profile_file.as_ref().display());

    let script_dir = PathBuf::from(
        CONFIG
            .lock()
            .as_ref()
            .unwrap()
            .get_str("global.script_dir")
            .unwrap_or_else(|_| constants::DEFAULT_SCRIPT_DIR.to_string()),
    );

    let profile_dir = PathBuf::from(
        CONFIG
            .lock()
            .as_ref()
            .unwrap()
            .get_str("global.profile_dir")
            .unwrap_or_else(|_| constants::DEFAULT_PROFILE_DIR.to_string()),
    );

    let profile_path = profile_dir.join(&profile_file);
    let profile = profiles::Profile::from(&profile_path)?;

    // verify script files first; better fail early if we can
    let script_files = profile.active_scripts.clone();
    for script_file in script_files.iter() {
        let script_path = script_dir.join(&script_file);

        if !util::is_script_file_accessible(&script_path)
            || !util::is_manifest_file_accessible(&script_path)
        {
            error!(
                "Script file or manifest inaccessible: {}",
                script_path.display()
            );
            return Err(MainError::SwitchProfileError {}.into());
        }
    }

    // now request termination of all Lua VMs
    let mut lua_txs = LUA_TXS.lock();

    for lua_tx in lua_txs.iter() {
        lua_tx
            .send(script::Message::Unload)
            .unwrap_or_else(|e| error!("Could not send an event to a Lua VM: {}", e));
    }

    // be safe and clear any leftover channels
    lua_txs.clear();

    // now spawn a new set of Lua VMs, with scripts from the new profile
    for (thread_idx, script_file) in script_files.iter().enumerate() {
        let script_path = script_dir.join(&script_file);

        let (lua_tx, lua_rx) = unbounded();
        spawn_lua_thread(
            thread_idx,
            lua_rx,
            script_path.clone(),
            keyboard_devices.to_owned(),
            mouse_devices.to_owned(),
        )
        .unwrap_or_else(|e| {
            error!("Could not spawn a thread: {}", e);
        });

        lua_txs.push(lua_tx);
    }

    // finally assign the globally active profile
    *ACTIVE_PROFILE.lock() = Some(profile);

    dbus_api_tx
        .send(DbusApiEvent::ActiveProfileChanged)
        .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));

    let active_slot = ACTIVE_SLOT.load(Ordering::SeqCst);
    let mut slot_profiles = SLOT_PROFILES.lock();
    slot_profiles.as_mut().unwrap()[active_slot] = profile_file.as_ref().into();

    Ok(())
}

/// Process file system related events
async fn process_filesystem_event(
    fsevent: &FileSystemEvent,
    dbus_api_tx: &Sender<DbusApiEvent>,
) -> Result<()> {
    match fsevent {
        FileSystemEvent::ProfilesChanged => {
            events::notify_observers(events::Event::FileSystemEvent(
                FileSystemEvent::ProfilesChanged,
            ))
            .unwrap_or_else(|e| error!("{}", e));

            dbus_api_tx
                .send(DbusApiEvent::ProfilesChanged)
                .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));
        }

        FileSystemEvent::ScriptsChanged => {}
    }

    Ok(())
}

/// Process D-Bus events
async fn process_dbus_event(
    dbus_event: &dbus_interface::Message,
    dbus_api_tx: &Sender<DbusApiEvent>,
    keyboard_devices: &[KeyboardDevice],
    mouse_devices: &[MouseDevice],
) -> Result<()> {
    match dbus_event {
        dbus_interface::Message::SwitchSlot(slot) => {
            info!("Switching to slot #{}", slot + 1);

            ACTIVE_SLOT.store(*slot, Ordering::SeqCst);
        }

        dbus_interface::Message::SwitchProfile(profile_path) => {
            info!("Loading profile: {}", profile_path.display());

            switch_profile(
                &profile_path,
                &dbus_api_tx,
                &keyboard_devices,
                &mouse_devices,
            )
            .unwrap_or_else(|e| error!("Could not switch profiles: {}", e));
        }
    }

    Ok(())
}

/// Process HID events
async fn process_keyboard_hid_events(
    keyboard_device: &KeyboardDevice,
    failed_txs: &HashSet<usize>,
) -> Result<()> {
    // limit the number of messages that will be processed during this iteration
    let mut loop_counter = 0;

    let mut event_processed = false;

    'HID_EVENTS_LOOP: loop {
        match keyboard_device.read().get_next_event_timeout(0) {
            Ok(result) if result != KeyboardHidEvent::Unknown => {
                event_processed = true;

                events::notify_observers(events::Event::KeyboardHidEvent(result))
                    .unwrap_or_else(|e| error!("{}", e));

                *UPCALL_COMPLETED_ON_KEYBOARD_HID_EVENT.0.lock() =
                    LUA_TXS.lock().len() - failed_txs.len();

                for (idx, lua_tx) in LUA_TXS.lock().iter().enumerate() {
                    if !failed_txs.contains(&idx) {
                        lua_tx
                            .send(script::Message::KeyboardHidEvent(result))
                            .unwrap_or_else(|e| {
                                error!("Could not send a pending HID event to a Lua VM: {}", e)
                            });
                    } else {
                        warn!("Not sending a message to a failed tx");
                    }
                }

                // wait until all Lua VMs completed the event handler
                loop {
                    let mut pending = UPCALL_COMPLETED_ON_KEYBOARD_HID_EVENT.0.lock();

                    UPCALL_COMPLETED_ON_KEYBOARD_HID_EVENT.1.wait_for(
                        &mut pending,
                        Duration::from_millis(constants::TIMEOUT_CONDITION_MILLIS),
                    );

                    if *pending == 0 {
                        break;
                    }
                }

                // translate HID event to keyboard event
                match result {
                    KeyboardHidEvent::KeyDown { code } => {
                        let index = keyboard_device.read().hid_event_code_to_key_index(&code);
                        if index > 0 {
                            KEY_STATES.lock()[index as usize] = true;

                            *UPCALL_COMPLETED_ON_KEY_DOWN.0.lock() =
                                LUA_TXS.lock().len() - failed_txs.len();

                            for (idx, lua_tx) in LUA_TXS.lock().iter().enumerate() {
                                if !failed_txs.contains(&idx) {
                                    lua_tx
                                        .send(script::Message::KeyDown(index))
                                        .unwrap_or_else(|e| {
                                            error!("Could not send a pending keyboard event to a Lua VM: {}", e)
                                        });
                                } else {
                                    warn!("Not sending a message to a failed tx");
                                }
                            }

                            // wait until all Lua VMs completed the event handler
                            loop {
                                let mut pending = UPCALL_COMPLETED_ON_KEY_DOWN.0.lock();

                                UPCALL_COMPLETED_ON_KEY_DOWN.1.wait_for(
                                    &mut pending,
                                    Duration::from_millis(constants::TIMEOUT_CONDITION_MILLIS),
                                );

                                if *pending == 0 {
                                    break;
                                }
                            }

                            // update AFK timer
                            *crate::LAST_INPUT_TIME.lock() = Instant::now();

                            events::notify_observers(events::Event::KeyDown(index))
                                .unwrap_or_else(|e| error!("{}", e));
                        }
                    }

                    KeyboardHidEvent::KeyUp { code } => {
                        let index = keyboard_device.read().hid_event_code_to_key_index(&code);
                        if index > 0 {
                            KEY_STATES.lock()[index as usize] = false;

                            *UPCALL_COMPLETED_ON_KEY_UP.0.lock() =
                                LUA_TXS.lock().len() - failed_txs.len();

                            for (idx, lua_tx) in LUA_TXS.lock().iter().enumerate() {
                                if !failed_txs.contains(&idx) {
                                    lua_tx.send(script::Message::KeyUp(index)).unwrap_or_else(
                                        |e| {
                                            error!("Could not send a pending keyboard event to a Lua VM: {}", e)
                                        },
                                    );
                                } else {
                                    warn!("Not sending a message to a failed tx");
                                }
                            }

                            // wait until all Lua VMs completed the event handler
                            loop {
                                let mut pending = UPCALL_COMPLETED_ON_KEY_UP.0.lock();

                                UPCALL_COMPLETED_ON_KEY_UP.1.wait_for(
                                    &mut pending,
                                    Duration::from_millis(constants::TIMEOUT_CONDITION_MILLIS),
                                );

                                if *pending == 0 {
                                    break;
                                }
                            }

                            // update AFK timer
                            *crate::LAST_INPUT_TIME.lock() = Instant::now();

                            events::notify_observers(events::Event::KeyUp(index))
                                .unwrap_or_else(|e| error!("{}", e));
                        }
                    }

                    _ => { /* ignore other events */ }
                }
            }

            Ok(_) => { /* Ignore unknown events */ }

            Err(_e) => {
                event_processed = false;
            }
        }

        if !event_processed || loop_counter >= constants::MAX_EVENTS_PER_ITERATION {
            break 'HID_EVENTS_LOOP; // no more events in queue or iteration limit reached
        }

        loop_counter += 1;
    }

    Ok(())
}

/// Process HID events
async fn process_mouse_hid_events(
    mouse_device: &MouseDevice,
    failed_txs: &HashSet<usize>,
) -> Result<()> {
    // limit the number of messages that will be processed during this iteration
    let mut loop_counter = 0;

    let mut event_processed = false;

    'HID_EVENTS_LOOP: loop {
        match mouse_device.read().get_next_event_timeout(0) {
            Ok(result) if result != MouseHidEvent::Unknown => {
                event_processed = true;

                events::notify_observers(events::Event::MouseHidEvent(result))
                    .unwrap_or_else(|e| error!("{}", e));

                *UPCALL_COMPLETED_ON_MOUSE_HID_EVENT.0.lock() =
                    LUA_TXS.lock().len() - failed_txs.len();

                for (idx, lua_tx) in LUA_TXS.lock().iter().enumerate() {
                    if !failed_txs.contains(&idx) {
                        lua_tx
                            .send(script::Message::MouseHidEvent(result))
                            .unwrap_or_else(|e| {
                                error!("Could not send a pending HID event to a Lua VM: {}", e)
                            });
                    } else {
                        warn!("Not sending a message to a failed tx");
                    }
                }

                // wait until all Lua VMs completed the event handler
                loop {
                    let mut pending = UPCALL_COMPLETED_ON_MOUSE_HID_EVENT.0.lock();

                    UPCALL_COMPLETED_ON_MOUSE_HID_EVENT.1.wait_for(
                        &mut pending,
                        Duration::from_millis(constants::TIMEOUT_CONDITION_MILLIS),
                    );

                    if *pending == 0 {
                        break;
                    }
                }

                //     _ => { /* ignore other events */ }
                // }
            }

            Ok(_) => { /* Ignore unknown events */ }

            Err(_e) => {
                event_processed = false;
            }
        }

        if !event_processed || loop_counter >= constants::MAX_EVENTS_PER_ITERATION {
            break 'HID_EVENTS_LOOP; // no more events in queue or iteration limit reached
        }

        loop_counter += 1;
    }

    Ok(())
}

/// Process mouse events
async fn process_mouse_event(
    raw_event: &evdev_rs::InputEvent,
    mouse_device: &MouseDevice,
    failed_txs: &HashSet<usize>,
    mouse_move_event_last_dispatched: &mut Instant,
    mouse_motion_buf: &mut (i32, i32, i32),
) -> Result<()> {
    // send pending mouse events to the Lua VMs and to the event dispatcher

    let mut mirror_event = true;

    // notify all observers of raw events
    events::notify_observers(events::Event::RawMouseEvent(raw_event.clone())).ok();

    if let evdev_rs::enums::EventCode::EV_REL(ref code) = raw_event.clone().event_code {
        match code {
            evdev_rs::enums::EV_REL::REL_X
            | evdev_rs::enums::EV_REL::REL_Y
            | evdev_rs::enums::EV_REL::REL_Z => {
                // mouse move event occurred

                mirror_event = false; // don't mirror pointer motion events, since they are
                                      // already mirrored by the mouse plugin

                // accumulate relative changes
                let direction = if *code == evdev_rs::enums::EV_REL::REL_X {
                    mouse_motion_buf.0 += raw_event.value;

                    1
                } else if *code == evdev_rs::enums::EV_REL::REL_Y {
                    mouse_motion_buf.1 += raw_event.value;

                    2
                } else if *code == evdev_rs::enums::EV_REL::REL_Z {
                    mouse_motion_buf.2 += raw_event.value;

                    3
                } else {
                    4
                };

                if *mouse_motion_buf != (0, 0, 0) &&
                    mouse_move_event_last_dispatched.elapsed().as_millis() > constants::EVENTS_UPCALL_RATE_LIMIT_MILLIS.into() {
                    *mouse_move_event_last_dispatched = Instant::now();

                    *UPCALL_COMPLETED_ON_MOUSE_MOVE.0.lock() =
                        LUA_TXS.lock().len() - failed_txs.len();

                    for (idx, lua_tx) in LUA_TXS.lock().iter().enumerate() {
                        if !failed_txs.contains(&idx) {
                            lua_tx.send(script::Message::MouseMove(mouse_motion_buf.0,
                                                                    mouse_motion_buf.1,
                                                                    mouse_motion_buf.2)).unwrap_or_else(
                        |e| {
                                error!("Could not send a pending mouse event to a Lua VM: {}", e);
                            });

                            // reset relative motion buffer, since it has been submitted
                            *mouse_motion_buf = (0, 0, 0);
                        } else {
                            warn!("Not sending a message to a failed tx");
                        }
                    }

                    // wait until all Lua VMs completed the event handler
                    loop {
                        let mut pending =
                            UPCALL_COMPLETED_ON_MOUSE_MOVE.0.lock();

                        UPCALL_COMPLETED_ON_MOUSE_MOVE.1.wait_for(
                            &mut pending,
                            Duration::from_millis(
                                constants::TIMEOUT_CONDITION_MILLIS,
                            ),
                        );

                        if *pending == 0 {
                            break;
                        }
                    }
                }

                events::notify_observers(events::Event::MouseMove(
                    direction,
                    raw_event.value,
                ))
                .unwrap_or_else(|e| error!("{}", e));
            }

            evdev_rs::enums::EV_REL::REL_WHEEL
            | evdev_rs::enums::EV_REL::REL_HWHEEL
            /* | evdev_rs::enums::EV_REL::REL_WHEEL_HI_RES
            | evdev_rs::enums::EV_REL::REL_HWHEEL_HI_RES */ => {
                // mouse scroll wheel event occurred

                let direction = if raw_event.value > 0 { 1 } else { 2 };

                *UPCALL_COMPLETED_ON_MOUSE_EVENT.0.lock() =
                    LUA_TXS.lock().len() - failed_txs.len();

                for (idx, lua_tx) in LUA_TXS.lock().iter().enumerate() {
                    if !failed_txs.contains(&idx) {
                        lua_tx.send(script::Message::MouseWheelEvent(direction)).unwrap_or_else(
                        |e| {
                            error!("Could not send a pending mouse event to a Lua VM: {}", e)
                        },
                    );
                    } else {
                        warn!("Not sending a message to a failed tx");
                    }
                }

                // wait until all Lua VMs completed the event handler
                loop {
                    let mut pending =
                        UPCALL_COMPLETED_ON_MOUSE_EVENT.0.lock();

                    UPCALL_COMPLETED_ON_MOUSE_EVENT.1.wait_for(
                        &mut pending,
                        Duration::from_millis(
                            constants::TIMEOUT_CONDITION_MILLIS,
                        ),
                    );

                    if *pending == 0 {
                        break;
                    }
                }

                events::notify_observers(events::Event::MouseWheelEvent(
                    direction,
                ))
                .unwrap_or_else(|e| error!("{}", e));
            }

            _ => (), // ignore other events
        }
    } else if let evdev_rs::enums::EventCode::EV_KEY(code) = raw_event.clone().event_code {
        // mouse button event occurred

        let is_pressed = raw_event.value > 0;
        let index = mouse_device.read().ev_key_to_button_index(code).unwrap();

        if is_pressed {
            *UPCALL_COMPLETED_ON_MOUSE_BUTTON_DOWN.0.lock() =
                LUA_TXS.lock().len() - failed_txs.len();

            for (idx, lua_tx) in LUA_TXS.lock().iter().enumerate() {
                if !failed_txs.contains(&idx) {
                    lua_tx
                        .send(script::Message::MouseButtonDown(index))
                        .unwrap_or_else(|e| {
                            error!("Could not send a pending mouse event to a Lua VM: {}", e)
                        });
                } else {
                    warn!("Not sending a message to a failed tx");
                }
            }

            // wait until all Lua VMs completed the event handler
            loop {
                let mut pending = UPCALL_COMPLETED_ON_MOUSE_BUTTON_DOWN.0.lock();

                UPCALL_COMPLETED_ON_MOUSE_BUTTON_DOWN.1.wait_for(
                    &mut pending,
                    Duration::from_millis(constants::TIMEOUT_CONDITION_MILLIS),
                );

                if *pending == 0 {
                    break;
                }
            }

            events::notify_observers(events::Event::MouseButtonDown(index))
                .unwrap_or_else(|e| error!("{}", e));
        } else {
            *UPCALL_COMPLETED_ON_MOUSE_BUTTON_UP.0.lock() = LUA_TXS.lock().len() - failed_txs.len();

            for (idx, lua_tx) in LUA_TXS.lock().iter().enumerate() {
                if !failed_txs.contains(&idx) {
                    lua_tx
                        .send(script::Message::MouseButtonUp(index))
                        .unwrap_or_else(|e| {
                            error!("Could not send a pending mouse event to a Lua VM: {}", e)
                        });
                } else {
                    warn!("Not sending a message to a failed tx");
                }
            }

            // wait until all Lua VMs completed the event handler
            loop {
                let mut pending = UPCALL_COMPLETED_ON_MOUSE_BUTTON_UP.0.lock();

                UPCALL_COMPLETED_ON_MOUSE_BUTTON_UP.1.wait_for(
                    &mut pending,
                    Duration::from_millis(constants::TIMEOUT_CONDITION_MILLIS),
                );

                if *pending == 0 {
                    break;
                }
            }

            events::notify_observers(events::Event::MouseButtonUp(index))
                .unwrap_or_else(|e| error!("{}", e));
        }
    }

    if mirror_event {
        // mirror all events, except pointer motion events.
        // Pointer motion events currently can not be overridden,
        // they are mirrored to the virtual mouse directly after they are
        // received by the mouse plugin. This is done to reduce input lag
        macros::UINPUT_TX
            .lock()
            .as_ref()
            .unwrap()
            .send(macros::Message::MirrorMouseEvent(raw_event.clone()))
            .unwrap_or_else(|e| error!("Could not send a pending mouse event: {}", e));
    }

    Ok(())
}

/// Process mouse events from a secondary sub-device on the primary mouse
// async fn process_mouse_secondary_events(
//     mouse_rx: &Receiver<Option<evdev_rs::InputEvent>>,
//     failed_txs: &HashSet<usize>,
// ) -> Result<()> {

//         // send pending mouse events to the Lua VMs and to the event dispatcher
//         match mouse_rx.recv_timeout(Duration::from_millis(0)) {
//             Ok(result) => {
//                 match result {
//                     Some(raw_event) => {
//                         // notify all observers of raw events
//                         events::notify_observers(events::Event::RawMouseEvent(raw_event.clone()))
//                             .ok();

//                         if let evdev_rs::enums::EventCode::EV_KEY(code) =
//                             raw_event.clone().event_code
//                         {
//                             // mouse button event occurred

//                             let is_pressed = raw_event.value > 0;
//                             let index = util::ev_key_to_button_index(code).unwrap();

//                             if is_pressed {
//                                 *UPCALL_COMPLETED_ON_MOUSE_BUTTON_DOWN.0.lock() =
//                                     LUA_TXS.lock().len() - failed_txs.len();

//                                 for (idx, lua_tx) in LUA_TXS.lock().iter().enumerate() {
//                                     if !failed_txs.contains(&idx) {
//                                         lua_tx.send(script::Message::MouseButtonDown(index)).unwrap_or_else(
//                                                 |e| {
//                                                     error!("Could not send a pending mouse event to a Lua VM: {}", e)
//                                                 },
//                                             );
//                                     } else {
//                                         warn!("Not sending a message to a failed tx");
//                                     }
//                                 }

//                                 // wait until all Lua VMs completed the event handler
//                                 loop {
//                                     let mut pending =
//                                         UPCALL_COMPLETED_ON_MOUSE_BUTTON_DOWN.0.lock();

//                                     UPCALL_COMPLETED_ON_MOUSE_BUTTON_DOWN.1.wait_for(
//                                         &mut pending,
//                                         Duration::from_millis(constants::TIMEOUT_CONDITION_MILLIS),
//                                     );

//                                     if *pending == 0 {
//                                         break;
//                                     }
//                                 }

//                                 events::notify_observers(events::Event::MouseButtonDown(index))
//                                     .unwrap_or_else(|e| error!("{}", e));
//                             } else {
//                                 *UPCALL_COMPLETED_ON_MOUSE_BUTTON_UP.0.lock() =
//                                     LUA_TXS.lock().len() - failed_txs.len();

//                                 for (idx, lua_tx) in LUA_TXS.lock().iter().enumerate() {
//                                     if !failed_txs.contains(&idx) {
//                                         lua_tx.send(script::Message::MouseButtonUp(index)).unwrap_or_else(
//                                                 |e| {
//                                                     error!("Could not send a pending mouse event to a Lua VM: {}", e)
//                                                 },
//                                             );
//                                     } else {
//                                         warn!("Not sending a message to a failed tx");
//                                     }
//                                 }

//                                 // wait until all Lua VMs completed the event handler
//                                 loop {
//                                     let mut pending = UPCALL_COMPLETED_ON_MOUSE_BUTTON_UP.0.lock();

//                                     UPCALL_COMPLETED_ON_MOUSE_BUTTON_UP.1.wait_for(
//                                         &mut pending,
//                                         Duration::from_millis(constants::TIMEOUT_CONDITION_MILLIS),
//                                     );

//                                     if *pending == 0 {
//                                         break;
//                                     }
//                                 }

//                                 events::notify_observers(events::Event::MouseButtonUp(index))
//                                     .unwrap_or_else(|e| error!("{}", e));
//                             }
//                         }

//                         // mirror all events, except pointer motion events.
//                         // Pointer motion events currently can not be overridden,
//                         // they are mirrored to the virtual mouse directly after they are
//                         // received by the mouse plugin. This is done to reduce input lag
//                         macros::UINPUT_TX
//                             .lock()
//                             .as_ref()
//                             .unwrap()
//                             .send(macros::Message::MirrorMouseEvent(raw_event.clone()))
//                             .unwrap_or_else(|e| {
//                                 error!("Could not send a pending mouse event: {}", e)
//                             });

//                         event_processed = true;
//                     }

//                 }
//             }
//         }

//     Ok(())
// }

/// Process keyboard events
async fn process_keyboard_event(
    raw_event: &evdev_rs::InputEvent,
    keyboard_device: &KeyboardDevice,
    failed_txs: &HashSet<usize>,
) -> Result<()> {
    // notify all observers of raw events
    events::notify_observers(events::Event::RawKeyboardEvent(raw_event.clone())).ok();

    if let evdev_rs::enums::EventCode::EV_KEY(ref code) = raw_event.event_code {
        let is_pressed = raw_event.value > 0;
        let index = keyboard_device.read().ev_key_to_key_index(code.clone());

        trace!("Key index: {:#x}", index);

        if is_pressed {
            *UPCALL_COMPLETED_ON_KEY_DOWN.0.lock() = LUA_TXS.lock().len() - failed_txs.len();

            for (idx, lua_tx) in LUA_TXS.lock().iter().enumerate() {
                if !failed_txs.contains(&idx) {
                    lua_tx
                        .send(script::Message::KeyDown(index))
                        .unwrap_or_else(|e| {
                            error!("Could not send a pending keyboard event to a Lua VM: {}", e)
                        });
                } else {
                    warn!("Not sending a message to a failed tx");
                }
            }

            // wait until all Lua VMs completed the event handler
            loop {
                let mut pending = UPCALL_COMPLETED_ON_KEY_DOWN.0.lock();

                UPCALL_COMPLETED_ON_KEY_DOWN.1.wait_for(
                    &mut pending,
                    Duration::from_millis(constants::TIMEOUT_CONDITION_MILLIS),
                );

                if *pending == 0 {
                    break;
                }
            }

            events::notify_observers(events::Event::KeyDown(index))
                .unwrap_or_else(|e| error!("{}", e));
        } else {
            *UPCALL_COMPLETED_ON_KEY_UP.0.lock() = LUA_TXS.lock().len() - failed_txs.len();

            for (idx, lua_tx) in LUA_TXS.lock().iter().enumerate() {
                if !failed_txs.contains(&idx) {
                    lua_tx
                        .send(script::Message::KeyUp(index))
                        .unwrap_or_else(|e| {
                            error!("Could not send a pending keyboard event to a Lua VM: {}", e)
                        });
                } else {
                    warn!("Not sending a message to a failed tx");
                }
            }

            // wait until all Lua VMs completed the event handler
            loop {
                let mut pending = UPCALL_COMPLETED_ON_KEY_UP.0.lock();

                UPCALL_COMPLETED_ON_KEY_UP.1.wait_for(
                    &mut pending,
                    Duration::from_millis(constants::TIMEOUT_CONDITION_MILLIS),
                );

                if *pending == 0 {
                    break;
                }
            }

            events::notify_observers(events::Event::KeyUp(index))
                .unwrap_or_else(|e| error!("{}", e));
        }
    }

    // handler for Message::MirrorKey will drop the key if a Lua VM
    // called inject_key(..), so that the key won't be reported twice
    macros::UINPUT_TX
        .lock()
        .as_ref()
        .unwrap()
        .send(macros::Message::MirrorKey(raw_event.clone()))
        .unwrap_or_else(|e| error!("Could not send a pending keyboard event: {}", e));

    Ok(())
}

async fn run_main_loop(
    keyboard_devices: Vec<(KeyboardDevice, Receiver<Option<evdev_rs::InputEvent>>)>,
    mouse_devices: Vec<(MouseDevice, Receiver<Option<evdev_rs::InputEvent>>)>,
    dbus_api_tx: &Sender<DbusApiEvent>,
    ctrl_c_rx: &Receiver<bool>,
    dbus_rx: &Receiver<dbus_interface::Message>,
    fsevents_rx: &Receiver<FileSystemEvent>,
) -> Result<()> {
    trace!("Entering main loop...");

    events::notify_observers(events::Event::DaemonStartup).unwrap();

    let keyboard_devices_c = keyboard_devices
        .iter()
        .map(|device| device.0.clone())
        .collect::<Vec<KeyboardDevice>>();
    let mouse_devices_c = mouse_devices
        .iter()
        .map(|device| device.0.clone())
        .collect::<Vec<MouseDevice>>();

    // main loop iterations, monotonic counter
    let mut ticks = 0;
    let mut start_time;
    let mut delay_time = Instant::now();

    // used to detect changes of the active slot
    let mut saved_slot = 0;

    let mut saved_brightness = BRIGHTNESS.load(Ordering::SeqCst);

    // used to detect changes to the AFK state
    let mut saved_afk_mode = false;

    // stores indices of failed Lua TXs
    let mut failed_txs = HashSet::new();

    // stores the generation number of the frame that is currently visible on the keyboard
    let saved_frame_generation = AtomicUsize::new(0);

    // used to calculate frames per second
    let mut fps_counter: i32 = 0;
    let mut fps_timer = Instant::now();

    let mut mouse_move_event_last_dispatched: Instant = Instant::now();
    let mut mouse_motion_buf: (i32, i32, i32) = (0, 0, 0);

    let mut sel = Select::new();

    let ctrl_c = sel.recv(&ctrl_c_rx);
    let fs_events = sel.recv(&fsevents_rx);
    let dbus_events = sel.recv(&dbus_rx);

    let mut keyboard_events = vec![];
    for device in keyboard_devices.iter() {
        let index = sel.recv(&device.1);
        keyboard_events.push((index, device));
    }

    let mut mouse_events = vec![];
    for device in mouse_devices.iter() {
        let index = sel.recv(&device.1);
        mouse_events.push((index, device));
    }

    'MAIN_LOOP: loop {
        // update timekeeping and state
        ticks += 1;
        start_time = Instant::now();

        {
            // slot changed?
            let active_slot = ACTIVE_SLOT.load(Ordering::SeqCst);
            if active_slot != saved_slot || ACTIVE_PROFILE.lock().is_none() {
                dbus_api_tx
                    .send(DbusApiEvent::ActiveSlotChanged)
                    .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));

                // reset the audio backend, it will be enabled again if needed
                plugins::audio::reset_audio_backend();

                let profile_path = {
                    let slot_profiles = SLOT_PROFILES.lock();
                    slot_profiles.as_ref().unwrap()[active_slot].clone()
                };

                switch_profile(
                    &profile_path,
                    &dbus_api_tx,
                    &keyboard_devices_c,
                    &mouse_devices_c,
                )?;

                saved_slot = active_slot;
                failed_txs.clear();
            }
        }

        // brightness changed?
        let current_brightness = BRIGHTNESS.load(Ordering::SeqCst);
        if current_brightness != saved_brightness {
            dbus_api_tx
                .send(DbusApiEvent::BrightnessChanged)
                .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));

            saved_brightness = current_brightness;
        }

        // user is AFK?
        let afk_mode = AFK.load(Ordering::SeqCst);
        if afk_mode != saved_afk_mode {
            if afk_mode {
                info!("Entering AFK mode now...");

                let afk_profile = crate::CONFIG
                    .lock()
                    .as_ref()
                    .unwrap()
                    .get::<String>("global.afk_profile")
                    .unwrap_or_else(|_| constants::DEFAULT_AFK_PROFILE.to_owned());

                let active_profile = &*ACTIVE_PROFILE.lock();
                let before_afk = active_profile
                    .as_ref()
                    .unwrap()
                    .profile_file
                    .file_name()
                    .unwrap();

                *ACTIVE_PROFILE_NAME_BEFORE_AFK.lock() =
                    Some(before_afk.to_string_lossy().to_string());

                ACTIVE_PROFILE_NAME.lock().replace(afk_profile);
            } else {
                info!("Leaving AFK mode now...");

                ACTIVE_PROFILE_NAME.lock().replace(
                    ACTIVE_PROFILE_NAME_BEFORE_AFK
                        .lock()
                        .as_ref()
                        .unwrap()
                        .clone(),
                );
            }

            saved_afk_mode = afk_mode;
        }

        {
            // active profile name changed?
            if let Some(active_profile) = &*ACTIVE_PROFILE_NAME.lock() {
                dbus_api_tx
                    .send(DbusApiEvent::ActiveProfileChanged)
                    .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));

                // reset the audio backend, it will be enabled again if needed
                plugins::audio::reset_audio_backend();

                let profile_path = Path::new(active_profile);

                switch_profile(
                    &profile_path,
                    &dbus_api_tx,
                    &keyboard_devices_c,
                    &mouse_devices_c,
                )
                .unwrap_or_else(|e| error!("Could not switch profiles: {}", e));

                failed_txs.clear();
            }

            *ACTIVE_PROFILE_NAME.lock() = None;
        }

        // prepare to call main loop hook
        let plugin_manager = plugin_manager::PLUGIN_MANAGER.read();
        let plugins = plugin_manager.get_plugins();

        // call main loop hook of each registered plugin
        let mut futures = vec![];
        for plugin in plugins.iter() {
            // call the sync main loop hook, intended to be used
            // for very short running pieces of code
            plugin.sync_main_loop_hook(ticks);

            // enqueue a call to the async main loop hook, intended
            // to be used for longer running pieces of code
            futures.push(plugin.main_loop_hook(ticks));
        }

        join_all(futures).await;

        // now, process events from all available sources...
        match sel.select_timeout(Duration::from_millis(1000 / constants::TARGET_FPS)) {
            Ok(oper) => match oper.index() {
                i if i == ctrl_c => {
                    // consume the event, so that we don't cause a panic
                    let _event = &oper.recv(&ctrl_c_rx);
                    break 'MAIN_LOOP;
                }

                i if i == fs_events => {
                    let event = &oper.recv(&fsevents_rx);
                    if let Ok(event) = event {
                        process_filesystem_event(&event, &dbus_api_tx)
                            .await
                            .unwrap_or_else(|e| {
                                error!("Could not process a filesystem event: {}", e)
                            })
                    } else {
                        error!(
                            "Could not process a filesystem event: {}",
                            event.as_ref().unwrap_err()
                        );
                    }
                }

                i if i == dbus_events => {
                    let event = &oper.recv(&dbus_rx);
                    if let Ok(event) = event {
                        process_dbus_event(
                            &event,
                            &dbus_api_tx,
                            &keyboard_devices_c,
                            &mouse_devices_c,
                        )
                        .await
                        .unwrap_or_else(|e| error!("Could not process a D-Bus event: {}", e));

                        failed_txs.clear();
                    } else {
                        error!(
                            "Could not process a D-Bus event: {}",
                            event.as_ref().unwrap_err()
                        );

                        sel.remove(dbus_events);
                    }
                }

                i => {
                    if let Some(event) = keyboard_events.iter().find(|&&e| e.0 == i) {
                        let event = &oper.recv(&(event.1).1);
                        if let Ok(Some(event)) = event {
                            process_keyboard_event(&event, &keyboard_devices[0].0, &failed_txs)
                                .await
                                .unwrap_or_else(|e| {
                                    error!("Could not process a keyboard event: {}", e)
                                });
                        } else {
                            error!(
                                "Could not process a keyboard event: {}",
                                event.as_ref().unwrap_err()
                            );
                        }
                    } else if let Some(event) = mouse_events.iter().find(|&&e| e.0 == i) {
                        let event = &oper.recv(&(event.1).1);
                        if let Ok(Some(event)) = event {
                            process_mouse_event(
                                &event,
                                &mouse_devices[0].0,
                                &failed_txs,
                                &mut mouse_move_event_last_dispatched,
                                &mut mouse_motion_buf,
                            )
                            .await
                            .unwrap_or_else(|e| error!("Could not process a mouse event: {}", e));
                        } else {
                            error!(
                                "Could not process a mouse event: {}",
                                event.as_ref().unwrap_err()
                            );
                        }
                    } else {
                        error!("Invalid or missing event type");
                    }
                }
            },

            Err(_e) => { /* do nothing */ }
        };

        if delay_time.elapsed() >= Duration::from_millis(1000 / (constants::TARGET_FPS * 4)) {
            // poll HID events on all available devices
            for device in keyboard_devices.iter() {
                process_keyboard_hid_events(&device.0, &failed_txs)
                    .await
                    .unwrap_or_else(|e| error!("Could not process a keyboard HID event: {}", e));
            }

            for device in mouse_devices.iter() {
                process_mouse_hid_events(&device.0, &failed_txs)
                    .await
                    .unwrap_or_else(|e| error!("Could not process a mouse HID event: {}", e));
            }
        }

        if delay_time.elapsed() >= Duration::from_millis(1000 / constants::TARGET_FPS) {
            let delta = (delay_time.elapsed().as_millis() as u64 / constants::TARGET_FPS) as u32;

            delay_time = Instant::now();

            // send timer tick events to the Lua VMs
            for (index, lua_tx) in LUA_TXS.lock().iter().enumerate() {
                // if this tx failed previously, then skip it completely
                if !failed_txs.contains(&index) {
                    lua_tx
                        .send(script::Message::Tick(delta))
                        .unwrap_or_else(|e| {
                            error!("Send error during timer tick event: {}", e);
                            failed_txs.insert(index);
                        });
                }
            }

            // finally, update the LEDs if necessary
            let current_frame_generation = script::FRAME_GENERATION_COUNTER.load(Ordering::SeqCst);
            if saved_frame_generation.load(Ordering::SeqCst) < current_frame_generation {
                // instruct the Lua VMs to realize their color maps, but only if at least one VM
                // submitted a new color map (performed a frame generation increment)

                // execute render "pipeline" now...
                let mut drop_frame = false;

                // first, clear the canvas
                script::LED_MAP.write().copy_from_slice(
                    &[hwdevices::RGBA {
                        r: 0,
                        g: 0,
                        b: 0,
                        a: 0,
                    }; constants::CANVAS_SIZE],
                );

                // instruct Lua VMs to realize their color maps,
                // e.g. to blend their local color maps with the canvas
                *COLOR_MAPS_READY_CONDITION.0.lock() = LUA_TXS.lock().len() - failed_txs.len();

                for (index, lua_tx) in LUA_TXS.lock().iter().enumerate() {
                    // if this tx failed previously, then skip it completely
                    if !failed_txs.contains(&index) {
                        // guarantee the right order of execution for the alpha blend
                        // operations, so we have to wait for the current Lua VM to
                        // complete its blending code, before continuing
                        let mut pending = COLOR_MAPS_READY_CONDITION.0.lock();

                        lua_tx
                            .send(script::Message::RealizeColorMap)
                            .unwrap_or_else(|e| {
                                error!("Send error during realization of color maps: {}", e);
                                failed_txs.insert(index);
                            });

                        let result = COLOR_MAPS_READY_CONDITION.1.wait_for(
                            &mut pending,
                            Duration::from_millis(constants::TIMEOUT_CONDITION_MILLIS),
                        );

                        if result.timed_out() {
                            drop_frame = true;
                            warn!("Frame dropped: Timeout while waiting for a lock!");
                            break;
                        }
                    } else {
                        drop_frame = true;
                    }
                }

                // number of pending blend ops should have reached zero by now
                // may currently occur during switching of profiles
                let ops_pending = *COLOR_MAPS_READY_CONDITION.0.lock();
                if ops_pending > 0 {
                    debug!(
                        "Pending blend ops before writing LED map to device: {}",
                        ops_pending
                    );
                }

                // send the final (combined) color map to all of the devices
                if !drop_frame {
                    for device in keyboard_devices.iter() {
                        device
                            .0
                            .write()
                            .send_led_map(&script::LED_MAP.read())
                            .unwrap_or_else(|e| {
                                error!("Could not send the LED map to the device: {}", e)
                            });
                    }

                    for device in mouse_devices.iter() {
                        device
                            .0
                            .write()
                            .send_led_map(&script::LED_MAP.read())
                            .unwrap_or_else(|e| {
                                error!("Could not send the LED map to the device: {}", e)
                            });
                    }

                    // update the current frame generation
                    saved_frame_generation.store(current_frame_generation, Ordering::SeqCst);

                    script::LAST_RENDERED_LED_MAP
                        .write()
                        .copy_from_slice(&script::LED_MAP.read());
                }

                fps_counter += 1;
            }

            // compute AFK time
            let afk_timeout_secs = CONFIG
                .lock()
                .as_ref()
                .unwrap()
                .get_int("global.afk_timeout_secs")
                .unwrap_or_else(|_| constants::AFK_TIMEOUT_SECS as i64)
                as u64;

            if afk_timeout_secs > 0 {
                let afk = LAST_INPUT_TIME.lock().elapsed() >= Duration::from_secs(afk_timeout_secs);
                AFK.store(afk, Ordering::SeqCst);
            }

            let elapsed_after_sleep = start_time.elapsed().as_millis();
            if elapsed_after_sleep > (1000 / constants::TARGET_FPS + 82_u64).into() {
                warn!("More than 82 milliseconds of jitter detected!");
                warn!("This means that we dropped at least one frame");
                warn!(
                    "Loop took: {} milliseconds, goal: {}",
                    elapsed_after_sleep,
                    1000 / constants::TARGET_FPS
                );
            } /* else if elapsed_after_sleep < 5_u128 {
                  debug!("Short loop detected");
                  debug!(
                      "Loop took: {} milliseconds, goal: {}",
                      elapsed_after_sleep,
                      1000 / constants::TARGET_FPS
                  );
              } else {
                  debug!(
                      "Loop took: {} milliseconds, goal: {}",
                      elapsed_after_sleep,
                      1000 / constants::TARGET_FPS
                  );
              } */
        }

        // calculate and log fps each second
        if fps_timer.elapsed().as_millis() >= 1000 {
            debug!("FPS: {}", fps_counter);

            fps_timer = Instant::now();
            fps_counter = 0;
        }

        // shall we quit the main loop?
        if QUIT.load(Ordering::SeqCst) {
            break 'MAIN_LOOP;
        }
    }

    events::notify_observers(events::Event::DaemonShutdown).unwrap();

    Ok(())
}

/// Watch profiles and script directory, as well as our
/// main configuration file for changes
pub fn register_filesystem_watcher(
    fsevents_tx: Sender<FileSystemEvent>,
    config_file: PathBuf,
    profile_dir: PathBuf,
    script_dir: PathBuf,
) -> Result<()> {
    debug!("Registering filesystem watcher...");

    thread::Builder::new()
        .name("hotwatch".to_owned())
        .spawn(
            move || match Hotwatch::new_with_custom_delay(Duration::from_millis(2000)) {
                Err(e) => error!("Could not initialize filesystem watcher: {}", e),

                Ok(ref mut hotwatch) => {
                    hotwatch
                        .watch(config_file, move |_event: Event| {
                            info!("Configuration File changed on disk, please restart eruption for the changes to take effect!");

                            Flow::Continue
                        })
                        .unwrap_or_else(|e| error!("Could not register file watch: {}", e));

                    let fsevents_tx_c = fsevents_tx.clone();

                    hotwatch
                        .watch(profile_dir, move |event: Event| {
                            if let Event::Write(event) = event {
                                info!("Existing profile modified: {:?}", event);
                            } else if let Event::Create(event) = event {
                                info!("New profile created: {:?}", event);
                            } else if let Event::Rename(from, to) = event {
                                info!("Profile file renamed: {:?}", (from, to));
                            } else if let Event::Remove(event) = event {
                                info!("Profile deleted: {:?}", event);
                            }

                            fsevents_tx_c.send(FileSystemEvent::ProfilesChanged).unwrap();

                            Flow::Continue
                        })
                        .unwrap_or_else(|e| error!("Could not register directory watch: {}", e));

                    let fsevents_tx_c = fsevents_tx.clone();

                    hotwatch
                        .watch(script_dir, move |event: Event| {
                            info!("Script file or manifest changed: {:?}", event);

                            fsevents_tx_c.send(FileSystemEvent::ScriptsChanged).unwrap();

                            Flow::Continue
                        })
                        .unwrap_or_else(|e| error!("Could not register directory watch: {}", e));


                    hotwatch.run();
                }
            },
        )?;

    Ok(())
}

#[cfg(debug_assertions)]
mod thread_util {
    use crate::Result;
    use log::*;
    use parking_lot::deadlock;
    use std::thread;
    use std::time::Duration;

    /// Creates a background thread which checks for deadlocks every 5 seconds
    pub(crate) fn deadlock_detector() -> Result<()> {
        thread::Builder::new()
            .name("deadlockd".to_owned())
            .spawn(move || loop {
                thread::sleep(Duration::from_secs(5));
                let deadlocks = deadlock::check_deadlock();
                if !deadlocks.is_empty() {
                    error!("{} deadlocks detected", deadlocks.len());

                    for (i, threads) in deadlocks.iter().enumerate() {
                        error!("Deadlock #{}", i);

                        for t in threads {
                            error!("Thread Id {:#?}", t.thread_id());
                            error!("{:#?}", t.backtrace());
                        }
                    }
                }
            })?;

        Ok(())
    }
}

/// open the control and LED devices of the keyboard
fn init_keyboard_device(keyboard_device: &KeyboardDevice, hidapi: &hidapi::HidApi) {
    info!("Opening keyboard device...");
    keyboard_device.write().open(&hidapi).unwrap_or_else(|e| {
        error!("Error opening the keyboard device: {}", e);
        error!(
            "This could be a permission problem, or maybe the device is locked by another process?"
        );
        process::exit(3);
    });

    // send initialization handshake
    info!("Initializing keyboard device...");
    keyboard_device
        .write()
        .send_init_sequence()
        .unwrap_or_else(|e| error!("Could not initialize the device: {}", e));

    // set LEDs to a known good initial state
    info!("Configuring keyboard LEDs...");
    keyboard_device
        .write()
        .set_led_init_pattern()
        .unwrap_or_else(|e| error!("Could not initialize LEDs: {}", e));

    info!(
        "Firmware revision: {}",
        keyboard_device.read().get_firmware_revision()
    );
}

/// open the sub-devices of the mouse
fn init_mouse_device(mouse_device: &MouseDevice, hidapi: &hidapi::HidApi) {
    info!("Opening mouse device...");

    mouse_device.write().open(&hidapi).unwrap_or_else(|e| {
        error!("Error opening the mouse device: {}", e);
        error!(
            "This could be a permission problem, or maybe the device is locked by another process?"
        );
    });

    // send initialization handshake
    info!("Initializing mouse device...");
    mouse_device
        .write()
        .send_init_sequence()
        .unwrap_or_else(|e| error!("Could not initialize the device: {}", e));

    // set LEDs to a known good initial state
    info!("Configuring mouse LEDs...");
    mouse_device
        .write()
        .set_led_init_pattern()
        .unwrap_or_else(|e| error!("Could not initialize LEDs: {}", e));

    info!(
        "Firmware revision: {}",
        mouse_device.read().get_firmware_revision()
    );
}

/// Main program entrypoint
#[tokio::main]
pub async fn main() -> std::result::Result<(), eyre::Error> {
    color_eyre::install()?;

    if unsafe { libc::isatty(0) != 0 } {
        print_header();
    }

    // start the thread deadlock detector
    #[cfg(debug_assertions)]
    thread_util::deadlock_detector()
        .unwrap_or_else(|e| error!("Could not spawn deadlock detector thread: {}", e));

    let matches = parse_commandline();

    // initialize logging
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG_OVERRIDE", "info");
        pretty_env_logger::init_custom_env("RUST_LOG_OVERRIDE");
    } else {
        pretty_env_logger::init();
    }

    info!(
        "Starting Eruption - Linux user-mode input and LED driver for keyboards, mice and other devices: Version {}",
        env!("CARGO_PKG_VERSION")
    );

    // register ctrl-c handler
    let (ctrl_c_tx, ctrl_c_rx) = unbounded();
    ctrlc::set_handler(move || {
        QUIT.store(true, Ordering::SeqCst);

        ctrl_c_tx
            .send(true)
            .unwrap_or_else(|e| error!("Could not send on a channel: {}", e));
    })
    .unwrap_or_else(|e| error!("Could not set CTRL-C handler: {}", e));

    // process configuration file
    let config_file = matches
        .value_of("config")
        .unwrap_or(constants::DEFAULT_CONFIG_FILE);

    let mut config = config::Config::default();
    config
        .merge(config::File::new(&config_file, config::FileFormat::Toml))
        .unwrap_or_else(|e| {
            error!("Could not parse configuration file: {}", e);
            process::exit(4);
        });

    *CONFIG.lock() = Some(config.clone());

    // enable support for experimental features?
    let enable_experimental_features = config
        .get::<bool>("global.enable_experimental_features")
        .unwrap_or(false);

    EXPERIMENTAL_FEATURES.store(enable_experimental_features, Ordering::SeqCst);

    if EXPERIMENTAL_FEATURES.load(Ordering::SeqCst) {
        warn!("** EXPERIMENTAL FEATURES are ENABLED, this may expose serious bugs! **");
    }

    // load and initialize global runtime state
    info!("Loading saved state...");
    state::init_global_runtime_state()
        .unwrap_or_else(|e| warn!("Could not parse state file: {}", e));

    // default directories
    let profile_dir = config
        .get_str("global.profile_dir")
        .unwrap_or_else(|_| constants::DEFAULT_PROFILE_DIR.to_string());
    let profile_path = PathBuf::from(&profile_dir);

    let script_dir = config
        .get_str("global.script_dir")
        .unwrap_or_else(|_| constants::DEFAULT_SCRIPT_DIR.to_string());

    // enable the mouse
    let enable_mouse = config.get::<bool>("global.enable_mouse").unwrap_or(true);

    // create the one and only hidapi instance
    match hidapi::HidApi::new() {
        Ok(hidapi) => {
            // initialize plugins
            info!("Registering plugins...");
            plugins::register_plugins()
                .unwrap_or_else(|_e| error!("Could not register one or more plugins"));

            // load plugin state from disk
            plugins::PersistencePlugin::load_persistent_data()
                .unwrap_or_else(|e| warn!("Could not load persisted state: {}", e));

            info!("Plugins loaded and initialized successfully");

            // enumerate devices
            info!("Enumerating connected devices...");

            if let Ok(devices) = hwdevices::probe_hid_devices(&hidapi) {
                // store device handles and associated sender/receiver pairs
                let mut keyboard_devices = vec![];
                let mut mouse_devices = vec![];

                // initialize keyboard devices
                for (index, device) in devices.0.iter().enumerate() {
                    init_keyboard_device(&device, &hidapi);

                    let usb_vid = device.read().get_usb_vid();
                    let usb_pid = device.read().get_usb_pid();

                    // spawn a thread to handle keyboard input
                    info!("Spawning keyboard input thread...");

                    let (kbd_tx, kbd_rx) = unbounded();
                    spawn_keyboard_input_thread(
                        kbd_tx.clone(),
                        device.clone(),
                        index,
                        usb_vid,
                        usb_pid,
                    )
                    .unwrap_or_else(|e| {
                        error!("Could not spawn a thread: {}", e);
                        panic!()
                    });

                    keyboard_devices.push((device, kbd_rx, kbd_tx));
                    crate::KEYBOARD_DEVICES.lock().push(device.clone());
                }

                // initialize mouse devices
                for (index, device) in devices.1.iter().enumerate() {
                    // enable mouse input
                    if enable_mouse {
                        init_mouse_device(&device, &hidapi);

                        let usb_vid = device.read().get_usb_vid();
                        let usb_pid = device.read().get_usb_pid();

                        let (mouse_tx, mouse_rx) = unbounded();
                        let (mouse_secondary_tx, _mouse_secondary_rx) = unbounded();

                        // spawn a thread to handle mouse input
                        info!("Spawning mouse input thread...");

                        spawn_mouse_input_thread(
                            mouse_tx.clone(),
                            device.clone(),
                            index,
                            usb_vid,
                            usb_pid,
                        )
                        .unwrap_or_else(|e| {
                            error!("Could not spawn a thread: {}", e);
                            panic!()
                        });

                        // spawn a thread to handle possible sub-devices
                        if EXPERIMENTAL_FEATURES.load(Ordering::SeqCst)
                            && device.read().has_secondary_device()
                        {
                            info!("Spawning mouse input thread for secondary sub-device...");
                            spawn_mouse_input_thread_secondary(
                                mouse_secondary_tx,
                                device.clone(),
                                index,
                                usb_vid,
                                usb_pid,
                            )
                            .unwrap_or_else(|e| {
                                error!("Could not spawn a thread: {}", e);
                                panic!()
                            });
                        }

                        mouse_devices.push((device, mouse_rx, mouse_tx));
                        crate::MOUSE_DEVICES.lock().push(device.clone());
                    } else {
                        info!("Found mouse device, but mouse support is DISABLED by configuration");
                    }
                }

                info!("Device enumeration completed");

                info!("Performing late initializations...");

                // initialize the D-Bus API
                info!("Initializing D-Bus API...");
                let (dbus_tx, dbus_rx) = unbounded();
                let dbus_api_tx = spawn_dbus_api_thread(dbus_tx).unwrap_or_else(|e| {
                    error!("Could not spawn a thread: {}", e);
                    panic!()
                });

                let (fsevents_tx, fsevents_rx) = unbounded();
                register_filesystem_watcher(
                    fsevents_tx,
                    PathBuf::from(&config_file),
                    profile_path,
                    PathBuf::from(&script_dir),
                )
                .unwrap_or_else(|e| error!("Could not register file changes watcher: {}", e));

                info!("Late initializations completed");

                info!("Startup completed");

                debug!("Entering the main loop now...");

                // enter the main loop
                run_main_loop(
                    keyboard_devices
                        .iter()
                        .map(|d| (d.0.clone(), d.1.clone()))
                        .collect::<Vec<_>>(),
                    mouse_devices
                        .iter()
                        .map(|d| (d.0.clone(), d.1.clone()))
                        .collect::<Vec<_>>(),
                    &dbus_api_tx,
                    &ctrl_c_rx,
                    &dbus_rx,
                    &fsevents_rx,
                )
                .await
                .unwrap_or_else(|e| error!("{}", e));

                debug!("Left the main loop");

                // we left the main loop, so send a final message to the running Lua VMs
                *UPCALL_COMPLETED_ON_QUIT.0.lock() = LUA_TXS.lock().len();

                for lua_tx in LUA_TXS.lock().iter() {
                    lua_tx
                        .send(script::Message::Quit(0))
                        .unwrap_or_else(|e| error!("Could not send quit message: {}", e));
                }

                // wait until all Lua VMs completed the event handler
                loop {
                    let mut pending = UPCALL_COMPLETED_ON_QUIT.0.lock();

                    let result = UPCALL_COMPLETED_ON_QUIT
                        .1
                        .wait_for(&mut pending, Duration::from_millis(2500));

                    if result.timed_out() {
                        warn!("Timed out while waiting for a Lua VM to shut down, terminating now");
                        break;
                    }

                    if *pending == 0 {
                        break;
                    }
                }

                // store plugin state to disk
                plugins::PersistencePlugin::store_persistent_data()
                    .unwrap_or_else(|e| error!("Could not write persisted state: {}", e));

                thread::sleep(Duration::from_millis(
                    constants::SHUTDOWN_TIMEOUT_MILLIS as u64,
                ));

                // set LEDs of all keyboards to a known final state, then close all associated devices
                for device in keyboard_devices.iter() {
                    device
                        .0
                        .write()
                        .set_led_off_pattern()
                        .unwrap_or_else(|e| error!("Could not finalize LEDs configuration: {}", e));

                    device.0.write().close_all().unwrap_or_else(|e| {
                        warn!("Could not close the device: {}", e);
                    });
                }

                // set LEDs of all mice to a known final state, then close all associated devices
                for device in mouse_devices.iter() {
                    device
                        .0
                        .write()
                        .set_led_off_pattern()
                        .unwrap_or_else(|e| error!("Could not finalize LEDs configuration: {}", e));

                    device.0.write().close_all().unwrap_or_else(|e| {
                        warn!("Could not close the device: {}", e);
                    });
                }
            } else {
                error!("Could not enumerate connected devices");
                process::exit(2);
            }
        }

        Err(_) => {
            error!("Could not open HIDAPI");
            process::exit(1);
        }
    }

    // save state
    debug!("Saving state...");
    state::save_runtime_state().unwrap_or_else(|e| error!("Could not save runtime state: {}", e));

    info!("Exiting now");

    Ok(())
}
