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

use crate::{dbus_client::Message, sensors::PROCESS_SENSOR_FAILED};

#[cfg(feature = "sensor-mutter")]
use crate::sensors::MutterSensorData;

#[cfg(feature = "sensor-wayland")]
use crate::sensors::WaylandSensorData;

#[cfg(feature = "sensor-x11")]
use crate::sensors::X11SensorData;

use clap::{IntoApp, Parser};
use clap_complete::Shell;
use config::Config;
use dbus::blocking::stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged;
use dbus::blocking::Connection;
use dbus_client::{profile, slot};
use flume::{unbounded, Receiver, Sender};
use hotwatch::{
    blocking::{Flow, Hotwatch},
    Event,
};
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};
use indexmap::IndexMap;
use lazy_static::lazy_static;
use log::*;
use parking_lot::{Mutex, RwLock};
use regex::Regex;
use rust_embed::RustEmbed;
use sensors::WindowSensorData;
use serde::{Deserialize, Serialize};
use std::{env, fmt, fs, path::PathBuf, process, sync::atomic::AtomicBool, sync::Arc};
use std::{sync::atomic::Ordering, thread, time::Duration};
use syslog::Facility;

mod constants;
mod dbus_client;
mod dbus_interface;

mod logger;
#[cfg(feature = "sensor-procmon")]
mod procmon;
mod sensors;
mod util;

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
    /// Global configuration
    pub static ref CONFIG: Arc<Mutex<Option<config::Config>>> = Arc::new(Mutex::new(None));

    /// Mapping between event selector => action
    pub static ref RULES_MAP: Arc<RwLock<IndexMap<Selector, (RuleMetadata, Action)>>> = Arc::new(RwLock::new(IndexMap::new()));

    /// Saved previous states
    pub static ref PREVIOUS_STATES_MAP: Arc<RwLock<IndexMap<i32, Action>>> = Arc::new(RwLock::new(IndexMap::new()));

    /// Currently selected slot and profile
    pub static ref CURRENT_STATE: Arc<RwLock<(Option<u64>, Option<String>)>> = Arc::new(RwLock::new((None, None)));

    // Flags

    /// Global "enable experimental features" flag
    pub static ref EXPERIMENTAL_FEATURES: AtomicBool = AtomicBool::new(false);

    /// Signals that we initiated a profile change
    pub static ref PROFILE_CHANGING: AtomicBool = AtomicBool::new(false);

    /// Global "polling works" for the X11 sensor flag
    pub static ref X11_POLL_SUCCEEDED: AtomicBool = AtomicBool::new(false);

    /// Global "quit" status flag
    pub static ref QUIT: AtomicBool = AtomicBool::new(false);
}

type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum MainError {
    #[error("Unknown error: {description}")]
    UnknownError { description: String },

    #[error("Sensor error: {description}")]
    SensorError { description: String },

    #[error("Could not register Linux process monitoring")]
    ProcMonError {},

    #[error("Could not switch profiles")]
    SwitchProfileError {},

    #[error("Could not parse syslog log-level")]
    SyslogLevelError {},
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum WindowFocusedSelectorMode {
    WindowName,
    WindowInstance,
    WindowClass,
}

impl fmt::Display for WindowFocusedSelectorMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            WindowFocusedSelectorMode::WindowName => {
                write!(f, "Name")?;
            }

            WindowFocusedSelectorMode::WindowInstance => {
                write!(f, "Instance")?;
            }

            WindowFocusedSelectorMode::WindowClass => {
                write!(f, "Class")?;
            }
        };

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum Selector {
    ProcessExec {
        comm: String,
    },
    WindowFocused {
        mode: WindowFocusedSelectorMode,
        regex: String,
    },
}

impl fmt::Display for Selector {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Selector::ProcessExec { comm } => {
                write!(f, "On process execution: comm: '{}'", comm)?;
            }

