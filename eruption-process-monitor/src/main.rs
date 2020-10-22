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

use crate::dbus_client::Message;
use clap::Clap;
use clap::*;
use crossbeam::channel::{unbounded, Receiver, Select, Sender};
use dbus::blocking::stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged;
use dbus::blocking::Connection;
use dbus_client::{profile, slot};
use hotwatch::{
    blocking::{Flow, Hotwatch},
    Event,
};
use indexmap::IndexMap;
use lazy_static::lazy_static;
use log::*;
use parking_lot::{Mutex, RwLock};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{env, fmt, fs, path::PathBuf, sync::atomic::AtomicBool, sync::Arc};
use std::{sync::atomic::Ordering, thread, time::Duration};

mod constants;
mod dbus_client;
mod procmon;
mod sensors;
mod util;

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
    pub static ref EXPERIMENTAL_FEATURES: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    /// Signals that we initiated a profile change
    pub static ref PROFILE_CHANGING: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    /// Global "quit" status flag
    pub static ref QUIT: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum MainError {
    #[error("Unknown error: {description}")]
    UnknownError { description: String },

    #[error("Could not register Linux process monitoring")]
    ProcMonError {},

    #[error("Could not switch profiles")]
    SwitchProfileError {},
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
#[derive(Debug, Clap)]
#[clap(
    version = env!("CARGO_PKG_VERSION"),
    author = "X3n0m0rph59 <x3n0m0rph59@gmail.com>",
    about = "A daemon to monitor and introspect system processes and events",
)]
pub struct Options {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, parse(from_occurrences))]
    verbose: u8,

    hostname: Option<String>,
    port: Option<u16>,

    /// Sets the configuration file to use
    #[clap(short, long)]
    config: Option<String>,

    #[clap(subcommand)]
    command: Subcommands,
}

// Subcommands
#[derive(Debug, Clap)]
pub enum Subcommands {
    /// Run in background and monitor running processes
    Daemon,

    /// Rules related subcommands
    Rules {
        #[clap(subcommand)]
        command: RulesSubcommands,
    },
}

/// Subcommands of the "rules" command
#[derive(Debug, Clap)]
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
"#
    );
}

/// Execute an action
async fn process_action(action: &Action) -> Result<()> {
    match action {
        Action::SwitchToProfile { profile_name } => {
            if CURRENT_STATE.read().1.is_none()
                || CURRENT_STATE.read().1.as_ref().unwrap() != profile_name
            {
                info!("Triggered action: {}", action);

                PROFILE_CHANGING.store(true, Ordering::SeqCst);

                dbus_client::switch_profile(&profile_name).await?;
            }

            CURRENT_STATE.write().1 = Some(profile_name.clone());
        }

        Action::SwitchToSlot { slot_index } => {
            if CURRENT_STATE.read().0.is_none()
                || CURRENT_STATE.read().0.as_ref().unwrap() != slot_index
            {
                info!("Triggered action: {}", action);

                PROFILE_CHANGING.store(true, Ordering::SeqCst);

                dbus_client::switch_slot(*slot_index).await?;
            }

            CURRENT_STATE.write().0 = Some(*slot_index);
        }
    }

    Ok(())
}

/// Process system related events
async fn process_system_event(event: &SystemEvent) -> Result<()> {
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
                                let re = Regex::new(&regex)?;

                                if re.is_match(&comm) {
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

                                    process_action(&action).await?;
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

                        dbus_client::switch_profile(&profile_name).await?;
                    }

                    Action::SwitchToSlot { slot_index } => {
                        debug!("Returning to slot: {}", slot_index + 1);

                        dbus_client::switch_slot(*slot_index).await?;
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
async fn process_fs_event(event: &FileSystemEvent) -> Result<()> {
    match event {
        FileSystemEvent::RulesChanged => {
            info!("Rules changed, reloading...");

            RULES_MAP.write().clear();

            load_rules_map().unwrap_or_else(|e| error!("Could not load rules: {}", e));

            for (selector, (metadata, action)) in RULES_MAP.read().iter() {
                debug!("{} => {} ({})", selector, action, metadata);
            }
        }
    }

    Ok(())
}

/// Process D-Bus related events
async fn process_dbus_event(event: &dbus_client::Message) -> Result<()> {
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
            } else {
                // we initiated the profile change
                PROFILE_CHANGING.store(false, Ordering::SeqCst);
            }
        }

        _ => { /* ignore other events */ }
    }

    Ok(())
}

