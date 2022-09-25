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

// use async_macros::join;
use clap::{Arg, Command};
use config::Config;
use flume::{unbounded, Receiver, Selector, Sender};
use hotwatch::{
    blocking::{Flow, Hotwatch},
    Event,
};
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};
use lazy_static::lazy_static;
use log::*;
use parking_lot::{Condvar, Mutex, RwLock};
use rust_embed::RustEmbed;
use std::fs::{self};
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::u64;
use std::{collections::HashMap, env};
use std::{collections::HashSet, thread};
use syslog::Facility;
use tokio::join;

mod logger;
mod threads;
use threads::*;

mod util;

mod hwdevices;
use hwdevices::{KeyboardDevice, KeyboardHidEvent, MiscDevice, MouseDevice, MouseHidEvent};

mod color_scheme;
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

use crate::plugins::{sdk_support, uleds};
use crate::{
    color_scheme::ColorScheme,
    hwdevices::{DeviceStatus, MaturityLevel, RGBA},
};

use crate::threads::DbusApiEvent;
#[cfg(feature = "mimalloc_allocator")]
use mimalloc::MiMalloc;

#[cfg(feature = "mimalloc_allocator")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(RustEmbed)]
#[folder = "i18n"] // path to the compiled localization resources
struct Localizations;

lazy_static! {
    /// Global configuration
    pub static ref STATIC_LOADER: Arc<Mutex<Option<FluentLanguageLoader>>> = Arc::new(Mutex::new(None));
}