            Selector::WindowFocused { mode, regex } => {
                write!(f, "On window focused: {}: '{}'", mode, regex)?;
            }
        };

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
    SwitchToProfile { profile_name: String },
    SwitchToSlot { slot_index: u64 },
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Action::SwitchToProfile { profile_name } => {
                write!(f, "Switch to profile: {}", profile_name)?;
            }

            Action::SwitchToSlot { slot_index } => {
                write!(f, "Switch to slot: {}", slot_index + 1)?;
            }
        };

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleMetadata {
    /// Specifies whether the rule is enabled
    pub enabled: bool,

    /// Set to true if the rule is auto-generated
    pub internal: bool,
}

impl std::default::Default for RuleMetadata {
    fn default() -> Self {
        RuleMetadata {
            enabled: true,
            internal: false,
        }
    }
}

impl fmt::Display for RuleMetadata {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "enabled: {}", self.enabled)?;
        write!(f, ", internal: {}", self.internal)?;

        Ok(())
    }
}

#[cfg(feature = "sensor-procmon")]
#[derive(Debug, Clone)]
pub enum SystemEvent {
    ProcessExec {
        event: procmon::Event,
        file_name: Option<String>,
        comm: Option<String>,
    },

    ProcessExit {
        event: procmon::Event,
    },
}

/// Supported command line arguments
#[derive(Debug, clap::Parser)]
#[clap(
    version = env!("CARGO_PKG_VERSION"),
    author = "X3n0m0rph59 <x3n0m0rph59@gmail.com>",
    about = "A daemon to monitor and introspect system processes and events",
)]
pub struct Options {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Sets the configuration file to use
    #[clap(short, long)]
    config: Option<String>,

    #[clap(subcommand)]
    command: Subcommands,
}

// Sub-commands
#[derive(Debug, clap::Parser)]
pub enum Subcommands {
    /// Run in background and monitor running processes
    Daemon,

    /// Rules related sub-commands
    Rules {
        #[clap(subcommand)]
        command: RulesSubcommands,
    },

    /// Generate shell completions
    Completions {
        // #[clap(subcommand)]
        shell: Shell,
    },
}

/// Sub-commands of the "rules" command
#[derive(Debug, clap::Parser)]
pub enum RulesSubcommands {
    /// List all available rules
    List,

    /// Mark a rule as enabled
    Enable { rule_index: usize },

    /// Mark a rule as disabled
    Disable { rule_index: usize },

    /// Add a new rule
    Add { rule: Vec<String> },

    /// Remove a rule by index
    Remove { rule_index: usize },
}

/// Subcommands of the "completions" command
#[derive(Debug, clap::Parser)]
pub enum CompletionsSubcommands {
    Bash,

    Elvish,

    Fish,

    PowerShell,

    Zsh,
}

#[derive(Debug, Clone)]
pub enum DebuggerEvent {
    ValueChanged { val: u32 },
}

#[derive(Debug, Clone)]
pub enum FileSystemEvent {
    RulesChanged,
}

/// Print license information
#[allow(dead_code)]
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

/// Execute an action
fn process_action(action: &Action) -> Result<()> {
    match action {
        Action::SwitchToProfile { profile_name } => {
            if CURRENT_STATE.read().1.is_none()
                || CURRENT_STATE.read().1.as_ref().unwrap() != profile_name
            {
                info!("Triggered action: {}", action);

                PROFILE_CHANGING.store(true, Ordering::SeqCst);

                dbus_client::switch_profile(profile_name)?;
            }

            CURRENT_STATE.write().1 = Some(profile_name.clone());
        }

        Action::SwitchToSlot { slot_index } => {
            if CURRENT_STATE.read().0.is_none()
                || CURRENT_STATE.read().0.as_ref().unwrap() != slot_index
            {
                info!("Triggered action: {}", action);

                PROFILE_CHANGING.store(true, Ordering::SeqCst);

                dbus_client::switch_slot(*slot_index)?;
            }

            CURRENT_STATE.write().0 = Some(*slot_index);
        }
    }

    Ok(())
}

