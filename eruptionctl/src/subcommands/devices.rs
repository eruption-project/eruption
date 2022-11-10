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

    Copyright (c) 2019-2022, The Eruption Development Team
*/

use std::collections::HashMap;
use std::thread;
use std::time::Duration;

use color_eyre::Help;
use colored::*;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, CellAlignment, ContentArrangement, Table};
use eyre::Context;
use std::sync::atomic::Ordering;

use crate::dbus_client::dbus_system_bus;
use crate::device;
use crate::tr;

type Result<T> = std::result::Result<T, eyre::Error>;

/// Sub-commands of the "devices" command
#[derive(Debug, clap::Parser)]
pub enum DevicesSubcommands {
    /// List connected devices and their indices (run this first)
    #[clap(display_order = 0)]
    List,

    /// Get information about a specific device
    #[clap(display_order = 1)]
    Info { device: String },

    /// Get status of a specific device
    #[clap(display_order = 2)]
    Status { device: String },

    /// Get or set the device specific brightness of the LEDs
    #[clap(display_order = 3)]
    Brightness {
        device: String,
        brightness: Option<i64>,
    },

    /// Get or set the current profile (applicable for some devices)
    #[clap(display_order = 4)]
    Profile {
        device: String,
        profile: Option<i32>,
    },

    /// Get or set the DPI parameter (applicable for some mice)
    #[clap(display_order = 5)]
    Dpi { device: String, dpi: Option<i32> },

    /// Get or set the bus poll rate
    #[clap(display_order = 6)]
    Rate { device: String, rate: Option<i32> },

    /// Get or set the debounce parameter (applicable for some mice)
    #[clap(display_order = 7)]
    Debounce {
        device: String,
        enable: Option<bool>,
    },

    /// Get or set the DCU parameter (applicable for some mice)
    #[clap(display_order = 8)]
    Distance { device: String, param: Option<i32> },

    /// Get or set the angle-snapping parameter (applicable for some mice)
    #[clap(display_order = 9)]
    AngleSnapping {
        device: String,
        enable: Option<bool>,
    },
}

pub async fn handle_command(command: DevicesSubcommands) -> Result<()> {
    match command {
        DevicesSubcommands::List => list_command().await,
        DevicesSubcommands::Info { device } => info_command(device).await,
        DevicesSubcommands::Status { device } => status_command(device).await,
        DevicesSubcommands::Profile { device, profile } => profile_command(device, profile).await,
        DevicesSubcommands::Dpi { device, dpi } => dpi_command(device, dpi).await,
        DevicesSubcommands::Rate { device, rate } => rate_command(device, rate).await,
        DevicesSubcommands::Distance { device, param } => distance_command(device, param).await,
        DevicesSubcommands::AngleSnapping { device, enable } => {
            angle_snapping_command(device, enable).await
        }
        DevicesSubcommands::Debounce { device, enable } => debounce_command(device, enable).await,
        DevicesSubcommands::Brightness { device, brightness } => {
            brightness_command(device, brightness).await
        }
    }
}