#[allow(unused)]
macro_rules! tr {
    ($message_id:literal) => {{
        let loader = $crate::STATIC_LOADER.lock();
        let loader = loader.as_ref().unwrap();

        i18n_embed_fl::fl!(loader, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        let loader = $crate::STATIC_LOADER.lock();
        let loader = loader.as_ref().unwrap();

        i18n_embed_fl::fl!(loader, $message_id, $($args), *)
    }};
}

lazy_static! {
    /// Managed keyboard devices
    pub static ref KEYBOARD_DEVICES: Arc<RwLock<Vec<hwdevices::KeyboardDevice>>> = Arc::new(RwLock::new(Vec::new()));

    pub static ref KEYBOARD_DEVICES_RX: Arc<RwLock<Vec<Receiver<Option<evdev_rs::InputEvent>>>>> = Arc::new(RwLock::new(Vec::new()));


    /// Managed mouse devices
    pub static ref MOUSE_DEVICES: Arc<RwLock<Vec<hwdevices::MouseDevice>>> = Arc::new(RwLock::new(Vec::new()));

    pub static ref MOUSE_DEVICES_RX: Arc<RwLock<Vec<Receiver<Option<evdev_rs::InputEvent>>>>> = Arc::new(RwLock::new(Vec::new()));


    /// Managed miscellaneous devices
    pub static ref MISC_DEVICES: Arc<RwLock<Vec<hwdevices::MiscDevice>>> = Arc::new(RwLock::new(Vec::new()));

    pub static ref MISC_DEVICES_RX: Arc<RwLock<Vec<Receiver<Option<evdev_rs::InputEvent>>>>> = Arc::new(RwLock::new(Vec::new()));


    /// Hidapi object
    pub static ref HIDAPI: Arc<RwLock<Option<hidapi::HidApi>>> = Arc::new(RwLock::new(None));

    /// Holds device status information, like e.g: current signal strength or battery levels
    pub static ref DEVICE_STATUS: Arc<Mutex<HashMap<u64, DeviceStatus>>> =
        Arc::new(Mutex::new(HashMap::new()));

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

    /// Named color schemes, for use in e.g. gradients
    pub static ref NAMED_COLOR_SCHEMES: Arc<RwLock<HashMap<String, ColorScheme>>> =
        Arc::new(RwLock::new(HashMap::new()));

    /// Global configuration
    pub static ref CONFIG: Arc<Mutex<Option<config::Config>>> = Arc::new(Mutex::new(None));

    // Flags

    /// Global "quit" status flag
    pub static ref QUIT: AtomicBool = AtomicBool::new(false);

    /// Global "quit and the re-enter the main loop" status flag
    pub static ref REENTER_MAIN_LOOP: AtomicBool = AtomicBool::new(false);

    /// Global "is AFK" status flag
    pub static ref AFK: AtomicBool = AtomicBool::new(false);

    /// Global "request to reload the profile" flag
    pub static ref REQUEST_PROFILE_RELOAD: AtomicBool = AtomicBool::new(false);

    /// Global "request to enter failsafe mode" flag
    pub static ref REQUEST_FAILSAFE_MODE: AtomicBool = AtomicBool::new(false);

    /// Global "enable experimental features" flag
    pub static ref EXPERIMENTAL_FEATURES: AtomicBool = AtomicBool::new(false);

    /// Global "driver maturity level" param
    pub static ref DRIVER_MATURITY_LEVEL: Arc<Mutex<MaturityLevel>> = Arc::new(Mutex::new(MaturityLevel::Stable));


    /// Global "enable SDK support" flag
    pub static ref SDK_SUPPORT_ACTIVE: AtomicBool = AtomicBool::new(false);

    /// Global "enable Linux Userspace LEDs support" flag
    pub static ref ULEDS_SUPPORT_ACTIVE: AtomicBool = AtomicBool::new(false);


    // Other state

    /// Global "keyboard brightness" modifier
    pub static ref BRIGHTNESS: AtomicIsize = AtomicIsize::new(100);

    /// Global modifier when fading into a profile
    pub static ref BRIGHTNESS_FADER: AtomicIsize = AtomicIsize::new(0);
    
    /// Global modifier to compare fading into a profile
    pub static ref BRIGHTNESS_FADER_BASE: AtomicIsize = AtomicIsize::new(0);
    
    /// AFK timer
    pub static ref LAST_INPUT_TIME: Arc<Mutex<Instant>> = Arc::new(Mutex::new(Instant::now()));

    /// Channel to the D-Bus interface
    pub static ref DBUS_API_TX: Arc<Mutex<Option<Sender<DbusApiEvent>>>> = Arc::new(Mutex::new(None));

    /// Channel to the device I/O thread
    pub static ref DEV_IO_TX: Arc<Mutex<Option<Sender<DeviceAction >>>> = Arc::new(Mutex::new(None));

    /// Channels to the Lua VMs
    pub static ref LUA_TXS: Arc<RwLock<Vec<LuaTx>>> = Arc::new(RwLock::new(vec![]));
    pub static ref FAILED_TXS: Arc<RwLock<HashSet<usize>>> = Arc::new(RwLock::new(HashSet::new()));

    /// Key states
    pub static ref KEY_STATES: Arc<RwLock<Vec<bool>>> = Arc::new(RwLock::new(vec![false; constants::MAX_KEYS]));

    pub static ref BUTTON_STATES: Arc<RwLock<Vec<bool>>> = Arc::new(RwLock::new(vec![false; constants::MAX_MOUSE_BUTTONS]));

    pub static ref MOUSE_MOVE_EVENT_LAST_DISPATCHED: Arc<RwLock<Instant>> = Arc::new(RwLock::new(Instant::now()));
    pub static ref MOUSE_MOTION_BUF: Arc<RwLock<(i32, i32, i32)>> = Arc::new(RwLock::new((0,0,0)));

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

    #[error("Could not parse syslog log-level")]
    SyslogLevelError {},
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

/// A LuaTx holds a Sender<T> as well as the path to the running script file
pub struct LuaTx {
    pub script_file: PathBuf,
    pub sender: Sender<script::Message>,
    pub is_failed: bool,
}

impl LuaTx {
    pub fn new(script_file: PathBuf, sender: Sender<script::Message>) -> Self {
        Self {
            script_file,
            sender,
            is_failed: false,
        }
    }
}

impl std::ops::Deref for LuaTx {
    type Target = Sender<script::Message>;

    fn deref(&self) -> &Self::Target {
        &self.sender
    }
}

#[derive(Debug, Clone)]
pub enum EventAction {
    Created,
    Modified,
    Deleted,
}

#[derive(Debug, Clone)]
pub enum FileSystemEvent {
    ProfileChanged { action: EventAction, path: PathBuf },
    ScriptChanged,
}

#[derive(Debug, Clone)]
pub enum DeviceAction {
    RenderNow,
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

 Copyright (c) 2019-2022, The Eruption Development Team
"#
    );
}

/// Process commandline options
fn parse_commandline() -> clap::ArgMatches {
    Command::new("Eruption")
        .version(
            format!(
                "{} ({}) ({} build)",
                env!("CARGO_PKG_VERSION"),
                env!("ERUPTION_GIT_PKG_VERSION"),
                if cfg!(debug_assertions) {
                    "debug"
                } else {
                    "release"
                }
            )
            .as_str(),
        )
        .author("X3n0m0rph59 <x3n0m0rph59@gmail.com>")
        .about("Realtime RGB LED Driver for Linux")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Sets the configuration file to use")
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

/// Switches the currently active profile to the profile file `profile_file`
/// Returns Ok(true) if the new profile has been activated or the old profile was kept,
/// otherwise returns Ok(false) when we entered failsafe mode. If an error occurred during
/// switching to failsafe mode, we return an Err() to signal a fatal error
pub fn switch_profile(
    profile_file: Option<&Path>,
    dbus_api_tx: &Sender<DbusApiEvent>,
    notify: bool,
) -> Result<bool> {
    fn switch_to_failsafe_profile(dbus_api_tx: &Sender<DbusApiEvent>, notify: bool) -> Result<()> {
        let mut errors_present = false;

        // force hardcoded directory for failsafe scripts
        let script_dir = PathBuf::from("/usr/share/eruption/scripts/");

        let profile = profiles::get_fail_safe_profile();

        // now spawn a new set of Lua VMs, with scripts from the failsafe profile
        for (thread_idx, script_file) in profile.active_scripts.iter().enumerate() {
            // TODO: use path from config
            let script_path = script_dir.join(&script_file);

            let (lua_tx, lua_rx) = unbounded();
            threads::spawn_lua_thread(thread_idx, lua_rx, script_path.clone(), None)
                .unwrap_or_else(|e| {
                    errors_present = true;

                    error!("Could not spawn a thread: {}", e);
                });

            let mut tx = LuaTx::new(script_path.clone(), lua_tx);

            if errors_present {
                tx.is_failed = true
            }

            LUA_TXS.write().push(tx);
        }

        // finally assign the globally active profile
        *ACTIVE_PROFILE.lock() = Some(profile);

        if notify {
            dbus_api_tx
                .send(DbusApiEvent::ActiveProfileChanged)
                .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));
        }

        // let active_slot = ACTIVE_SLOT.load(Ordering::SeqCst);

        // let mut slot_profiles = SLOT_PROFILES.lock();
        // slot_profiles.as_mut().unwrap()[active_slot] = "failsafe.profile".into();

        if errors_present {
            error!("Fatal error: An error occurred while loading the failsafe profile");
            Err(MainError::SwitchProfileError {}.into())
        } else {
            Ok(())
        }
    }

    if REQUEST_FAILSAFE_MODE.load(Ordering::SeqCst) {
        debug!("Preparing to enter failsafe mode...");

        // request termination of all Lua VMs

        for lua_tx in LUA_TXS.read().iter() {
            if !lua_tx.is_failed {
                lua_tx
                    .send(script::Message::Unload)
                    .unwrap_or_else(|e| error!("Could not send an event to a Lua VM: {}", e));
            } else {
                warn!("Skipping unload of a failed tx");
            }
        }

        // be safe and clear any leftover channels
        LUA_TXS.write().clear();

        switch_to_failsafe_profile(dbus_api_tx, notify)?;
        REQUEST_FAILSAFE_MODE.store(false, Ordering::SeqCst);

        debug!("Successfully entered failsafe mode");

        Ok(false)
    } else {
        // we require profile_file to be set in this branch
        let profile_file = if let Some(profile_file) = profile_file {
            profile_file
        } else {
            error!("Undefined profile");
            return Err(MainError::SwitchProfileError {}.into());
        };

        info!("Switching to profile: {}", &profile_file.display());

        let profile = profiles::Profile::from(profile_file);

        if let Ok(profile) = profile {
            let mut errors_present = false;

            // verify script files first; better fail early if we can
            let script_files = profile.active_scripts.clone();
            for script_file in script_files.iter() {
                let script_path = util::match_script_path(&script_file);

                let mut is_script_file_accessible = false;
                let mut is_manifest_file_accessible = false;

                if let Ok(script_path) = script_path {
                    is_script_file_accessible = util::is_script_file_accessible(&script_path);
                    is_manifest_file_accessible = util::is_manifest_file_accessible(&script_path);
                }

                if !is_script_file_accessible || !is_manifest_file_accessible {
                    error!(
                        "Script file or manifest inaccessible: {}",
                        &script_file.display()
                    );

                    // errors_present = true;

                    // the profile to switch to refers to invalid script files, so we need to refuse to
                    // switch profiles and simply keep the current one, or load a failsafe profile if we
                    // do not have a currently active profile, like e.g. during startup
                    if crate::ACTIVE_PROFILE.lock().is_none() {
                        error!("An error occurred during switching of profiles, loading failsafe profile now");
                        switch_to_failsafe_profile(dbus_api_tx, notify)?;

                        return Ok(false);
                    } else {
                        error!(
                            "Invalid profile: {}, refusing to switch profiles",
                            profile_file.display()
                        );

                        return Ok(true);
                    }
                }
            }

            // now request termination of all Lua VMs

            for lua_tx in LUA_TXS.read().iter() {
                if !lua_tx.is_failed {
                    lua_tx
                        .send(script::Message::Unload)
                        .unwrap_or_else(|e| error!("Could not send an event to a Lua VM: {}", e));
                } else {
                    warn!("Skipping unload of a failed tx");
                }
            }

            // be safe and clear any leftover channels
            LUA_TXS.write().clear();

            // we passed the point of no return, from here on we can't just go back
            // but need to switch to failsafe mode when we encounter any critical errors

            let mut num_vms = 0; // only valid if no errors occurred

            // now spawn a new set of Lua VMs, with scripts from the new profile
            for (thread_idx, script_file) in script_files.iter().enumerate() {
                let script_path = util::match_script_path(&script_file)?;

                let (lua_tx, lua_rx) = unbounded();
                if let Err(e) = threads::spawn_lua_thread(
                    thread_idx,
                    lua_rx,
                    script_path.clone(),
                    Some(profile.clone()),
                ) {
                    errors_present = true;

                    error!("Could not spawn a thread: {}", e);
                }

                let mut tx = LuaTx::new(script_path.clone(), lua_tx);

                if !errors_present {
                    num_vms += 1;
                } else {
                    tx.is_failed = true;
                }

                LUA_TXS.write().push(tx);
            }

            // it seems that at least one Lua VM failed during loading of the new profile,
            // so we have to switch to failsafe mode to be safe
            if errors_present || num_vms == 0 {
                error!(
                    "An error occurred during switching of profiles, loading failsafe profile now"
                );
                switch_to_failsafe_profile(dbus_api_tx, notify)?;

                Ok(false)
            } else {
                // everything is fine, finally assign the globally active profile
                debug!("Switch successful");

                let fade_millis = crate::CONFIG
                    .lock()
                    .as_ref()
                    .unwrap()
                    .get_int("global.profile_fade_milliseconds")
                    .unwrap_or(constants::FADE_MILLIS as i64);
                let fade_frames = (fade_millis * constants::TARGET_FPS as i64 / 1000) as isize;
                crate::BRIGHTNESS_FADER.store(fade_frames, Ordering::SeqCst);
                crate::BRIGHTNESS_FADER_BASE.store(fade_frames, Ordering::SeqCst);

                *ACTIVE_PROFILE.lock() = Some(profile);

                if notify {
                    dbus_api_tx
                        .send(DbusApiEvent::ActiveProfileChanged)
                        .unwrap_or_else(|e| {
                            error!("Could not send a pending dbus API event: {}", e)
                        });
                }

                let active_slot = ACTIVE_SLOT.load(Ordering::SeqCst);
                let mut slot_profiles = SLOT_PROFILES.lock();
                slot_profiles.as_mut().unwrap()[active_slot] = profile_file.into();

                Ok(true)
            }
        } else {
            // the profile file to switch to is corrupted, so we need to refuse to switch profiles
            // and simply keep the current one, or load a failsafe profile if we do not have a
            // currently active profile, like e.g. during startup of the daemon
            if crate::ACTIVE_PROFILE.lock().is_none() {
                error!(
                    "An error occurred during switching of profiles, loading failsafe profile now"
                );
                switch_to_failsafe_profile(dbus_api_tx, notify)?;

                Ok(false)
            } else {
                error!(
                    "Invalid profile: {}, refusing to switch profiles",
                    profile_file.display()
                );

                Ok(true)
            }
        }
    }
}

