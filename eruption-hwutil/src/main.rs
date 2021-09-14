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

use clap::{lazy_static::lazy_static, Clap};
use colored::*;
use crossbeam::channel::unbounded;
use log::error;
use parking_lot::Mutex;
use std::{
    env,
    process::{self},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

mod constants;
mod device;
mod hwdevices;
mod util;

// type Result<T> = std::result::Result<T, eyre::Error>;

#[macro_export]
macro_rules! println_v {
    () => {
        println!()
    };

    ($verbosity : expr, $l : literal $(,$params : tt) *) => {
        if crate::OPTIONS.lock().as_ref().unwrap().verbose >= $verbosity as u8 {
            println!($l, $($params),*)
        }
    };

    ($verbosity : expr, $($params : tt) *) => {
        if crate::OPTIONS.lock().as_ref().unwrap().verbose >= $verbosity as u8 {
            println!($($params),*)
        }
    };
}

#[macro_export]
macro_rules! eprintln_v {
    () => {
        eprintln!()
    };

    ($verbosity : expr, $l : literal $(,$params : tt) *) => {
        if crate::OPTIONS.lock().as_ref().unwrap().verbose >= $verbosity as u8 {
            eprintln!($l, $($params),*)
        }
    };

    ($verbosity : expr, $($params : tt) *) => {
        if crate::OPTIONS.lock().as_ref().unwrap().verbose >= $verbosity as u8 {
            eprintln!($($params),*)
        }
    };
}

lazy_static! {
    /// Global configuration
    pub static ref CONFIG: Arc<Mutex<Option<config::Config>>> = Arc::new(Mutex::new(None));

    /// Global command line options
    pub static ref OPTIONS: Arc<Mutex<Option<Options>>> = Arc::new(Mutex::new(None));

    /// Global "quit" status flag
    pub static ref QUIT: AtomicBool = AtomicBool::new(false);
}

#[derive(Debug, thiserror::Error)]
pub enum MainError {
    #[error("Unknown error: {description}")]
    UnknownError { description: String },
}

/// Supported command line arguments
#[derive(Debug, Clone, Clap)]
#[clap(
    version = env!("CARGO_PKG_VERSION"),
    author = "X3n0m0rph59 <x3n0m0rph59@gmail.com>",
    about = "A CLI control utility for hardware supported by the Eruption Linux user-mode driver",
)]

pub struct Options {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Repeat output until ctrl+c is pressed
    #[clap(short, long)]
    repeat: bool,

    /// Sets the configuration file to use
    #[clap(short = 'c', long)]
    config: Option<String>,

    #[clap(subcommand)]
    command: Subcommands,
}

// Sub-commands
#[derive(Debug, Clone, Clap)]
pub enum Subcommands {
    /// List available devices, use this first to find out the index of the device to address
    #[clap(display_order = 0)]
    List,

    /// Query device specific status like e.g.: Signal Strength/Battery Level
    #[clap(display_order = 1)]
    Status { device: usize },

    /// Turn off all LEDs, but otherwise leave the device completely usable
    #[clap(display_order = 2)]
    Blackout { device: usize },

    /// Firmware related subcommands (DANGEROUS, may brick the device)
    #[clap(display_order = 3)]
    Firmware {
        #[clap(subcommand)]
        command: FirmwareSubcommands,
    },

    /// Generate shell completions
    #[clap(display_order = 4)]
    Completions {
        #[clap(subcommand)]
        command: CompletionsSubcommands,
    },
}

/// Subcommands of the "firmware" command
#[derive(Debug, Clone, Clap)]
pub enum FirmwareSubcommands {
    /// Get some information about the currently installed firmware
    #[clap(display_order = 0)]
    Info { device: u64 },

    /// Flash firmware to device (DANGEROUS, may brick the device)
    #[clap(display_order = 1)]
    Flash { device: u64 },
}

