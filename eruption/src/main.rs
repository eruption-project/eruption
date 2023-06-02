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

use clap::{Arg, Command};
use color_eyre::owo_colors::OwoColorize;
use config::Config;
use flume::{select::SelectError, unbounded, Receiver, Selector, Sender};
use hotwatch::{
    blocking::{Flow, Hotwatch},
    Event,
};
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};
use lazy_static::lazy_static;
use parking_lot::{Condvar, Mutex, RwLock};
use rayon::prelude::{IndexedParallelIterator, IntoParallelRefIterator, ParallelIterator};
use rust_embed::RustEmbed;
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::u64;
use std::{collections::HashMap, env};
use std::{collections::HashSet, thread};
use std::{
    fs,
    path::{Path, PathBuf},
};
use std::{
    process,
    sync::atomic::{AtomicBool, AtomicIsize, AtomicUsize, Ordering},
};
use tracing::*;
use util::ratelimited;

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

#[cfg(not(target_os = "windows"))]
use crate::{plugins::macros, plugins::uleds};

#[cfg(target_os = "windows")]
use windows_named_pipe::PipeStream;

use crate::{
    color_scheme::ColorScheme,
    hwdevices::{DeviceStatus, MaturityLevel, RGBA},
    plugins::sdk_support,
    profiles::Profile,
    scripting::script,
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

    pub static ref VERSION: String = {
        format!("version {version} ({build_type} build) [{branch}:{commit} {dirty}]",
            version = env!("CARGO_PKG_VERSION"),
            branch = env!("GIT_BRANCH"),
            commit = env!("GIT_COMMIT"),
            dirty = if env!("GIT_DIRTY") == "true" {
                "dirty"
            } else {
                "clean"
            },
            // timestamp = env!("SOURCE_TIMESTAMP"),
            build_type = if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            })
        };
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

#[cfg(not(target_os = "windows"))]
lazy_static! {
    #[cfg(not(target_os = "windows"))]
    pub static ref KEYBOARD_DEVICES_RX: Arc<RwLock<Vec<Receiver<Option<evdev_rs::InputEvent>>>>> = Arc::new(RwLock::new(Vec::new()));

    #[cfg(not(target_os = "windows"))]
    pub static ref MOUSE_DEVICES_RX: Arc<RwLock<Vec<Receiver<Option<evdev_rs::InputEvent>>>>> = Arc::new(RwLock::new(Vec::new()));

    #[cfg(not(target_os = "windows"))]
    pub static ref MISC_DEVICES_RX: Arc<RwLock<Vec<Receiver<Option<evdev_rs::InputEvent>>>>> = Arc::new(RwLock::new(Vec::new()));

}

lazy_static! {
    /// Managed keyboard devices
    pub static ref KEYBOARD_DEVICES: Arc<RwLock<Vec<hwdevices::KeyboardDevice>>> = Arc::new(RwLock::new(Vec::new()));

    /// Managed mouse devices
    pub static ref MOUSE_DEVICES: Arc<RwLock<Vec<hwdevices::MouseDevice>>> = Arc::new(RwLock::new(Vec::new()));

    /// Managed miscellaneous devices
    pub static ref MISC_DEVICES: Arc<RwLock<Vec<hwdevices::MiscDevice>>> = Arc::new(RwLock::new(Vec::new()));

    /// Hidapi object
    pub static ref HIDAPI: Arc<RwLock<Option<hidapi::HidApi>>> = Arc::new(RwLock::new(None));

    /// Holds device status information, like e.g: current signal strength or battery levels
    pub static ref DEVICE_STATUS: Arc<RwLock<HashMap<u64, DeviceStatus>>> =
        Arc::new(RwLock::new(HashMap::new()));

    /// The currently active slot (1-4)
    pub static ref ACTIVE_SLOT: AtomicUsize = AtomicUsize::new(0);

    /// The custom names of each slot
    pub static ref SLOT_NAMES: Arc<RwLock<Vec<String>>> = Arc::new(RwLock::new(Vec::new()));

    /// The slot to profile associations
    pub static ref SLOT_PROFILES: Arc<RwLock<Option<Vec<PathBuf>>>> = Arc::new(RwLock::new(None));

    /// The currently active profile
    pub static ref ACTIVE_PROFILE: Arc<RwLock<Option<Profile>>> = Arc::new(RwLock::new(None));

    /// Contains the file name part of the active profile;
    /// may be used to switch profiles at runtime
    pub static ref ACTIVE_PROFILE_NAME: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));

    /// The profile that was active before we entered AFK mode
    pub static ref ACTIVE_PROFILE_NAME_BEFORE_AFK: Arc<RwLock<Option<String>>> = Arc::new(RwLock::new(None));

    /// Named color schemes, for use in e.g. gradients
    pub static ref NAMED_COLOR_SCHEMES: Arc<RwLock<HashMap<String, ColorScheme>>> =
        Arc::new(RwLock::new(HashMap::new()));

    /// Global configuration
    pub static ref CONFIG: Arc<RwLock<Option<config::Config>>> = Arc::new(RwLock::new(None));

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
    pub static ref DRIVER_MATURITY_LEVEL: Arc<RwLock<MaturityLevel>> = Arc::new(RwLock::new(MaturityLevel::Stable));

    /// Global "enable Linux Userspace LEDs support" flag
    pub static ref ULEDS_SUPPORT_ACTIVE: AtomicBool = AtomicBool::new(false);

    // Other state

    /// Global brightness modifier
    pub static ref BRIGHTNESS: AtomicIsize = AtomicIsize::new(100);

    /// Fade in on profile switch
    pub static ref FADER: AtomicIsize = AtomicIsize::new(0);

    /// Global modifier to compare fading into a profile
    pub static ref FADER_BASE: AtomicIsize = AtomicIsize::new(0);

    /// Canvas post-processing parameters
    pub static ref CANVAS_HSL: Arc<RwLock<(f64, f64, f64)>> = Arc::new(RwLock::new((0.0, 0.0, 0.0)));

    /// AFK timer
    pub static ref LAST_INPUT_TIME: Arc<RwLock<Instant>> = Arc::new(RwLock::new(Instant::now()));

    /// Channel to the D-Bus interface
    pub static ref DBUS_API_TX: Arc<RwLock<Option<Sender<DbusApiEvent>>>> = Arc::new(RwLock::new(None));

    /// Channel to the device I/O thread
    pub static ref DEV_IO_TX: Arc<RwLock<Option<Sender<DeviceAction >>>> = Arc::new(RwLock::new(None));

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
        let config = &*crate::CONFIG.read();
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

    // This is the "switch profile fence" condition variable
    pub static ref PROFILE_SWITCHING_COMPLETED_CONDITION: Arc<(Mutex<bool>, Condvar)> =
    Arc::new((Mutex::new(true), Condvar::new()));

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

    #[error("A device failed")]
    DeviceFailed {},

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
        r#"Eruption is free software: you can redistribute it and/or modify
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
"#
    );
}