async fn list_command() -> Result<()> {
    let verbose = crate::VERBOSE.load(Ordering::SeqCst);
    let mut base_index = 0;

    let (keyboards, mice, misc) = get_devices()
        .await
        .wrap_err("Could not connect to the Eruption daemon")
        .suggestion("Please verify that the Eruption daemon is running")?;

    if verbose > 0 {
        println!(
            "
Use the `eruptionctl devices list` sub-command to find out the index of the device that
you want to operate on. All the other device-related commands require a device index.

Examples:

Set the brightness of the first connected keyboard to 80 percent:

$ eruptionctl devices brightness 0 80


Query the DPI configuration of the first connected mouse (second device):

$ eruptionctl devices dpi 1

"
        );
    }

    println!("{}\n", tr!("dumping-devices").bold());

    println!("{}", tr!("keyboard-devices"));

    if keyboards.is_empty() {
        println!("{}", "<No supported devices detected>\n".italic());
    } else {
        for (_index, dev) in keyboards.iter().enumerate() {
            if verbose > 0 {
                println!(
                    "Index: {}: ID: {}:{} {} {}",
                    format!("{:02}", base_index).bold(),
                    format!("{:04x}", dev.0),
                    format!("{:04x}", dev.1),
                    device::get_device_make(dev.0, dev.1)
                        .unwrap_or("<unknown make>")
                        .bold(),
                    device::get_device_model(dev.0, dev.1)
                        .unwrap_or("<unknown model>")
                        .bold()
                );
            } else {
                println!(
                    "{}: {} {}",
                    format!("{:02}", base_index).bold(),
                    device::get_device_make(dev.0, dev.1)
                        .unwrap_or("<unknown make>")
                        .bold(),
                    device::get_device_model(dev.0, dev.1)
                        .unwrap_or("<unknown model>")
                        .bold()
                );
            }

            base_index += 1;
        }
    }

    println!("\n{}", tr!("mouse-devices"));

    if mice.is_empty() {
        println!("{}", "<No supported devices detected>\n".italic());
    } else {
        for (_index, dev) in mice.iter().enumerate() {
            if verbose > 0 {
                println!(
                    "Index: {}: ID: {}:{} {} {}",
                    format!("{:02}", base_index).bold(),
                    format!("{:04x}", dev.0),
                    format!("{:04x}", dev.1),
                    device::get_device_make(dev.0, dev.1)
                        .unwrap_or("<unknown make>")
                        .bold(),
                    device::get_device_model(dev.0, dev.1)
                        .unwrap_or("<unknown model>")
                        .bold()
                );
            } else {
                println!(
                    "{}: {} {}",
                    format!("{:02}", base_index).bold(),
                    device::get_device_make(dev.0, dev.1)
                        .unwrap_or("<unknown make>")
                        .bold(),
                    device::get_device_model(dev.0, dev.1)
                        .unwrap_or("<unknown model>")
                        .bold()
                );
            }

            base_index += 1;
        }
    }

    println!("\n{}", tr!("misc-devices"));

    if misc.is_empty() {
        println!("{}", "<No supported devices detected>\n".italic());
    } else {
        for (_index, dev) in misc.iter().enumerate() {
            if verbose > 0 {
                println!(
                    "Index: {}: ID: {}:{} {} {}",
                    format!("{:02}", base_index).bold(),
                    format!("{:04x}", dev.0),
                    format!("{:04x}", dev.1),
                    device::get_device_make(dev.0, dev.1)
                        .unwrap_or("<unknown make>")
                        .bold(),
                    device::get_device_model(dev.0, dev.1)
                        .unwrap_or("<unknown model>")
                        .bold()
                );
            } else {
                println!(
                    "{}: {} {}",
                    format!("{:02}", base_index).bold(),
                    device::get_device_make(dev.0, dev.1)
                        .unwrap_or("<unknown make>")
                        .bold(),
                    device::get_device_model(dev.0, dev.1)
                        .unwrap_or("<unknown model>")
                        .bold()
                );
            }

            base_index += 1;
        }
    }

    Ok(())
}

async fn info_command(device: String) -> Result<()> {
    let device = device.parse::<u64>()?;

    print_device_header(device)
        .await
        .wrap_err("Could not connect to the Eruption daemon")
        .suggestion("Please verify that the Eruption daemon is running")?;

    let result = get_device_config(device, "info").await?;

    println!("{}", format!("{}", result.bold()));

    Ok(())
}

async fn status_command(device: String) -> Result<()> {
    let device = device.parse::<u64>()?;

    print_device_header(device)
        .await
        .wrap_err("Could not connect to the Eruption daemon")
        .suggestion("Please verify that the Eruption daemon is running")?;

    let term = console::Term::stdout();

    // stores how many lines we printed in the previous iteration
    let mut prev = 0;

    loop {
        let result = get_device_status(device).await?;

        let mut table = Table::new();
        table
            .load_preset(UTF8_FULL)
            .apply_modifier(UTF8_ROUND_CORNERS)
            .set_content_arrangement(ContentArrangement::Dynamic)
            .set_width(40)
            .set_header(vec!["Parameter", "Value"]);

        // counts the number of lines that we printed
        let mut cntr = 3;

        let mut v = result.iter().collect::<Vec<(&String, &String)>>();
        v.sort_by_key(|&v| v.0);

        v.iter().for_each(|(k, v)| {
            table.add_row(vec![
                Cell::new(k.to_owned()).set_alignment(CellAlignment::Left),
                Cell::new(v.to_owned()).set_alignment(CellAlignment::Right),
            ]);

            cntr += 2;
        });

        // empty table requires special handling
        if cntr <= 3 {
            cntr = 4
        }

        term.clear_last_lines(prev)?;
        prev = cntr;

        println!("{}", table);

        if !crate::REPEAT.load(Ordering::SeqCst) || crate::QUIT.load(Ordering::SeqCst) {
            break;
        }

        thread::sleep(Duration::from_millis(250));
    }

    Ok(())
}