/// Process system related events
#[cfg(feature = "sensor-procmon")]
fn process_system_event(event: &SystemEvent) -> Result<()> {
    match event {
        SystemEvent::ProcessExec {
            event,
            file_name: _,
            comm,
        } => {
            if let Some(comm) = comm {
                for (selector, (metadata, action)) in RULES_MAP.read().iter() {
                    match selector {
                        Selector::ProcessExec { comm: regex } => {
                            if metadata.enabled {
                                let re = Regex::new(regex)?;

                                if re.is_match(comm) {
                                    debug!("Matching rule for: {}", comm);

                                    match action {
                                        Action::SwitchToProfile { profile_name: _ } => {
                                            let profile_name = dbus_client::get_active_profile()?;
                                            let return_action =
                                                Action::SwitchToProfile { profile_name };
                                            PREVIOUS_STATES_MAP
                                                .write()
                                                .insert(event.pid, return_action);
                                        }

                                        Action::SwitchToSlot { slot_index: _ } => {
                                            let slot_index = dbus_client::get_active_slot()?;
                                            let return_action = Action::SwitchToSlot { slot_index };
                                            PREVIOUS_STATES_MAP
                                                .write()
                                                .insert(event.pid, return_action);
                                        }
                                    }

                                    process_action(action)?;
                                    break;
                                }
                            }
                        }

                        _ => { /* Ignore others */ }
                    }
                }
            } else {
                debug!("Could not get the process comm. The process vanished.");
            }
        }

        SystemEvent::ProcessExit { event } => {
            match PREVIOUS_STATES_MAP.read().get(&event.pid) {
                Some(action) => match action {
                    Action::SwitchToProfile { profile_name } => {
                        debug!("Returning to profile: {}", profile_name);

                        dbus_client::switch_profile(profile_name)?;
                    }

                    Action::SwitchToSlot { slot_index } => {
                        debug!("Returning to slot: {}", slot_index + 1);

                        dbus_client::switch_slot(*slot_index)?;
                    }
                },

                None => {
                    // no saved state available
                }
            }
        }
    }

    Ok(())
}

/// Process filesystem related events
fn process_fs_event(event: &FileSystemEvent, dbus_api_tx: &Sender<DbusApiEvent>) -> Result<()> {
    match event {
        FileSystemEvent::RulesChanged => {
            info!("Rules changed, reloading...");

            RULES_MAP.write().clear();

            load_rules_map().unwrap_or_else(|e| error!("Could not load rules: {}", e));

            for (selector, (metadata, action)) in RULES_MAP.read().iter() {
                debug!("{} => {} ({})", selector, action, metadata);
            }

            dbus_api_tx.send(DbusApiEvent::RulesChanged {})?;
        }
    }

    Ok(())
}

/// Process D-Bus related events
fn process_dbus_event(event: &dbus_client::Message) -> Result<()> {
    match event {
        Message::ProfileChanged(profile_name) => {
            // update the default rule to use the newly selected profile,
            // but only if we did not initiate the profile change
            if !PROFILE_CHANGING.load(Ordering::SeqCst) {
                let selector = Selector::WindowFocused {
                    mode: WindowFocusedSelectorMode::WindowInstance,
                    regex: ".*".to_string(),
                };

                if let Some((_metadata, action)) = RULES_MAP.write().get_mut(&selector) {
                    info!(
                        "Updating the default rule to use the profile: {}",
                        profile_name
                    );

                    *action = Action::SwitchToProfile {
                        profile_name: profile_name.clone(),
                    };
                } else {
                    error!("Could not get the default rule");
                }

                // update global state
                CURRENT_STATE.write().1 = Some(profile_name.clone());
            } else {
                // we initiated the profile change
                PROFILE_CHANGING.store(false, Ordering::SeqCst);
            }
        }

        _ => { /* ignore other events */ }
    }

    Ok(())
}