/// Process commandline options
fn parse_commandline() -> clap::ArgMatches {
    Command::new("Eruption")
        .version(VERSION.as_str())
        .author("X3n0m0rph59 <x3n0m0rph59@gmail.com>")
        .about("Realtime RGB LED Driver for Linux")
        .subcommand(
            Command::new("daemon")
                .name("daemon")
                .about("Run in background"),
        )
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Sets the configuration file to use"),
        )
        // .arg(
        //     Arg::new("completions")
        //         .long("completions")
        //         .value_name("SHELL")
        //         .about("Generate shell completions"),
        // )
        .get_matches()
}

pub fn switch_profile_please(profile_file: Option<&Path>) -> Result<SwitchProfileResult> {
    let dbus_api_tx = crate::DBUS_API_TX.read();
    let dbus_api_tx = dbus_api_tx.as_ref().unwrap();

    switch_profile(profile_file, dbus_api_tx, true)
}

#[derive(PartialEq, Eq)]
pub enum SwitchProfileResult {
    Switched,
    InvalidProfile,
    FallbackToFailsafe,
}

/// Switches the currently active profile to the profile file `profile_file`
/// Returns Ok(Switched) if the new profile has been activated, Ok(InvalidProfile)
/// if the old profile was kept, or else Ok(FallbackToFailsafe) when we entered
/// failsafe mode. If an error occurred during switching to failsafe mode, we
/// return an Err() to signal a fatal error
pub fn switch_profile(
    profile_file: Option<&Path>,
    dbus_api_tx: &Sender<DbusApiEvent>,
    notify: bool,
) -> Result<SwitchProfileResult> {
    fn switch_to_failsafe_profile(dbus_api_tx: &Sender<DbusApiEvent>, notify: bool) -> Result<()> {
        let mut errors_present = false;

        let profile = Profile::new_fail_safe();

        // spawn a new set of Lua VMs, with scripts from the failsafe profile
        for (thread_idx, manifest) in profile.manifests.values().enumerate() {
            let (lua_tx, lua_rx) = unbounded();
            let parameters = &manifest.get_merged_parameters(&profile);
            threads::spawn_lua_thread(thread_idx, lua_rx, &manifest.script_file, parameters)
                .unwrap_or_else(|e| {
                    errors_present = true;

                    error!("Could not spawn a thread: {}", e);
                });

            let mut tx = LuaTx::new(manifest.script_file.to_owned(), lua_tx);

            if errors_present {
                tx.is_failed = true
            }

            LUA_TXS.write().push(tx);
        }

        // finally assign the globally active profile
        *ACTIVE_PROFILE.write() = Some(profile);

        if notify {
            dbus_api_tx
                .send(DbusApiEvent::ActiveProfileChanged)
                .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));
        }

        // let active_slot = ACTIVE_SLOT.load(Ordering::SeqCst);

        // let mut slot_profiles = SLOT_PROFILES.write();
        // slot_profiles.as_mut().unwrap()[active_slot] = "failsafe.profile".into();

        if errors_present {
            error!("Fatal error: An error occurred while loading the failsafe profile");
            Err(MainError::SwitchProfileError {}.into())
        } else {
            Ok(())
        }
    }

    let mut switch_completed = crate::PROFILE_SWITCHING_COMPLETED_CONDITION.0.lock();
    if !*switch_completed {
        // wait for previous profile switching requests to complete...

        let result = PROFILE_SWITCHING_COMPLETED_CONDITION.1.wait_for(
            &mut switch_completed,
            Duration::from_millis(constants::LONG_TIMEOUT_MILLIS),
        );

        if result.timed_out() {
            ratelimited::error!("Invalid state in profile switching code");
            crate::REENTER_MAIN_LOOP.store(true, Ordering::SeqCst);

            return Err(MainError::SwitchProfileError {}.into());
        }
    }

    // ok, no previous operations pending, begin switching the profile now...

    *switch_completed = false;

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

        *switch_completed = true;
        crate::PROFILE_SWITCHING_COMPLETED_CONDITION.1.notify_all();

        Ok(SwitchProfileResult::FallbackToFailsafe)
    } else {
        // we require profile_file to be set in this branch
        let profile_file = if let Some(profile_file) = profile_file {
            profile_file
        } else {
            *switch_completed = true;
            crate::PROFILE_SWITCHING_COMPLETED_CONDITION.1.notify_all();

            error!("Undefined profile");
            return Err(MainError::SwitchProfileError {}.into());
        };

        info!("Switching to profile: {}", &profile_file.display());

        let profile = profiles::Profile::load_fully(profile_file);

        match profile {
            Ok(profile) => {
                let mut errors_present = false;

                // take a snapshot of the last rendered LED map
                *script::SAVED_LED_MAP.write() = script::LAST_RENDERED_LED_MAP.read().clone();

                // request termination of all Lua VMs
                for lua_tx in LUA_TXS.read().iter() {
                    if !lua_tx.is_failed {
                        lua_tx.send(script::Message::Unload).unwrap_or_else(|e| {
                            error!("Could not send an event to a Lua VM: {}", e)
                        });
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
                for (thread_idx, manifest) in profile.manifests.values().enumerate() {
                    let (lua_tx, lua_rx) = unbounded();
                    if let Err(e) = threads::spawn_lua_thread(
                        thread_idx,
                        lua_rx,
                        &manifest.script_file,
                        &manifest.get_merged_parameters(&profile),
                    ) {
                        errors_present = true;

                        error!("Could not spawn a thread: {}", e);
                    }

                    let mut tx = LuaTx::new(manifest.script_file.to_owned(), lua_tx);

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

                    *switch_completed = true;
                    crate::PROFILE_SWITCHING_COMPLETED_CONDITION.1.notify_all();

                    Ok(SwitchProfileResult::FallbackToFailsafe)
                } else {
                    // everything is fine, finally assign the globally active profile
                    debug!("Switch successful");

                    // now clear all led maps, so that we don't get leftover artifacts in
                    // case the new scripts don't paint the whole canvas
                    script::LED_MAP.write().fill(RGBA {
                        r: 0x00,
                        g: 0x00,
                        b: 0x00,
                        a: 0x00,
                    });

                    script::LAST_RENDERED_LED_MAP.write().fill(RGBA {
                        r: 0x00,
                        g: 0x00,
                        b: 0x00,
                        a: 0x00,
                    });

                    sdk_support::LED_MAP.write().fill(RGBA {
                        r: 0x00,
                        g: 0x00,
                        b: 0x00,
                        a: 0x00,
                    });

                    let fade_millis = crate::CONFIG
                        .read()
                        .as_ref()
                        .unwrap()
                        .get_int("global.profile_fade_milliseconds")
                        .unwrap_or(constants::FADE_MILLIS as i64);
                    let fade_frames = (fade_millis * constants::TARGET_FPS as i64 / 1000) as isize;
                    crate::FADER.store(fade_frames, Ordering::SeqCst);
                    crate::FADER_BASE.store(fade_frames, Ordering::SeqCst);

                    *ACTIVE_PROFILE.write() = Some(profile);

                    if notify {
                        dbus_api_tx
                            .send(DbusApiEvent::ActiveProfileChanged)
                            .unwrap_or_else(|e| {
                                error!("Could not send a pending dbus API event: {}", e)
                            });
                    }

                    let active_slot = ACTIVE_SLOT.load(Ordering::SeqCst);
                    let mut slot_profiles = SLOT_PROFILES.write();
                    slot_profiles.as_mut().unwrap()[active_slot] = profile_file.into();

                    *switch_completed = true;
                    crate::PROFILE_SWITCHING_COMPLETED_CONDITION.1.notify_all();

                    Ok(SwitchProfileResult::Switched)
                }
            }
            Err(e) => {
                // the profile file to switch to is corrupted, so we need to refuse to switch profiles
                // and simply keep the current one, or load a failsafe profile if we do not have a
                // currently active profile, like e.g. during startup of the daemon
                if crate::ACTIVE_PROFILE.read().is_none() {
                    error!(
                        "An error occurred during switching of profiles, loading failsafe profile now. {}",
                        e
                    );
                    switch_to_failsafe_profile(dbus_api_tx, notify)?;

                    *switch_completed = true;
                    crate::PROFILE_SWITCHING_COMPLETED_CONDITION.1.notify_all();

                    Ok(SwitchProfileResult::FallbackToFailsafe)
                } else {
                    error!(
                        "Invalid profile: {}, refusing to switch profiles. {}",
                        profile_file.display(),
                        e
                    );

                    *switch_completed = true;
                    crate::PROFILE_SWITCHING_COMPLETED_CONDITION.1.notify_all();

                    Ok(SwitchProfileResult::InvalidProfile)
                }
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

    let afk_timeout_secs = crate::CONFIG
        .read()
        .as_ref()
        .unwrap()
        .get_int("global.afk_timeout_secs")
        .unwrap_or(constants::AFK_TIMEOUT_SECS as i64) as u64;

    // main loop iterations, monotonic counter
    let mut ticks = 0;
    let mut start_time;
    let mut delay_time_hid_poll = Instant::now();
    let mut delay_time_tick = Instant::now();
    let mut delay_time_render = Instant::now();
    let mut last_status_poll = Instant::now();

    // used to detect changes of the active slot
    let mut saved_slot = 0;

    let mut saved_brightness = BRIGHTNESS.load(Ordering::SeqCst);

    let mut saved_hue = CANVAS_HSL.read().0;
    let mut saved_saturation = CANVAS_HSL.read().1;
    let mut saved_lightness = CANVAS_HSL.read().2;

    // used to detect changes to the AFK state
    let mut saved_afk_mode = false;

    // fade-in from an all black canvas to the saved profile on startup
    let fade_frames = (constants::STARTUP_FADE_IN_MILLIS * constants::TARGET_FPS / 1000) as isize;
    crate::FADER.store(fade_frames, Ordering::SeqCst);
    crate::FADER_BASE.store(fade_frames, Ordering::SeqCst);

    'MAIN_LOOP: loop {
        #[cfg(feature = "profiling")]
        coz::scope!("main loop");

        let mut device_has_failed = false;

        #[cfg(not(target_os = "windows"))]
        let kbd_rxs = crate::KEYBOARD_DEVICES_RX.write();
        #[cfg(not(target_os = "windows"))]
        let mouse_rxs = crate::MOUSE_DEVICES_RX.write();

        #[cfg(not(target_os = "windows"))]
        let kbd_rxs_clone = kbd_rxs.clone();
        #[cfg(not(target_os = "windows"))]
        let mouse_rxs_clone = mouse_rxs.clone();

        let mut sel = Selector::new()
            .recv(ctrl_c_rx, |_event| {
                QUIT.store(true, Ordering::SeqCst);
            })
            .recv(fsevents_rx, |event| {
                if let Ok(event) = event {
                    events::process_filesystem_event(&event, dbus_api_tx).unwrap_or_else(|e| {
                        ratelimited::error!("Could not process a filesystem event: {}", e)
                    })
                } else {
                    ratelimited::error!(
                        "Could not process a filesystem event: {}",
                        event.as_ref().unwrap_err()
                    );
                }
            })
            .recv(dbus_rx, |event| {
                if let Ok(event) = event {
                    events::process_dbus_event(&event, dbus_api_tx).unwrap_or_else(|e| {
                        ratelimited::error!("Could not process a D-Bus event: {}", e)
                    });

                    // FAILED_TXS.write().clear();
                } else {
                    ratelimited::error!(
                        "Fatal: Could not process a D-Bus event: {}",
                        event.as_ref().unwrap_err()
                    );
                }
            });

        #[cfg(not(target_os = "windows"))]
        let failed_kbd_rxs = Arc::new(RwLock::new(HashSet::new()));

        #[cfg(not(target_os = "windows"))]
        for (index, rx) in kbd_rxs_clone.iter().enumerate() {
            let failed_kbd_rxs = failed_kbd_rxs.clone();

            let mapper = move |event| {
                if let Ok(Some(event)) = event {
                    // TODO: support multiple keyboards
                    events::process_keyboard_event(&event, &crate::KEYBOARD_DEVICES.read()[0])
                        .unwrap_or_else(|e| {
                            device_has_failed = true;

                            // let make = hwdevices::get_device_make(
                            //     device.read().get_usb_vid(),
                            //     device.read().get_usb_pid(),
                            // )
                            // .unwrap_or_else(|| "<unknown>");
                            // let model = hwdevices::get_device_model(
                            //     device.read().get_usb_vid(),
                            //     device.read().get_usb_pid(),
                            // )
                            // .unwrap_or_else(|| "<unknown>");

                            ratelimited::error!(
                                "Could not process a keyboard event: {}. Trying to close the device now...",
                                e
                            );

                            (*crate::KEYBOARD_DEVICES.read()[0])
                                .write()
                                .as_device_mut()
                                .close_all()
                                .map_err(|e| ratelimited::error!("An error occurred while closing the device: {e}"))
                                .ok();
                        });
                } else {
                    device_has_failed = true;

                    // let make = hwdevices::get_device_make(
                    //     device.read().get_usb_vid(),
                    //     device.read().get_usb_pid(),
                    // )
                    // .unwrap_or_else(|| "<unknown>");
                    // let model = hwdevices::get_device_model(
                    //     device.read().get_usb_vid(),
                    //     device.read().get_usb_pid(),
                    // )
                    // .unwrap_or_else(|| "<unknown>");

                    ratelimited::error!(
                        "Could not process a keyboard event from: {}",
                        event.as_ref().unwrap_err()
                    );

                    (*crate::KEYBOARD_DEVICES.read()[0])
                        .write()
                        .as_device_mut()
                        .close_all()
                        .map_err(|e| {
                            ratelimited::error!("An error occurred while closing the device: {e}")
                        })
                        .ok();
                }

                if device_has_failed {
                    failed_kbd_rxs.write().insert(index);
                }
            };

            sel = sel.recv(rx, mapper);
        }

        #[cfg(not(target_os = "windows"))]
        let failed_mouse_rxs = Arc::new(RwLock::new(HashSet::new()));

        #[cfg(not(target_os = "windows"))]
        for (index, rx) in mouse_rxs_clone.iter().enumerate() {
            let failed_mouse_rxs = failed_mouse_rxs.clone();

            let mapper = move |event| {
                if let Ok(Some(event)) = event {
                    events::process_mouse_event(&event, &crate::MOUSE_DEVICES.read()[0])
                        .unwrap_or_else(|e| {
                            device_has_failed = true;

                            // let make = hwdevices::get_device_make(
                            //     device.read().get_usb_vid(),
                            //     device.read().get_usb_pid(),
                            // )
                            // .unwrap_or_else(|| "<unknown>");
                            // let model = hwdevices::get_device_model(
                            //     device.read().get_usb_vid(),
                            //     device.read().get_usb_pid(),
                            // )
                            // .unwrap_or_else(|| "<unknown>");

                            ratelimited::error!("Could not process a mouse event from: {}. Trying to close the device now...", e);

                            (*crate::MOUSE_DEVICES.read()[0])
                                .write()
                                .as_device_mut()
                                .close_all()
                                .map_err(|e| ratelimited::error!("An error occurred while closing the device: {e}"))
                                .ok();
                        });
                } else {
                    device_has_failed = true;

                    // let make = hwdevices::get_device_make(
                    //     device.read().get_usb_vid(),
                    //     device.read().get_usb_pid(),
                    // )
                    // .unwrap_or_else(|| "<unknown>");
                    // let model = hwdevices::get_device_model(
                    //     device.read().get_usb_vid(),
                    //     device.read().get_usb_pid(),
                    // )
                    // .unwrap_or_else(|| "<unknown>");

                    ratelimited::error!(
                        "Could not process a mouse event from: {}",
                        event.as_ref().unwrap_err()
                    );

                    (*crate::MOUSE_DEVICES.read()[0])
                        .write()
                        .as_device_mut()
                        .close_all()
                        .map_err(|e| {
                            ratelimited::error!("An error occurred while closing the device: {e}")
                        })
                        .ok();
                }

                if device_has_failed {
                    failed_mouse_rxs.write().insert(index);
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
                *ACTIVE_PROFILE_NAME.write() = None;
                saved_slot = ACTIVE_SLOT.load(Ordering::SeqCst);

                if let Err(e) = switch_profile(None, dbus_api_tx, true) {
                    error!("Could not switch profiles: {}", e);
                } else {
                    // reset the audio backend, it will be enabled again if needed
                    #[cfg(not(target_os = "windows"))]
                    plugins::audio::reset_audio_backend();

                    FAILED_TXS.write().clear();
                }
            }
        }

        {
            // slot changed?
            let active_slot = ACTIVE_SLOT.load(Ordering::SeqCst);
            if active_slot != saved_slot || ACTIVE_PROFILE.read().is_none() {
                let profile_path = {
                    let slot_profiles = SLOT_PROFILES.read();
                    slot_profiles.as_ref().unwrap()[active_slot].clone()
                };

                if let Err(e) = switch_profile(Some(&profile_path), dbus_api_tx, false) {
                    error!("Could not switch profiles: {}", e);
                } else {
                    // reset the audio backend, it will be enabled again if needed
                    #[cfg(not(target_os = "windows"))]
                    plugins::audio::reset_audio_backend();

                    dbus_api_tx
                        .send(DbusApiEvent::ActiveSlotChanged)
                        .unwrap_or_else(|e| {
                            error!("Could not send a pending dbus API event: {}", e)
                        });

                    dbus_api_tx
                        .send(DbusApiEvent::ActiveProfileChanged)
                        .unwrap_or_else(|e| {
                            error!("Could not send a pending dbus API event: {}", e)
                        });

                    saved_slot = active_slot;
                    FAILED_TXS.write().clear();
                }
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

        // post-processing parameters changed?
        let current_hsl = *CANVAS_HSL.read();

        if current_hsl.0 != saved_hue {
            dbus_api_tx
                .send(DbusApiEvent::HueChanged)
                .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));

            saved_hue = current_hsl.0;
        }

        if current_hsl.1 != saved_saturation {
            dbus_api_tx
                .send(DbusApiEvent::SaturationChanged)
                .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));

            saved_saturation = current_hsl.1;
        }

        if current_hsl.2 != saved_lightness {
            dbus_api_tx
                .send(DbusApiEvent::LightnessChanged)
                .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));

            saved_lightness = current_hsl.2;
        }

        // user is AFK?
        let afk_mode = AFK.load(Ordering::SeqCst);
        if afk_mode != saved_afk_mode {
            if afk_mode {
                info!("Entering AFK mode now...");

                let afk_profile = crate::CONFIG
                    .read()
                    .as_ref()
                    .unwrap()
                    .get::<String>("global.afk_profile")
                    .unwrap_or_else(|_| constants::DEFAULT_AFK_PROFILE.to_owned());

                let active_profile = &*ACTIVE_PROFILE.read();
                let before_afk = &active_profile.as_ref().unwrap().profile_file;

                *ACTIVE_PROFILE_NAME_BEFORE_AFK.write() =
                    Some(before_afk.to_string_lossy().to_string());

                ACTIVE_PROFILE_NAME.write().replace(afk_profile);
            } else {
                info!("Leaving AFK mode now...");

                ACTIVE_PROFILE_NAME.write().replace(
                    ACTIVE_PROFILE_NAME_BEFORE_AFK
                        .read()
                        .as_ref()
                        .unwrap()
                        .clone(),
                );
            }

            saved_afk_mode = afk_mode;
        }

        {
            // active profile name changed?
            if let Some(active_profile) = &*ACTIVE_PROFILE_NAME.read() {
                let profile_path = Path::new(active_profile);

                if let Err(e) = switch_profile(Some(profile_path), dbus_api_tx, true) {
                    error!("Could not switch profiles: {}", e);
                } else {
                    // reset the audio backend, it will be enabled again if needed
                    #[cfg(not(target_os = "windows"))]
                    plugins::audio::reset_audio_backend();

                    FAILED_TXS.write().clear();
                }
            }

            *ACTIVE_PROFILE_NAME.write() = None;
        }

        {
            // reload of current profile requested?
            if REQUEST_PROFILE_RELOAD.load(Ordering::SeqCst) {
                REQUEST_PROFILE_RELOAD.store(false, Ordering::SeqCst);

                let active_profile = ACTIVE_PROFILE.read();
                let profile_clone = active_profile.clone();

                // ACTIVE_PROFILE lock needs to be released here, or otherwise we may deadlock
                drop(active_profile);

                if let Some(profile) = &profile_clone {
                    if let Err(e) = switch_profile(Some(&profile.profile_file), dbus_api_tx, false)
                    {
                        error!("Could not reload profile: {}", e);
                    } else {
                        // reset the audio backend, it will be enabled again if needed
                        #[cfg(not(target_os = "windows"))]
                        plugins::audio::reset_audio_backend();

                        // don't notify "active profile changed", since it may deadlock

                        // dbus_api_tx
                        //     .send(DbusApiEvent::ActiveProfileChanged)
                        //     .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));

                        FAILED_TXS.write().clear();
                    }
                }
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

            let saved_status = crate::DEVICE_STATUS.as_ref().read().clone();

            if let Err(_e) = events::process_timer_event() {
                /* do nothing  */

                // if e.type_id() == (HwDeviceError::NoOpResult {}).type_id() {
                //     error!("Could not process a timer event: {}", e);
                // } else {
                //     trace!("Result is a NoOp");
                // }
            }

            last_status_poll = Instant::now();

            let current_status = crate::DEVICE_STATUS.read().clone();

            if current_status != saved_status {
                dbus_api_tx
                    .send(DbusApiEvent::DeviceStatusChanged)
                    .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));
            }

            // use 'device status poll' code to detect failed/disconnected devices as well,
            // by forcing a write to the device. This is required for hotplug to work correctly in
            // case we didn't transfer data to the device for an extended period of time
            script::FRAME_GENERATION_COUNTER.fetch_add(1, Ordering::SeqCst);
        }

        // now, process events from all available sources...
        let result = sel.wait_timeout(Duration::from_millis(1000 / (constants::TARGET_FPS * 2)));

        let timedout = if let Err(result) = result {
            match result {
                SelectError::Timeout => true,
            }
        } else {
            false
        };

        // remove all failed devices
        #[cfg(not(target_os = "windows"))]
        for idx in failed_kbd_rxs.read().iter() {
            // warn!("Removing keyboard rx with index {idx}");
            // kbd_rxs.remove(*idx);

            if let Some(device) = crate::KEYBOARD_DEVICES.write().get_mut(*idx) {
                let _ = device
                    .write()
                    .as_device_mut()
                    .fail()
                    .map_err(|_e| ratelimited::error!("Could not mark the device as failed"));
            }
        }

        #[cfg(not(target_os = "windows"))]
        for idx in failed_mouse_rxs.read().iter() {
            // warn!("Removing mouse rx with index {idx}");
            // mouse_rxs.remove(*idx);

            if let Some(device) = crate::MOUSE_DEVICES.write().get_mut(*idx) {
                let _ = device
                    .write()
                    .as_device_mut()
                    .fail()
                    .map_err(|_e| ratelimited::error!("Could not mark the device as failed"));
            }
        }

        // #[cfg(not(target_os = "windows"))]
        // for idx in failed_misc_rxs.read().iter() {
        //     // warn!("Removing misc rx with index {idx}");
        //     // misc_rxs.remove(*idx);
        //
        //     if let Some(device) = crate::MISC_DEVICES.write().get_mut(*idx) {
        //         let _ = device
        //             .write()
        //             .as_device_mut()
        //             .fail()
        //             .map_err(|_e| ratelimited::error!("Could not mark the device as failed"));
        //     }
        // }

        // terminate the main loop (and later re-enter it) on device failure
        // in most cases eruption should better be restarted

        #[cfg(target_os = "windows")]
        if device_has_failed || (result.is_err() && !timedout) {
            return Err(MainError::DeviceFailed {}.into());
        }

        #[cfg(not(target_os = "windows"))]
        if device_has_failed
            || (result.is_err() && !timedout)
            || !failed_kbd_rxs.read().is_empty()
            || !failed_mouse_rxs.read().is_empty()
        {
            return Err(MainError::DeviceFailed {}.into());
        }

        if delay_time_hid_poll.elapsed()
            >= Duration::from_millis(1000 / (constants::TARGET_FPS * 8))
        {
            #[cfg(feature = "profiling")]
            coz::scope!("HID events polling");

            delay_time_hid_poll = Instant::now();

            // poll HID events on all available devices
            for device in crate::KEYBOARD_DEVICES.read().iter() {
                events::process_keyboard_hid_events(device).unwrap_or_else(|e| {
                    ratelimited::error!("Could not process a keyboard HID event: {}", e)
                });
            }

            for device in crate::MOUSE_DEVICES.read().iter() {
                events::process_mouse_hid_events(device).unwrap_or_else(|e| {
                    ratelimited::error!("Could not process a mouse HID event: {}", e)
                });
            }
        }

        if delay_time_tick.elapsed() >= Duration::from_millis(1000 / constants::TICK_FPS) {
            #[cfg(feature = "profiling")]
            coz::scope!("timer tick code");

            let delta =
                (delay_time_tick.elapsed().as_millis() as u64 / constants::TARGET_FPS) as u32;

            delay_time_tick = Instant::now();

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
        }

        if !device_has_failed
            && delay_time_render.elapsed() >= Duration::from_millis(1000 / constants::TARGET_FPS)
        {
            #[cfg(feature = "profiling")]
            coz::scope!("render code");

            delay_time_render = Instant::now();

            // send timer tick events to the Lua VMs
            for (index, lua_tx) in LUA_TXS.read().iter().enumerate() {
                // if this tx failed previously, then skip it completely
                if !FAILED_TXS.read().contains(&index) {
                    lua_tx.send(script::Message::Render).unwrap_or_else(|e| {
                        error!("Send error during timer tick event: {}", e);
                        FAILED_TXS.write().insert(index);
                    });
                }
            }

            // finally, send request to update the LEDs if necessary
            DEV_IO_TX
                .read()
                .as_ref()
                .unwrap()
                .send(DeviceAction::RenderNow)
                .unwrap_or_else(|e| {
                    error!("Send error: {}", e);
                });
        }

        // update the monotonic timer used for fading-effect when switching between profiles
        let delta = start_time.elapsed().as_millis() as isize;
        crate::FADER.fetch_add(delta, Ordering::SeqCst);

        // compute AFK time
        if afk_timeout_secs > 0 {
            let afk = LAST_INPUT_TIME.read().elapsed() >= Duration::from_secs(afk_timeout_secs);
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
        }
        /* else if elapsed_after_sleep < 5_u128 {
            debug!("Short loop detected");
            debug!(
                "Loop took: {} milliseconds, goal: {}",
                elapsed_after_sleep,
                1000 / constants::TARGET_FPS
            );
        }
        else {
            debug!(
                "Loop took: {} milliseconds, goal: {}",
                elapsed_after_sleep,
                1000 / constants::TARGET_FPS
            );
        } */

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

        cfg_if::cfg_if! {
            if #[cfg(not(target_os = "windows"))] {
                let mut devices_rx = crate::KEYBOARD_DEVICES_RX.write();
                assert!(devices_rx.len() > index);
                devices_rx.remove(index);
            }
        }

        assert!(keyboard_devices.len() > index);

        let usb_id = keyboard_devices
            .get(index)
            .map(|d| (d.read().get_usb_vid(), d.read().get_usb_pid()));

        keyboard_devices.remove(index);

        result = true;

        debug!("Sending device hot remove notification...");

        let dbus_api_tx = crate::DBUS_API_TX.read();
        let dbus_api_tx = dbus_api_tx.as_ref().unwrap();

        dbus_api_tx
            .send(DbusApiEvent::DeviceHotplug(usb_id.unwrap_or((0, 0)), true))
            .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));
    }

    let mut mouse_devices = crate::MOUSE_DEVICES.write();
    if let Some(index) = mouse_devices
        .iter()
        .position(|device: &hwdevices::MouseDevice| device.read().has_failed().unwrap_or(true))
    {
        info!("Unplugging a failed mouse device...");

        cfg_if::cfg_if! {
            if #[cfg(not(target_os = "windows"))] {
                let mut devices_rx = crate::MOUSE_DEVICES_RX.write();
                assert!(devices_rx.len() > index);
                devices_rx.remove(index);
            }
        }

        assert!(mouse_devices.len() > index);

        let usb_id = mouse_devices
            .get(index)
            .map(|d| (d.read().get_usb_vid(), d.read().get_usb_pid()));

        mouse_devices.remove(index);

        result = true;

        debug!("Sending device hot remove notification...");

        let dbus_api_tx = crate::DBUS_API_TX.read();
        let dbus_api_tx = dbus_api_tx.as_ref().unwrap();

        dbus_api_tx
            .send(DbusApiEvent::DeviceHotplug(usb_id.unwrap_or((0, 0)), true))
            .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));
    }

    let mut misc_devices = crate::MISC_DEVICES.write();
    if let Some(index) = misc_devices
        .iter()
        .position(|device: &hwdevices::MiscDevice| device.read().has_failed().unwrap_or(true))
    {
        info!("Unplugging a failed misc device...");

        cfg_if::cfg_if! {
            if #[cfg(not(target_os = "windows"))] {
                let mut devices_rx = crate::MISC_DEVICES_RX.write();
                assert!(devices_rx.len() > index);
                devices_rx.remove(index);
            }
        }

        assert!(misc_devices.len() > index);
        misc_devices.remove(index);

        let usb_id = misc_devices
            .get(index)
            .map(|d| (d.read().get_usb_vid(), d.read().get_usb_pid()));

        result = true;

        debug!("Sending device hot remove notification...");

        let dbus_api_tx = crate::DBUS_API_TX.read();
        let dbus_api_tx = dbus_api_tx.as_ref().unwrap();

        dbus_api_tx
            .send(DbusApiEvent::DeviceHotplug(usb_id.unwrap_or((0, 0)), true))
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