async fn profile_command(device: String, profile: Option<i32>) -> Result<()> {
    let device = device.parse::<u64>()?;

    print_device_header(device)
        .await
        .wrap_err("Could not connect to the Eruption daemon")
        .suggestion("Please verify that the Eruption daemon is running")?;

    if let Some(profile) = profile {
        let value = &format!("{}", profile);

        set_device_config(device, "profile", value).await?;
    } else {
        let result = get_device_config(device, "profile").await?;

        println!("{}", format!("Current profile: {}", result.bold()));
    }

    Ok(())
}

async fn dpi_command(device: String, dpi: Option<i32>) -> Result<()> {
    let device = device.parse::<u64>()?;

    print_device_header(device)
        .await
        .wrap_err("Could not connect to the Eruption daemon")
        .suggestion("Please verify that the Eruption daemon is running")?;

    if let Some(dpi) = dpi {
        let value = &format!("{}", dpi);

        set_device_config(device, "dpi", value).await?
    } else {
        let result = get_device_config(device, "dpi").await?;

        println!("{}", format!("DPI config: {}", result.bold()));
    }

    Ok(())
}

async fn rate_command(device: String, rate: Option<i32>) -> Result<()> {
    let device = device.parse::<u64>()?;

    print_device_header(device)
        .await
        .wrap_err("Could not connect to the Eruption daemon")
        .suggestion("Please verify that the Eruption daemon is running")?;

    if let Some(rate) = rate {
        let value = &format!("{}", rate);

        set_device_config(device, "rate", value).await?
    } else {
        let result = get_device_config(device, "rate").await?;

        println!("{}", format!("Poll rate: {}", result.bold()));
    }

    Ok(())
}

async fn distance_command(device: String, param: Option<i32>) -> Result<()> {
    let device = device.parse::<u64>()?;

    print_device_header(device)
        .await
        .wrap_err("Could not connect to the Eruption daemon")
        .suggestion("Please verify that the Eruption daemon is running")?;

    if let Some(param) = param {
        let value = &format!("{}", param);

        set_device_config(device, "dcu", value).await?
    } else {
        let result = get_device_config(device, "dcu").await?;

        println!("{}", format!("DCU config: {}", result.bold()));
    }

    Ok(())
}

async fn angle_snapping_command(device: String, enable: Option<bool>) -> Result<()> {
    let device = device.parse::<u64>()?;

    print_device_header(device)
        .await
        .wrap_err("Could not connect to the Eruption daemon")
        .suggestion("Please verify that the Eruption daemon is running")?;

    if let Some(enable) = enable {
        let value = &format!("{}", enable);

        set_device_config(device, "angle-snapping", value).await?
    } else {
        let result = get_device_config(device, "angle-snapping").await?;

        println!("{}", format!("Angle-snapping: {}", result.bold()));
    }

    Ok(())
}

async fn debounce_command(device: String, enable: Option<bool>) -> Result<()> {
    let device = device.parse::<u64>()?;

    print_device_header(device)
        .await
        .wrap_err("Could not connect to the Eruption daemon")
        .suggestion("Please verify that the Eruption daemon is running")?;

    if let Some(enable) = enable {
        let value = &format!("{}", enable);

        set_device_config(device, "debounce", value).await?
    } else {
        let result = get_device_config(device, "debounce").await?;

        println!("{}", format!("Debounce: {}", result.bold()));
    }

    Ok(())
}