#[allow(dead_code)]
fn process_window_event(event: &dyn WindowSensorData) -> Result<()> {
    trace!("Sensor data: {:#?}", event);

    for (selector, (metadata, action)) in RULES_MAP.read().iter() {
        match selector {
            Selector::WindowFocused { mode, regex } => {
                if metadata.enabled {
                    let re = Regex::new(regex)?;

                    match mode {
                        WindowFocusedSelectorMode::WindowName => {
                            if re.is_match(event.window_name().unwrap_or_default()) {
                                process_action(action)?;
                                break;
                            }
                        }

                        WindowFocusedSelectorMode::WindowInstance => {
                            if re.is_match(event.window_instance().unwrap_or_default()) {
                                process_action(action)?;
                                break;
                            }
                        }

                        WindowFocusedSelectorMode::WindowClass => {
                            if re.is_match(event.window_class().unwrap_or_default()) {
                                process_action(action)?;
                                break;
                            }
                        }
                    }
                }
            }

            _ => { /* not a window related selector */ }
        }
    }

    Ok(())
}

/// Watch filesystem events
pub fn register_filesystem_watcher(
    fsevents_tx: Sender<FileSystemEvent>,
    rule_file: PathBuf,
) -> Result<()> {
    debug!("Registering filesystem watcher...");

    thread::Builder::new()
        .name("hotwatch".to_owned())
        .spawn(
            move || match Hotwatch::new_with_custom_delay(Duration::from_millis(1000)) {
                Err(e) => error!("Could not initialize filesystem watcher: {}", e),

                Ok(ref mut hotwatch) => {
                    hotwatch
                        .watch(&rule_file, move |event: Event| {
                            // check if we shall terminate the thread
                            if QUIT.load(Ordering::SeqCst) {
                                return Flow::Exit;
                            }

                            match event {
                                Event::Write(path) => {
                                    debug!("Rule file changed: {}", path.display());

                                    fsevents_tx
                                        .send(FileSystemEvent::RulesChanged)
                                        .unwrap_or_else(|e| {
                                            error!("Could not send on a channel: {}", e)
                                        });
                                }

                                _ => { /* do nothing */ }
                            }

                            Flow::Continue
                        })
                        .unwrap_or_else(|e| error!("Could not register file watch: {}", e));

                    hotwatch.run();
                }
            },
        )?;

    Ok(())
}

/// Spawn the dbus listener thread
pub fn spawn_dbus_thread(dbus_event_tx: Sender<dbus_client::Message>) -> Result<()> {
    thread::Builder::new()
        .name("dbus".to_owned())
        .spawn(move || -> Result<()> {
            let conn = Connection::new_system().unwrap();

            let slot_proxy = conn.with_proxy(
                "org.eruption",
                "/org/eruption/slot",
                Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
            );

            let profile_proxy = conn.with_proxy(
                "org.eruption",
                "/org/eruption/profile",
                Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
            );

            let config_proxy = conn.with_proxy(
                "org.eruption",
                "/org/eruption/config",
                Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
            );

            let tx = dbus_event_tx.clone();
            let _id1 = slot_proxy.match_signal(
                move |h: slot::OrgEruptionSlotActiveSlotChanged,
                      _: &Connection,
                      _message: &dbus::Message| {
                    tx.send(Message::SlotChanged(h.new_slot as usize)).unwrap();

                    true
                },
            )?;

            let tx = dbus_event_tx.clone();
            let _id1_1 = slot_proxy.match_signal(
                move |h: slot::OrgFreedesktopDBusPropertiesPropertiesChanged,
                      _: &Connection,
                      _message: &dbus::Message| {
                    // slot names have been changed
                    if let Some(args) = h.changed_properties.get("SlotNames") {
                        let slot_names = args
                            .0
                            .as_iter()
                            .unwrap()
                            .map(|v| v.as_str().unwrap().to_string())
                            .collect::<Vec<String>>();
                        tx.send(Message::SlotNamesChanged(slot_names)).unwrap();
                    }

                    true
                },
            )?;

            let tx = dbus_event_tx.clone();
            let _id2 = profile_proxy.match_signal(
                move |h: profile::OrgEruptionProfileActiveProfileChanged,
                      _: &Connection,
                      _message: &dbus::Message| {
                    let _ = tx
                        .send(Message::ProfileChanged(h.new_profile_name))
                        .map_err(|e| log::error!("Could not send a message: {}", e));

                    true
                },
            )?;

            let tx = dbus_event_tx;
            let _id3 = config_proxy.match_signal(
                move |h: PropertiesPropertiesChanged, _: &Connection, _message: &dbus::Message| {
                    if let Some(brightness) = h.changed_properties.get("Brightness") {
                        let brightness = brightness.0.as_i64().unwrap() as usize;

                        tx.send(Message::BrightnessChanged(brightness)).unwrap();
                    }

                    if let Some(result) = h.changed_properties.get("EnableSfx") {
                        let enabled = result.0.as_u64().unwrap() != 0;

                        tx.send(Message::SoundFxChanged(enabled)).unwrap();
                    }

                    true
                },
            )?;

            loop {
                if let Err(e) = conn.process(Duration::from_millis(constants::DBUS_TIMEOUT_MILLIS))
                {
                    error!("Could not process a D-Bus message: {}", e);
                }
            }
        })?;

    Ok(())
}