/// Subcommands of the "completions" command
#[derive(Debug, Clone, Clap)]
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

#[allow(dead_code)]
fn print_notice() {
    println!(
        r#"
 Please stop the Eruption daemon prior to running this tool:
 $ sudo systemctl mask eruption.service && sudo systemctl stop eruption.service

 You can re-enable Eruption with this command afterwards:
 $ sudo systemctl unmask eruption.service && sudo systemctl start eruption.service
 "#
    );
}

#[tokio::main(flavor = "multi_thread")]
pub async fn main() -> std::result::Result<(), eyre::Error> {
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
        // print_header();
        print_notice();
    }

    // initialize logging
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG_OVERRIDE", "info");
        pretty_env_logger::init_custom_env("RUST_LOG_OVERRIDE");
    } else {
        pretty_env_logger::init();
    }

    // register ctrl-c handler
    let (ctrl_c_tx, _ctrl_c_rx) = unbounded();
    ctrlc::set_handler(move || {
        QUIT.store(true, Ordering::SeqCst);

        ctrl_c_tx
            .send(true)
            .unwrap_or_else(|e| error!("Could not send on a channel: {}", e));
    })
    .unwrap_or_else(|e| error!("Could not set CTRL-C handler: {}", e));

    let opts = Options::parse();
    *OPTIONS.lock() = Some(opts.clone());

    // process configuration file
    let config_file = opts
        .config
        .unwrap_or(constants::DEFAULT_CONFIG_FILE.to_string());

    let mut config = config::Config::default();
    config
        .merge(config::File::new(&config_file, config::FileFormat::Toml))
        .unwrap_or_else(|e| {
            log::error!("Could not parse configuration file: {}", e);
            process::exit(4);
        });

    *CONFIG.lock() = Some(config);

    match opts.command {
        // device specific sub-commands
        Subcommands::List => {
            println!();
            println!("Please find the device you want to address below and use its respective");
            println!("index number (column 1) as the device index for the other sub-commands of this tool\n");

            // create the one and only hidapi instance
            match hidapi::HidApi::new() {
                Ok(hidapi) => {
                    for (index, device) in hidapi.device_list().enumerate() {
                        if device.interface_number() == 0 {
                            println!(
                                "Index: {}: ID: {:x}:{:x} {}/{}",
                                format!("{:02}", index).bold(),
                                device.vendor_id(),
                                device.product_id(),
                                device.manufacturer_string().unwrap_or("<unknown>").bold(),
                                device.product_string().unwrap_or("<unknown>").bold(),
                            )
                        }
                    }

                    println_v!(1, "\nEnumeration of HID devices completed");

                    // println!("\nSpecial devices\n");

                    // for device_index in 0..4 {
                    //     let device_file = format!("/dev/ttyACM{}", device_index);

                    //     println!(
                    //         "Index: {}: Serial Port {} ({})",
                    //         &format!("{}", 255 - device_index).bold(),
                    //         device_index + 1,
                    //         &device_file
                    //     );
                    // }
                }

                Err(_) => {
                    error!("Could not open HIDAPI");
                }
            };
        }

        Subcommands::Status {
            device: device_index,
        } => {
            // create the one and only hidapi instance
            match hidapi::HidApi::new() {
                Ok(hidapi) => {
                    if let Some((index, device)) =
                        hidapi.device_list().enumerate().nth(device_index)
                    {
                        let term = console::Term::stdout();

                        println!(
                            "Index: {}: ID: {:x}:{:x} {}/{}",
                            format!("{:02}", index).bold(),
                            device.vendor_id(),
                            device.product_id(),
                            device.manufacturer_string().unwrap_or("<unknown>").bold(),
                            device.product_string().unwrap_or("<unknown>").bold()
                        );

                        if let Ok(dev) = device.open_device(&hidapi) {
                            let hwdev = hwdevices::bind_device(
                                dev,
                                &hidapi,
                                device.vendor_id(),
                                device.product_id(),
                            )?;

                            hwdev.send_init_sequence()?;

                            println_v!(1, "Polling device status...");

                            // reserve a few lines of space for the output
                            println!();
                            println!();
                            println!();
                            println!();

                            loop {
                                let status = hwdev.device_status()?;

                                term.clear_last_lines(4)?;

                                println!();

                                println!(
                                    "Transceiver enabled: {:>}",
                                    status
                                        .get("transceiver-enabled")
                                        .map(|e| e.to_string())
                                        .unwrap_or_else(|| "---".to_string())
                                        .bold()
                                );

                                println!(
                                    "Signal Strength:     {:>4} ({})",
                                    status
                                        .get("signal-strength-percent")
                                        .map(|e| e.to_string())
                                        .unwrap_or_else(|| "---".to_string())
                                        .bold(),
                                    status
                                        .get("signal-strength-raw")
                                        .map(|e| e.to_string())
                                        .unwrap_or_else(|| "---".to_string())
                                );

                                println!(
                                    "Battery Level:       {:>4} ({})",
                                    status
                                        .get("battery-level-percent")
                                        .map(|e| e.to_string())
                                        .unwrap_or_else(|| "---".to_string())
                                        .bold(),
                                    status
                                        .get("battery-level-raw")
                                        .map(|e| e.to_string())
                                        .unwrap_or_else(|| "---".to_string())
                                );

                                if !opts.repeat || QUIT.load(Ordering::SeqCst) {
                                    break;
                                }

                                term.flush()?;

                                thread::sleep(Duration::from_millis(1));
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

        Subcommands::Blackout {
            device: device_index,
        } => {
            // create the one and only hidapi instance
            match hidapi::HidApi::new() {
                Ok(hidapi) => {
                    if let Some((index, device)) =
                        hidapi.device_list().enumerate().nth(device_index)
                    {
                        println!(
                            "Index: {}: ID: {:x}:{:x} {}/{}",
                            format!("{:02}", index).bold(),
                            device.vendor_id(),
                            device.product_id(),
                            device.manufacturer_string().unwrap_or("<unknown>").bold(),
                            device.product_string().unwrap_or("<unknown>").bold()
                        );

                        if let Ok(dev) = device.open_device(&hidapi) {
                            let hwdev = hwdevices::bind_device(
                                dev,
                                &hidapi,
                                device.vendor_id(),
                                device.product_id(),
                            )?;

                            hwdev.send_init_sequence()?;

                            println_v!(1, "Polling device status...");

                            hwdev.send_led_map(
                                &[hwdevices::RGBA {
                                    r: 0x00,
                                    g: 0x00,
                                    b: 0x00,
                                    a: 0x00,
                                }; constants::CANVAS_SIZE],
                            )?;
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

        Subcommands::Firmware { command: _ } => {}

        Subcommands::Completions { command } => {
            use clap::IntoApp;
            use clap_generate::{generate, generators::*};

            const BIN_NAME: &str = env!("CARGO_PKG_NAME");

            let mut app = Options::into_app();
            let mut fd = std::io::stdout();

            match command {
                CompletionsSubcommands::Bash => {
                    generate::<Bash, _>(&mut app, BIN_NAME, &mut fd);
                }

                CompletionsSubcommands::Elvish => {
                    generate::<Elvish, _>(&mut app, BIN_NAME, &mut fd);
                }

                CompletionsSubcommands::Fish => {
                    generate::<Fish, _>(&mut app, BIN_NAME, &mut fd);
                }

                CompletionsSubcommands::PowerShell => {
                    generate::<PowerShell, _>(&mut app, BIN_NAME, &mut fd);
                }

                CompletionsSubcommands::Zsh => {
                    generate::<Zsh, _>(&mut app, BIN_NAME, &mut fd);
                }
            }
        }
    };

    Ok(())
}
