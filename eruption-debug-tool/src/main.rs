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

use clap::Clap;
use color_eyre;
use colored::*;
use log::*;
use std::time::Instant;
use std::{env, thread};
use std::{path::PathBuf, time::Duration};

mod constants;
mod hwdevices;
mod util;

use util::{DeviceState, HexSlice};

type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum MainError {
    #[error("Unknown error: {description}")]
    UnknownError { description: String },
}

/// Supported command line arguments
#[derive(Debug, Clap)]
#[clap(
    version = env!("CARGO_PKG_VERSION"),
    author = "X3n0m0rph59 <x3n0m0rph59@gmail.com>",
    about = "A CLI utility to debug USB devices",
)]
pub struct Options {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, parse(from_occurrences))]
    verbose: u8,

    #[clap(subcommand)]
    command: Subcommands,
}

// Subcommands
#[derive(Debug, Clap)]
pub enum Subcommands {
    /// List available devices, use this first to find out the index of the device to use
    List,

    /// Generate a report of the specified device
    Report {
        /// The index of the device, can be found with the list subcommand
        device: usize,
    },

    /// Dump a trace of events originating from the specified device
    Trace {
        /// The index of the device, can be found with the list subcommand
        device: usize,
    },

    /// Read out the device state and show differences to previous state (May hang the device)
    StateDiff {
        /// The index of the device, can be found with the list subcommand
        device: usize,
    },

    /// Read a single USB HID feature report from device
    Read {
        /// The index of the device, can be found with the list subcommand
        device: usize,

        /// ID of the USB HID report
        #[clap(parse(try_from_str = util::parse_report_id))]
        report_id: u8,

        /// Length in bytes to read
        length: usize,
    },

    /// Send a single USB HID feature report to device (dangerous)
    Write {
        /// The index of the device, can be found with the list subcommand
        device: usize,

        /// Hex bytes e.g.: [0x09, 0x00, 0x1f]
        data: String,
    },

    /// Send test data to a device (dangerous)
    Test {
        /// The index of the device, can be found with the list subcommand
        device: usize,
    },
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

#[tokio::main]
pub async fn main() -> std::result::Result<(), eyre::Error> {
    color_eyre::install()?;

    // if unsafe { libc::isatty(0) != 0 } {
    //     print_header();
    // }

    // initialize logging
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG_OVERRIDE", "info");
        pretty_env_logger::init_custom_env("RUST_LOG_OVERRIDE");
    } else {
        pretty_env_logger::init();
    }