#[derive(Debug, Clone)]
pub enum DbusApiEvent {
    RulesChanged,
}

/// Spawns the D-Bus API thread and executes it's main loop
fn spawn_dbus_api_thread(dbus_tx: Sender<dbus_interface::Message>) -> Result<Sender<DbusApiEvent>> {
    let (dbus_api_tx, dbus_api_rx) = unbounded();

    thread::Builder::new()
        .name("dbus_interface".into())
        .spawn(move || -> Result<()> {
            let dbus = dbus_interface::initialize(dbus_tx)?;

            loop {
                // process events, destined for the dbus api
                match dbus_api_rx.recv_timeout(Duration::from_millis(0)) {
                    Ok(result) => match result {
                        DbusApiEvent::RulesChanged => dbus.notify_rules_changed(),
                    },

                    // ignore timeout errors
                    Err(_e) => (),
                }

                dbus.get_next_event_timeout(constants::DBUS_TIMEOUT_MILLIS as u32)
                    .unwrap_or_else(|e| error!("Could not get the next D-Bus event: {}", e));
            }
        })?;

    Ok(dbus_api_tx)
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

pub fn run_main_loop(
    #[cfg(feature = "sensor-procmon")] sysevents_rx: &Receiver<SystemEvent>,
    fsevents_rx: &Receiver<FileSystemEvent>,
    dbusevents_rx: &Receiver<dbus_client::Message>,
    ctrl_c_rx: &Receiver<bool>,
    dbus_api_tx: &Sender<DbusApiEvent>,
) -> Result<()> {
    trace!("Entering main loop...");

    'MAIN_LOOP: loop {
        if QUIT.load(Ordering::SeqCst) {
            break 'MAIN_LOOP;
        }

        let mut sel = flume::Selector::new()
            .recv(ctrl_c_rx, |_| {
                QUIT.store(true, Ordering::SeqCst);
            })
            .recv(fsevents_rx, |event| {
                if let Ok(event) = event {
                    process_fs_event(&event, dbus_api_tx)
                        .unwrap_or_else(|e| error!("Could not process a filesystem event: {}", e))
                } else {
                    error!("{}", event.as_ref().unwrap_err());
                }
            })
            .recv(dbusevents_rx, |event| {
                if let Ok(event) = event {
                    process_dbus_event(&event)
                        .unwrap_or_else(|e| error!("Could not process a D-Bus event: {}", e))
                } else {
                    error!("{}", event.as_ref().unwrap_err());
                }
            });

        #[cfg(feature = "sensor-procmon")]
        {
            if !PROCESS_SENSOR_FAILED.load(Ordering::SeqCst) {
                sel = sel.recv(sysevents_rx, |event| {
                    if let Ok(event) = event {
                        process_system_event(&event)
                            .unwrap_or_else(|e| error!("Could not process a system event: {}", e));
                    } else {
                        error!("{}", event.as_ref().unwrap_err());
                    }
                });
            }
        }

        let _result = sel.wait_timeout(Duration::from_millis(constants::MAIN_LOOP_SLEEP_MILLIS));

        // poll all pollable sensors that do not notify us via messages
        for sensor in sensors::SENSORS.lock().iter_mut() {
            if sensor.is_pollable() && !sensor.is_failed() {
                match sensor.poll() {
                    #[allow(unused_variables)]
                    Ok(data) => {
                        // debug!("Sensor data: {}", data);

                        #[allow(unused_mut)]
                        let mut handled = false;

                        #[cfg(feature = "sensor-mutter")]
                        if let Some(data) = data.as_any().downcast_ref::<MutterSensorData>() {
                            process_window_event(data)?;

                            handled = true;
                        }

                        #[cfg(feature = "sensor-wayland")]
                        if let Some(data) = data.as_any().downcast_ref::<WaylandSensorData>() {
                            process_window_event(data)?;

                            handled = true;
                        }

                        #[cfg(feature = "sensor-x11")]
                        if let Some(data) = data.as_any().downcast_ref::<X11SensorData>() {
                            process_window_event(data)?;

                            X11_POLL_SUCCEEDED.store(true, Ordering::SeqCst);

                            handled = true;
                        }

                        if !handled {
                            return Err(MainError::SensorError {
                                description: "Unhandled sensor data type".to_string(),
                            }
                            .into());
                        }
                    }

                    Err(e) => {
                        error!("Could not poll sensor '{}': {}", sensor.get_id(), e);

                        // sensor.set_failed(true);
                    }
                }
            }
        }
    }

    Ok(())
}

