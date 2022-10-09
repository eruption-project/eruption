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

use clap::CommandFactory;
use clap::Parser;
use clap_complete::Shell;
use colored::*;
use evdev_rs::{Device, DeviceWrapper, GrabMode};
use flume::{unbounded, Sender};
use hwdevices::{EvdevError, HwDevice, KeyboardHidEvent, RGBA};
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};
use lazy_static::lazy_static;
use log::*;
use parking_lot::Mutex;
use rust_embed::RustEmbed;
use std::{
    env,
    fs::File,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

mod constants;
mod hwdevices;
mod util;

type Result<T> = std::result::Result<T, eyre::Error>;

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
    /// Global "quit" status flag
    pub static ref QUIT: AtomicBool = AtomicBool::new(false);
}

#[derive(Debug, thiserror::Error)]
pub enum MainError {
    #[error("Unknown error: {description}")]
    UnknownError { description: String },
}

/// Supported command line arguments
#[derive(Debug, clap::Parser)]
#[clap(
    version = env!("CARGO_PKG_VERSION"),
    author = "X3n0m0rph59 <x3n0m0rph59@gmail.com>",
    about = "A CLI developer support utility for the Eruption Linux user-mode driver",
)]
pub struct Options {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[clap(subcommand)]
    command: Subcommands,
}

// Sub-commands
#[derive(Debug, clap::Parser)]
pub enum Subcommands {
    /// List available devices, use this first to find out the index of the device to use
    List,

    /// Record key index information subcommands
    RecordKeyIndices {
        #[clap(subcommand)]
        command: RecordKeyIndicesSubcommands,
    },

    /// Test key index information subcommands
    TestKeyIndices {
        #[clap(subcommand)]
        command: TestKeyIndicesSubcommands,
    },

    /// Record key topology information subcommands
    RecordTopology {
        #[clap(subcommand)]
        command: RecordTopologySubcommands,
    },

    /// Test key topology maps subcommands
    TestTopology {
        #[clap(subcommand)]
        command: TestTopologySubcommands,
    },

    /// Generate shell completions
    Completions {
        // #[clap(subcommand)]
        shell: Shell,
    },
}

/// Sub-commands of the "RecordKeyIndices" command
#[derive(Debug, clap::Parser)]
pub enum RecordKeyIndicesSubcommands {
    /// Generate evdev event-code to key index mapping table
    EvDev { device_index: usize },
}

/// Sub-commands of the "TestKeyIndices" command
#[derive(Debug, clap::Parser)]
pub enum TestKeyIndicesSubcommands {
    /// Test mapping of evdev event-codes to key index
    EvDev { device_index: usize },
}

/// Sub-commands of the "RecordTopology" command
#[derive(Debug, clap::Parser)]
pub enum RecordTopologySubcommands {
    /// Generate rows topology information
    Rows { device_index: usize },

    /// Generate columns topology information
    Columns { device_index: usize },

    /// Generate neighbor topology information
    Neighbor { device_index: usize },
}

/// Sub-commands of the "TestTopology" command
#[derive(Debug, clap::Parser)]
pub enum TestTopologySubcommands {
    /// Test rows topology information
    Rows { device_index: usize },

    /// Test columns topology information
    Columns { device_index: usize },

    /// Test neighbor topology information
    Neighbor { device_index: usize },
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

/// Print license information
#[allow(dead_code)]
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

Copyright (c) 2019-2022, The Eruption Development Team
"#
    );
}

/*
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
*/