fn run_main_loop(
    dbus_api_tx: &Sender<DbusApiEvent>,
    ctrl_c_rx: &Receiver<bool>,
    dbus_rx: &Receiver<dbus_interface::Message>,
    fsevents_rx: &Receiver<FileSystemEvent>,
) -> Result<()> {
    trace!("Entering main loop...");

    events::notify_observers(events::Event::DaemonStartup).unwrap();

    // main loop iterations, monotonic counter
    let mut ticks = 0;
    let mut start_time;
    let mut watchdog_time = Instant::now();
    let mut delay_time_hid_poll = Instant::now();
    let mut delay_time_render = Instant::now();
    let mut last_status_poll = Instant::now();

    // used to detect changes of the active slot
    let mut saved_slot = 0;

    let mut saved_brightness = BRIGHTNESS.load(Ordering::SeqCst);

    // used to detect changes to the AFK state
    let mut saved_afk_mode = false;

    let kbd_rxs = crate::KEYBOARD_DEVICES_RX.read();
    let mouse_rxs = crate::MOUSE_DEVICES_RX.read();

    'MAIN_LOOP: loop {
        #[cfg(feature = "profiling")]
        coz::scope!("main loop");

        let mut sel = Selector::new()
            .recv(ctrl_c_rx, |_event| {
                QUIT.store(true, Ordering::SeqCst);
            })
            .recv(fsevents_rx, |event| {
                if let Ok(event) = event {
                    events::process_filesystem_event(&event, dbus_api_tx)
                        .unwrap_or_else(|e| error!("Could not process a filesystem event: {}", e))
                } else {
                    error!(
                        "Could not process a filesystem event: {}",
                        event.as_ref().unwrap_err()
                    );
                }
            })
            .recv(dbus_rx, |event| {
                if let Ok(event) = event {
                    events::process_dbus_event(&event, dbus_api_tx)
                        .unwrap_or_else(|e| error!("Could not process a D-Bus event: {}", e));

                    // FAILED_TXS.write().clear();
                } else {
                    error!(
                        "Fatal: Could not process a D-Bus event: {}",
                        event.as_ref().unwrap_err()
                    );

                    QUIT.store(true, Ordering::SeqCst);
                }
            });

        for rx in kbd_rxs.iter() {
            let mapper = move |event| {
                if let Ok(Some(event)) = event {
                    // TODO: support multiple keyboards
                    events::process_keyboard_event(&event, &crate::KEYBOARD_DEVICES.read()[0])
                        .unwrap_or_else(|e| error!("Could not process a keyboard event: {}", e));
                } else {
                    error!(
                        "Could not process a keyboard event: {}",
                        event.as_ref().unwrap_err()
                    );
                }
            };

            sel = sel.recv(rx, mapper);
        }

        for (index, rx) in mouse_rxs.iter().enumerate() {
            let mapper = move |event| {
                if let Ok(Some(event)) = event {
                    events::process_mouse_event(&event, &crate::MOUSE_DEVICES.read()[0])
                        .unwrap_or_else(|e| error!("Could not process a mouse event: {}", e));
                } else {
                    error!(
                        "Could not process a mouse event: {}",
                        event.as_ref().unwrap_err()
                    );

                    FAILED_TXS.write().insert(index);

                    // remove failed devices
                    REENTER_MAIN_LOOP.store(true, Ordering::SeqCst);
                }
            };

            sel = sel.recv(rx, mapper);
        }

        // update timekeeping and state
        ticks += 1;
        start_time = Instant::now();

        // check if we shall terminate the main loop (and later re-enter it)
        // this is needed e.g. after a device hotplug event or after device removal
        if REENTER_MAIN_LOOP.load(Ordering::SeqCst) {
            // reset flag
            crate::REENTER_MAIN_LOOP.store(false, Ordering::SeqCst);

            return Ok(());
        }

        {
            if REQUEST_FAILSAFE_MODE.load(Ordering::SeqCst) {
                warn!("Entering failsafe mode now, due to previous irrecoverable errors");

                // forbid changing of profile and/or slots now
                *ACTIVE_PROFILE_NAME.lock() = None;
                saved_slot = ACTIVE_SLOT.load(Ordering::SeqCst);

                dbus_api_tx
                    .send(DbusApiEvent::ActiveProfileChanged)
                    .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));

                // reset the audio backend, it will be enabled again if needed
                plugins::audio::reset_audio_backend();

                if let Err(e) = switch_profile(None, dbus_api_tx, true) {
                    error!("Could not switch profiles: {}", e);
                }

                FAILED_TXS.write().clear();
            }
        }

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

                switch_profile(Some(&profile_path), dbus_api_tx, true)?;

                saved_slot = active_slot;
                FAILED_TXS.write().clear();
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
                let before_afk = &active_profile.as_ref().unwrap().profile_file;

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

                if let Err(e) = switch_profile(Some(profile_path), dbus_api_tx, true) {
                    error!("Could not switch profiles: {}", e);
                }

                FAILED_TXS.write().clear();
            }

            *ACTIVE_PROFILE_NAME.lock() = None;
        }

        {
            // reload of current profile requested?
            if REQUEST_PROFILE_RELOAD.load(Ordering::SeqCst) {
                // don't notify "active profile changed", since it may deadlock

                // dbus_api_tx
                //     .send(DbusApiEvent::ActiveProfileChanged)
                //     .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));

                // reset the audio backend, it will be enabled again if needed
                plugins::audio::reset_audio_backend();

                let active_profile = ACTIVE_PROFILE.lock();
                let profile_clone = active_profile.clone();
                // ACTIVE_PROFILE.lock() needs to be released here, or otherwise we will deadlock
                drop(active_profile);

                if let Some(profile) = &profile_clone {
                    if let Err(e) = switch_profile(Some(&profile.profile_file), dbus_api_tx, false)
                    {
                        error!("Could not reload profile: {}", e);
                    }
                }

                REQUEST_PROFILE_RELOAD.store(false, Ordering::SeqCst);
            }
        }

        {
            #[cfg(feature = "profiling")]
            coz::scope!("main loop hooks");

            // prepare to call main loop hook
            let plugin_manager = plugin_manager::PLUGIN_MANAGER.read();
            let plugins = plugin_manager.get_plugins();

            // call main loop hook of each registered plugin
            // let mut futures = vec![];
            for plugin in plugins.iter() {
                // call the sync main loop hook, intended to be used
                // for very short running pieces of code
                plugin.sync_main_loop_hook(ticks);

                // enqueue a call to the async main loop hook, intended
                // to be used for longer running pieces of code
                // futures.push(plugin.main_loop_hook(ticks));
            }

            // join_all(futures);
        }

        if last_status_poll.elapsed()
            >= Duration::from_millis(constants::POLL_TIMER_INTERVAL_MILLIS)
        {
            #[cfg(feature = "profiling")]
            coz::scope!("device status polling");

            let saved_status = crate::DEVICE_STATUS.as_ref().lock().clone();

            if let Err(_e) = events::process_timer_event() {
                /* do nothing  */

                // if e.type_id() == (HwDeviceError::NoOpResult {}).type_id() {
                //     error!("Could not process a timer event: {}", e);
                // } else {
                //     trace!("Result is a NoOp");
                // }
            }

            last_status_poll = Instant::now();

            let current_status = crate::DEVICE_STATUS.lock().clone();

            if current_status != saved_status {
                dbus_api_tx
                    .send(DbusApiEvent::DeviceStatusChanged)
                    .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));
            }
        }

        // now, process events from all available sources...
        let _result = sel.wait_timeout(Duration::from_millis(1000 / (constants::TARGET_FPS * 2)));

        if delay_time_hid_poll.elapsed()
            >= Duration::from_millis(1000 / (constants::TARGET_FPS * 8))
        {
            #[cfg(feature = "profiling")]
            coz::scope!("HID events polling");

            delay_time_hid_poll = Instant::now();

            // poll HID events on all available devices
            for device in crate::KEYBOARD_DEVICES.read().iter() {
                events::process_keyboard_hid_events(device)
                    .unwrap_or_else(|e| error!("Could not process a keyboard HID event: {}", e));
            }

            for device in crate::MOUSE_DEVICES.read().iter() {
                events::process_mouse_hid_events(device)
                    .unwrap_or_else(|e| error!("Could not process a mouse HID event: {}", e));
            }
        }

        if delay_time_render.elapsed() >= Duration::from_millis(1000 / constants::TARGET_FPS) {
            #[cfg(feature = "profiling")]
            coz::scope!("render code");

            let delta =
                (delay_time_render.elapsed().as_millis() as u64 / constants::TARGET_FPS) as u32;

            delay_time_render = Instant::now();

            // send timer tick events to the Lua VMs
            for (index, lua_tx) in LUA_TXS.read().iter().enumerate() {
                // if this tx failed previously, then skip it completely
                if !FAILED_TXS.read().contains(&index) {
                    lua_tx
                        .send(script::Message::Tick(delta))
                        .unwrap_or_else(|e| {
                            error!("Send error during timer tick event: {}", e);
                            FAILED_TXS.write().insert(index);
                        });
                }
            }

            // finally, update the LEDs if necessary
            DEV_IO_TX
                .lock()
                .as_ref()
                .unwrap()
                .send(DeviceAction::RenderNow)
                .unwrap_or_else(|e| {
                    error!("Send error: {}", e);
                });
        }

        let fader = crate::BRIGHTNESS_FADER.load(Ordering::SeqCst);
        if fader > 0 {
            crate::BRIGHTNESS_FADER.store(fader - 1, Ordering::SeqCst);
        }

        // compute AFK time
        let afk_timeout_secs = crate::CONFIG
            .lock()
            .as_ref()
            .unwrap()
            .get_int("global.afk_timeout_secs")
            .unwrap_or(constants::AFK_TIMEOUT_SECS as i64) as u64;

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
          } */
        /*
        else {
            debug!(
                "Loop took: {} milliseconds, goal: {}",
                elapsed_after_sleep,
                1000 / constants::TARGET_FPS
            );
        } */

        // notify the software watchdog that we are still "alive"
        if watchdog_time.elapsed() >= Duration::from_millis(constants::WATCHDOG_NOTIFY_MILLIS) {
            let result =
                systemd::daemon::notify(false, [(systemd::daemon::STATE_WATCHDOG, "1")].iter());
            if result.is_err() || !result.unwrap() {
                error!("Could not notify the systemd software watchdog");
            }

            watchdog_time = Instant::now();
        }

        // shall we quit the main loop?
        if QUIT.load(Ordering::SeqCst) {
            break 'MAIN_LOOP;
        }

        #[cfg(feature = "profiling")]
        coz::progress!();
    }

    Ok(())
}

