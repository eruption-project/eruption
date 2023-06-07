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
use flume::unbounded;
use hwdevices::RGBA;
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use rust_embed::RustEmbed;
use std::sync::Arc;
use std::{env, thread};
use std::{path::PathBuf, time::Duration};
use std::{sync::atomic::AtomicBool, sync::atomic::Ordering, time::Instant};
use tracing::*;

mod constants;
mod hwdevices;
mod util;

use util::{DeviceState, HexSlice};

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
    about = "A CLI utility to debug USB HID devices",
)]
pub struct Options {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[clap(subcommand)]
    command: Subcommands,
}

// Sub-commands
#[derive(Debug, Clone, clap::Parser)]
pub enum Subcommands {
    /// List available devices, use this first to find out the index of the device to use
    List,

    /// Generate a report for the specified device
    Report {
        /// The index of the device, can be found with the list sub-command
        device: usize,
    },

    /// Dump a trace of events originating from the specified device (May hang the device)
    Trace {
        /// The index of the device, can be found with the list sub-command
        device: usize,
    },

    /// Read out the device state and show differences to previous state (May hang the device)
    StateDiff {
        /// The index of the device, can be found with the list sub-command
        device: usize,
    },

    /// Read a single USB HID feature report from device
    Read {
        /// The index of the device, can be found with the list sub-command
        device: usize,

        /// ID of the USB HID report
        #[clap(value_parser = util::parse_report_id)]
        report_id: u8,

        /// Length in bytes to read
        length: usize,
    },

    /// Send a single USB HID feature report to device (dangerous)
    Write {
        /// The index of the device, can be found with the list sub-command
        device: usize,

        /// Hex bytes e.g.: [0x09, 0x00, 0x1f]
        data: String,
    },

    /// Read data from device
    ReadRaw {
        /// The index of the device, can be found with the list sub-command
        device: usize,

        /// ID of the USB HID report
        #[clap(value_parser = util::parse_report_id)]
        report_id: u8,

        /// Length in bytes to read
        length: usize,
    },

    /// Send data to device (dangerous)
    WriteRaw {
        /// The index of the device, can be found with the list sub-command
        device: usize,

        /// Hex bytes e.g.: [0x09, 0x00, 0x1f]
        data: String,
    },

    /// Send a device specific init sequence and try to set colors
    RunTests {
        /// The index of the device, can be found with the list sub-command
        device: usize,
    },

    /// Special utility functions, like searching for CRC polynoms and parameters
    Utils {
        #[clap(subcommand)]
        command: UtilsSubcommands,
    },

    /// Generate shell completions
    #[clap(hide = true, about(tr!("completions-about")))]
    Completions {
        // #[clap(subcommand)]
        shell: Shell,
    },
}

/// Subcommands of the "utils" command
#[derive(Debug, Clone, clap::Parser)]
pub enum UtilsSubcommands {
    /// Find CRC8 polynoms and init params by performing an exhaustive search
    ReverseCrc8 {
        /// Hex byte vectors each starting with expected CRC8, no spaces allowed.
        /// [0x32,0xff,0x00,0x00,0x00,0x00,0xff] [0x31,0x59,0xa5,0xff,0x00,0x00,0x00] [0x31,0x00,0x00,0xff,0xff,0x00,0x00]
        data: Vec<String>,
    },
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
    #[cfg(not(target_os = "windows"))]
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

