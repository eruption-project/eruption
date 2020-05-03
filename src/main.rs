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

// enable these if you want to compile-in the frontend (browser-based GUI)

//#![feature(plugin)]
//#![feature(proc_macro_hygiene, decl_macro)]
//#![feature(vec_into_raw_parts)]

use clap::{App, Arg};
use failure::Fail;
use hotwatch::{
    blocking::{Flow, Hotwatch},
    Event,
};
use lazy_static::lazy_static;
use log::*;
use parking_lot::{Condvar, Mutex};
use std::convert::TryInto;
use std::env;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicBool, AtomicIsize, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use std::u64;

mod util;

mod rvdevice;
use rvdevice::RvDeviceState;

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

#[cfg(feature = "frontend")]
mod frontend;

#[cfg(not(feature = "frontend"))]
#[allow(dead_code)]
mod frontend {
    // provide a dummy implementation of Message
    pub enum Message {}
}

lazy_static! {
    /// The currently active profile
    pub static ref ACTIVE_PROFILE: Arc<Mutex<Option<Profile>>> = Arc::new(Mutex::new(None));

    /// The current "pipeline" of scripts
    pub static ref ACTIVE_SCRIPTS: Arc<Mutex<Vec<Manifest>>> = Arc::new(Mutex::new(vec![]));

    /// Global configuration
    pub static ref CONFIG: Arc<Mutex<Option<config::Config>>> = Arc::new(Mutex::new(None));

    // Flags

    /// Global "quit" status flag
    pub static ref QUIT: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));

    // Color maps of Lua VMs ready?
    pub static ref COLOR_MAPS_READY_CONDITION: Arc<(Mutex<usize>, Condvar)> =
        Arc::new((Mutex::new(0), Condvar::new()));

    // All upcalls (event handlers) in Lua VM completed?
    pub static ref UPCALL_COMPLETED_ON_KEY_DOWN: Arc<(Mutex<usize>, Condvar)> =
        Arc::new((Mutex::new(0), Condvar::new()));
    pub static ref UPCALL_COMPLETED_ON_KEY_UP: Arc<(Mutex<usize>, Condvar)> =
        Arc::new((Mutex::new(0), Condvar::new()));

    // Other state

    /// Global "keyboard brightness" modifier
    pub static ref BRIGHTNESS: AtomicIsize = AtomicIsize::new(100);

    static ref LUA_TXS: Arc<Mutex<Vec<Sender<script::Message>>>> = Arc::new(Mutex::new(vec![]));
}

pub type Result<T> = std::result::Result<T, MainError>;

#[derive(Debug, Fail)]
pub enum MainError {
    #[fail(display = "Could not spawn a thread")]
    ThreadSpawnError {},

    #[fail(display = "Could not switch profiles")]
    SwitchProfileError {},

    #[fail(display = "Could not execute Lua script")]
    ScriptExecError {},
    // #[fail(display = "Unknown error: {}", description)]
    // UnknownError { description: String },
}