    let opts = Options::parse();
    match opts.command {
        Subcommands::List => {
            debug!("Enumerating devices...");

            // create the one and only hidapi instance
            match hidapi::HidApi::new() {
                Ok(hidapi) => {
                    for (index, device) in hidapi.device_list().enumerate() {
                        info!(
                            "#{}: USB ID: {:x}:{:x} {}/{} Subdev: {}",
                            format!("{:02}", index).bold(),
                            device.vendor_id(),
                            device.product_id(),
                            device
                                .manufacturer_string()
                                .unwrap_or_else(|| "<unknown>")
                                .bold(),
                            device
                                .product_string()
                                .unwrap_or_else(|| "<unknown>")
                                .bold(),
                            device.interface_number()
                        )
                    }
                }

                Err(_) => {
                    error!("Could not open HIDAPI");
                }
            };

            debug!("Enumeration complete");
        }

        Subcommands::Report { device } => {
            info!("-- Start of report --");

            // create the one and only hidapi instance
            match hidapi::HidApi::new() {
                Ok(hidapi) => {
                    if let Some((index, device)) = hidapi.device_list().enumerate().nth(device) {
                        info!(
                            "#{}: USB ID: {:x}:{:x} {}/{} Subdev: {}",
                            format!("{:02}", index).bold(),
                            device.vendor_id(),
                            device.product_id(),
                            device
                                .manufacturer_string()
                                .unwrap_or_else(|| "<unknown>")
                                .bold(),
                            device
                                .product_string()
                                .unwrap_or_else(|| "<unknown>")
                                .bold(),
                            device.interface_number()
                        );

                        if let Ok(dev) = device.open_device(&hidapi) {
                            for i in 0..256 {
                                if let Ok(result) = dev.get_indexed_string(i) {
                                    if let Some(s) = result {
                                        info!("{:03}: {}", i, s);
                                    }
                                } else {
                                    if opts.verbose > 0 {
                                        error!("{:03}: {}", i, "Failed");
                                    }
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

            info!("-- End of report --");
        }

        Subcommands::Trace { device } => {
            info!("-- Start of trace --");

            // create the one and only hidapi instance
            match hidapi::HidApi::new() {
                Ok(hidapi) => {
                    if let Some((index, device)) = hidapi.device_list().enumerate().nth(device) {
                        info!(
                            "#{}: USB ID: {:x}:{:x} {}/{} Subdev: {}",
                            format!("{:02}", index).bold(),
                            device.vendor_id(),
                            device.product_id(),
                            device
                                .manufacturer_string()
                                .unwrap_or_else(|| "<unknown>")
                                .bold(),
                            device
                                .product_string()
                                .unwrap_or_else(|| "<unknown>")
                                .bold(),
                            device.interface_number()
                        );

                        if let Ok(dev) = device.open_device(&hidapi) {
                            info!("Initializing...");
                            let mut buf: [u8; 256] = [0; 256];
                            buf[0] = 0x0f;

                            let bytes_read = dev.get_feature_report(&mut buf)?;

                            info!("{} bytes", bytes_read);
                            hexdump::hexdump_iter(&buf).for_each(|s| info!("  {}", s));

                            // wait to settle
                            thread::sleep(Duration::from_millis(1000));

                            let mut buf: [u8; 8] = [0; 8];
                            buf[0] = 0x04;

                            let bytes_read = dev.get_feature_report(&mut buf)?;

                            info!("{} bytes", bytes_read);
                            hexdump::hexdump_iter(&buf).for_each(|s| info!("  {}", s));

                            // wait to settle
                            thread::sleep(Duration::from_millis(1000));

                            info!("Entering loop:");

                            loop {
                                let mut buf: [u8; 16] = [0; 16];
                                buf[0] = 0xff;

                                let bytes_read = dev.read(&mut buf)?;

                                info!("{:?}: {} bytes", Instant::now(), bytes_read);
                                hexdump::hexdump_iter(&buf).for_each(|s| info!("  {}", s));
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

            info!("-- End of trace --");
        }

        Subcommands::StateDiff {
            device: device_index,
        } => {
            info!("Reading data from device...");

            // create the one and only hidapi instance
            match hidapi::HidApi::new() {
                Ok(hidapi) => {
                    if let Some((index, device)) =
                        hidapi.device_list().enumerate().nth(device_index)
                    {
                        info!(
                            "#{}: USB ID: {:x}:{:x} {}/{} Subdev: {}",
                            format!("{:02}", index).bold(),
                            device.vendor_id(),
                            device.product_id(),
                            device
                                .manufacturer_string()
                                .unwrap_or_else(|| "<unknown>")
                                .bold(),
                            device
                                .product_string()
                                .unwrap_or_else(|| "<unknown>")
                                .bold(),
                            device.interface_number()
                        );

                        if let Ok(dev) = device.open_device(&hidapi) {
                            let path = PathBuf::from("/tmp/eruption-debug-tool.data");

                            let mut data_store = util::load_data_from_file(&path)?;
                            let mut state = DeviceState::new(
                                device.serial_number().unwrap_or_else(|| "<unknown>"),
                                device.product_string().unwrap_or_else(|| "<unknown>"),
                            );

                            for report_id in 0..=255 {
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

                            util::print_diff(&state, &data_store);

                            data_store.push(state);
                            util::save_data_to_file(&path, &data_store)?;
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
            info!("Reading data from device...");

            // create the one and only hidapi instance
            match hidapi::HidApi::new() {
                Ok(hidapi) => {
                    if let Some((index, device)) =
                        hidapi.device_list().enumerate().nth(device_index)
                    {
                        info!(
                            "#{}: USB ID: {:x}:{:x} {}/{} Subdev: {}",
                            format!("{:02}", index).bold(),
                            device.vendor_id(),
                            device.product_id(),
                            device
                                .manufacturer_string()
                                .unwrap_or_else(|| "<unknown>")
                                .bold(),
                            device
                                .product_string()
                                .unwrap_or_else(|| "<unknown>")
                                .bold(),
                            device.interface_number()
                        );

                        if let Ok(dev) = device.open_device(&hidapi) {
                            let mut buf = Vec::new();

                            buf.resize(length + 1, 0x00);
                            buf[0] = report_id;

                            dev.get_feature_report(&mut buf)?;

                            info!("[{}]", HexSlice::new(&buf));
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
            info!("Writing data to device...");

            // create the one and only hidapi instance
            match hidapi::HidApi::new() {
                Ok(hidapi) => {
                    if let Some((index, device)) =
                        hidapi.device_list().enumerate().nth(device_index)
                    {
                        info!(
                            "#{}: USB ID: {:x}:{:x} {}/{} Subdev: {}",
                            format!("{:02}", index).bold(),
                            device.vendor_id(),
                            device.product_id(),
                            device
                                .manufacturer_string()
                                .unwrap_or_else(|| "<unknown>")
                                .bold(),
                            device
                                .product_string()
                                .unwrap_or_else(|| "<unknown>")
                                .bold(),
                            device.interface_number()
                        );

                        if let Ok(dev) = device.open_device(&hidapi) {
                            let buf = util::parse_hex_vec(&data)?;

                            info!("[{}]", HexSlice::new(&buf));

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

        Subcommands::Test {
            device: device_index,
        } => {
            warn!("Sending test data to device...");

            // create the one and only hidapi instance
            match hidapi::HidApi::new() {
                Ok(hidapi) => {
                    if let Some((index, device)) =
                        hidapi.device_list().enumerate().nth(device_index)
                    {
                        info!(
                            "#{}: USB ID: {:x}:{:x} {}/{} Subdev: {}",
                            format!("{:02}", index).bold(),
                            device.vendor_id(),
                            device.product_id(),
                            device
                                .manufacturer_string()
                                .unwrap_or_else(|| "<unknown>")
                                .bold(),
                            device
                                .product_string()
                                .unwrap_or_else(|| "<unknown>")
                                .bold(),
                            device.interface_number()
                        );

                        if let Ok(dev) = device.open_device(&hidapi) {
                            fn settle(dev: &hidapi::HidDevice) -> Result<()> {
                                let mut buf: [u8; 8] = [0; 8];
                                buf[0] = 0x04;
                                let _result = dev.get_feature_report(&mut buf)?;

                                info!("Settle: {:x?}", &buf);
                                thread::sleep(Duration::from_millis(250));

                                Ok(())
                            }

                            fn init(dev: &hidapi::HidDevice) -> Result<()> {
                                let buf: [u8; 76] = [
                                    0x9, 0x4c, 0x6a, 0x65, 0x9, 0x47, 0x3, 0x0, 0x0, 0x10, 0x0,
                                    0x18, 0x0, 0x20, 0x0, 0x40, 0x0, 0x8, 0x0, 0x10, 0x0, 0x18,
                                    0x0, 0x20, 0x0, 0x40, 0x0, 0x0, 0x2, 0x3, 0x4, 0x6, 0xff, 0xf,
                                    0x0, 0x0, 0xff, 0xff, 0xc5, 0xb, 0xdc, 0xff, 0xff, 0xe6, 0x8c,
                                    0x0, 0x15, 0xff, 0xf, 0xf, 0xff, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
                                    0x0, 0x0, 0x0, 0x0, 0xd8, 0xb, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
                                    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x8c,
                                ];

                                // [
                                //     /*0x9, 0x49, 0x6a, 0x65,*/ 0x9, 0x47, 0x3, 0x0, 0x0, 0x10,
                                //     0x0, 0x18, 0x0, 0x20, 0x0, 0x40, 0x0, 0x8, 0x0, 0x10, 0x0,
                                //     0x18, 0x0, 0x1, 0x1, 0x1, 0xff, r, g, b, 0x50, 0x0, 0xff, 0xf,
                                //     0x50, 0x0, 0x15, 0xff, 0xf, 0xf, 0xff, 0x15, 0xff, 0xf, 0xf,
                                //     0xff, 0x15, 0xff, 0xf, 0xf, 0xff, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
                                //     0x0, 0x0, 0x0, 0x0, 0x59, 0xa, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
                                //     0x0, 0x0, 0x0, 0x0,
                                // ];

                                info!("{:x?}", &buf[4..]);

                                // 37-39

                                // [9, 4c, 6a, 65, 9, 47, 3, 0, 0, 10, 0, 18, 0, 20, 0, 40, 0, 8, 0, 10, 0, 18, 0, 20, 0, 40, 0, 0, 2, 3, 4, 6, ff, f, 0, 0, ff, ff, 51, ff, 0, ff, ff, e6, 8c, 0, 15, ff, f, f, ff, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 7b, b, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8c]
                                // [9, 4c, 6a, 65, 9, 47, 3, 0, 0, 10, 0, 18, 0, 20, 0, 40, 0, 8, 0, 10, 0, 18, 0, 20, 0, 40, 0, 0, 2, 3, 4, 6, ff, f, 0, 0, ff, ff, c5, b, dc, ff, ff, e6, 8c, 0, 15, ff, f, f, ff, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, d8, b, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 8c]

                                dev.send_feature_report(&buf[4..])?;

                                Ok(())
                            }

                            fn set_led_color(
                                dev: &hidapi::HidDevice,
                                r: u8,
                                g: u8,
                                b: u8,
                            ) -> Result<()> {
                                let mut buf = vec![0x09];
                                buf.resize(76, 0x00);

                                dev.get_feature_report(&mut buf)?;

                                // settle(&dev)?;

                                info!("{:x?}", buf);

                                /*
                                let mut buf = buf[4..].to_vec();
                                // buf.resize(32, 0x00);

                                const base: usize = 1;
                                buf[0] = 0x06;
                                // buf[base + 1] = 0x1f;
                                buf[base + 1] = 0x00; // profile
                                buf[base + 23] = 0x01;
                                buf[base + 25] = 0xff;
                                buf[base + 26] = r;
                                buf[base + 27] = g;
                                buf[base + 28] = b;
                                buf[base + 29] = 0x01;
                                buf[base + 30] = 0x00;


                                */

                                // let brightness = 80;

                                let buf: [u8; 77] = [
                                    0x9, 0x4c, 0x6a, 0x65, 0x9, 0x47, 0x3, 0x0, 0x0, 0x10, 0x0,
                                    0x18, 0x0, 0x20, 0x0, 0x40, 0x0, 0x8, 0x0, 0x10, 0x0, 0x18,
                                    0x0, 0x20, 0x0, 0x40, 0x0, 0x1, 0x2, 0x3, 0x4, 0x6, 0xff, 0xf,
                                    0x0, 0x0, 0xff, 0xff, r, g, b, 0xff, 0xff, 0xe6, 0x8c, 0x0,
                                    0x15, 0xff, 0xf, 0xf, 0xff, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
                                    0x0, 0x0, 0x0, 0xd8, 0xb, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
                                    0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
                                ];

                                // [
                                //     /*0x9, 0x49, 0x6a, 0x65,*/ 0x9, 0x47, 0x3, 0x0, 0x0, 0x10,
                                //     0x0, 0x18, 0x0, 0x20, 0x0, 0x40, 0x0, 0x8, 0x0, 0x10, 0x0,
                                //     0x18, 0x0, 0x1, 0x1, 0x1, 0xff, r, g, b, 0x50, 0x0, 0xff, 0xf,
                                //     0x50, 0x0, 0x15, 0xff, 0xf, 0xf, 0xff, 0x15, 0xff, 0xf, 0xf,
                                //     0xff, 0x15, 0xff, 0xf, 0xf, 0xff, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
                                //     0x0, 0x0, 0x0, 0x0, 0x59, 0xa, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
                                //     0x0, 0x0, 0x0, 0x0,
                                // ];

                                info!("{:x?}", &buf[4..]);

                                dev.send_feature_report(&buf[4..])?;

                                settle(&dev)?;

                                Ok(())
                            }

                            init(&dev)?;
                            settle(&dev)?;
                            set_led_color(&dev, 59, 245, 0)?;
                        } else {
                            error!("Could not open the device, is the device in use?");
                        }
                    }
                }

                Err(_) => {
                    error!("Could not open HIDAPI");
                }
            };

            info!("-- End of test --");
        }
    };

    Ok(())
}
