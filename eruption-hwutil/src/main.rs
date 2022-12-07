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

use clap::CommandFactory;
use clap::Parser;
use clap_complete::Shell;
use colored::*;
use config::Config;
use flume::unbounded;
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use rust_embed::RustEmbed;
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
use tracing::error;

mod constants;
mod device;
mod hwdevices;
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

// type Result<T> = std::result::Result<T, eyre::Error>;

#[macro_export]
macro_rules! println_v {
    () => {
        println!()
    };

    ($verbosity : expr, $l : literal $(,$params : tt) *) => {
        if $crate::OPTIONS.lock().as_ref().unwrap().verbose >= $verbosity as u8 {
            println!($l, $($params),*)
        }
    };

    ($verbosity : expr, $($params : tt) *) => {
        if $crate::OPTIONS.lock().as_ref().unwrap().verbose >= $verbosity as u8 {
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
        if $crate::OPTIONS.lock().as_ref().unwrap().verbose >= $verbosity as u8 {
            eprintln!($l, $($params),*)
        }
    };

    ($verbosity : expr, $($params : tt) *) => {
        if $crate::OPTIONS.lock().as_ref().unwrap().verbose >= $verbosity as u8 {
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
#[derive(Debug, Clone, clap::Parser)]
#[clap(
    version = env!("CARGO_PKG_VERSION"),
    author = "X3n0m0rph59 <x3n0m0rph59@gmail.com>",
    about = "A CLI control utility for hardware supported by the Eruption Linux user-mode driver",
)]

pub struct Options {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, action = clap::ArgAction::Count)]
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
#[derive(Debug, Clone, clap::Parser)]
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
    #[clap(hide = true, about(tr!("completions-about")))]
    Completions {
        // #[clap(subcommand)]
        shell: Shell,
    },
}

/// Subcommands of the "firmware" command
#[derive(Debug, Clone, clap::Parser)]
pub enum FirmwareSubcommands {
    /// Get some information about the currently installed firmware
    #[clap(display_order = 0)]
    Info { device: u64 },

    /// Flash firmware to device (DANGEROUS, may brick the device)
    #[clap(display_order = 1)]
    Flash { device: u64 },
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

Copyright (c) 2019-2023, The Eruption Development Team
"#
    );
}

#[allow(dead_code)]
fn print_notice() {
    println!(
        r#"
 Please stop the Eruption daemon prior to running this tool:
 $ sudo systemctl stop eruption.service && sudo systemctl mask eruption.service

 You can re-enable Eruption with this command afterwards:
 $ sudo systemctl unmask eruption.service && sudo systemctl start eruption.service
 "#
    );
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

        if util::is_eruption_daemon_running() {
            print_notice();
        }
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
        .unwrap_or_else(|| constants::DEFAULT_CONFIG_FILE.to_string());

    let config = Config::builder()
        .add_source(config::File::new(&config_file, config::FileFormat::Toml))
        .build()
        .unwrap_or_else(|e| {
            tracing::error!("Could not parse configuration file: {}", e);
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
                            if opts.verbose > 0 {
                                println!(
                                    "Index: {}: ID: {:x}:{:x} {}/{}",
                                    format!("{index:02}").bold(),
                                    device.vendor_id(),
                                    device.product_id(),
                                    device.manufacturer_string().unwrap_or("<unknown>").bold(),
                                    device.product_string().unwrap_or("<unknown>").bold()
                                );
                            } else {
                                println!(
                                    "{}: {}/{}",
                                    format!("{index:02}").bold(),
                                    device.manufacturer_string().unwrap_or("<unknown>").bold(),
                                    device.product_string().unwrap_or("<unknown>").bold()
                                );
                            }
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

                        if opts.verbose > 0 {
                            println!(
                                "Index: {}: ID: {:x}:{:x} {}/{}",
                                format!("{index:02}").bold(),
                                device.vendor_id(),
                                device.product_id(),
                                device.manufacturer_string().unwrap_or("<unknown>").bold(),
                                device.product_string().unwrap_or("<unknown>").bold()
                            );
                        } else {
                            println!(
                                "{}: {}/{}",
                                format!("{index:02}").bold(),
                                device.manufacturer_string().unwrap_or("<unknown>").bold(),
                                device.product_string().unwrap_or("<unknown>").bold()
                            );
                        }

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
                                match hwdev.device_status() {
                                    Ok(status) => {
                                        term.clear_last_lines(4)?;

                                        println!();

                                        println!(
                                            "Transceiver enabled:  {:>}",
                                            status
                                                .get("transceiver-enabled")
                                                .map(|e| e.to_string())
                                                .unwrap_or_else(|| " ---".to_string())
                                                .bold()
                                        );

                                        println!(
                                            "Signal strength:      {:>4}% ({})",
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
                                            "Battery level:        {:>4}% ({})",
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
                                    }

                                    Err(_e) => {
                                        // term.clear_last_lines(1)?;
                                        // eprintln!("{}", e)
                                    }
                                }

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
                        if opts.verbose > 1 {
                            println!(
                                "Index: {}: ID: {:x}:{:x} {}/{}",
                                format!("{index:02}").bold(),
                                device.vendor_id(),
                                device.product_id(),
                                device.manufacturer_string().unwrap_or("<unknown>").bold(),
                                device.product_string().unwrap_or("<unknown>").bold()
                            );
                        } else if opts.verbose > 0 {
                            println!(
                                "{}: {}/{}",
                                format!("{index:02}").bold(),
                                device.manufacturer_string().unwrap_or("<unknown>").bold(),
                                device.product_string().unwrap_or("<unknown>").bold()
                            );
                        }

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
    // let filter = tracing_subscriber::EnvFilter::from_default_env();
    // let journald_layer = tracing_journald::layer()?.with_filter(filter);

    // let filter = tracing_subscriber::EnvFilter::from_default_env();
    // let format_layer = tracing_subscriber::fmt::layer()
    //     .compact()
    //     .with_filter(filter);

    cfg_if::cfg_if! {
        if #[cfg(feature = "debug-async")] {
            console_layer = console_subscriber::ConsoleLayer::builder()
                .with_default_env()
                .spawn();

            tracing_subscriber::registry()
                // .with(journald_layer)
                .with(console_layer)
                // .with(format_layer)
                .init();
        } else {
            // tracing_subscriber::registry()
            //     // .with(journald_layer)
            //     // .with(console_layer)
            //     // .with(format_layer)
            //     .init();
        }
    };

    // i18n/l10n support
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