/// Hot-unplug all failed or disconnected devices
fn remove_failed_devices() -> Result<bool> {
    let mut result = false;

    let mut keyboard_devices = crate::KEYBOARD_DEVICES.write();
    if let Some(index) = keyboard_devices
        .iter()
        .position(|device: &hwdevices::KeyboardDevice| device.read().has_failed().unwrap_or(true))
    {
        info!("Unplugging a failed keyboard device...");

        let mut devices_rx = crate::KEYBOARD_DEVICES_RX.write();
        assert!(devices_rx.len() > index);
        devices_rx.remove(index);

        assert!(keyboard_devices.len() > index);
        keyboard_devices.remove(index);

        result = true;

        debug!("Sending device hot remove notification...");

        let dbus_api_tx = crate::DBUS_API_TX.lock();
        let dbus_api_tx = dbus_api_tx.as_ref().unwrap();

        dbus_api_tx
            .send(DbusApiEvent::DeviceHotplug((0, 0), true))
            .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));
    }

    let mut mouse_devices = crate::MOUSE_DEVICES.write();
    if let Some(index) = mouse_devices
        .iter()
        .position(|device: &hwdevices::MouseDevice| device.read().has_failed().unwrap_or(true))
    {
        info!("Unplugging a failed mouse device...");

        let mut devices_rx = crate::MOUSE_DEVICES_RX.write();
        assert!(devices_rx.len() > index);
        devices_rx.remove(index);

        assert!(mouse_devices.len() > index);
        mouse_devices.remove(index);

        result = true;

        debug!("Sending device hot remove notification...");

        let dbus_api_tx = crate::DBUS_API_TX.lock();
        let dbus_api_tx = dbus_api_tx.as_ref().unwrap();

        dbus_api_tx
            .send(DbusApiEvent::DeviceHotplug((0, 0), true))
            .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));
    }

    let mut misc_devices = crate::MISC_DEVICES.write();
    if let Some(index) = misc_devices
        .iter()
        .position(|device: &hwdevices::MiscDevice| device.read().has_failed().unwrap_or(true))
    {
        info!("Unplugging a failed misc device...");

        let mut devices_rx = crate::MISC_DEVICES_RX.write();
        assert!(devices_rx.len() > index);
        devices_rx.remove(index);

        assert!(misc_devices.len() > index);
        misc_devices.remove(index);

        result = true;

        debug!("Sending device hot remove notification...");

        let dbus_api_tx = crate::DBUS_API_TX.lock();
        let dbus_api_tx = dbus_api_tx.as_ref().unwrap();

        dbus_api_tx
            .send(DbusApiEvent::DeviceHotplug((0, 0), true))
            .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));
    }

    Ok(result)
}