/*
#[cfg(debug_assertions)]
mod thread_util {
    use crate::Result;
    use parking_lot::deadlock;
    use std::thread;
    use std::time::Duration;
    use tracing::*;

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
*/

/// open the control and LED devices of the keyboard
pub fn init_keyboard_device(keyboard_device: &KeyboardDevice) {
    info!("Opening keyboard device...");

    let hidapi = crate::HIDAPI.read();
    let hidapi = hidapi.as_ref().unwrap();

    let make = hwdevices::get_device_make(
        keyboard_device.read().get_usb_vid(),
        keyboard_device.read().get_usb_pid(),
    )
    .unwrap_or("<unknown>");
    let model = hwdevices::get_device_model(
        keyboard_device.read().get_usb_vid(),
        keyboard_device.read().get_usb_pid(),
    )
    .unwrap_or("<unknown>");

    keyboard_device.write().open(hidapi).unwrap_or_else(|e| {
        error!("Error opening the keyboard device '{make} {model}': {}", e);
        error!(
            "This could be a permission problem, or maybe the device is locked by another process?"
        );
    });

    // send initialization handshake
    info!("Initializing keyboard device...");
    keyboard_device
        .write()
        .send_init_sequence()
        .unwrap_or_else(|e| error!("Could not initialize the device '{make} {model}': {}", e));

    // set LEDs to a known good initial state
    info!("Configuring keyboard LEDs...");
    keyboard_device
        .write()
        .set_led_init_pattern()
        .unwrap_or_else(|e| {
            error!(
                "Could not initialize LEDs of the device '{make} {model}': {}",
                e
            )
        });

    info!(
        "Firmware revision: '{make} {model}': {}",
        keyboard_device.read().get_firmware_revision()
    );
}