/// Spawns the keyboard events thread and executes it's main loop
fn spawn_keyboard_input_thread(
    _keyboard_device: Arc<Mutex<Box<HwDevice>>>,
    kbd_tx: Sender<Option<evdev_rs::InputEvent>>,
    device_index: usize,
    usb_vid: u16,
    usb_pid: u16,
) -> Result<()> {
    thread::Builder::new()
        .name(format!("events/kbd:{}", device_index))
        .spawn(move || -> Result<()> {
            let device = match hwdevices::get_input_dev_from_udev(usb_vid, usb_pid) {
                Ok(filename) => match File::open(filename.clone()) {
                    Ok(devfile) => match Device::new_from_file(devfile) {
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

                        kbd_tx.send(Some(k.1)).unwrap_or_else(|e| {
                            error!("Could not send a keyboard event to the main thread: {}", e)
                        });
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

    // initialize logging
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG_OVERRIDE", "info");
        pretty_env_logger::init_custom_env("RUST_LOG_OVERRIDE");
    } else {
        pretty_env_logger::init();
    }

    // start the thread deadlock detector
    // #[cfg(debug_assertions)]
    // thread_util::deadlock_detector()
    //     .unwrap_or_else(|e| error!("Could not spawn deadlock detector thread: {}", e));

    // register ctrl-c handler
    let (ctrl_c_tx, ctrl_c_rx) = unbounded();
    ctrlc::set_handler(move || {
        QUIT.store(true, Ordering::SeqCst);

        ctrl_c_tx
            .send(true)
            .unwrap_or_else(|e| error!("Could not send on a channel: {}", e));
    })
    .unwrap_or_else(|e| error!("Could not set CTRL-C handler: {}", e));

    let opts = Options::parse();
    match opts.command {
        Subcommands::List => {
            println!();
            println!("Please find the device you want to use below and use its respective");
            println!("index number (column 1) as the device index for the other sub-commands of this tool\n");

            // create the one and only hidapi instance
            match hidapi::HidApi::new() {
                Ok(hidapi) => {
                    for (index, device) in hidapi.device_list().enumerate() {
                        println!(
                            "Index: {}: ID: {:x}:{:x} {}/{} Subdev: {}",
                            format!("{:02}", index).bold(),
                            device.vendor_id(),
                            device.product_id(),
                            device.manufacturer_string().unwrap_or("<unknown>").bold(),
                            device.product_string().unwrap_or("<unknown>").bold(),
                            device.interface_number()
                        )
                    }

                    println!("\nEnumeration completed");
                }

                Err(_) => {
                    error!("Could not open HIDAPI");
                }
            };
        }

        // Key index recording related sub-commands
        Subcommands::RecordKeyIndices { command } => match command {
            RecordKeyIndicesSubcommands::EvDev { device_index } => {
                println!();
                println!("Generate evdev event-code to key index mapping table");
                println!();
                println!("Press ESC to skip a key");
                println!();

                // create the one and only hidapi instance
                match hidapi::HidApi::new() {
                    Ok(hidapi) => {
                        if let Some((index, device)) =
                            hidapi.device_list().enumerate().nth(device_index)
                        {
                            println!("Please specify the number of keys, to iterate over");
                            let num_keys = util::get_input("Number of keys: ")
                                .expect("Input error")
                                .parse::<usize>()
                                .expect("Invalid number");

                            // the table that will be filled
                            let mut ev_to_index: Vec<u8> = vec![0xff; 0x2ff + 1];

                            println!(
                                "Index: {}: ID: {:x}:{:x} {}/{} Subdev: {}",
                                format!("{:02}", index).bold(),
                                device.vendor_id(),
                                device.product_id(),
                                device.manufacturer_string().unwrap_or("<unknown>").bold(),
                                device.product_string().unwrap_or("<unknown>").bold(),
                                device.interface_number()
                            );

                            if let Ok(dev) = device.open_device(&hidapi) {
                                let hwdev = Arc::new(Mutex::new(hwdevices::bind_device(
                                    dev,
                                    &hidapi,
                                    device.vendor_id(),
                                    device.product_id(),
                                )?));

                                let led_map = [RGBA {
                                    r: 0,
                                    g: 0,
                                    b: 0,
                                    a: 0,
                                }; 144];

                                hwdev.lock().send_init_sequence()?;
                                hwdev.lock().send_led_map(&led_map)?;

                                // clear any pending/leftover events
                                println!();
                                println!("Clearing any pending events...");

                                loop {
                                    let ev = hwdev.lock().get_next_event_timeout(1000)?;

                                    // println!("{:?}", ev);

                                    if ev == KeyboardHidEvent::Unknown {
                                        break;
                                    }
                                }

                                println!("done");
                                println!();

                                let (kbd_tx, kbd_rx) = unbounded();
                                info!("Spawning evdev input thread...");
                                spawn_keyboard_input_thread(
                                    hwdev.clone(),
                                    kbd_tx,
                                    device_index,
                                    device.vendor_id(),
                                    device.product_id(),
                                )?;

                                let mut key_index = 0;
                                loop {
                                    if key_index >= num_keys {
                                        break;
                                    }

                                    if QUIT.load(Ordering::SeqCst) {
                                        info!("Terminating now");
                                        break;
                                    }

                                    let mut led_map = [RGBA {
                                        r: 0,
                                        g: 0,
                                        b: 0,
                                        a: 0,
                                    }; 144];

                                    // set highlighted LEDs
                                    led_map[key_index] = RGBA {
                                        r: 255,
                                        g: 0,
                                        b: 0,
                                        a: 0,
                                    };

                                    hwdev.lock().send_led_map(&led_map)?;

                                    flume::Selector::new()
                                        .recv(&kbd_rx, |msg| -> Result<()> {
                                            match msg.unwrap() {
                                            Some(ev) => {
                                                // debug!("{:?}", ev);

                                                if ev.value >= 1 {
                                                    if let evdev_rs::enums::EventCode::EV_KEY(code) = ev.event_code {
                                                        if code == evdev_rs::enums::EV_KEY::KEY_ESC {
                                                            info!("Skipping key index: {}", &key_index);
                                                            key_index += 1;
                                                        } else {
                                                            info!("Event code: 0x{:02x} has key index: {}", code as u8, &key_index);

                                                            if ev_to_index[(code as u8) as usize] != 0xff {
                                                                error!("Duplicate indices detected, please retry");
                                                            } else {
                                                                // seems to be valid
                                                                ev_to_index[(code as u8) as usize] = key_index as u8;
                                                                key_index += 1;
                                                            }
                                                        }
                                                    } else {
                                                        // warn!("Event ignored");
                                                    }
                                                }
                                            }

                                            None => error!("Received an invalid event"),
                                        }

                                            Ok(())
                                        })
                                        .recv(&ctrl_c_rx, |_| {
                                            // signal that we want to quit
                                            QUIT.store(true, Ordering::SeqCst);

                                            Ok(())
                                        })
                                        .wait()?;

                                    if QUIT.load(Ordering::SeqCst) {
                                        info!("Terminating now");
                                        break;
                                    }
                                }

                                // processing done
                                println!();
                                println!("Dumping generated table:");
                                println!();

                                println!("let EV_TO_INDEX_<model>: [u8; 0x2ff + 1] = [");
                                for row in ev_to_index.chunks(16) {
                                    print!("\t");

                                    for e in row {
                                        print!("0x{:02x}, ", e);
                                    }

                                    println!();
                                }
                                println!("];");
                            } else {
                                error!("Could not open the device, is the device in use?");
                            }
                        }
                    }

                    Err(_) => {
                        error!("Could not open HIDAPI");
                    }
                }
            }
        },

        // Testing of key indices related sub-commands
        Subcommands::TestKeyIndices { command } => match command {
            TestKeyIndicesSubcommands::EvDev { device_index } => {
                println!();
                println!("Test evdev event-code to key index mapping");
                println!("Each key press should highlight the pressed key");
                println!();
                println!("Press CTRL+C at any time to cancel");
                println!();

                // create the one and only hidapi instance
                match hidapi::HidApi::new() {
                    Ok(hidapi) => {
                        if let Some((index, device)) =
                            hidapi.device_list().enumerate().nth(device_index)
                        {
                            println!(
                                "Index: {}: ID: {:x}:{:x} {}/{} Subdev: {}",
                                format!("{:02}", index).bold(),
                                device.vendor_id(),
                                device.product_id(),
                                device.manufacturer_string().unwrap_or("<unknown>").bold(),
                                device.product_string().unwrap_or("<unknown>").bold(),
                                device.interface_number()
                            );

                            if let Ok(dev) = device.open_device(&hidapi) {
                                let hwdev = Arc::new(Mutex::new(hwdevices::bind_device(
                                    dev,
                                    &hidapi,
                                    device.vendor_id(),
                                    device.product_id(),
                                )?));

                                let mut led_map = [RGBA {
                                    r: 0,
                                    g: 0,
                                    b: 0,
                                    a: 0,
                                }; 144];

                                hwdev.lock().send_init_sequence()?;
                                hwdev.lock().send_led_map(&led_map)?;

                                // clear any pending/leftover events
                                println!();
                                println!("Clearing any pending events...");

                                loop {
                                    let ev = hwdev.lock().get_next_event_timeout(1000)?;

                                    // println!("{:?}", ev);

                                    if ev == KeyboardHidEvent::Unknown {
                                        break;
                                    }
                                }

                                println!("done");
                                println!();

                                let (kbd_tx, kbd_rx) = unbounded();
                                info!("Spawning evdev input thread...");
                                spawn_keyboard_input_thread(
                                    hwdev.clone(),
                                    kbd_tx,
                                    device_index,
                                    device.vendor_id(),
                                    device.product_id(),
                                )?;

                                loop {
                                    if QUIT.load(Ordering::SeqCst) {
                                        info!("Terminating now");
                                        break;
                                    }

                                    flume::Selector::new()
                                        .recv(&kbd_rx, |msg| -> Result<()> {
                                            match msg.unwrap() {
                                                Some(ev) => {
                                                    info!("{:?}", ev);

                                                    if let evdev_rs::enums::EventCode::EV_KEY(
                                                        code,
                                                    ) = ev.event_code
                                                    {
                                                        let key_index =
                                                            hwdev.lock().ev_key_to_key_index(code)
                                                                as usize
                                                                - 1;

                                                        // set highlighted LEDs
                                                        led_map[key_index] = RGBA {
                                                            r: 255,
                                                            g: 0,
                                                            b: 0,
                                                            a: 0,
                                                        };

                                                        hwdev.lock().send_led_map(&led_map)?;
                                                    }
                                                }

                                                None => error!("Received an invalid event"),
                                            }

                                            Ok(())
                                        })
                                        .recv(&ctrl_c_rx, |_| {
                                            // signal that we want to quit
                                            QUIT.store(true, Ordering::SeqCst);
                                            Ok(())
                                        })
                                        .wait()?;

                                    if QUIT.load(Ordering::SeqCst) {
                                        info!("Terminating now");
                                        break;
                                    }
                                }
                            } else {
                                error!("Could not open the device, is the device in use?");
                            }
                        }
                    }

                    Err(_) => {
                        error!("Could not open HIDAPI");
                    }
                }
            }
        },

        // Topology recording related sub-commands
        Subcommands::RecordTopology { command } => match command {
            RecordTopologySubcommands::Rows { device_index } => {
                println!();
                println!("Generate row topology information for the selected device");
                // println!();
                // println!("Press CTRL+C at any time to cancel");
                println!();

                // create the one and only hidapi instance
                match hidapi::HidApi::new() {
                    Ok(hidapi) => {
                        if let Some((index, device)) =
                            hidapi.device_list().enumerate().nth(device_index)
                        {
                            // println!("Please specify the number of keys to iterate over");
                            // let num_keys = util::get_input("Number of keys: ")
                            //     .expect("Input error")
                            //     .parse::<usize>()
                            //     .expect("Invalid number");

                            println!(
                                "Index: {}: ID: {:x}:{:x} {}/{} Subdev: {}",
                                format!("{:02}", index).bold(),
                                device.vendor_id(),
                                device.product_id(),
                                device.manufacturer_string().unwrap_or("<unknown>").bold(),
                                device.product_string().unwrap_or("<unknown>").bold(),
                                device.interface_number()
                            );

                            if let Ok(dev) = device.open_device(&hidapi) {
                                let hwdev = Arc::new(Mutex::new(hwdevices::bind_device(
                                    dev,
                                    &hidapi,
                                    device.vendor_id(),
                                    device.product_id(),
                                )?));

                                let led_map = [RGBA {
                                    r: 0,
                                    g: 0,
                                    b: 0,
                                    a: 0,
                                }; 144];

                                hwdev.lock().send_init_sequence()?;
                                hwdev.lock().send_led_map(&led_map)?;

                                // clear any pending/leftover events
                                println!();
                                println!("Clearing any pending events...");

                                loop {
                                    let ev = hwdev.lock().get_next_event_timeout(1000)?;

                                    // println!("{:?}", ev);

                                    if ev == KeyboardHidEvent::Unknown {
                                        break;
                                    }
                                }

                                println!("done");
                                println!();

                                let (kbd_tx, kbd_rx) = unbounded();
                                info!("Spawning evdev input thread...");
                                spawn_keyboard_input_thread(
                                    hwdev.clone(),
                                    kbd_tx,
                                    device_index,
                                    device.vendor_id(),
                                    device.product_id(),
                                )?;

                                thread::sleep(Duration::from_millis(1000));
                                println!();

                                let keys_per_row = hwdev.lock().get_num_cols() + 1;
                                let num_cols = hwdev.lock().get_num_cols();
                                let num_rows = hwdev.lock().get_num_rows();

                                // the table that will be filled
                                let mut topology: Vec<u8> = vec![0xff; num_cols * num_rows];

                                for i in 0..num_rows {
                                    let mut led_map = [RGBA {
                                        r: 0,
                                        g: 0,
                                        b: 0,
                                        a: 0,
                                    }; 144];

                                    // set highlighted LEDs
                                    // led_map[i] = RGBA {
                                    //     r: 255,
                                    //     g: 0,
                                    //     b: 0,
                                    //     a: 0,
                                    // };

                                    // hwdev.lock().send_led_map(&led_map)?;

                                    println!(
                                        "Please press all keys in row {}, press ESC to skip",
                                        i
                                    );

                                    let mut key_index = 0;
                                    loop {
                                        if key_index >= keys_per_row {
                                            break;
                                        }

                                        if QUIT.load(Ordering::SeqCst) {
                                            info!("Terminating now");
                                            break;
                                        }

                                        flume::Selector::new()
                                            .recv(&kbd_rx, |msg| -> Result<()> {
                                                match msg.unwrap() {
                                                    Some(ev) => {
                                                        // debug!("{:?}", ev);

                                                        if ev.value >= 1 {
                                                            if let evdev_rs::enums::EventCode::EV_KEY(code) = ev.event_code {
                                                                if code == evdev_rs::enums::EV_KEY::KEY_ESC {
                                                                    info!("Skipping key index: {}", &key_index);
                                                                    key_index += 1;
                                                                } else {
                                                                    let idx = hwdev.lock().ev_key_to_key_index(code) - 1;

                                                                    info!("Recorded key with index {}", idx);

                                                                    topology[(i * keys_per_row) + key_index] = idx;
                                                                    key_index += 1;

                                                                    // set highlighted LEDs
                                                                    led_map[idx as usize] = RGBA {
                                                                        r: 255,
                                                                        g: 0,
                                                                        b: 0,
                                                                        a: 0,
                                                                    };

                                                                    hwdev.lock().send_led_map(&led_map)?;
                                                                }
                                                            } else {
                                                                // warn!("Event ignored");
                                                            }
                                                        }
                                                    }

                                                    None => error!("Received an invalid event"),
                                                }

                                                Ok(())
                                            })
                                            .recv(&ctrl_c_rx, |_| {
                                                // signal that we want to quit
                                                QUIT.store(true, Ordering::SeqCst);
                                                Ok(())
                                            })
                                            .wait()?;

                                        if QUIT.load(Ordering::SeqCst) {
                                            info!("Terminating now");
                                            break;
                                        }
                                    }
                                }

                                // processing done
                                println!();
                                println!("Dumping generated table:");
                                println!();

                                println!("pub static ROWS_TOPOLOGY: [u8; {}] = [", topology.len());
                                for row in topology.chunks(keys_per_row) {
                                    print!("\t");

                                    for e in row {
                                        print!("0x{:02x}, ", e);
                                    }

                                    println!();
                                }
                                println!("];");

                                println!();

                                println!("rows_topology = {{");
                                for row in topology.chunks(keys_per_row) {
                                    print!("\t");

                                    for e in row {
                                        print!("0x{:02x}, ", e);
                                    }

                                    println!();
                                }
                                println!("}}");
                            } else {
                                error!("Could not open the device, is the device in use?");
                            }
                        }
                    }

                    Err(_) => {
                        error!("Could not open HIDAPI");
                    }
                }
            }

            RecordTopologySubcommands::Columns { device_index } => {
                println!();
                println!("Generate column topology information for the selected device");
                // println!();
                // println!("Press CTRL+C at any time to cancel");
                println!();

                // create the one and only hidapi instance
                match hidapi::HidApi::new() {
                    Ok(hidapi) => {
                        if let Some((index, device)) =
                            hidapi.device_list().enumerate().nth(device_index)
                        {
                            // println!("Please specify the number of keys to iterate over");
                            // let num_keys = util::get_input("Number of keys: ")
                            //     .expect("Input error")
                            //     .parse::<usize>()
                            //     .expect("Invalid number");

                            println!(
                                "Index: {}: ID: {:x}:{:x} {}/{} Subdev: {}",
                                format!("{:02}", index).bold(),
                                device.vendor_id(),
                                device.product_id(),
                                device.manufacturer_string().unwrap_or("<unknown>").bold(),
                                device.product_string().unwrap_or("<unknown>").bold(),
                                device.interface_number()
                            );

                            if let Ok(dev) = device.open_device(&hidapi) {
                                let hwdev = Arc::new(Mutex::new(hwdevices::bind_device(
                                    dev,
                                    &hidapi,
                                    device.vendor_id(),
                                    device.product_id(),
                                )?));

                                let led_map = [RGBA {
                                    r: 0,
                                    g: 0,
                                    b: 0,
                                    a: 0,
                                }; 144];

                                hwdev.lock().send_init_sequence()?;
                                hwdev.lock().send_led_map(&led_map)?;

                                // clear any pending/leftover events
                                println!();
                                println!("Clearing any pending events...");

                                loop {
                                    let ev = hwdev.lock().get_next_event_timeout(1000)?;

                                    // println!("{:?}", ev);

                                    if ev == KeyboardHidEvent::Unknown {
                                        break;
                                    }
                                }

                                println!("done");
                                println!();

                                let (kbd_tx, kbd_rx) = unbounded();
                                info!("Spawning evdev input thread...");
                                spawn_keyboard_input_thread(
                                    hwdev.clone(),
                                    kbd_tx,
                                    device_index,
                                    device.vendor_id(),
                                    device.product_id(),
                                )?;

                                thread::sleep(Duration::from_millis(1000));
                                println!();

                                let keys_per_col = hwdev.lock().get_num_rows() + 1;
                                let num_cols = hwdev.lock().get_num_cols();
                                let num_rows = hwdev.lock().get_num_rows();

                                // the table that will be filled
                                let mut topology: Vec<u8> = vec![0xff; num_cols * num_rows];

                                for i in 0..num_cols {
                                    let mut led_map = [RGBA {
                                        r: 0,
                                        g: 0,
                                        b: 0,
                                        a: 0,
                                    }; 144];

                                    // set highlighted LEDs
                                    // led_map[i] = RGBA {
                                    //     r: 255,
                                    //     g: 0,
                                    //     b: 0,
                                    //     a: 0,
                                    // };

                                    // hwdev.lock().send_led_map(&led_map)?;

                                    println!(
                                        "Please press all keys in column {}, press ESC to skip",
                                        i
                                    );

                                    let mut key_index = 0;
                                    loop {
                                        if key_index >= keys_per_col {
                                            break;
                                        }

                                        if QUIT.load(Ordering::SeqCst) {
                                            info!("Terminating now");
                                            break;
                                        }

                                        flume::Selector::new()
                                            .recv(&kbd_rx, |msg| -> Result<()> {
                                                match msg.unwrap() {
                                                    Some(ev) => {
                                                        // debug!("{:?}", ev);

                                                        if ev.value >= 1 {
                                                            if let evdev_rs::enums::EventCode::EV_KEY(code) = ev.event_code {
                                                                if code == evdev_rs::enums::EV_KEY::KEY_ESC {
                                                                    info!("Skipping key index: {}", &key_index);
                                                                    key_index += 1;
                                                                } else {
                                                                    let idx = hwdev.lock().ev_key_to_key_index(code) - 1;

                                                                    info!("Recorded key with index {}", idx);

                                                                    topology[(i * num_rows) + key_index] = idx;
                                                                    key_index += 1;

                                                                    // set highlighted LEDs
                                                                    led_map[idx as usize] = RGBA {
                                                                        r: 255,
                                                                        g: 0,
                                                                        b: 0,
                                                                        a: 0,
                                                                    };

                                                                    hwdev.lock().send_led_map(&led_map)?;
                                                                }
                                                            } else {
                                                                // warn!("Event ignored");
                                                            }
                                                        }
                                                    }

                                                    None => error!("Received an invalid event"),
                                                }

                                                Ok(())
                                            })
                                            .recv(&ctrl_c_rx, |_| {
                                                // signal that we want to quit
                                                QUIT.store(true, Ordering::SeqCst);
                                                Ok(())
                                            })
                                            .wait()?;

                                        if QUIT.load(Ordering::SeqCst) {
                                            info!("Terminating now");
                                            break;
                                        }
                                    }
                                }

                                // processing done
                                println!();
                                println!("Dumping generated table:");
                                println!();

                                println!("pub static COLS_TOPOLOGY: [u8; {}] = [", topology.len());
                                for row in topology.chunks(num_rows) {
                                    print!("\t");

                                    for e in row {
                                        print!("0x{:02x}, ", e);
                                    }

                                    println!();
                                }
                                println!("];");

                                println!();

                                println!("cols_topology = {{");
                                for row in topology.chunks(num_rows) {
                                    print!("\t");

                                    for e in row {
                                        print!("0x{:02x}, ", e);
                                    }

                                    println!();
                                }
                                println!("}}");
                            } else {
                                error!("Could not open the device, is the device in use?");
                            }
                        }
                    }

                    Err(_) => {
                        error!("Could not open HIDAPI");
                    }
                }
            }

            RecordTopologySubcommands::Neighbor { device_index } => {
                println!();
                println!("Generate neighbor topology information table");
                println!("This feature needs an updated evdev event-code to key index mapping!");
                println!();

                // create the one and only hidapi instance
                match hidapi::HidApi::new() {
                    Ok(hidapi) => {
                        if let Some((index, device)) =
                            hidapi.device_list().enumerate().nth(device_index)
                        {
                            println!("Please specify the number of keys to iterate over");
                            let num_keys = util::get_input("Number of keys: ")
                                .expect("Input error")
                                .parse::<usize>()
                                .expect("Invalid number");

                            // the table that will be filled
                            let mut neighbor_topology: Vec<u8> = vec![0xff; 2900];

                            println!(
                                "Index: {}: ID: {:x}:{:x} {}/{} Subdev: {}",
                                format!("{:02}", index).bold(),
                                device.vendor_id(),
                                device.product_id(),
                                device.manufacturer_string().unwrap_or("<unknown>").bold(),
                                device.product_string().unwrap_or("<unknown>").bold(),
                                device.interface_number()
                            );

                            if let Ok(dev) = device.open_device(&hidapi) {
                                let hwdev = Arc::new(Mutex::new(hwdevices::bind_device(
                                    dev,
                                    &hidapi,
                                    device.vendor_id(),
                                    device.product_id(),
                                )?));

                                let led_map = [RGBA {
                                    r: 0,
                                    g: 0,
                                    b: 0,
                                    a: 0,
                                }; 144];

                                hwdev.lock().send_init_sequence()?;
                                hwdev.lock().send_led_map(&led_map)?;

                                // clear any pending/leftover events
                                println!();
                                println!("Clearing any pending events...");

                                loop {
                                    let ev = hwdev.lock().get_next_event_timeout(1000)?;

                                    // println!("{:?}", ev);

                                    if ev == KeyboardHidEvent::Unknown {
                                        break;
                                    }
                                }

                                println!("done");
                                println!();

                                let (kbd_tx, kbd_rx) = unbounded();
                                info!("Spawning evdev input thread...");
                                spawn_keyboard_input_thread(
                                    hwdev.clone(),
                                    kbd_tx,
                                    device_index,
                                    device.vendor_id(),
                                    device.product_id(),
                                )?;

                                thread::sleep(Duration::from_millis(1000));
                                println!();

                                for i in 1..=num_keys {
                                    let mut led_map = [RGBA {
                                        r: 0,
                                        g: 0,
                                        b: 0,
                                        a: 0,
                                    }; 144];

                                    // set highlighted LEDs
                                    led_map[i] = RGBA {
                                        r: 255,
                                        g: 0,
                                        b: 0,
                                        a: 0,
                                    };

                                    hwdev.lock().send_led_map(&led_map)?;

                                    println!("Please press all direct neighbor keys of the highlighted (red) key, press ESC to skip");

                                    let mut key_index = 0;
                                    loop {
                                        if key_index >= 10 {
                                            break;
                                        }

                                        if QUIT.load(Ordering::SeqCst) {
                                            info!("Terminating now");
                                            break;
                                        }

                                        flume::Selector::new()
                                            .recv(&kbd_rx, |msg| -> Result<()> {
                                                match msg.unwrap() {
                                                    Some(ev) => {
                                                        // debug!("{:?}", ev);

                                                        if ev.value >= 1 {
                                                            if let evdev_rs::enums::EventCode::EV_KEY(code) = ev.event_code {
                                                                if code == evdev_rs::enums::EV_KEY::KEY_ESC {
                                                                    info!("Skipping key index: {}", &key_index);
                                                                    key_index += 1;
                                                                } else {
                                                                    let idx = hwdev.lock().ev_key_to_key_index(code) - 1;

                                                                    info!("Recorded neighbor with index {} for key: {}", idx, i);

                                                                    neighbor_topology[(i * 10) + key_index] = idx;
                                                                    key_index += 1;

                                                                    // set highlighted LEDs
                                                                    led_map[idx as usize] = RGBA {
                                                                        r: 255,
                                                                        g: 200,
                                                                        b: 200,
                                                                        a: 0,
                                                                    };

                                                                    hwdev.lock().send_led_map(&led_map)?;
                                                                }
                                                            } else {
                                                                // warn!("Event ignored");
                                                            }
                                                        }
                                                    }

                                                    None => error!("Received an invalid event"),
                                                }

                                                Ok(())
                                            })
                                            .recv(&ctrl_c_rx, |_| {
                                                // signal that we want to quit
                                                QUIT.store(true, Ordering::SeqCst);
                                                Ok(())
                                            })
                                            .wait()?;

                                        if QUIT.load(Ordering::SeqCst) {
                                            info!("Terminating now");
                                            break;
                                        }
                                    }
                                }

                                // processing done
                                println!();
                                println!("Dumping generated table:");
                                println!();

                                println!(
                                    "pub static NEIGHBOR_TOPOLOGY: [u8; {}] = [",
                                    neighbor_topology.len()
                                );
                                for row in neighbor_topology.chunks(10) {
                                    print!("\t");

                                    for e in row {
                                        print!("0x{:02x}, ", e);
                                    }

                                    println!();
                                }
                                println!("];");

                                println!();

                                println!("neighbor_topology = {{");
                                for (_idx, row) in neighbor_topology.chunks(10).enumerate() {
                                    print!("\t");

                                    for e in row {
                                        print!("0x{:02x}, ", e);
                                    }

                                    println!();
                                }
                                println!("}}");
                            } else {
                                error!("Could not open the device, is the device in use?");
                            }
                        }
                    }

                    Err(_) => {
                        error!("Could not open HIDAPI");
                    }
                }
            }
        },

        // Topology testing related sub-commands
        Subcommands::TestTopology { command } => match command {
            TestTopologySubcommands::Rows { device_index } => {
                println!();
                println!("Testing compiled-in row topology map for the selected device");
                println!("Each row should light up consistently");
                println!();
                println!("Press CTRL+C at any time to cancel");
                println!();

                // create the one and only hidapi instance
                match hidapi::HidApi::new() {
                    Ok(hidapi) => {
                        if let Some((index, device)) =
                            hidapi.device_list().enumerate().nth(device_index)
                        {
                            println!(
                                "Index: {}: ID: {:x}:{:x} {}/{} Subdev: {}",
                                format!("{:02}", index).bold(),
                                device.vendor_id(),
                                device.product_id(),
                                device.manufacturer_string().unwrap_or("<unknown>").bold(),
                                device.product_string().unwrap_or("<unknown>").bold(),
                                device.interface_number()
                            );

                            if let Ok(dev) = device.open_device(&hidapi) {
                                let hwdev = hwdevices::bind_device(
                                    dev,
                                    &hidapi,
                                    device.vendor_id(),
                                    device.product_id(),
                                )?;

                                hwdev.send_init_sequence()?;

                                let topology = hwdev.get_row_topology();
                                let keys_per_row = hwdev.get_num_cols() + 1;

                                // main loop: highlight full rows at once
                                for key_indices in topology.chunks(keys_per_row) {
                                    thread::sleep(Duration::from_millis(800));

                                    // clear all LEDs
                                    let mut led_map = [RGBA {
                                        r: 0,
                                        g: 0,
                                        b: 0,
                                        a: 0,
                                    }; 144];

                                    for key_index in key_indices {
                                        if (0..144).contains(key_index) {
                                            // set highlighted LEDs
                                            led_map[*key_index as usize] = RGBA {
                                                r: 255,
                                                g: 0,
                                                b: 0,
                                                a: 0,
                                            };
                                        } else {
                                            println!("Sentinel element");
                                        }

                                        hwdev.send_led_map(&led_map)?;
                                        println!("Highlighted key: 0x{:02x}", key_index);
                                    }
                                }

                                thread::sleep(Duration::from_millis(1000));

                                println!();
                                println!("Now lighting up individual keys");
                                println!();

                                // main loop: highlight keys
                                for key_index in topology {
                                    thread::sleep(Duration::from_millis(400));

                                    // clear all LEDs
                                    let mut led_map = [RGBA {
                                        r: 0,
                                        g: 0,
                                        b: 0,
                                        a: 0,
                                    }; 144];

                                    if (0..144).contains(&key_index) {
                                        // set highlighted LEDs
                                        led_map[key_index as usize] = RGBA {
                                            r: 255,
                                            g: 0,
                                            b: 0,
                                            a: 0,
                                        };

                                        hwdev.send_led_map(&led_map)?;
                                        println!("Highlighted key: 0x{:02x}", key_index);
                                    } else {
                                        println!("Sentinel element");
                                    }
                                }
                            } else {
                                error!("Could not open the device, is the device in use?");
                            }
                        }
                    }

                    Err(_) => {
                        error!("Could not open HIDAPI");
                    }
                }
            }

            TestTopologySubcommands::Columns { device_index } => {
                println!();
                println!("Testing compiled-in column topology map for the selected device");
                println!("Each column should light up consistently");
                println!();
                println!("Press CTRL+C at any time to cancel");
                println!();

                // create the one and only hidapi instance
                match hidapi::HidApi::new() {
                    Ok(hidapi) => {
                        if let Some((index, device)) =
                            hidapi.device_list().enumerate().nth(device_index)
                        {
                            println!(
                                "Index: {}: ID: {:x}:{:x} {}/{} Subdev: {}",
                                format!("{:02}", index).bold(),
                                device.vendor_id(),
                                device.product_id(),
                                device.manufacturer_string().unwrap_or("<unknown>").bold(),
                                device.product_string().unwrap_or("<unknown>").bold(),
                                device.interface_number()
                            );

                            if let Ok(dev) = device.open_device(&hidapi) {
                                let hwdev = hwdevices::bind_device(
                                    dev,
                                    &hidapi,
                                    device.vendor_id(),
                                    device.product_id(),
                                )?;

                                hwdev.send_init_sequence()?;

                                let topology = hwdev.get_col_topology();
                                let keys_per_col = hwdev.get_num_cols() + 1;

                                // main loop: highlight full columns at once
                                for key_indices in topology.chunks(keys_per_col) {
                                    thread::sleep(Duration::from_millis(800));

                                    // clear all LEDs
                                    let mut led_map = [RGBA {
                                        r: 0,
                                        g: 0,
                                        b: 0,
                                        a: 0,
                                    }; 144];

                                    for key_index in key_indices {
                                        if (0..144).contains(key_index) {
                                            // set highlighted LEDs
                                            led_map[*key_index as usize] = RGBA {
                                                r: 255,
                                                g: 0,
                                                b: 0,
                                                a: 0,
                                            };
                                        } else {
                                            println!("Sentinel element");
                                        }

                                        hwdev.send_led_map(&led_map)?;
                                        println!("Highlighted key: 0x{:02x}", key_index);
                                    }
                                }

                                thread::sleep(Duration::from_millis(1000));

                                println!();
                                println!("Now lighting up individual keys");
                                println!();

                                // main loop: highlight keys
                                for key_index in topology {
                                    thread::sleep(Duration::from_millis(400));

                                    // clear all LEDs
                                    let mut led_map = [RGBA {
                                        r: 0,
                                        g: 0,
                                        b: 0,
                                        a: 0,
                                    }; 144];

                                    if (0..144).contains(&key_index) {
                                        // set highlighted LEDs
                                        led_map[key_index as usize] = RGBA {
                                            r: 255,
                                            g: 0,
                                            b: 0,
                                            a: 0,
                                        };

                                        hwdev.send_led_map(&led_map)?;
                                        println!("Highlighted key: 0x{:02x}", key_index);
                                    } else {
                                        println!("Sentinel element");
                                    }
                                }
                            } else {
                                error!("Could not open the device, is the device in use?");
                            }
                        }
                    }

                    Err(_) => {
                        error!("Could not open HIDAPI");
                    }
                }
            }

            TestTopologySubcommands::Neighbor { device_index } => {
                println!();
                println!("Testing compiled-in neighbor topology map for the selected device");
                println!("Press CTRL+C at any time to cancel");
                println!();

                // create the one and only hidapi instance
                match hidapi::HidApi::new() {
                    Ok(hidapi) => {
                        if let Some((index, device)) =
                            hidapi.device_list().enumerate().nth(device_index)
                        {
                            println!(
                                "Index: {}: ID: {:x}:{:x} {}/{} Subdev: {}",
                                format!("{:02}", index).bold(),
                                device.vendor_id(),
                                device.product_id(),
                                device.manufacturer_string().unwrap_or("<unknown>").bold(),
                                device.product_string().unwrap_or("<unknown>").bold(),
                                device.interface_number()
                            );

                            let num_keys = 144; // TODO: Implement this

                            if let Ok(dev) = device.open_device(&hidapi) {
                                let hwdev = hwdevices::bind_device(
                                    dev,
                                    &hidapi,
                                    device.vendor_id(),
                                    device.product_id(),
                                )?;

                                hwdev.send_init_sequence()?;

                                // main loop: highlight keys
                                for i in 1..=num_keys {
                                    thread::sleep(Duration::from_millis(800));

                                    // clear all LEDs
                                    let mut led_map = [RGBA {
                                        r: 0,
                                        g: 0,
                                        b: 0,
                                        a: 0,
                                    }; 144];

                                    // set highlighted LEDs
                                    led_map[i as usize] = RGBA {
                                        r: 255,
                                        g: 0,
                                        b: 0,
                                        a: 0,
                                    };

                                    let topology = hwdev.get_neighbor_topology();
                                    let topology =
                                        topology.chunks(10).nth(i).unwrap().iter().take(10);

                                    for key_index in topology {
                                        if (0..144).contains(key_index) {
                                            // set highlighted LEDs
                                            led_map[*key_index as usize] = RGBA {
                                                r: 255,
                                                g: 200,
                                                b: 200,
                                                a: 0,
                                            };

                                            hwdev.send_led_map(&led_map)?;
                                            println!("Highlighted key: 0x{:02x}", key_index);
                                        } else {
                                            println!("Sentinel element");
                                        }
                                    }
                                }
                            } else {
                                error!("Could not open the device, is the device in use?");
                            }
                        }
                    }

                    Err(_) => {
                        error!("Could not open HIDAPI");
                    }
                }
            }
        },

        Subcommands::Completions { shell } => {
            const BIN_NAME: &str = env!("CARGO_PKG_NAME");

            let mut command = Options::command();
            let mut fd = std::io::stdout();

            clap_complete::generate(shell, &mut command, BIN_NAME.to_string(), &mut fd);
        }
    };

    Ok(())
}

/// Main program entrypoint
pub fn main() -> std::result::Result<(), eyre::Error> {
    let language_loader: FluentLanguageLoader = fluent_language_loader!();

    let requested_languages = DesktopLanguageRequester::requested_languages();
    i18n_embed::select(&language_loader, &Localizations, &requested_languages)?;

    STATIC_LOADER.lock().replace(language_loader);

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("worker")
        .enable_all()
        // .worker_threads(4)
        .build()?;

    runtime.block_on(async move { async_main().await })
}