/// Watch profiles and script directory, as well as our
/// main configuration file for changes
pub fn register_filesystem_watcher(
    fsevents_tx: Sender<FileSystemEvent>,
    config_file: PathBuf,
) -> Result<()> {
    debug!("Registering filesystem watcher...");

    thread::Builder::new()
        .name("hotwatch".to_owned())
        .spawn(
            move || {
                #[cfg(feature = "profiling")]
                coz::thread_init();

                match Hotwatch::new_with_custom_delay(Duration::from_millis(1000)) {
                    Err(e) => error!("Could not initialize filesystem watcher: {}", e),

                    Ok(ref mut hotwatch) => {
                        hotwatch
                            .watch(config_file, move |_event: Event| {
                                info!("Configuration File changed on disk, please restart eruption for the changes to take effect!");

                                Flow::Continue
                            })
                            .unwrap_or_else(|e| error!("Could not register file watch: {}", e));

                        for profile_dir in profiles::get_profile_dirs() {
                            let fsevents_tx_c = fsevents_tx.clone();

                            hotwatch
                                .watch(&profile_dir, move |event: Event| {
                                    if let Event::Write(event) = event {
                                        if event.extension().unwrap_or_default().to_string_lossy() == "state" {
                                            info!("Existing profile state modified: {:?}", event);

                                            // crate::REQUEST_PROFILE_RELOAD.store(true, Ordering::SeqCst);
                                        } else if event.extension().unwrap_or_default().to_string_lossy() == "profile" {
                                            info!("Existing profile modified: {:?}", event);

                                            fsevents_tx_c.send(FileSystemEvent::ProfileChanged { action: EventAction::Modified, path: event }).unwrap();
                                        }
                                    } else if let Event::Create(event) = event {
                                        if event.extension().unwrap_or_default().to_string_lossy() == "state" {
                                            info!("New profile state created: {:?}", event);

                                            // crate::REQUEST_PROFILE_RELOAD.store(true, Ordering::SeqCst);
                                        } else if event.extension().unwrap_or_default().to_string_lossy() == "profile" {
                                            info!("New profile created: {:?}", event);

                                            fsevents_tx_c.send(FileSystemEvent::ProfileChanged { action: EventAction::Created, path: event }).unwrap();
                                        }
                                    } else if let Event::Rename(from, to) = event {
                                        if to.extension().unwrap_or_default().to_string_lossy() == "profile" {
                                            info!("Profile file renamed: {:?}", (&from, &to));

                                            fsevents_tx_c.send(FileSystemEvent::ProfileChanged { action: EventAction::Modified, path: to }).unwrap();
                                        }
                                    } else if let Event::Remove(event) = event {
                                        if event.extension().unwrap_or_default().to_string_lossy() == "state" {
                                            info!("Profile state deleted: {:?}", event);

                                            crate::REQUEST_PROFILE_RELOAD.store(true, Ordering::SeqCst);
                                        } else if event.extension().unwrap_or_default().to_string_lossy() == "profile" {
                                            info!("Profile deleted: {:?}", event);

                                            fsevents_tx_c.send(FileSystemEvent::ProfileChanged { action: EventAction::Deleted, path: event }).unwrap();
                                        }
                                    }

                                    Flow::Continue
                                })
                                .unwrap_or_else(|e| error!("Could not register directory watch for {}: {}", &profile_dir.display(), e));
                        }

                        for script_dir in util::get_script_dirs() {
                            let fsevents_tx_c = fsevents_tx.clone();

                            hotwatch
                                .watch(&script_dir, move |event: Event| {
                                    if let Event::Write(event) | Event::Create(event) |
                                           Event::Remove(event) | Event::Rename(_, event) = event {
                                        if event.extension().unwrap_or_default().to_string_lossy() == "lua" ||
                                           event.extension().unwrap_or_default().to_string_lossy() == "manifest" {
                                            info!("Script file, manifest or keymap changed: {:?}", event);

                                            fsevents_tx_c.send(FileSystemEvent::ScriptChanged).unwrap();
                                        }
                                    }

                                    Flow::Continue
                                })
                                .unwrap_or_else(|e| error!("Could not register directory watch for {}: {}", &script_dir.display(), e));
                        }

                        hotwatch.run();
                    }
                }
            })?;

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
            .spawn(move || {
                #[cfg(feature = "profiling")]
                coz::thread_init();

                loop {
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
                }
            })?;

        Ok(())
    }
}