#[derive(Debug)]
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
fn parse_commandline<'a>() -> clap::ArgMatches<'a> {
    App::new("Eruption")
        .version(env!("CARGO_PKG_VERSION"))
        .author("X3n0m0rph59 <x3n0m0rph59@gmail.com>")
        .about("Linux user-mode driver for the ROCCAT Vulcan 100/12x series keyboards")
        .arg(
            Arg::with_name("config")
                .short("c")
                .long("config")
                .value_name("FILE")
                .help("Sets the configuration file to use")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("profile")
                .short("p")
                .long("profile")
                .value_name("profile")
                .help("Sets the profile to activate")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("scripts")
                .help("The Lua scripts to execute")
                .multiple(true)
                // .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .subcommand(App::new("list-scripts").about("Display a listing of all available scripts"))
        .subcommand(
            App::new("check-syntax")
                .about("Validate a Lua script for syntactical correctness")
                .arg(
                    Arg::with_name("script")
                        .help("The Lua script to check")
                        .required(true)
                        .index(1),
                ),
        )
        .get_matches()
}

/// Spawns the web-frontend thread
#[cfg(feature = "frontend")]
fn spawn_frontend_thread(
    frontend_tx: Sender<frontend::Message>,
    profile_path: PathBuf,
    script_paths: Vec<PathBuf>,
) -> plugins::Result<()> {
    let builder = thread::Builder::new().name("frontend".into());
    builder
        .spawn(move || {
            frontend::initialize(frontend_tx, profile_path, script_paths).unwrap_or_else(|e| {
                error!("Could not initialize the Web-Frontend: {}", e);
                panic!();
            });
        })
        .map_err(|_e| MainError::ThreadSpawnError {})?;

    Ok(())
}

#[derive(Debug, Clone)]
pub enum DbusApiEvent {
    ProfilesChanged,
    ActiveProfileChanged,
}

/// Spawns the dbus thread and executes it's main loop
#[cfg(feature = "dbus")]
fn spawn_dbus_thread(
    dbus_tx: Sender<dbus_interface::Message>,
) -> plugins::Result<Sender<DbusApiEvent>> {
    let (dbus_api_tx, dbus_api_rx) = channel();

    let builder = thread::Builder::new().name("dbus".into());
    builder
        .spawn(move || -> Result<()> {
            let dbus =
                dbus_interface::initialize(dbus_tx).map_err(|_e| MainError::ThreadSpawnError {})?;

            loop {
                // process events, destined for the dbus api
                match dbus_api_rx.recv_timeout(Duration::from_millis(0)) {
                    Ok(result) => match result {
                        DbusApiEvent::ProfilesChanged => dbus.notify_profiles_changed(),

                        DbusApiEvent::ActiveProfileChanged => dbus.notify_active_profile_changed(),
                    },

                    // ignore timeout errors
                    Err(std::sync::mpsc::RecvTimeoutError::Timeout) => (),

                    Err(e) => {
                        // print warning but continue
                        warn!("Channel error: {}", e);
                    }
                }

                dbus.get_next_event()
                    .unwrap_or_else(|e| error!("Could not get the next D-Bus event: {}", e));
            }
        })
        .map_err(|_e| MainError::ThreadSpawnError {})?;

    Ok(dbus_api_tx)
}

/// Spawns the keyboard events thread and executes it's main loop
fn spawn_input_thread(kbd_tx: Sender<Option<evdev_rs::InputEvent>>) -> plugins::Result<()> {
    let builder = thread::Builder::new().name("events".into());
    builder
        .spawn(move || {
            {
                // initialize thread local state of the keyboard plugin
                let mut plugin_manager = plugin_manager::PLUGIN_MANAGER.write();
                let keyboard_plugin = plugin_manager
                    .find_plugin_by_name_mut("Keyboard".to_string())
                    .unwrap_or_else(|| {
                        error!("Could not find a required plugin");
                        panic!()
                    })
                    .as_any_mut()
                    .downcast_mut::<plugins::KeyboardPlugin>()
                    .unwrap();

                keyboard_plugin
                    .initialize_thread_locals()
                    .unwrap_or_else(|e| {
                        error!("Could not initialize the keyboard plugin: {}", e);
                        panic!()
                    })
            }

            let plugin_manager = plugin_manager::PLUGIN_MANAGER.read();
            let keyboard_plugin = plugin_manager
                .find_plugin_by_name("Keyboard".to_string())
                .unwrap_or_else(|| {
                    error!("Could not find a required plugin");
                    panic!()
                })
                .as_any()
                .downcast_ref::<plugins::KeyboardPlugin>()
                .unwrap();

            loop {
                if let Ok(event) = keyboard_plugin.get_next_event() {
                    kbd_tx.send(event).unwrap_or_else(|e| {
                        error!("Could not send a keyboard event to the main thread: {}", e)
                    });
                } else {
                    // ignore spurious events
                    // error!("Could not get next keyboard event");
                }
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
    rvdevice: &RvDeviceState,
) -> Result<()> {
    let result = util::is_file_accessible(&script_path);
    if let Err(result) = result {
        error!(
            "Script file '{}' is not accessible: {}",
            script_path.display(),
            result
        );

        return Err(MainError::ScriptExecError {});
    }

    let result = util::is_file_accessible(util::get_manifest_for(&script_path));
    if let Err(result) = result {
        error!(
            "Manifest file for script '{}' is not accessible: {}",
            script_path.display(),
            result
        );

        return Err(MainError::ScriptExecError {});
    }

    let rvdevice = rvdevice.clone();

    let builder = thread::Builder::new().name(format!(
        "{}:{}",
        thread_idx,
        script_path.file_name().unwrap().to_string_lossy(),
    ));
    builder
        .spawn(move || -> Result<()> {
            loop {
                let rvdevice = rvdevice.clone();

                let result = script::run_script(script_path.clone(), rvdevice, &lua_rx)
                    .map_err(|_e| MainError::ScriptExecError {})?;

                match result {
                    //script::RunScriptResult::ReExecuteOtherScript(script_file) => {
                    //script_path = script_file;
                    //continue;
                    //}
                    script::RunScriptResult::TerminatedGracefully => break,
                }
            }

            Ok(())
        })
        .map_err(|_e| MainError::ThreadSpawnError {})?;

    Ok(())
}

/// Switches the currently active profile to the profile file `profile_path`
fn switch_profile<P: AsRef<Path>>(
    profile_file: P,
    rvdevice: &RvDeviceState,
    #[cfg(feature = "dbus")] dbus_api_tx: &Sender<DbusApiEvent>,
) -> Result<()> {
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
    let profile =
        profiles::Profile::from(&profile_path).map_err(|_e| MainError::SwitchProfileError {})?;

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
            return Err(MainError::SwitchProfileError {});
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

        let (lua_tx, lua_rx) = channel();
        spawn_lua_thread(thread_idx, lua_rx, script_path.clone(), &rvdevice).unwrap_or_else(|e| {
            error!("Could not spawn a thread: {}", e);
        });

        lua_txs.push(lua_tx);
    }

    // finally assign the globally active profile
    *ACTIVE_PROFILE.lock() = Some(profile);

    #[cfg(feature = "dbus")]
    dbus_api_tx
        .send(DbusApiEvent::ActiveProfileChanged)
        .unwrap_or_else(|e| error!("Could not send a pending dbus API event: {}", e));

    Ok(())
}

#[allow(clippy::cognitive_complexity)]
fn run_main_loop(
    rvdevice: &mut RvDeviceState,
    #[cfg(feature = "dbus")] dbus_api_tx: &Sender<DbusApiEvent>,
    #[cfg(feature = "frontend")] frontend_rx: &Receiver<frontend::Message>,
    dbus_rx: &Receiver<dbus_interface::Message>,
    kbd_rx: &Receiver<Option<evdev_rs::InputEvent>>,
    fsevents_rx: &Receiver<FileSystemEvent>,
) {
    trace!("Entering main loop...");

    events::notify_observers(events::Event::DaemonStartup).unwrap();

    // main loop iterations, monotonic counter
    let mut ticks = 0;

    // used to calculate frames per second
    let mut fps_cntr = 0;
    let mut fps_timer = Instant::now();

    let mut start_time = Instant::now();

    // enter the main loop on the main thread
    'MAIN_LOOP: loop {
        // prepare to call main loop hook
        let plugin_manager = plugin_manager::PLUGIN_MANAGER.read();
        let plugins = plugin_manager.get_plugins();

        // call main loop hook of each registered plugin
        for plugin in plugins.iter() {
            plugin.main_loop_hook(ticks);
        }

        // process file system related events
        match fsevents_rx.recv_timeout(Duration::from_millis(0)) {
            Ok(result) => match result {
                FileSystemEvent::ProfilesChanged => {
                    events::notify_observers(events::Event::FileSystemEvent(
                        FileSystemEvent::ProfilesChanged,
                    ))
                    .unwrap_or_else(|e| error!("{}", e));

                    #[cfg(feature = "dbus")]
                    dbus_api_tx
                        .send(DbusApiEvent::ProfilesChanged)
                        .unwrap_or_else(|e| {
                            error!("Could not send a pending dbus API event: {}", e)
                        });
                }
                FileSystemEvent::ScriptsChanged => {}
            },

            // ignore timeout errors
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => (),

            Err(e) => {
                // print warning but continue
                warn!("Channel error: {}", e);
            }
        }

        // process Web-Frontend events
        #[cfg(feature = "frontend")]
        match frontend_rx.recv_timeout(Duration::from_millis(0)) {
            Ok(result) => match result {
                frontend::Message::SwitchProfile(profile_path) => {
                    info!("Loading Profile: {}", profile_path.display());

                    switch_profile(&profile_path, &rvdevice, &dbus_api_tx)
                        .unwrap_or_else(|e| error!("Could not switch profiles: {}", e));
                }
            },

            // ignore timeout errors
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => (),

            Err(e) => {
                error!("Channel error: {}", e);
                break 'MAIN_LOOP;
            }
        }

        // process D-Bus events
        #[cfg(feature = "dbus")]
        match dbus_rx.recv_timeout(Duration::from_millis(0)) {
            Ok(result) => match result {
                dbus_interface::Message::SwitchProfile(profile_path) => {
                    info!("Loading Profile: {}", profile_path.display());

                    switch_profile(&profile_path, &rvdevice, &dbus_api_tx)
                        .unwrap_or_else(|e| error!("Could not switch profiles: {}", e));
                }
            },

            // ignore timeout errors
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => (),

            Err(e) => {
                error!("Channel error: {}", e);
                break 'MAIN_LOOP;
            }
        }

        // execute render "pipeline" now

        // send timer tick events to the Lua VMs
        for lua_tx in LUA_TXS.lock().iter() {
            lua_tx
                .send(script::Message::Tick(
                    start_time.elapsed().as_millis().try_into().unwrap(),
                ))
                .unwrap_or_else(|e| error!("Send error: {}", e));
        }

        let mut drop_frame = false;

        // execute render "pipeline" now
        // first, clear the canvas
        script::LED_MAP.lock().copy_from_slice(
            &[rvdevice::RGBA {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            }; rvdevice::NUM_KEYS],
        );

        // instruct Lua VMs to realize their color maps, e.g. to blend their
        // local color maps with the canvas
        *COLOR_MAPS_READY_CONDITION.0.lock() = LUA_TXS.lock().len();

        for lua_tx in LUA_TXS.lock().iter() {
            // guarantee the right order of execution for the alpha blend
            // operations, so we have to wait for the current Lua VM to
            // complete its blending code, before continuing
            let mut pending = COLOR_MAPS_READY_CONDITION.0.lock();

            lua_tx
                .send(script::Message::RealizeColorMap)
                .unwrap_or_else(|e| error!("Send error: {}", e));

            // yield to thread
            //thread::sleep(Duration::from_millis(0));

            let result = COLOR_MAPS_READY_CONDITION
                .1
                .wait_for(&mut pending, Duration::from_millis(50));

            if result.timed_out() {
                drop_frame = true;
                warn!("Frame dropped: Timeout while waiting for a lock!");
                break;
            }
        }

        // yield main thread
        //thread::sleep(Duration::from_millis(0));
        // number of pending blend ops should have reached zero by now
        //assert!(*COLOR_MAPS_READY_CONDITION.0.lock() == 0);
        // send the final (combined) color map to the keyboard
        if !drop_frame {
            rvdevice
                .send_led_map(&script::LED_MAP.lock())
                .unwrap_or_else(|e| error!("Could not send led map to the device: {}", e));
        }

        // sync to MAIN_LOOP_DELAY_MILLIS iteration time
        let elapsed: u64 = start_time.elapsed().as_millis().try_into().unwrap();
        let sleep_millis = u64::min(
            constants::MAIN_LOOP_DELAY_MILLIS.saturating_sub(elapsed),
            constants::MAIN_LOOP_DELAY_MILLIS,
        );

        // send pending keyboard events to the Lua VMs and to the event dispatcher
        match kbd_rx.recv_timeout(Duration::from_millis(sleep_millis)) {
            Ok(result) => match result {
                Some(raw_event) => {
                    // notify all observers of raw events
                    events::notify_observers(events::Event::RawKeyboardEvent(raw_event.clone()))
                        .unwrap();

                    if let evdev_rs::enums::EventCode::EV_KEY(ref code) = raw_event.event_code {
                        let is_pressed = raw_event.value > 0;
                        let index = util::ev_key_to_key_index(code.clone());

                        trace!("Key index: {:#x}", index);

                        if is_pressed {
                            *UPCALL_COMPLETED_ON_KEY_DOWN.0.lock() = LUA_TXS.lock().len();

                            for lua_tx in LUA_TXS.lock().iter() {
                                lua_tx
                                    .send(script::Message::KeyDown(index))
                                    .unwrap_or_else(|e| {
                                        error!("Could not send a pending keyboard event: {}", e)
                                    });
                            }

                            // yield to thread
                            //thread::sleep(Duration::from_millis(0));

                            // wait until all Lua VMs completed the event handler
                            loop {
                                let mut pending = UPCALL_COMPLETED_ON_KEY_DOWN.0.lock();

                                UPCALL_COMPLETED_ON_KEY_DOWN
                                    .1
                                    .wait_for(&mut pending, Duration::from_millis(50));

                                if *pending == 0 {
                                    break;
                                }
                            }

                            events::notify_observers(events::Event::KeyDown(index))
                                .unwrap_or_else(|e| error!("{}", e));
                        } else {
                            *UPCALL_COMPLETED_ON_KEY_UP.0.lock() = LUA_TXS.lock().len();

                            for lua_tx in LUA_TXS.lock().iter() {
                                lua_tx
                                    .send(script::Message::KeyUp(index))
                                    .unwrap_or_else(|e| {
                                        error!("Could not send a pending keyboard event: {}", e)
                                    });
                            }

                            // yield to thread
                            //thread::sleep(Duration::from_millis(0));

                            // wait until all Lua VMs completed the event handler
                            loop {
                                let mut pending = UPCALL_COMPLETED_ON_KEY_UP.0.lock();

                                UPCALL_COMPLETED_ON_KEY_UP
                                    .1
                                    .wait_for(&mut pending, Duration::from_millis(50));

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
                        .unwrap_or_else(|e| {
                            error!("Could not send a pending keyboard event: {}", e)
                        });
                }

                // ignore spurious events
                None => trace!("Spurious keyboard event ignored"),
            },

            // ignore timeout errors
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => (),

            Err(e) => {
                error!("Channel error: {}", e);
                break 'MAIN_LOOP;
            }
        }

        let elapsed_after_sleep = start_time.elapsed().as_millis();
        if elapsed_after_sleep != constants::MAIN_LOOP_DELAY_MILLIS.into() {
            if elapsed_after_sleep > (constants::MAIN_LOOP_DELAY_MILLIS + 15u64).into() {
                warn!("More than 15 milliseconds of jitter detected!");
                warn!(
                    "Loop took: {} milliseconds, goal: {}",
                    elapsed_after_sleep,
                    constants::MAIN_LOOP_DELAY_MILLIS
                );
            } else {
                trace!(
                    "Loop took: {} milliseconds, goal: {}",
                    elapsed_after_sleep,
                    constants::MAIN_LOOP_DELAY_MILLIS
                );
            }
        }

        // calculate and log fps each second
        if fps_timer.elapsed().as_millis() >= 1000 {
            debug!("FPS: {}", fps_cntr);

            fps_timer = Instant::now();
            fps_cntr = 0;
        }

        // shall we quit the main loop?
        if QUIT.load(Ordering::SeqCst) {
            break 'MAIN_LOOP;
        }

        fps_cntr += 1;
        ticks += 1;

        start_time = Instant::now();
    }

    events::notify_observers(events::Event::DaemonShutdown).unwrap();
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
        )
        .map_err(|_e| MainError::ThreadSpawnError {})?;

    Ok(())
}

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
            })
            .map_err(|_e| crate::MainError::ThreadSpawnError {})?;

        Ok(())
    }
}

