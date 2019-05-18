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

use lazy_static::lazy_static;
use log::*;
use pretty_env_logger;
use std::convert::TryInto;
use std::error::Error;
use std::process;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

mod util;

use hidapi;

mod rvdevice;
use rvdevice::RvDeviceState;

mod plugin_manager;
mod plugins;
mod scripting;
mod errors;

lazy_static! {
    pub static ref QUIT: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

/// Main program entrypoint
fn main() {
    pretty_env_logger::init();

    info!("Starting user-mode driver for ROCCAT Vulcan 100/12x series devices");

    let q = QUIT.clone();
    ctrlc::set_handler(move || {
        q.store(true, Ordering::SeqCst);
    })
    .unwrap_or_else(|e| error!("Could not set CTRL-C handler: {}", e));

    // create the one and only hidapi instance
    match hidapi::HidApi::new() {
        Ok(hidapi) => match RvDeviceState::enumerate_devices(&hidapi) {
            Ok(mut rvdevice) => {
                // open the control and led device
                info!("Opening devices...");
                rvdevice
                    .open(&hidapi)
                    .unwrap_or_else(|e| error!("{}", e.description()));

                // send initialization handshake
                info!("Initializing devices...");
                rvdevice.send_init_sequence().unwrap_or_else(|e| {
                    error!("Could not initialize the device: {}", e.description())
                });

                // set leds to a known initial state
                info!("Configuring LEDs...");
                rvdevice
                    .set_led_init_pattern()
                    .unwrap_or_else(|e| error!("Could not initialize LEDs: {}", e.description()));

                // initialize plugins
                // info!("Registering plugins...");
                plugins::register_plugins()
                    .unwrap_or_else(|_e| error!("Could not register plugin"));

                // spawn a thread for the Lua VM, and then load and execute a script
                info!("Loading Lua scripts...");
                let (lua_tx, lua_rx) = channel();
                thread::spawn(move || {
                    scripting::run_scripts(rvdevice, &lua_rx)
                        .unwrap_or_else(|e| error!("Could not load script: {}", e));
                });

                // spawn a thread to handle keyboard input
                info!("Spawning input thread...");
                let (kbd_tx, kbd_rx) = channel();
                thread::spawn(move || {
                    {
                        // initialize thread local state of the keyboard plugin
                        let mut plugin_manager = plugin_manager::PLUGIN_MANAGER.write().unwrap();
                        let mut keyboard_plugin = plugin_manager
                            .find_plugin_by_name_mut("Keyboard".to_string())
                            .as_any_mut()
                            .downcast_mut::<plugins::KeyboardPlugin>()
                            .unwrap();

                        keyboard_plugin.initialize_thread_locals();
                    }

                    loop {
                        let mut plugin_manager = plugin_manager::PLUGIN_MANAGER.read().unwrap();
                        let mut keyboard_plugin = plugin_manager
                            .find_plugin_by_name("Keyboard".to_string())
                            .as_any()
                            .downcast_ref::<plugins::KeyboardPlugin>()
                            .unwrap();

                        match keyboard_plugin.get_next_event() {
                            Ok(event) => kbd_tx.send(event).unwrap(),
                            _ => (),
                        }
                    }
                });

                lua_tx.send(scripting::Message::Startup).unwrap();

                // trace!("Entering main loop...");

                // let mut cntr = 0;
                let mut start_time = Instant::now();

                // enter the main loop on the main thread
                'MAIN_LOOP: loop {
                    let quit = QUIT.load(Ordering::SeqCst);

                    let plugin_manager = plugin_manager::PLUGIN_MANAGER.read().unwrap();
                    let plugins = plugin_manager.get_plugins();

                    for plugin in plugins.iter() {
                        plugin.main_loop_hook();
                    }

                    lua_tx
                        .send(scripting::Message::Tick(
                            start_time.elapsed().as_millis().try_into().unwrap(),
                        ))
                        .unwrap();

                    match kbd_rx.recv_timeout(Duration::from_millis(1)) {
                        Ok(index) => lua_tx.send(scripting::Message::KeyDown(index)).unwrap(),

                        // ignore timeout errors
                        Err(std::sync::mpsc::RecvTimeoutError::Timeout) => (),

                        Err(e) => error!("{}", e.description()),
                    }

                    thread::sleep(Duration::from_millis(19));
                    // thread::yield_now();

                    if quit {
                        break 'MAIN_LOOP;
                    }

                    // trace!("Loop time: {} millis", start_time.elapsed().as_millis());

                    // cntr += 1;
                    start_time = Instant::now();
                }

                // we left the main loop, send a final message to the running Lua VM
                lua_tx.send(scripting::Message::Quit(0)).unwrap();

                // TODO: Ugly hack, find a better way to wait for exit of Lua VM
                thread::sleep(Duration::from_millis(250));
            }

            Err(_) => {
                error!("Could not enumerate system HID devices");
                process::exit(2);
            }
        },

        Err(_) => {
            error!("Could not open HIDAPI");
            process::exit(1);
        }
    }

    info!("Exiting now");
}