fn load_rules_map() -> Result<()> {
    let rules_file = util::tilde_expand(constants::STATE_DIR)?.join("process-monitor.rules");

    let s = fs::read_to_string(&rules_file)?;

    let rules_vec: Vec<(Selector, (RuleMetadata, Action))> = serde_json::from_str(&s)?;
    let rules_map = rules_vec
        .iter()
        .cloned()
        .collect::<IndexMap<Selector, (RuleMetadata, Action)>>();

    RULES_MAP.write().extend(rules_map);

    // add auto-generated rules
    let default_profile = crate::CONFIG
        .lock()
        .as_ref()
        .unwrap()
        .get_string("global.default_profile")
        .unwrap_or_else(|_| {
            dbus_client::get_active_profile()
                .unwrap_or_else(|_| constants::DEFAULT_PROFILE.to_string())
        });

    let selector = Selector::WindowFocused {
        mode: WindowFocusedSelectorMode::WindowInstance,
        regex: ".*".to_string(),
    };

    let metadata = RuleMetadata {
        internal: true,
        ..Default::default()
    };

    let action = Action::SwitchToProfile {
        profile_name: default_profile,
    };

    RULES_MAP.write().insert(selector, (metadata, action));

    Ok(())
}

fn save_rules_map() -> Result<()> {
    let rules_dir = util::tilde_expand(constants::STATE_DIR)?;
    let rules_file = rules_dir.join("process-monitor.rules");

    util::create_dir(&rules_dir)?;

    let rules_map = RULES_MAP.read();
    let v = rules_map
        .iter()
        // do not save internal auto-generated rules, they will be regenerated anyway
        .filter(|(_, (meta, _))| !meta.internal)
        .collect::<Vec<(&Selector, &(RuleMetadata, Action))>>();

    let s = serde_json::to_string_pretty(&v)?;
    fs::write(&rules_file, s)?;

    Ok(())
}

