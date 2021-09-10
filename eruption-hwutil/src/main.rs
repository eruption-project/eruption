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
    env, process,
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

    /// Sets the configuration file to use
    #[clap(short = 'c', long)]
    config: Option<String>,

    #[clap(subcommand)]
    command: Subcommands,
}

// Sub-commands
#[derive(Debug, Clone, Clap)]
pub enum Subcommands {
    /// Devices related subcommands
    Devices {
        #[clap(subcommand)]
        command: DevicesSubcommands,
    },

    /// Firmware update related subcommands
    FirmwareUpdate {
        #[clap(subcommand)]
        command: FirmwareUpdateSubcommands,
    },

    /// Generate shell completions
    Completions {
        #[clap(subcommand)]
        command: CompletionsSubcommands,
    },
}

/// Subcommands of the "devices" command
#[derive(Debug, Clone, Clap)]
pub enum DevicesSubcommands {
    /// List devices
    List,

    /// Query device specific status
    Status { device: usize },
}

/// Subcommands of the "firmware-update" command
#[derive(Debug, Clone, Clap)]
pub enum FirmwareUpdateSubcommands {
    /// Get some information about the currently installed firmware
    Info { device: u64 },

    /// Flash firmware
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
        Subcommands::Devices { command } => match command {
            DevicesSubcommands::List => {
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

                        println!("\nEnumeration of HID devices completed");

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

            DevicesSubcommands::Status {
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
                                device.product_string().unwrap_or("<unknown>").bold(),
                            );

                            if let Ok(dev) = device.open_device(&hidapi) {
                                let hwdev = hwdevices::bind_device(
                                    dev,
                                    &hidapi,
                                    device.vendor_id(),
                                    device.product_id(),
                                )?;

                                hwdev.send_init_sequence()?;

                                println!("Polling device status...");

                                loop {
                                    if QUIT.load(Ordering::SeqCst) {
                                        break;
                                    }

                                    let status = hwdev.device_status()?;

                                    println!();
                                    println!("Battery Level:   {}", status["battery-level"]);
                                    println!("Signal Strength: {}", status["signal-strength"]);

                                    thread::sleep(Duration::from_millis(250));
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

        Subcommands::FirmwareUpdate { command: _ } => {}

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