async fn process_window_event(event: &sensors::X11SensorData) -> Result<()> {
    trace!("Sensor data: {:#?}", event);

    for (selector, (metadata, action)) in RULES_MAP.read().iter() {
        match selector {
            Selector::WindowFocused { mode, regex } => {
                if metadata.enabled {
                    let re = Regex::new(&regex)?;

                    match mode {
                        WindowFocusedSelectorMode::WindowName => {
                            if re.is_match(&event.window_name) {
                                process_action(&action).await?;
                                break;
                            }
                        }

                        WindowFocusedSelectorMode::WindowInstance => {
                            if re.is_match(&event.window_instance) {
                                process_action(&action).await?;
                                break;
                            }
                        }
                        WindowFocusedSelectorMode::WindowClass => {
                            if re.is_match(&event.window_class) {
                                process_action(&action).await?;
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
                    tx.send(Message::ProfileChanged(h.new_profile_name))
                        .unwrap();

                    true
                },
            )?;

            let tx = dbus_event_tx.clone();
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

pub async fn run_main_loop(
    sysevents_rx: &Receiver<SystemEvent>,
    fsevents_rx: &Receiver<FileSystemEvent>,
    dbusevents_rx: &Receiver<dbus_client::Message>,
    ctrl_c_rx: &Receiver<bool>,
) -> Result<()> {
    trace!("Entering main loop...");

    let mut sel = Select::new();

    let ctrl_c = sel.recv(ctrl_c_rx);
    let fsevents = sel.recv(fsevents_rx);
    let dbusevents = sel.recv(dbusevents_rx);
    let sysevents = sel.recv(sysevents_rx);

    'MAIN_LOOP: loop {
        if QUIT.load(Ordering::SeqCst) {
            break 'MAIN_LOOP;
        }

        match sel.select_timeout(Duration::from_millis(constants::MAIN_LOOP_SLEEP_MILLIS)) {
            Ok(oper) => match oper.index() {
                i if i == ctrl_c => {
                    // consume the event, so that we don't cause a panic
                    let _event = &oper.recv(&ctrl_c_rx);
                    break 'MAIN_LOOP;
                }

                i if i == fsevents => {
                    let event = &oper.recv(&fsevents_rx);
                    if let Ok(event) = event {
                        process_fs_event(&event).await.unwrap_or_else(|e| {
                            error!("Could not process a filesystem event: {}", e)
                        })
                    } else {
                        error!("{}", event.as_ref().unwrap_err());
                    }
                }

                i if i == dbusevents => {
                    let event = &oper.recv(&dbusevents_rx);
                    if let Ok(event) = event {
                        process_dbus_event(&event)
                            .await
                            .unwrap_or_else(|e| error!("Could not process a D-Bus event: {}", e))
                    } else {
                        error!("{}", event.as_ref().unwrap_err());
                    }
                }

                i if i == sysevents => {
                    let event = &oper.recv(&sysevents_rx);
                    if let Ok(event) = event {
                        process_system_event(&event)
                            .await
                            .unwrap_or_else(|e| error!("Could not process a system event: {}", e));
                    } else {
                        error!("{}", event.as_ref().unwrap_err());
                    }
                }

                _ => unreachable!(),
            },

            Err(_) => {}
        }

        // poll all pollable sensors that do not notify us via messages
        for sensor in sensors::SENSORS.lock().iter_mut() {
            if sensor.is_pollable() {
                match sensor.poll() {
                    Ok(data) => {
                        if let Some(data) = data.as_any().downcast_ref::<sensors::X11SensorData>() {
                            process_window_event(&data).await?;
                        } else {
                            warn!("Unknown sensor data: {:#?}", data);
                        }
                    }

                    Err(e) => warn!("Could not poll a sensor: {}", e),
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
        .get_str("global.default_profile")
        .unwrap_or_else(|_| {
            dbus_client::get_active_profile()
                .unwrap_or_else(|_| constants::DEFAULT_PROFILE.to_string())
        });

    let selector = Selector::WindowFocused {
        mode: WindowFocusedSelectorMode::WindowInstance,
        regex: ".*".to_string(),
    };

    let mut metadata = RuleMetadata::default();
    metadata.internal = true;

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

#[tokio::main]
pub async fn main() -> std::result::Result<(), eyre::Error> {
    color_eyre::install()?;

    // if unsafe { libc::isatty(0) != 0 } {
    //     print_header();
    // }

    let opts = Options::parse();

    // enable logging if we are running as a daemon
    let daemon = match opts.command {
        Subcommands::Daemon => true,

        _ => false,
    };

    if unsafe { libc::isatty(0) == 0 } || daemon {
        // initialize logging
        if env::var("RUST_LOG").is_err() {
            env::set_var("RUST_LOG_OVERRIDE", "info");
            pretty_env_logger::init_custom_env("RUST_LOG_OVERRIDE");
        } else {
            pretty_env_logger::init();
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
    let q = QUIT.clone();
    ctrlc::set_handler(move || {
        q.store(true, Ordering::SeqCst);

        ctrl_c_tx
            .send(true)
            .unwrap_or_else(|e| error!("Could not send on a channel: {}", e));
    })
    .unwrap_or_else(|e| error!("Could not set CTRL-C handler: {}", e));

    // process configuration file
    let config_file = opts
        .config
        .unwrap_or_else(|| constants::PROCESS_MONITOR_CONFIG_FILE.to_string());

    let mut config = config::Config::default();
    if let Err(e) = config.merge(config::File::new(&config_file, config::FileFormat::Toml)) {
        warn!("Could not parse configuration file: {}", e);
    }

    *CONFIG.lock() = Some(config.clone());

    // enable support for experimental features?
    let enable_experimental_features = config
        .get::<bool>("global.enable_experimental_features")
        .unwrap_or(false);

    EXPERIMENTAL_FEATURES.store(enable_experimental_features, Ordering::SeqCst);

    if EXPERIMENTAL_FEATURES.load(Ordering::SeqCst) {
        warn!("** EXPERIMENTAL FEATURES are ENABLED, this may expose serious bugs! **");
    }

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

            let (fsevents_tx, fsevents_rx) = unbounded();
            register_filesystem_watcher(fsevents_tx, rules_file)?;

            let (dbusevents_tx, dbusevents_rx) = unbounded();
            spawn_dbus_thread(dbusevents_tx)?;

            // configure plugins
            let (sysevents_tx, sysevents_rx) = unbounded();
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
            run_main_loop(&sysevents_rx, &fsevents_rx, &dbusevents_rx, &ctrl_c_rx)
                .await
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
    }

    info!("Saving rules...");
    save_rules_map().unwrap_or_else(|e| error!("Could not save rules: {}", e));

    info!("Exiting now");

    Ok(())
}
