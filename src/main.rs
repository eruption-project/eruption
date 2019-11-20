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

#![feature(plugin)]
#![feature(proc_macro_hygiene, decl_macro)]

use clap::{App, Arg};
use failure::Fail;
use lazy_static::lazy_static;
use log::*;
use pretty_env_logger;
use std::convert::TryInto;
use std::env;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};
use std::u64;

mod util;

use hidapi;

mod rvdevice;
use rvdevice::RvDeviceState;

mod constants;
mod dbus_interface;
mod plugin_manager;
mod plugins;
mod profiles;
mod runtime_state;
mod scripting;

use profiles::Profile;
use runtime_state::RuntimeState;
use scripting::script;

#[cfg(feature = "frontend")]
mod frontend;

#[cfg(not(feature = "frontend"))]
mod frontend {
    // provide a dummy implementation of Message
    pub enum Message {}
}

lazy_static! {
    pub static ref GLOBALS: Arc<RwLock<RuntimeState>> =
        Arc::new(RwLock::new(RuntimeState::default()));
    pub static ref CONFIG: Arc<RwLock<Option<config::Config>>> = Arc::new(RwLock::new(None));
    pub static ref QUIT: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

pub type Result<T> = std::result::Result<T, MainError>;

#[derive(Debug, Fail)]
pub enum MainError {
    #[fail(display = "Could not execute Lua script")]
    ScriptExecError {},
    // #[fail(display = "Unknown error: {}", description)]
    // UnknownError { description: String },
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
        .version("0.0.9")
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
            Arg::with_name("script")
                .help("The Lua script to execute")
                // .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .multiple(true)
                .help("Sets the level of verbosity"),
        )
        .subcommand(
            App::new("list-effects").about("Display a listing of all available effects scripts"),
        )
        .subcommand(
            App::new("check-syntax")
                .about("Validate a Lua effect script for syntactical correctness")
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
    script_path: PathBuf,
) -> plugins::Result<()> {
    let builder = thread::Builder::new().name("frontend".into());
    builder
        .spawn(move || {
            frontend::initialize(frontend_tx, profile_path, script_path).unwrap_or_else(|e| {
                error!("Could not initialize the Web-Frontend: {}", e);
                panic!()
            });
        })
        .unwrap_or_else(|e| {
            error!("Could not spawn a thread: {}", e);
            panic!()
        });

    Ok(())
}

/// Spawns the dbus thread and executes it's main loop
fn spawn_dbus_thread(dbus_tx: Sender<dbus_interface::Message>) -> plugins::Result<()> {
    let builder = thread::Builder::new().name("dbus".into());
    builder
        .spawn(move || {
            let dbus = dbus_interface::initialize(dbus_tx).unwrap_or_else(|e| {
                error!("Could not initialize D-Bus: {}", e);
                panic!()
            });

            loop {
                dbus.get_next_event()
                    .unwrap_or_else(|e| error!("Could not get the next D-Bus event: {}", e));
            }
        })
        .unwrap_or_else(|e| {
            error!("Could not spawn a thread: {}", e);
            panic!()
        });

    Ok(())
}

/// Spawns the keyboard events thread and executes it's main loop
fn spawn_input_thread(kbd_tx: Sender<Option<u8>>) -> plugins::Result<()> {
    let builder = thread::Builder::new().name("events".into());
    builder
        .spawn(move || {
            {
                // initialize thread local state of the keyboard plugin
                let mut plugin_manager = plugin_manager::PLUGIN_MANAGER.write().unwrap();
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

            let plugin_manager = plugin_manager::PLUGIN_MANAGER.read().unwrap();
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
    lua_rx: Receiver<script::Message>,
    mut script_path: PathBuf,
    rvdevice: &RvDeviceState,
) -> Result<()> {
    let result = util::is_file_accessible(&script_path);
    if let Err(result) = result {
        error!(
            "Script file '{}' is not accessible: {}",
            script_path.display(),
            result
        );
        process::exit(3);
    }

    let result = util::is_file_accessible(util::get_manifest_for(&script_path));
    if let Err(result) = result {
        error!(
            "Manifest file for script '{}' is not accessible: {}",
            script_path.display(),
            result
        );
        process::exit(3);
    }

    let rvdevice = rvdevice.clone();

    let builder = thread::Builder::new().name("lua-vm".into());
    builder
        .spawn(move || loop {
            let rvdevice = rvdevice.clone();

            let result =
                script::run_script(script_path.clone(), rvdevice, &lua_rx).unwrap_or_else(|e| {
                    error!(
                        "Could not load script file '{}': {}",
                        script_path.display(),
                        e
                    );
                    panic!();
                });

            match result {
                script::RunScriptResult::ReExecuteOtherScript(script_file) => {
                    script_path = script_file;
                    continue;
                }
            }
        })
        .unwrap_or_else(|e| {
            error!("Could not spawn a thread: {}", e);
            panic!();
        });

    Ok(())
}

#[allow(clippy::cognitive_complexity)]
fn run_main_loop(
    frontend_rx: &Receiver<frontend::Message>,
    dbus_rx: &Receiver<dbus_interface::Message>,
    kbd_rx: &Receiver<Option<u8>>,
    lua_tx: &Sender<script::Message>,
) {
    trace!("Entering main loop...");

    // main loop iterations, monotonic counter
    let mut ticks = 0;

    // used to calculate frames per second
    let mut fps_cntr = 0;
    let mut fps_timer = Instant::now();

    let mut start_time = Instant::now();

    // enter the main loop on the main thread
    'MAIN_LOOP: loop {
        // prepare to call main loop hook
        let plugin_manager = plugin_manager::PLUGIN_MANAGER.read().unwrap();
        let plugins = plugin_manager.get_plugins();

        // call main loop hook of each registered plugin
        for plugin in plugins.iter() {
            plugin.main_loop_hook(ticks);
        }

        // process Web-Frontend events
        #[cfg(feature = "frontend")]
        match frontend_rx.recv_timeout(Duration::from_millis(0)) {
            Ok(result) => match result {
                frontend::Message::LoadScript(script_path) => {
                    info!("Loading Script: {}", script_path.display());

                    if util::is_script_file_accessible(&script_path)
                        && util::is_manifest_file_accessible(&script_path)
                    {
                        lua_tx
                            .send(script::Message::LoadScript(script_path))
                            .unwrap_or_else(|e| {
                                error!("Could not send an event to the Lua VM: {}", e)
                            });
                    } else {
                        error!(
                            "Script and/or manifest file '{}' is not accessible: ",
                            script_path.display()
                        );
                    }
                }

                frontend::Message::SwitchProfile(profile_path) => {
                    info!("Loading Profile: {}", profile_path.display());

                    let profile = profiles::Profile::from(&profile_path).unwrap();

                    let mut globals = GLOBALS.write().unwrap();
                    globals.active_profile = Some(profile);

                    let script_path = &globals.active_script.as_ref().unwrap().script_file;

                    if util::is_script_file_accessible(&script_path)
                        && util::is_manifest_file_accessible(&script_path)
                    {
                        lua_tx
                            .send(script::Message::LoadScript(script_path.to_path_buf()))
                            .unwrap_or_else(|e| {
                                error!("Could not send an event to the Lua VM: {}", e)
                            });
                    } else {
                        error!(
                            "Script and/or manifest file '{}' is not accessible: ",
                            script_path.display()
                        );
                    }
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
                dbus_interface::Message::LoadScript(script_path) => {
                    info!("Loading Script: {}", script_path.display());

                    match util::is_file_accessible(&script_path) {
                        Ok(_) => lua_tx
                            .send(script::Message::LoadScript(script_path))
                            .unwrap_or_else(|e| {
                                error!("Could not send an event to the Lua VM: {}", e)
                            }),

                        Err(e) => error!(
                            "Script file '{}' is not accessible: {}",
                            script_path.display(),
                            e
                        ),
                    }
                }
            },

            // ignore timeout errors
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => (),

            Err(e) => {
                error!("Channel error: {}", e);
                break 'MAIN_LOOP;
            }
        }

        // send pending keyboard events to the Lua VM
        match kbd_rx.recv_timeout(Duration::from_millis(0)) {
            Ok(result) => match result {
                Some(index) => lua_tx
                    .send(script::Message::KeyDown(index))
                    .unwrap_or_else(|e| error!("Could not send a pending keyboard event: {}", e)),

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

        // send a timer tick event to the Lua VM
        lua_tx
            .send(script::Message::Tick(
                start_time.elapsed().as_millis().try_into().unwrap(),
            ))
            .unwrap();

        // sync to MAIN_LOOP_DELAY_MILLIS iteration time
        let elapsed: u64 = start_time.elapsed().as_millis().try_into().unwrap();
        let sleep_millis = u64::max(
            constants::MAIN_LOOP_DELAY_MILLIS.saturating_sub(elapsed),
            constants::MAIN_LOOP_DELAY_MILLIS,
        );
        thread::sleep(Duration::from_millis(sleep_millis));

        let elapsed_after_sleep = start_time.elapsed().as_millis();
        if elapsed_after_sleep != constants::MAIN_LOOP_DELAY_MILLIS.into() {
            if elapsed_after_sleep > (constants::MAIN_LOOP_DELAY_MILLIS + 5u64).into() {
                warn!("More than 5 milliseconds of jitter detected!");
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
            trace!("FPS: {}", fps_cntr);

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
}

/// Main program entrypoint
fn main() {
    if unsafe { libc::isatty(0) != 0 } {
        print_header();
    }

    let matches = parse_commandline();

    // initialize logging
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG_OVERRIDE", "info");
        pretty_env_logger::init_custom_env("RUST_LOG_OVERRIDE");
    } else {
        pretty_env_logger::init();
    }

    info!("Starting user-mode driver for ROCCAT Vulcan 100/12x series devices");

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
            panic!()
        });

    *CONFIG.write().unwrap() = Some(config.clone());

    // default directories
    let profile_dir = config
        .get_str("global.profile_dir")
        .unwrap_or_else(|_| constants::DEFAULT_PROFILE_DIR.to_string());
    let profile_path = PathBuf::from(&profile_dir);

    let script_dir = config
        .get_str("global.script_dir")
        .unwrap_or_else(|_| constants::DEFAULT_SCRIPT_DIR.to_string());

    // active sript file
    let default_script_file = config
        .get_str("global.script_file")
        .unwrap_or_else(|_| constants::DEFAULT_EFFECT_SCRIPT.to_string());
    let script_file = matches.value_of("script").unwrap_or(&default_script_file);
    let script_path = PathBuf::from(&script_dir).join(Path::new(&script_file));

    // active runtime profile
    let default_profile_name = config
        .get_str("global.profile")
        .unwrap_or_else(|_| "default".into());
    let profile_name = matches.value_of("profile").unwrap_or(&default_profile_name);
    let mut profile_file = PathBuf::from(&profile_name);
    profile_file.set_extension("profile");

    let profile_file = PathBuf::from(&profile_dir).join(&profile_file);

    // load profile
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

    // set globals
    {
        let mut globals = GLOBALS.write().unwrap();
        globals.active_profile = Some(profile);
    }

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
                        process::exit(4);
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
                    spawn_dbus_thread(dbus_tx).unwrap_or_else(|e| {
                        error!("Could not spawn a thread: {}", e);
                        panic!()
                    });

                    // initialize plugins
                    info!("Registering plugins...");
                    plugins::register_plugins()
                        .unwrap_or_else(|_e| error!("Could not register plugin"));

                    // spawn a thread to handle keyboard input
                    info!("Spawning input thread...");

                    let (kbd_tx, kbd_rx) = channel();
                    spawn_input_thread(kbd_tx).unwrap_or_else(|e| {
                        error!("Could not spawn a thread: {}", e);
                        panic!()
                    });

                    // spawn the Lua VM thread, and then load and execute a script
                    info!("Loading Lua script...");

                    let script_path_clone = script_path.clone();

                    let (lua_tx, lua_rx) = channel();
                    spawn_lua_thread(lua_rx, script_path_clone, &rvdevice).unwrap_or_else(|e| {
                        error!("Could not spawn a thread: {}", e);
                        panic!()
                    });

                    // spawn a thread to handle the web-frontend
                    let (frontend_tx, frontend_rx) = channel();

                    if frontend_enabled {
                        #[cfg(feature = "frontend")]
                        info!("Spawning Web-Frontend thread...");

                        #[cfg(feature = "frontend")]
                        spawn_frontend_thread(frontend_tx, profile_path, script_path)
                            .unwrap_or_else(|e| {
                                error!("Could not spawn a thread: {}", e);
                                panic!()
                            });
                    } else {
                        info!("Web-Frontend DISABLED by configuration");
                    }

                    // enter the main loop
                    run_main_loop(&frontend_rx, &dbus_rx, &kbd_rx, &lua_tx);

                    // we left the main loop, so send a final message to the running Lua VM
                    lua_tx
                        .send(script::Message::Quit(0))
                        .unwrap_or_else(|e| error!("Could not send quit message: {}", e));

                    // TODO: Ugly hack, find a better way to wait for exit of the Lua VM
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

    info!("Exiting now");
}