/// open the sub-devices of the mouse
pub fn init_mouse_device(mouse_device: &MouseDevice) {
    info!("Opening mouse device...");

    let hidapi = crate::HIDAPI.read();
    let hidapi = hidapi.as_ref().unwrap();

    let make = hwdevices::get_device_make(
        mouse_device.read().get_usb_vid(),
        mouse_device.read().get_usb_pid(),
    )
    .unwrap_or("<unknown>");
    let model = hwdevices::get_device_model(
        mouse_device.read().get_usb_vid(),
        mouse_device.read().get_usb_pid(),
    )
    .unwrap_or("<unknown>");

    mouse_device.write().open(hidapi).unwrap_or_else(|e| {
        error!("Error opening the mouse device '{make} {model}': {}", e);
        error!(
            "This could be a permission problem, or maybe the device is locked by another process?"
        );
    });

    // send initialization handshake
    info!("Initializing mouse device...");
    mouse_device
        .write()
        .send_init_sequence()
        .unwrap_or_else(|e| error!("Could not initialize the device '{make} {model}': {}", e));

    // set LEDs to a known good initial state
    info!("Configuring mouse LEDs...");
    mouse_device
        .write()
        .set_led_init_pattern()
        .unwrap_or_else(|e| {
            error!(
                "Could not initialize LEDs of the device '{make} {model}': {}",
                e
            )
        });

    info!(
        "Firmware revision: '{make} {model}': {}",
        mouse_device.read().get_firmware_revision()
    );
}