/// Main program entrypoint
#[allow(clippy::cognitive_complexity)]
fn main() {
    if unsafe { libc::isatty(0) != 0 } {
        print_header();
    }

    // start the thread deadlock detector
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
        "Starting user-mode driver for ROCCAT Vulcan 100/12x series keyboards: Version {}",
        env!("CARGO_PKG_VERSION")
    );

    // register ctrl-c handler
    let q = QUIT.clone();
    ctrlc::set_handler(move || {
        q.store(true, Ordering::SeqCst);
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

    // load and initialize global runtime state
    debug!("Loading saved state...");
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

    // active runtime profile
    let default_profile_name = config
        .get_str("global.profile")
        .unwrap_or_else(|_| "default".into());
    let profile_name = matches.value_of("profile").unwrap_or(&default_profile_name);
    let mut profile_file = PathBuf::from(&profile_name);
    profile_file.set_extension("profile");

    let profile_file = PathBuf::from(&profile_dir).join(&profile_file);

    // try to load saved profile state
    let state = state::STATE.read();
    let saved_profile = state
        .as_ref()
        .unwrap()
        .get("profile")
        .unwrap_or_else(|_| profile_file);

    let profile_file = PathBuf::from(&profile_dir).join(saved_profile);

    // finally, load the profile
    trace!("Loading profile data from '{}'", profile_file.display());
    let profile = Profile::from(&profile_file).unwrap_or_else(|e| {
        warn!(
            "Error opening the profile file '{}': {}",
            profile_file.display(),
            e
        );

        Profile::default()
    });

    info!("Loaded profile: {}", &profile.name);

    // active sript files
    //let default_script_files: Vec<PathBuf> = config
    //.get_array("global.script_files")
    //.unwrap_or_else(|_| vec![constants::DEFAULT_EFFECT_SCRIPT.into()])
    //.iter()
    //.map(|v| PathBuf::from(v.clone().into_str().unwrap()))
    //.collect();

    let script_files = if profile.active_scripts.is_empty() {
        matches
            .values_of("scripts")
            .unwrap_or_default()
            .map(|p| PathBuf::from(p.to_owned()))
            .collect::<Vec<PathBuf>>()
    } else {
        profile.active_scripts.clone()
    };

    let script_paths: Vec<PathBuf> = script_files
        .iter()
        .map(|p| PathBuf::from(&script_dir).join(p))
        .collect();

    *ACTIVE_PROFILE.lock() = Some(profile);

    // frontend enable
    let frontend_enabled = config
        .get::<bool>("frontend.enabled")
        .unwrap_or_else(|_| true);

    // others
    let _verbosity = matches.occurrences_of("v");

    // try to set timer slack to a low value
    // prctl::set_timer_slack(1).unwrap_or_else(|e| warn!("Could not set process timer slack: {}", e));

    // request realtime priority
    // crate::util::set_process_priority();

    // create the one and only hidapi instance
    match hidapi::HidApi::new() {
        Ok(hidapi) => {
            match RvDeviceState::enumerate_devices(&hidapi) {
                Ok(mut rvdevice) => {
                    // open the control and led devices
                    info!("Opening devices...");
                    rvdevice
                    .open(&hidapi)
                    .unwrap_or_else(|e| {
                        error!("Error opening the keyboard device: {}", e);
                        error!("This could be a permission problem, or maybe the device is locked by another process?");
                        process::exit(3);
                    });

                    // send initialization handshake
                    info!("Initializing devices...");
                    rvdevice
                        .send_init_sequence()
                        .unwrap_or_else(|e| error!("Could not initialize the device: {}", e));

                    // set leds to a known initial state
                    info!("Configuring LEDs...");
                    rvdevice
                        .set_led_init_pattern()
                        .unwrap_or_else(|e| error!("Could not initialize LEDs: {}", e));

                    // initialize the D-Bus API
                    #[cfg(feature = "dbus")]
                    info!("Initializing D-Bus API...");

                    let (dbus_tx, dbus_rx) = channel();
                    #[cfg(feature = "dbus")]
                    let dbus_api_tx = spawn_dbus_thread(dbus_tx).unwrap_or_else(|e| {
                        error!("Could not spawn a thread: {}", e);
                        panic!()
                    });

                    // initialize plugins
                    info!("Registering plugins...");
                    plugins::register_plugins()
                        .unwrap_or_else(|_e| error!("Could not register one or more plugins"));

                    // spawn a thread to handle keyboard input
                    info!("Spawning input thread...");

                    let (kbd_tx, kbd_rx) = channel();
                    spawn_input_thread(kbd_tx).unwrap_or_else(|e| {
                        error!("Could not spawn a thread: {}", e);
                        panic!()
                    });

                    // spawn Lua VM threads
                    info!("Loading Lua scripts...");

                    for (thread_idx, script_path) in script_paths.iter().enumerate() {
                        let script_path = script_path.clone();

                        let (lua_tx, lua_rx) = channel();
                        let result =
                            spawn_lua_thread(thread_idx, lua_rx, script_path.clone(), &rvdevice);

                        if result.is_err() {
                            error!("Could not spawn a Lua VM thread");
                        } else {
                            LUA_TXS.lock().push(lua_tx);
                        }
                    }

                    // spawn a thread to handle the web-frontend
                    #[cfg(feature = "frontend")]
                    let (frontend_tx, frontend_rx) = channel();

                    if frontend_enabled {
                        #[cfg(feature = "frontend")]
                        info!("Spawning Web-Frontend thread...");

                        #[cfg(feature = "frontend")]
                        spawn_frontend_thread(frontend_tx, profile_path.clone(), script_paths)
                            .unwrap_or_else(|e| {
                                error!("Could not spawn a thread: {}", e);
                                panic!()
                            });
                    } else {
                        info!("Web-Frontend DISABLED by configuration");
                    }

                    let (fsevents_tx, fsevents_rx) = channel();
                    register_filesystem_watcher(
                        fsevents_tx,
                        PathBuf::from(&config_file),
                        profile_path,
                        PathBuf::from(&script_dir),
                    )
                    .unwrap_or_else(|e| error!("Could not register file changes watcher: {}", e));

                    // enter the main loop
                    run_main_loop(
                        &mut rvdevice,
                        #[cfg(feature = "dbus")]
                        &dbus_api_tx,
                        #[cfg(feature = "frontend")]
                        &frontend_rx,
                        &dbus_rx,
                        &kbd_rx,
                        &fsevents_rx,
                    );

                    // we left the main loop, so send a final message to the running Lua VMs
                    for lua_tx in LUA_TXS.lock().iter() {
                        lua_tx
                            .send(script::Message::Quit(0))
                            .unwrap_or_else(|e| error!("Could not send quit message: {}", e));
                    }

                    // TODO: Ugly hack, find a better way to wait for exit of the Lua VMs
                    thread::sleep(Duration::from_millis(250));

                    // close the control and LED devices
                    info!("Closing devices...");
                    rvdevice.close_all().unwrap_or_else(|e| {
                        warn!("Could not close the keyboard device: {}", e);
                    });
                }

                Err(_) => {
                    error!("Could not enumerate system HID devices");
                    process::exit(2);
                }
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
}