async fn brightness_command(device: String, brightness: Option<i64>) -> Result<()> {
    let device = device.parse::<u64>()?;

    print_device_header(device)
        .await
        .wrap_err("Could not connect to the Eruption daemon")
        .suggestion("Please verify that the Eruption daemon is running")?;

    if let Some(brightness) = brightness {
        let value = &format!("{}", brightness);

        set_device_config(device, "brightness", value).await?
    } else {
        let result = get_device_config(device, "brightness").await?;

        println!("{}", format!("Device brightness: {}%", result.bold()));
    }

    Ok(())
}

/// Enumerate all available devices
async fn get_devices() -> Result<(Vec<(u16, u16)>, Vec<(u16, u16)>, Vec<(u16, u16)>)> {
    let ((keyboards, mice, misc),): ((Vec<(u16, u16)>, Vec<(u16, u16)>, Vec<(u16, u16)>),) =
        dbus_system_bus("/org/eruption/devices")
            .await?
            .method_call("org.eruption.Device", "GetManagedDevices", ())
            .await?;

    Ok((keyboards, mice, misc))
}

/// Get a device specific config param
async fn get_device_config(device: u64, param: &str) -> Result<String> {
    let (result,): (String,) = dbus_system_bus("/org/eruption/devices")
        .await?
        .method_call(
            "org.eruption.Device",
            "GetDeviceConfig",
            (device, param.to_owned()),
        )
        .await?;

    Ok(result)
}

/// Get device specific status
async fn get_device_status(device: u64) -> Result<HashMap<String, String>> {
    let (status,): (String,) = dbus_system_bus("/org/eruption/devices")
        .await?
        .method_call("org.eruption.Device", "GetDeviceStatus", (device,))
        .await?;

    let result: HashMap<String, String> = serde_json::from_str(&status)?;

    Ok(result)
}

/// Set a device specific config param
async fn set_device_config(device: u64, param: &str, value: &str) -> Result<()> {
    let (_result,): (bool,) = dbus_system_bus("/org/eruption/devices")
        .await?
        .method_call(
            "org.eruption.Device",
            "SetDeviceConfig",
            (device, param.to_owned(), value.to_owned()),
        )
        .await?;

    Ok(())
}

async fn print_device_header(device: u64) -> Result<()> {
    let mut base_index = 0;

    let (keyboards, mice, misc) = get_devices().await?;

    print!("Selected device: ");

    if !keyboards.is_empty() {
        for (_index, dev) in keyboards.iter().enumerate() {
            if base_index == device {
                println!(
                    // "{}: ID: {}:{} {} {}",
                    // format!("{:02}", base_index).bold(),
                    // format!("{:04x}", dev.0),
                    // format!("{:04x}", dev.1),
                    "{} {}",
                    device::get_device_make(dev.0, dev.1)
                        .unwrap_or("<unknown make>")
                        .bold(),
                    device::get_device_model(dev.0, dev.1)
                        .unwrap_or("<unknown model>")
                        .bold(),
                );
            }

            base_index += 1;
        }
    }

    if !mice.is_empty() {
        for (_index, dev) in mice.iter().enumerate() {
            if base_index == device {
                println!(
                    // "{}: ID: {}:{} {} {}",
                    // format!("{:02}", base_index).bold(),
                    // format!("{:04x}", dev.0),
                    // format!("{:04x}", dev.1),
                    "{} {}",
                    device::get_device_make(dev.0, dev.1)
                        .unwrap_or("<unknown make>")
                        .bold(),
                    device::get_device_model(dev.0, dev.1)
                        .unwrap_or("<unknown model>")
                        .bold(),
                );
            }

            base_index += 1;
        }
    }

    if !misc.is_empty() {
        for (_index, dev) in misc.iter().enumerate() {
            if base_index == device {
                println!(
                    // "{}: ID: {}:{} {} {}",
                    // format!("{:02}", base_index).bold(),
                    // format!("{:04x}", dev.0),
                    // format!("{:04x}", dev.1),
                    "{} {}",
                    device::get_device_make(dev.0, dev.1)
                        .unwrap_or("<unknown make>")
                        .bold(),
                    device::get_device_model(dev.0, dev.1)
                        .unwrap_or("<unknown model>")
                        .bold(),
                );
            }

            base_index += 1;
        }
    }

    Ok(())
}