/// open the misc device
pub fn init_misc_device(misc_device: &MiscDevice) {
    info!("Opening misc device...");

    let hidapi = crate::HIDAPI.read();
    let hidapi = hidapi.as_ref().unwrap();

    let make = hwdevices::get_device_make(
        misc_device.read().get_usb_vid(),
        misc_device.read().get_usb_pid(),
    )
    .unwrap_or("<unknown>");
    let model = hwdevices::get_device_model(
        misc_device.read().get_usb_vid(),
        misc_device.read().get_usb_pid(),
    )
    .unwrap_or("<unknown>");

    misc_device.write().open(hidapi).unwrap_or_else(|e| {
        error!("Error opening the misc device '{make} {model}': {}", e);
        error!(
            "This could be a permission problem, or maybe the device is locked by another process?"
        );
    });

    // send initialization handshake
    info!("Initializing misc device...");
    misc_device
        .write()
        .send_init_sequence()
        .unwrap_or_else(|e| error!("Could not initialize the device '{make} {model}': {}", e));

    // set LEDs to a known good initial state
    info!("Configuring misc device LEDs...");
    misc_device
        .write()
        .set_led_init_pattern()
        .unwrap_or_else(|e| {
            error!(
                "Could not initialize LEDs of the device '{make} {model}': {}",
                e
            )
        });

    info!(
        "Firmware revision: '{make} {model}': {}",
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

    if atty::is(atty::Stream::Stdout) {
        if !env::args().any(|a| a.eq_ignore_ascii_case("daemon"))
            && !env::args().any(|a| a.eq_ignore_ascii_case("completions"))
        {
            // we require the "daemon" subcommand to be specified, as a safety measure to
            // prevent the accidental execution of another instance of the eruption daemon
            eprintln!(
                "Did you probably intend to run the `{}` command instead?",
                "eruptionctl".bold()
            );
            eprintln!("");
            eprintln!("");
            eprintln!("If you meant to run the Eruption daemon, please use:");
            eprintln!(
                "{}",
                "sudo systemctl unmask eruption.service && sudo systemctl restart eruption.service"
                    .bold()
            );
            eprintln!("");
            eprintln!("To run the Eruption daemon from the current shell, please use:");
            eprintln!(
                "{}",
                "sudo -u eruption RUST_LOG=info eruption daemon".bold()
            );

            return Ok(());
        } else {
            // print a license header, except if we are generating shell completions
            if !env::args().any(|a| a.eq_ignore_ascii_case("completions")) {
                print_header();
            }
        }
    }

    // start the thread deadlock detector
    // #[cfg(debug_assertions)]
    // thread_util::deadlock_detector()
    //     .unwrap_or_else(|e| error!("Could not spawn deadlock detector thread: {}", e));

    let matches = parse_commandline();

    info!(
        "Starting Eruption - Realtime RGB LED Driver for Linux: {}",
        format!(
            "version {version} ({build_type} build) [{branch}:{commit} {dirty}]",
            version = env!("CARGO_PKG_VERSION"),
            branch = env!("GIT_BRANCH"),
            commit = env!("GIT_COMMIT"),
            dirty = if env!("GIT_DIRTY") == "true" {
                "dirty"
            } else {
                "clean"
            },
            // timestamp = env!("SOURCE_TIMESTAMP"),
            build_type = if cfg!(debug_assertions) {
                "debug"
            } else {
                "release"
            }
        )
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
    #[cfg(not(target_os = "windows"))]
    let _result = util::write_pid_file();

    // process configuration file
    let config_file = matches
        .get_one("config")
        .unwrap_or(&constants::DEFAULT_CONFIG_FILE.to_string())
        .to_string();

    let config = Config::builder()
        .add_source(config::File::new(&config_file, config::FileFormat::Toml))
        .build()
        .unwrap_or_else(|e| {
            tracing::error!("Could not parse configuration file: {}", e);
            process::exit(1);
        });

    *CONFIG.write() = Some(config.clone());

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

    *DRIVER_MATURITY_LEVEL.write() = driver_maturity_level;

    match *DRIVER_MATURITY_LEVEL.read() {
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
                thread::scope(|s| -> Result<()> {
                    let keyboard_init_thread = thread::Builder::new()
                        .name("init/kbd:all".to_string())
                        .spawn_scoped(s, move || {
                            // initialize keyboard devices
                            devices.0.par_iter().enumerate().for_each_init(
                                || {},
                                |_, (index, device)| {
                                    init_keyboard_device(device);

                                    let usb_vid = device.read().get_usb_vid();
                                    let usb_pid = device.read().get_usb_pid();

                                    cfg_if::cfg_if! {
                                        if #[cfg(not(target_os = "windows"))] {
                                            // spawn a thread to handle keyboard input
                                            info!("Spawning keyboard input thread...");

                                            let (kbd_tx, kbd_rx) = unbounded();
                                            threads::spawn_keyboard_input_thread(
                                                kbd_tx,
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
                                        }
                                    }

                                    crate::KEYBOARD_DEVICES.write().push(device.clone());
                                },
                            )
                        })?;

                    let mice_init_thread = thread::Builder::new()
                        .name("init/mice:all".to_string())
                        .spawn_scoped(s, move  || {
                            // initialize mouse devices
                            devices.1.par_iter().enumerate().for_each_init(
                                || {},
                                |_, (index, device)| {
                                    // enable mouse input
                                    if enable_mouse {
                                        init_mouse_device(device);

                                        let usb_vid = device.read().get_usb_vid();
                                        let usb_pid = device.read().get_usb_pid();

                                        cfg_if::cfg_if! {
                                            if #[cfg(not(target_os = "windows"))] {
                                                let (mouse_tx, mouse_rx) = unbounded();
                                                // let (mouse_secondary_tx, _mouse_secondary_rx) = unbounded();

                                                // spawn a thread to handle mouse input
                                                info!("Spawning mouse input thread...");

                                                spawn_mouse_input_thread(
                                                    mouse_tx,
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
                                            }
                                        }

                                        crate::MOUSE_DEVICES.write().push(device.clone());
                                    } else {
                                        info!("Found mouse device, but mouse support is DISABLED by configuration");
                                    }
                                },
                            )
                        })?;

                    let misc_init_thread = thread::Builder::new()
                        .name("init/misc:all".to_string())
                        .spawn_scoped(s, || {
                            // initialize misc devices
                            devices.2.par_iter().enumerate().for_each_init(
                                || {},
                                move |_, (index, device)| {
                                    init_misc_device(device);

                                    cfg_if::cfg_if! {
                                    if #[cfg(not(target_os = "windows"))] {
                                            if device.read().has_input_device() {
                                                let usb_vid = device.read().get_usb_vid();
                                                let usb_pid = device.read().get_usb_pid();

                                                // spawn a thread to handle keyboard input
                                                info!("Spawning misc device input thread...");

                                                let (misc_tx, misc_rx) = unbounded();
                                                threads::spawn_misc_input_thread(
                                                    misc_tx,
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
                                        }
                                    }

                                    crate::MISC_DEVICES.write().push(device.clone());
                                },
                            );
                        })?;

                    info!("Device enumeration completed");

                    // optionally wait for devices to settle; this should not be required
                    thread::sleep(Duration::from_millis(constants::DEVICE_SETTLE_DELAY));

                    let _ = keyboard_init_thread.join().map_err(|_| {
                        error!("Error during initialization of at least one device occurred")
                    });
                    let _ = mice_init_thread.join().map_err(|_| {
                        error!("Error during initialization of at least one device occurred")
                    });
                    let _ = misc_init_thread.join().map_err(|_| {
                        error!("Error during initialization of at least one device occurred")
                    });

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
                    #[cfg(not(target_os = "windows"))]
                    info!("Initializing Linux Userspace LEDs interface...");

                    #[cfg(not(target_os = "windows"))]
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

                    *DBUS_API_TX.write() = Some(dbus_api_tx.clone());

                    let (fsevents_tx, fsevents_rx) = unbounded();
                    register_filesystem_watcher(fsevents_tx, PathBuf::from(&config_file))
                        .unwrap_or_else(|e| {
                            error!("Could not register file changes watcher: {}", e)
                        });

                    // initialize the device I/O thread
                    info!("Initializing device I/O thread...");
                    let (dev_io_tx, dev_io_rx) = unbounded();
                    threads::spawn_device_io_thread(dev_io_rx).unwrap_or_else(|e| {
                        error!("Could not spawn the render thread: {}", e);
                        panic!()
                    });

                    *DEV_IO_TX.write() = Some(dev_io_tx);

                    info!("Late initializations completed");

                    info!("Startup completed");

                    'OUTER_LOOP: loop {
                        info!("Entering the main loop now...");

                        let mut errors_present = false;

                        // enter the main loop
                        run_main_loop(&dbus_api_tx, &ctrl_c_rx, &dbus_rx, &fsevents_rx)
                            .unwrap_or_else(|e| {
                                warn!("Left the main loop due to an irrecoverable error: {}", e);
                                errors_present = true;
                            });

                        if !errors_present {
                            info!("Main loop terminated gracefully");
                        }

                        if crate::QUIT.load(Ordering::SeqCst) {
                            break 'OUTER_LOOP;
                        }

                        // wait a few milliseconds to give devices time to settle
                        thread::sleep(Duration::from_millis(constants::DEVICE_SETTLE_MILLIS));

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
                    for device in crate::KEYBOARD_DEVICES.read().iter() {
                        device.write().set_led_off_pattern().unwrap_or_else(|e| {
                            error!("Could not finalize LEDs configuration: {}", e)
                        });

                        device.write().close_all().unwrap_or_else(|e| {
                            warn!("Could not close the device: {}", e);
                        });
                    }

                    // set LEDs of all mice to a known final state, then close all associated devices
                    for device in crate::MOUSE_DEVICES.read().iter() {
                        device.write().set_led_off_pattern().unwrap_or_else(|e| {
                            error!("Could not finalize LEDs configuration: {}", e)
                        });

                        device.write().close_all().unwrap_or_else(|e| {
                            warn!("Could not close the device: {}", e);
                        });
                    }

                    // set LEDs of all misc devices to a known final state, then close all associated devices
                    for device in crate::MISC_DEVICES.read().iter() {
                        device.write().set_led_off_pattern().unwrap_or_else(|e| {
                            error!("Could not finalize LEDs configuration: {}", e)
                        });

                        device.write().close_all().unwrap_or_else(|e| {
                            warn!("Could not close the device: {}", e);
                        });
                    }

                    Ok(())
                })?;
            } else {
                error!("Could not enumerate connected devices");
            }
        }

        Err(_) => {
            error!("Could not open HIDAPI");
            process::exit(1);
        }
    }

    info!("Exiting now");

    Ok(())
}

/// Main program entrypoint
pub fn main() -> std::result::Result<(), eyre::Error> {
    // initialize logging
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::util::SubscriberInitExt;

    if atty::is(atty::Stream::Stdout) {
        // let filter = tracing_subscriber::EnvFilter::from_default_env();
        // let journald_layer = tracing_journald::layer()?.with_filter(filter);

        #[cfg(not(target_os = "windows"))]
        let ansi = true;

        #[cfg(target_os = "windows")]
        let ansi = false;

        let filter = tracing_subscriber::EnvFilter::from_default_env();
        let format_layer = tracing_subscriber::fmt::layer()
            .compact()
            .with_ansi(ansi)
            .with_filter(filter);

        cfg_if::cfg_if! {
            if #[cfg(feature = "debug-async")] {
                let console_layer = console_subscriber::ConsoleLayer::builder()
                    .with_default_env()
                    .spawn();

                tracing_subscriber::registry()
                    // .with(journald_layer)
                    .with(console_layer)
                    .with(format_layer)
                    .init();
            } else {
                tracing_subscriber::registry()
                    // .with(journald_layer)
                    .with(format_layer)
                    .init();
            }
        };
    } else {
        let filter = tracing_subscriber::EnvFilter::from_default_env();
        let journald_layer = tracing_journald::layer()?.with_filter(filter);

        tracing_subscriber::registry().with(journald_layer).init();
    }

    #[cfg(feature = "profiling")]
    coz::thread_init();

    // i18n/l10n support
    let language_loader: FluentLanguageLoader = fluent_language_loader!();

    let requested_languages = DesktopLanguageRequester::requested_languages();
    i18n_embed::select(&language_loader, &Localizations, &requested_languages)?;

    STATIC_LOADER.lock().replace(language_loader);

    let runtime = tokio::runtime::Builder::new_current_thread()
        // .enable_all()
        .build()?;

    runtime.block_on(async move { async_main().await })
}