/// open the control and LED devices of the keyboard
pub fn init_keyboard_device(keyboard_device: &KeyboardDevice) {
    info!("Opening keyboard device...");

    let hidapi = crate::HIDAPI.read();
    let hidapi = hidapi.as_ref().unwrap();

    keyboard_device.write().open(hidapi).unwrap_or_else(|e| {
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
pub fn init_mouse_device(mouse_device: &MouseDevice) {
    info!("Opening mouse device...");

    let hidapi = crate::HIDAPI.read();
    let hidapi = hidapi.as_ref().unwrap();

    mouse_device.write().open(hidapi).unwrap_or_else(|e| {
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

/// open the misc device
pub fn init_misc_device(misc_device: &MiscDevice) {
    info!("Opening misc device...");

    let hidapi = crate::HIDAPI.read();
    let hidapi = hidapi.as_ref().unwrap();

    misc_device.write().open(hidapi).unwrap_or_else(|e| {
        error!("Error opening the misc device: {}", e);
        error!(
            "This could be a permission problem, or maybe the device is locked by another process?"
        );
    });

    // send initialization handshake
    info!("Initializing misc device...");
    misc_device
        .write()
        .send_init_sequence()
        .unwrap_or_else(|e| error!("Could not initialize the device: {}", e));

    // set LEDs to a known good initial state
    info!("Configuring misc device LEDs...");
    misc_device
        .write()
        .set_led_init_pattern()
        .unwrap_or_else(|e| error!("Could not initialize LEDs: {}", e));

    info!(
        "Firmware revision: {}",
        misc_device.read().get_firmware_revision()
    );
}

pub async fn async_main() -> std::result::Result<(), eyre::Error> {
    #[cfg(feature = "profiling")]
    coz::thread_init();

    cfg_if::cfg_if! {
        if #[cfg(debug_assertions)] {
            color_eyre::config::HookBuilder::default()
            .panic_section("Please consider reporting a bug at https://github.com/X3n0m0rph59/eruption")
            .install()?;
        } else {
            color_eyre::config::HookBuilder::default()
            .panic_section("Please consider reporting a bug at https://github.com/X3n0m0rph59/eruption")
            .display_env_section(false)
            .install()?;
        }
    }

    if unsafe { libc::isatty(0) != 0 } {
        // print a license header, except if we are generating shell completions
        if !env::args().any(|a| a.eq_ignore_ascii_case("completions")) && env::args().count() < 2 {
            print_header();
        }

        // initialize logging on console
        logger::initialize_logging(&env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()))?;
    } else {
        // initialize logging to syslog
        let mut errors_present = false;

        let level_filter = match env::var("RUST_LOG")
            .unwrap_or_else(|_| "info".to_string())
            .to_lowercase()
            .as_str()
        {
            "off" => log::LevelFilter::Off,
            "error" => log::LevelFilter::Error,
            "warn" => log::LevelFilter::Warn,
            "info" => log::LevelFilter::Info,
            "debug" => log::LevelFilter::Debug,
            "trace" => log::LevelFilter::Trace,

            _ => {
                errors_present = true;
                log::LevelFilter::Info
            }
        };

        syslog::init(
            Facility::LOG_DAEMON,
            level_filter,
            Some(env!("CARGO_PKG_NAME")),
        )
        .map_err(|_e| MainError::SyslogLevelError {})?;

        if errors_present {
            log::error!("Could not parse syslog log-level");
        }
    }

    // start the thread deadlock detector
    #[cfg(debug_assertions)]
    thread_util::deadlock_detector()
        .unwrap_or_else(|e| error!("Could not spawn deadlock detector thread: {}", e));

    let matches = parse_commandline();

    info!(
        "Starting Eruption - Realtime RGB LED Driver for Linux: Version {} ({}) ({} build)",
        env!("CARGO_PKG_VERSION"),
        env!("ERUPTION_GIT_PKG_VERSION"),
        if cfg!(debug_assertions) {
            "debug"
        } else {
            "release"
        }
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

    // create a directory in /run in case it does not already exist
    let _result = fs::create_dir(constants::RUN_ERUPTION_DIR);

    // write out our current PID
    let _result = util::write_pid_file();

    // process configuration file
    let config_file = matches
        .value_of("config")
        .unwrap_or(constants::DEFAULT_CONFIG_FILE);

    let config = Config::builder()
        .add_source(config::File::new(config_file, config::FileFormat::Toml))
        .build()
        .unwrap_or_else(|e| {
            log::error!("Could not parse configuration file: {}", e);
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

    // driver maturity level
    let driver_maturity_level = config
        .get::<MaturityLevel>("global.driver_maturity_level")
        .unwrap_or(MaturityLevel::Stable);

    *DRIVER_MATURITY_LEVEL.lock() = driver_maturity_level;

    match *DRIVER_MATURITY_LEVEL.lock() {
        MaturityLevel::Stable => {
            info!("Using only drivers that are marked as stable (default)")
        }
        MaturityLevel::Testing => {
            info!("Using drivers that are marked as testing, this may expose some bugs!")
        }
        MaturityLevel::Experimental => {
            warn!("** EXPERIMENTAL DRIVERS are ENABLED, this may expose serious bugs! **")
        }
    }

    // load and initialize global runtime state
    info!("Loading saved state...");
    state::init_global_runtime_state()
        .unwrap_or_else(|e| warn!("Could not parse state file: {}", e));

    // restore saved color-schemes
    state::load_color_schemes()
        .unwrap_or_else(|e| warn!("Could not restore previously saved color-schemes: {}", e));

    // enable the mouse
    let enable_mouse = config.get::<bool>("global.enable_mouse").unwrap_or(true);

    // create the one and only hidapi instance
    match hidapi::HidApi::new() {
        Ok(hidapi) => {
            {
                *crate::HIDAPI.write() = Some(hidapi);
            }

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

            if let Ok(devices) = hwdevices::probe_devices() {
                // initialize keyboard devices
                for (index, device) in devices.0.iter().enumerate() {
                    init_keyboard_device(device);

                    let usb_vid = device.read().get_usb_vid();
                    let usb_pid = device.read().get_usb_pid();

                    // spawn a thread to handle keyboard input
                    info!("Spawning keyboard input thread...");

                    let (kbd_tx, kbd_rx) = unbounded();
                    threads::spawn_keyboard_input_thread(
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

                    crate::KEYBOARD_DEVICES_RX.write().push(kbd_rx);
                    crate::KEYBOARD_DEVICES.write().push(device.clone());
                }

                // initialize mouse devices
                for (index, device) in devices.1.iter().enumerate() {
                    // enable mouse input
                    if enable_mouse {
                        init_mouse_device(device);

                        let usb_vid = device.read().get_usb_vid();
                        let usb_pid = device.read().get_usb_pid();

                        let (mouse_tx, mouse_rx) = unbounded();
                        // let (mouse_secondary_tx, _mouse_secondary_rx) = unbounded();

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
                        /* if EXPERIMENTAL_FEATURES.load(Ordering::SeqCst)
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
                        } */

                        crate::MOUSE_DEVICES_RX.write().push(mouse_rx);
                        crate::MOUSE_DEVICES.write().push(device.clone());
                    } else {
                        info!("Found mouse device, but mouse support is DISABLED by configuration");
                    }
                }

                // initialize misc devices
                for (index, device) in devices.2.iter().enumerate() {
                    init_misc_device(device);

                    if device.read().has_input_device() {
                        let usb_vid = device.read().get_usb_vid();
                        let usb_pid = device.read().get_usb_pid();

                        // spawn a thread to handle keyboard input
                        info!("Spawning misc device input thread...");

                        let (misc_tx, misc_rx) = unbounded();
                        threads::spawn_misc_input_thread(
                            misc_tx.clone(),
                            device.clone(),
                            index,
                            usb_vid,
                            usb_pid,
                        )
                        .unwrap_or_else(|e| {
                            error!("Could not spawn a thread: {}", e);
                            panic!()
                        });

                        crate::MISC_DEVICES_RX.write().push(misc_rx);
                    } else {
                        // insert an unused rx
                        let (_misc_tx, misc_rx) = unbounded();
                        crate::MISC_DEVICES_RX.write().push(misc_rx);
                    }

                    crate::MISC_DEVICES.write().push(device.clone());
                }

                info!("Device enumeration completed");

                if crate::KEYBOARD_DEVICES.read().is_empty()
                    && crate::MOUSE_DEVICES.read().is_empty()
                    && crate::MISC_DEVICES.read().is_empty()
                {
                    warn!("No supported devices found!");
                }

                info!("Performing late initializations...");

                // load and initialize global runtime state (late init)
                info!("Loading saved device state...");
                state::init_global_runtime_state_late()
                    .unwrap_or_else(|e| warn!("Could not parse state file: {}", e));

                // initialize the Linux uleds interface
                info!("Initializing Linux Userspace LEDs interface...");
                plugins::UledsPlugin::spawn_uleds_thread().unwrap_or_else(|e| {
                    warn!("Could not spawn a thread: {}", e);
                    panic!()
                });

                // initialize the D-Bus API
                info!("Initializing D-Bus API...");
                let (dbus_tx, dbus_rx) = unbounded();
                let dbus_api_tx = threads::spawn_dbus_api_thread(dbus_tx).unwrap_or_else(|e| {
                    error!("Could not spawn a thread: {}", e);
                    panic!()
                });

                *DBUS_API_TX.lock() = Some(dbus_api_tx.clone());

                let (fsevents_tx, fsevents_rx) = unbounded();
                register_filesystem_watcher(fsevents_tx, PathBuf::from(&config_file))
                    .unwrap_or_else(|e| error!("Could not register file changes watcher: {}", e));

                // initialize the device I/O thread
                info!("Initializing device I/O thread...");
                let (dev_io_tx, dev_io_rx) = unbounded();
                threads::spawn_device_io_thread(dev_io_rx).unwrap_or_else(|e| {
                    error!("Could not spawn the render thread: {}", e);
                    panic!()
                });

                *DEV_IO_TX.lock() = Some(dev_io_tx.clone());

                info!("Late initializations completed");

                info!("Startup completed");

                'OUTER_LOOP: loop {
                    info!("Entering the main loop now...");

                    let mut errors_present = false;

                    // enter the main loop
                    run_main_loop(&dbus_api_tx, &ctrl_c_rx, &dbus_rx, &fsevents_rx).unwrap_or_else(
                        |e| {
                            warn!("Left the main loop due to an irrecoverable error: {}", e);
                            errors_present = true;
                        },
                    );

                    if !errors_present {
                        info!("Main loop terminated gracefully");
                    }

                    if crate::QUIT.load(Ordering::SeqCst) {
                        break 'OUTER_LOOP;
                    }

                    // wait a few miliseconds to give devices time to settle
                    thread::sleep(Duration::from_millis(50));

                    // remove disconnected or failed devices
                    remove_failed_devices()?;
                }

                events::notify_observers(events::Event::DaemonShutdown)?;

                // we left the main loop, so send a final message to the running Lua VMs
                info!("Shutting down all Lua VMs now...");

                *UPCALL_COMPLETED_ON_QUIT.0.lock() = LUA_TXS.read().len();

                for lua_tx in LUA_TXS.read().iter() {
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
                        warn!("Timed out while waiting for a Lua VM to shut down");
                        break;
                    }

                    if *pending == 0 {
                        break;
                    }
                }

                // store plugin state to disk
                plugins::PersistencePlugin::store_persistent_data()
                    .unwrap_or_else(|e| error!("Could not write persisted state: {}", e));

                // save state
                info!("Saving global runtime state...");
                state::save_runtime_state()
                    .unwrap_or_else(|e| error!("Could not save runtime state: {}", e));

                // save color-schemes
                state::save_color_schemes()
                    .unwrap_or_else(|e| error!("Could not save color-schemes: {}", e));

                // close all managed devices
                info!("Closing all devices now...");

                thread::sleep(Duration::from_millis(
                    constants::SHUTDOWN_TIMEOUT_MILLIS as u64,
                ));

                // set LEDs of all keyboards to a known final state, then close all associated devices
                let shutdown_keyboards = async {
                    for device in crate::KEYBOARD_DEVICES.read().iter() {
                        device.write().set_led_off_pattern().unwrap_or_else(|e| {
                            error!("Could not finalize LEDs configuration: {}", e)
                        });

                        device.write().close_all().unwrap_or_else(|e| {
                            warn!("Could not close the device: {}", e);
                        });
                    }
                };

                // set LEDs of all mice to a known final state, then close all associated devices
                let shutdown_mice = async {
                    for device in crate::MOUSE_DEVICES.read().iter() {
                        device.write().set_led_off_pattern().unwrap_or_else(|e| {
                            error!("Could not finalize LEDs configuration: {}", e)
                        });

                        device.write().close_all().unwrap_or_else(|e| {
                            warn!("Could not close the device: {}", e);
                        });
                    }
                };

                // set LEDs of all misc devices to a known final state, then close all associated devices
                let shutdown_misc = async {
                    for device in crate::MISC_DEVICES.read().iter() {
                        device.write().set_led_off_pattern().unwrap_or_else(|e| {
                            error!("Could not finalize LEDs configuration: {}", e)
                        });

                        device.write().close_all().unwrap_or_else(|e| {
                            warn!("Could not close the device: {}", e);
                        });
                    }
                };

                join!(shutdown_keyboards, shutdown_mice, shutdown_misc);
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

    if util::file_exists("/run/lock/eruption-hotplug-helper.lock") {
        debug!("Removing stale eruption-hotplug-helper.lock file...");

        fs::remove_file("/run/lock/eruption-hotplug-helper.lock")
            .unwrap_or_else(|e| warn!("Could not remove lock file: {}", e));
    }

    info!("Exiting now");

    Ok(())
}

/// Main program entrypoint
pub fn main() -> std::result::Result<(), eyre::Error> {
    #[cfg(feature = "profiling")]
    coz::thread_init();

    let language_loader: FluentLanguageLoader = fluent_language_loader!();

    let requested_languages = DesktopLanguageRequester::requested_languages();
    i18n_embed::select(&language_loader, &Localizations, &requested_languages)?;

    STATIC_LOADER.lock().replace(language_loader);

    let runtime = tokio::runtime::Builder::new_current_thread()
        // .enable_all()
        .build()?;

    runtime.block_on(async move { async_main().await })
}