    match opts.command {
        Subcommands::List => {
            println!();
            println!("Please find the device you want to debug below and use its respective");
            println!("index number (column 1) as the device index for the other sub-commands of this tool\n");

            // create the one and only hidapi instance
            match hidapi::HidApi::new() {
                Ok(hidapi) => {
                    for (index, device) in hidapi.device_list().enumerate() {
                        println!(
                            "Index: {}: ID: {:x}:{:x} {}/{} iface: {}:{:x}",
                            format!("{index:02}").bold(),
                            device.vendor_id(),
                            device.product_id(),
                            device.manufacturer_string().unwrap_or("<unknown>").bold(),
                            device.product_string().unwrap_or("<unknown>").bold(),
                            device.interface_number(),
                            device.usage_page(),
                        )
                    }

                    println!("\nEnumeration of HID devices completed");

                    println!("\nSpecial devices\n");

                    for device_index in 0..4 {
                        let device_file = format!("/dev/ttyACM{device_index}");

                        println!(
                            "Index: {}: Serial Port {} ({})",
                            &format!("{}", 255 - device_index).bold(),
                            device_index + 1,
                            &device_file
                        );
                    }
                }

                Err(_) => {
                    error!("Could not open HIDAPI");
                }
            };
        }

        Subcommands::Report { device } => {
            println!("-- Start of report --");

            // create the one and only hidapi instance
            match hidapi::HidApi::new() {
                Ok(hidapi) => {
                    if let Some((index, device)) = hidapi.device_list().enumerate().nth(device) {
                        println!(
                            "Index: {}: ID: {:x}:{:x} {}/{} iface: {}:{:x}",
                            format!("{index:02}").bold(),
                            device.vendor_id(),
                            device.product_id(),
                            device.manufacturer_string().unwrap_or("<unknown>").bold(),
                            device.product_string().unwrap_or("<unknown>").bold(),
                            device.interface_number(),
                            device.usage_page(),
                        );

                        if let Ok(dev) = device.open_device(&hidapi) {
                            for i in 0..256 {
                                if let Ok(result) = dev.get_indexed_string(i) {
                                    if let Some(s) = result {
                                        println!("{i:03}: {s}");
                                    }
                                } else if opts.verbose > 0 {
                                    error!("{:03}: {}", i, "Failed");
                                }
                            }
                        } else {
                            error!("Could not open the device, is the device in use?");
                        }
                    } else {
                        error!("Invalid device index");
                    }
                }

                Err(_) => {
                    error!("Could not open HIDAPI");
                }
            };

            println!("-- End of report --");
        }

        Subcommands::Trace { device } => {
            println!("-- Start of trace --");

            // create the one and only hidapi instance
            match hidapi::HidApi::new() {
                Ok(hidapi) => {
                    if let Some((index, device)) = hidapi.device_list().enumerate().nth(device) {
                        println!(
                            "Index: {}: ID: {:x}:{:x} {}/{} iface: {}:{:x}",
                            format!("{index:02}").bold(),
                            device.vendor_id(),
                            device.product_id(),
                            device.manufacturer_string().unwrap_or("<unknown>").bold(),
                            device.product_string().unwrap_or("<unknown>").bold(),
                            device.interface_number(),
                            device.usage_page(),
                        );

                        if let Ok(dev) = device.open_device(&hidapi) {
                            println!("Initializing...");

                            // let mut buf: [u8; 8] = [0; 8];
                            // buf[0] = 0x04;

                            // let bytes_read = dev.get_feature_report(&mut buf)?;

                            // println!("{} bytes", bytes_read);
                            // hexdump::hexdump_iter(&buf).for_each(|s| println!("  {}", s));

                            // wait to settle
                            thread::sleep(Duration::from_millis(500));

                            println!("Entering polling loop:");

                            loop {
                                let mut buf: [u8; 16] = [0; 16];
                                buf[0] = 0xff;

                                let bytes_read = dev.read(&mut buf)?;

                                println!("{:?}: {} bytes", Instant::now(), bytes_read);
                                hexdump::hexdump_iter(&buf).for_each(|s| println!("  {s}"));
                            }
                        } else {
                            error!("Could not open the device, is the device in use?");
                        }
                    } else {
                        error!("Invalid device index");
                    }
                }

                Err(_) => {
                    error!("Could not open HIDAPI");
                }
            };

            println!("-- End of trace --");
        }

        Subcommands::StateDiff {
            device: device_index,
        } => {
            // create the one and only hidapi instance
            match hidapi::HidApi::new() {
                Ok(hidapi) => {
                    if let Some((index, device)) =
                        hidapi.device_list().enumerate().nth(device_index)
                    {
                        println!(
                            "Index: {}: ID: {:x}:{:x} {}/{} iface: {}:{:x}",
                            format!("{index:02}").bold(),
                            device.vendor_id(),
                            device.product_id(),
                            device.manufacturer_string().unwrap_or("<unknown>").bold(),
                            device.product_string().unwrap_or("<unknown>").bold(),
                            device.interface_number(),
                            device.usage_page(),
                        );

                        if let Ok(dev) = device.open_device(&hidapi) {
                            let path = PathBuf::from("/tmp/eruption-debug-tool.data");

                            let mut data_store = util::load_data_from_file(&path)?;
                            let mut state = DeviceState::new(
                                device.serial_number().unwrap_or("<unknown>"),
                                device.product_string().unwrap_or("<unknown>"),
                            );

                            println!("Reading data from device...");

                            for report_id in 0x00..=0xff {
                                let length = 128;

                                let mut buf = vec![0; length];
                                buf[0] = report_id;

                                match dev.get_feature_report(&mut buf) {
                                    Ok(_len) => {
                                        if opts.verbose > 0 {
                                            println!(
                                                // "0x{:02x} (len:{}): [{}]",
                                                "0x{:02x} [{}]",
                                                report_id,
                                                // len,
                                                HexSlice::new(&buf)
                                            );
                                        }

                                        state.data.insert(report_id, buf);
                                    }

                                    Err(e) => {
                                        if opts.verbose > 0 {
                                            warn!(
                                                "Report ID not implemented?: 0x{:02x}: {}",
                                                report_id, e
                                            );
                                        }
                                    }
                                }
                            }

                            println!("The following USB HID report IDs have changed bytes:\n");

                            util::print_diff(&state, &data_store);

                            println!("Saving state data...");

                            data_store.push(state);
                            util::save_data_to_file(&path, &data_store)?;

                            println!("Done");
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

        Subcommands::Read {
            device: device_index,
            report_id,
            length,
        } => {
            // create the one and only hidapi instance
            match hidapi::HidApi::new() {
                Ok(hidapi) => {
                    if let Some((index, device)) =
                        hidapi.device_list().enumerate().nth(device_index)
                    {
                        println!(
                            "Index: {}: ID: {:x}:{:x} {}/{} iface: {}:{:x}",
                            format!("{index:02}").bold(),
                            device.vendor_id(),
                            device.product_id(),
                            device.manufacturer_string().unwrap_or("<unknown>").bold(),
                            device.product_string().unwrap_or("<unknown>").bold(),
                            device.interface_number(),
                            device.usage_page(),
                        );

                        if let Ok(dev) = device.open_device(&hidapi) {
                            println!("Reading data from device...");

                            let mut buf = Vec::new();

                            buf.resize(length + 1, 0x00);
                            buf[0] = report_id;

                            dev.get_feature_report(&mut buf)?;

                            println!("[{}]", HexSlice::new(&buf));
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

        Subcommands::Write {
            device: device_index,
            data,
        } => {
            // create the one and only hidapi instance
            match hidapi::HidApi::new() {
                Ok(hidapi) => {
                    if let Some((index, device)) =
                        hidapi.device_list().enumerate().nth(device_index)
                    {
                        println!(
                            "Index: {}: ID: {:x}:{:x} {}/{} iface: {}:{:x}",
                            format!("{index:02}").bold(),
                            device.vendor_id(),
                            device.product_id(),
                            device.manufacturer_string().unwrap_or("<unknown>").bold(),
                            device.product_string().unwrap_or("<unknown>").bold(),
                            device.interface_number(),
                            device.usage_page(),
                        );

                        if let Ok(dev) = device.open_device(&hidapi) {
                            println!("Writing data to device...");

                            let buf = util::parse_hex_vec(&data)?;

                            println!("[{}]", HexSlice::new(&buf));

                            dev.send_feature_report(&buf)?;
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

        Subcommands::ReadRaw {
            device: device_index,
            report_id,
            length,
        } => {
            // create the one and only hidapi instance
            match hidapi::HidApi::new() {
                Ok(hidapi) => {
                    if let Some((index, device)) =
                        hidapi.device_list().enumerate().nth(device_index)
                    {
                        println!(
                            "Index: {}: ID: {:x}:{:x} {}/{} iface: {}:{:x}",
                            format!("{index:02}").bold(),
                            device.vendor_id(),
                            device.product_id(),
                            device.manufacturer_string().unwrap_or("<unknown>").bold(),
                            device.product_string().unwrap_or("<unknown>").bold(),
                            device.interface_number(),
                            device.usage_page(),
                        );

                        if let Ok(dev) = device.open_device(&hidapi) {
                            println!("Reading data from device...");

                            let mut buf = Vec::new();

                            buf.resize(length + 1, 0x00);
                            buf[0] = report_id;

                            dev.read(&mut buf)?;

                            println!("[{}]", HexSlice::new(&buf));
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

        Subcommands::WriteRaw {
            device: device_index,
            data,
        } => {
            // create the one and only hidapi instance
            match hidapi::HidApi::new() {
                Ok(hidapi) => {
                    if let Some((index, device)) =
                        hidapi.device_list().enumerate().nth(device_index)
                    {
                        println!(
                            "Index: {}: ID: {:x}:{:x} {}/{} iface: {}:{:x}",
                            format!("{index:02}").bold(),
                            device.vendor_id(),
                            device.product_id(),
                            device.manufacturer_string().unwrap_or("<unknown>").bold(),
                            device.product_string().unwrap_or("<unknown>").bold(),
                            device.interface_number(),
                            device.usage_page(),
                        );

                        if let Ok(dev) = device.open_device(&hidapi) {
                            println!("Writing data to device...");

                            let buf = util::parse_hex_vec(&data)?;

                            println!("[{}]", HexSlice::new(&buf));

                            dev.write(&buf)?;
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

        Subcommands::RunTests {
            device: device_index,
        } => {
            if device_index < 255 - 4 {
                // create the one and only hidapi instance
                match hidapi::HidApi::new() {
                    Ok(hidapi) => {
                        if let Some((index, device)) =
                            hidapi.device_list().enumerate().nth(device_index)
                        {
                            println!(
                                "Index: {}: ID: {:x}:{:x} {}/{} iface: {}:{:x}",
                                format!("{index:02}").bold(),
                                device.vendor_id(),
                                device.product_id(),
                                device.manufacturer_string().unwrap_or("<unknown>").bold(),
                                device.product_string().unwrap_or("<unknown>").bold(),
                                device.interface_number(),
                                device.usage_page(),
                            );

                            if let Ok(dev) = device.open_device(&hidapi) {
                                let hwdev = hwdevices::bind_device(
                                    dev,
                                    &hidapi,
                                    device.vendor_id(),
                                    device.product_id(),
                                )?;

                                hwdev.send_init_sequence()?;
                                hwdev.send_test_pattern()?;
                            } else {
                                error!("Could not open the device, is the device in use?");
                            }
                        }
                    }

                    Err(_) => {
                        error!("Could not open HIDAPI");
                    }
                }
            } else {
                // test serial LED devices
                let device_file = format!("/dev/ttyACM{}", 255 - device_index);

                println!(
                    "Index: {}: Device file: {}",
                    device_index,
                    device_file.bold()
                );

                match hwdevices::CustomSerialLeds::bind(&device_file) {
                    Ok(mut device) => {
                        device.send_init_sequence()?;

                        let mut cntr = 0;
                        loop {
                            if QUIT.load(Ordering::SeqCst) {
                                break;
                            }

                            let led_map = [RGBA {
                                r: cntr % 255,
                                g: cntr % 255,
                                b: cntr % 255,
                                a: cntr % 255,
                            }; 0x50];

                            if let Err(e) = device.send_led_map(&led_map) {
                                error!("{}", e);
                            }

                            if cntr == 255 {
                                cntr = 0;
                            } else {
                                cntr += 1;
                            }
                        }

                        device.send_led_off_sequence()?;
                    }

                    Err(e) => {
                        tracing::error!("Could not bind the device: {}", e);
                    }
                }
            }
        }

        Subcommands::Utils { command } => match command {
            UtilsSubcommands::ReverseCrc8 { data } => {
                let mut result = Vec::new();
                let mut buf = Vec::new();

                for s in data.iter() {
                    buf.push(util::parse_hex_vec(s)?);
                }

                println!("Performing exhaustive search...");

                for init in 0x00..=0xff {
                    for poly in 0x00..=0xff {
                        result.push((init, poly));
                    }
                }

                for b in buf.iter() {
                    if opts.verbose > 0 {
                        println!("{buf:?}");
                    }

                    if QUIT.load(Ordering::SeqCst) {
                        break;
                    }

                    for init in 0x00..0xff {
                        for poly in 0x00..0xff {
                            if opts.verbose > 0 {
                                println!("Processing: init: 0x{init:02x}, Poly: 0x{poly:02x}");
                            }

                            let crc8 = util::crc8_slow_with_poly(&b[1..], init, poly);

                            if crc8 == b[0] {
                                if opts.verbose > 0 {
                                    println!("Found a match!");
                                }

                                let params = util::find_crc8_from_params(crc8, &b[1..], &result);
                                if !params.is_empty() {
                                    result = params;
                                }

                                if opts.verbose > 1 {
                                    println!("{result:?}");
                                }
                            }
                        }
                    }
                }

                if !QUIT.load(Ordering::SeqCst) {
                    println!("Search completed\n");

                    result.sort_unstable();

                    if result.is_empty() {
                        println!("No matches found");
                    } else {
                        println!("The following combinations were found:");

                        for e in result.iter() {
                            println!("Init: 0x{:02x}, Poly: 0x{:02x}", e.0, e.1);
                        }
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
    }

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
            // initialize logging
            use tracing_subscriber::prelude::*;
            use tracing_subscriber::util::SubscriberInitExt;

            let console_layer = console_subscriber::ConsoleLayer::builder()
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