pub async fn async_main() -> std::result::Result<(), eyre::Error> {
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

    // print a license header, except if we are generating shell completions
    if !env::args().any(|a| a.eq_ignore_ascii_case("completions")) && env::args().count() < 2 {
        print_header();
    }

    let opts = Options::parse();
    let daemon = matches!(opts.command, Subcommands::Daemon);

    if unsafe { libc::isatty(0) != 0 } && daemon {
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
            Facility::LOG_USER,
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

    info!(
        "Starting eruption-process-monitor: Version {}",
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
    let config_file = opts
        .config
        .unwrap_or_else(|| constants::PROCESS_MONITOR_CONFIG_FILE.to_string());

    let config = Config::builder()
        .add_source(config::File::new(&config_file, config::FileFormat::Toml))
        .build()
        .unwrap_or_else(|e| {
            log::error!("Could not parse configuration file: {}", e);
            process::exit(4);
        });

    // enable support for experimental features?
    let enable_experimental_features = config
        .get::<bool>("global.enable_experimental_features")
        .unwrap_or(false);

    EXPERIMENTAL_FEATURES.store(enable_experimental_features, Ordering::SeqCst);

    if EXPERIMENTAL_FEATURES.load(Ordering::SeqCst) {
        warn!("** EXPERIMENTAL FEATURES are ENABLED, this may expose serious bugs! **");
    }

    *CONFIG.lock() = Some(config);

    // initialize plugins
    info!("Registering plugins...");

    sensors::register_sensors()?;

    info!("Loading rules...");
    load_rules_map().unwrap_or_else(|e| error!("Could not load rules: {}", e));

    match opts.command {
        Subcommands::Daemon => {
            for (index, (selector, (metadata, action))) in RULES_MAP.read().iter().enumerate() {
                info!("{:3}: {} => {} ({})", index, selector, action, metadata);
            }

            let rules_dir = util::tilde_expand(constants::STATE_DIR)?;
            let rules_file = rules_dir.join("process-monitor.rules");

            util::create_dir(&rules_dir)?;
            util::create_rules_file_if_not_exists(&rules_file)?;

            let (dbusevents_tx, dbusevents_rx) = unbounded();
            spawn_dbus_thread(dbusevents_tx)?;

            // initialize the D-Bus API
            let (dbus_tx, _dbus_rx) = unbounded();
            let dbus_api_tx = spawn_dbus_api_thread(dbus_tx)?;

            let (fsevents_tx, fsevents_rx) = unbounded();
            register_filesystem_watcher(fsevents_tx, rules_file)?;

            // configure plugins
            #[cfg(feature = "sensor-procmon")]
            let (sysevents_tx, sysevents_rx) = unbounded();

            #[cfg(feature = "sensor-procmon")]
            if let Some(mut s) = sensors::find_sensor_by_id("process") {
                let process_sensor = s
                    .as_any_mut()
                    .downcast_mut::<sensors::ProcessSensor>()
                    .unwrap();

                process_sensor.spawn_system_monitor_thread(sysevents_tx)?;
            }

            info!("Startup completed");

            debug!("Entering the main loop now...");

            // enter the main loop
            run_main_loop(
                #[cfg(feature = "sensor-procmon")]
                &sysevents_rx,
                &fsevents_rx,
                &dbusevents_rx,
                &ctrl_c_rx,
                &dbus_api_tx,
            )
            .unwrap_or_else(|e| error!("{}", e));

            debug!("Left the main loop");
        }

        Subcommands::Rules { command } => match command {
            RulesSubcommands::List => {
                for (index, (selector, (metadata, action))) in RULES_MAP.read().iter().enumerate() {
                    println!("{:3}: {} => {} ({})", index, selector, action, metadata);
                }
            }

            RulesSubcommands::Add { rule } => {
                fn print_usage_examples() {
                    eprintln!("\nPlease see below for some examples:");

                    for s in sensors::SENSORS.lock().iter() {
                        eprintln!("{}", s.get_usage_example());
                    }
                }

                if rule.len() != 3 {
                    eprintln!("Malformed rule definition");
                    print_usage_examples();
                } else {
                    let sensor = &rule[0];
                    let selector = &rule[1];
                    let action = &rule[2];

                    let mut parsed_selector = None;
                    let parsed_action;

                    // TODO: move this to the plugin code
                    if sensor.contains("exec") {
                        parsed_selector = Some(Selector::ProcessExec {
                            comm: selector.clone(),
                        });
                    } else if sensor.contains("window-class") {
                        parsed_selector = Some(Selector::WindowFocused {
                            mode: WindowFocusedSelectorMode::WindowClass,
                            regex: selector.clone(),
                        });
                    } else if sensor.contains("window-instance") {
                        parsed_selector = Some(Selector::WindowFocused {
                            mode: WindowFocusedSelectorMode::WindowInstance,
                            regex: selector.clone(),
                        });
                    } else if sensor.contains("window-name") {
                        parsed_selector = Some(Selector::WindowFocused {
                            mode: WindowFocusedSelectorMode::WindowName,
                            regex: selector.clone(),
                        });
                    }

                    if parsed_selector.is_none() {
                        eprintln!("Syntax error in selector");
                        print_usage_examples();
                    } else {
                        if action.contains(".profile") {
                            parsed_action = Action::SwitchToProfile {
                                profile_name: action.clone(),
                            };

                            RULES_MAP.write().insert(
                                parsed_selector.clone().unwrap(),
                                (RuleMetadata::default(), parsed_action.clone()),
                            );
                        } else {
                            parsed_action = Action::SwitchToSlot {
                                slot_index: action.parse::<u64>()? - 1,
                            };

                            RULES_MAP.write().insert(
                                parsed_selector.clone().unwrap(),
                                (RuleMetadata::default(), parsed_action.clone()),
                            );
                        }

                        // print resulting action to console
                        println!("{} => {}", parsed_selector.unwrap(), parsed_action);

                        save_rules_map()?;
                    }
                }
            }

            RulesSubcommands::Enable { rule_index } => {
                match RULES_MAP.write().get_index_mut(rule_index) {
                    Some((ref selector, (metadata, action))) => {
                        if !metadata.internal {
                            metadata.enabled = true;

                            println!(
                                "{:3}: {} => {} ({})",
                                rule_index, selector, action, metadata
                            );
                        } else {
                            eprintln!("Trying to change an internal (auto-generated) rule, this is a noop!");
                        }
                    }

                    None => eprintln!("No matching rules found!"),
                }

                save_rules_map()?;
            }

            RulesSubcommands::Disable { rule_index } => {
                match RULES_MAP.write().get_index_mut(rule_index) {
                    Some((ref selector, (metadata, action))) => {
                        if !metadata.internal {
                            metadata.enabled = false;

                            println!(
                                "{:3}: {} => {} ({})",
                                rule_index, selector, action, metadata
                            );
                        } else {
                            eprintln!("Trying to change an internal (auto-generated) rule, this is a noop!");
                        }
                    }

                    None => eprintln!("No matching rules found!"),
                }

                save_rules_map()?;
            }

            RulesSubcommands::Remove { rule_index } => {
                // print results to console
                match RULES_MAP.write().shift_remove_index(rule_index) {
                    Some((selector, (metadata, action))) => {
                        if !metadata.internal {
                            println!(
                                "{:3}: {} => {} ({})",
                                rule_index, selector, action, metadata
                            );
                        } else {
                            eprintln!("Trying to remove an internal (auto-generated) rule, this is a noop!");
                        }
                    }

                    None => eprintln!("No matching rules found!"),
                }

                save_rules_map()?;
            }
        },

        Subcommands::Completions { shell } => {
            const BIN_NAME: &str = env!("CARGO_PKG_NAME");

            let mut command = Options::command();
            let mut fd = std::io::stdout();

            clap_complete::generate(shell, &mut command, BIN_NAME.to_string(), &mut fd);
        }
    }

    info!("Saving rules...");
    save_rules_map().unwrap_or_else(|e| error!("Could not save rules: {}", e));

    info!("Exiting now");

    Ok(())
}

/// Main program entrypoint
pub fn main() -> std::result::Result<(), eyre::Error> {
    let language_loader: FluentLanguageLoader = fluent_language_loader!();

    let requested_languages = DesktopLanguageRequester::requested_languages();
    i18n_embed::select(&language_loader, &Localizations, &requested_languages)?;

    STATIC_LOADER.lock().replace(language_loader);

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async move { async_main().await })
}
