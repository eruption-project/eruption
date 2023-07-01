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

use std::sync::atomic::Ordering;
use std::time::Duration;

use color_eyre::Help;
use colored::Colorize;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, CellAlignment, ContentArrangement, Row, Table};
use dbus::blocking::Connection;
use eyre::Context;

use crate::dbus_client::{canvas, dbus_system_bus};
use crate::device::get_device_info;
use crate::{constants, tr};

type Result<T> = std::result::Result<T, eyre::Error>;

/// Assign an [Option<T>] to $dst if the optional value is not [None]
macro_rules! assign_conditionally {
    ($opt:ident, $dst:expr) => {
        if let Some(val) = $opt {
            $dst = val;
        }
    };
}

/// Sub-commands of the "zones" command
#[derive(Debug, clap::Parser)]
pub enum ZonesSubcommands {
    /// List all allocated zones
    List,

    /// Get the allocated zone of a specific device
    Get { device: String },

    /// Set the allocated zone of a specific device
    Set {
        device: String,
        x: Option<i32>,
        y: Option<i32>,
        width: Option<i32>,
        height: Option<i32>,
        enabled: Option<bool>,
    },

    /// Enable rendering to the zone of a specific device
    Enable { device: String },

    /// Disable rendering to the zone of a specific device
    Disable { device: String },
}

/// Represents a rectangular zone on the canvas that is allocated to a device
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Zone {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub enabled: bool,
}

impl dbus::arg::Arg for Zone {
    const ARG_TYPE: dbus::arg::ArgType = dbus::arg::ArgType::Struct;

    fn signature() -> dbus::Signature<'static> {
        dbus::Signature::from("(iiiib)")
    }
}

impl dbus::arg::Append for Zone {
    fn append_by_ref(&self, i: &mut dbus::arg::IterAppend) {
        i.append((self.x, self.y, self.width, self.height, self.enabled));
    }
}

pub async fn handle_command(command: ZonesSubcommands) -> Result<()> {
    match command {
        ZonesSubcommands::List => list_zones_command().await?,
        ZonesSubcommands::Get { device } => get_zone_command(&device).await?,
        ZonesSubcommands::Set {
            device,
            x,
            y,
            width,
            height,
            enabled,
        } => set_zone_command(&device, x, y, width, height, enabled).await?,
        ZonesSubcommands::Enable { device } => enable_zone_command(&device).await?,
        ZonesSubcommands::Disable { device } => disable_zone_command(&device).await?,
    }

    Ok(())
}

async fn list_zones_command() -> Result<()> {
    let verbose = crate::VERBOSE.load(Ordering::SeqCst);
    let mut base_index = 0;

    let (keyboards, mice, misc) = get_devices()
        .await
        .wrap_err("Could not connect to the Eruption daemon")
        .suggestion("Please verify that the Eruption daemon is running")?;

    let zones = get_zones().await?;

    println!("{}\n", tr!("dumping-devices").bold());

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);

    let row = Row::from(vec![
        Cell::new("#").set_alignment(CellAlignment::Right),
        Cell::new("Make"),
        Cell::new("Model"),
        Cell::new("x").set_alignment(CellAlignment::Right),
        Cell::new("y").set_alignment(CellAlignment::Right),
        Cell::new("width").set_alignment(CellAlignment::Right),
        Cell::new("height").set_alignment(CellAlignment::Right),
        Cell::new("Enabled").set_alignment(CellAlignment::Right),
    ]);
    table.set_header(row);

    if keyboards.is_empty() {
        println!("{}", "<No supported devices detected>\n".italic());
    } else {
        let row = Row::from(vec![Cell::new(tr!("keyboard-devices"))]);
        table.add_row_if(|_, _| verbose > 0, row);

        for (index, dev) in keyboards.iter().enumerate() {
            let device_info = get_device_info(dev.0, dev.1);

            if let Some((_device, zone)) = zones.iter().find(|e| e.0 == base_index as u64) {
                if let Some(device) = device_info {
                    let row = Row::from(vec![
                        Cell::new(format!("{base_index:02}"))
                            .add_attribute(Attribute::Bold)
                            .set_alignment(CellAlignment::Right),
                        Cell::new(device.make).add_attribute(Attribute::Bold),
                        Cell::new(device.model).add_attribute(Attribute::Bold),
                        Cell::new(zone.x)
                            .add_attribute(Attribute::Bold)
                            .set_alignment(CellAlignment::Right),
                        Cell::new(zone.y)
                            .add_attribute(Attribute::Bold)
                            .set_alignment(CellAlignment::Right),
                        Cell::new(zone.width)
                            .add_attribute(Attribute::Bold)
                            .set_alignment(CellAlignment::Right),
                        Cell::new(zone.height)
                            .add_attribute(Attribute::Bold)
                            .set_alignment(CellAlignment::Right),
                        Cell::new(zone.enabled).add_attribute(Attribute::Bold),
                    ]);
                    table.add_row(row);
                } else {
                    println!("{}: <Unknown device>", format!("{index:02}").bold());
                }
            } else if let Some(device) = device_info {
                let row = Row::from(vec![
                    Cell::new(format!("{base_index:02}"))
                        .add_attribute(Attribute::Bold)
                        .set_alignment(CellAlignment::Right),
                    Cell::new(device.make).add_attribute(Attribute::Bold),
                    Cell::new(device.model).add_attribute(Attribute::Bold),
                    Cell::new("n.a.")
                        .add_attribute(Attribute::Italic)
                        .set_alignment(CellAlignment::Right),
                    Cell::new("n.a.")
                        .add_attribute(Attribute::Italic)
                        .set_alignment(CellAlignment::Right),
                    Cell::new("n.a.")
                        .add_attribute(Attribute::Italic)
                        .set_alignment(CellAlignment::Right),
                    Cell::new("n.a.")
                        .add_attribute(Attribute::Italic)
                        .set_alignment(CellAlignment::Right),
                    Cell::new("n.a.").add_attribute(Attribute::Bold),
                ]);
                table.add_row(row);
            } else {
                println!("{}: <Unknown device>", format!("{index:02}").bold());
            }

            base_index += 1;
        }
    }

    if mice.is_empty() {
        println!("{}", "<No supported devices detected>\n".italic());
    } else {
        let row = Row::from(vec![Cell::new(tr!("mouse-devices"))]);
        table.add_row_if(|_, _| verbose > 0, row);

        for (index, dev) in mice.iter().enumerate() {
            let device_info = get_device_info(dev.0, dev.1);

            if let Some((_device, zone)) = zones.iter().find(|e| e.0 == base_index as u64) {
                if let Some(device) = device_info {
                    let row = Row::from(vec![
                        Cell::new(format!("{base_index:02}"))
                            .add_attribute(Attribute::Bold)
                            .set_alignment(CellAlignment::Right),
                        Cell::new(device.make).add_attribute(Attribute::Bold),
                        Cell::new(device.model).add_attribute(Attribute::Bold),
                        Cell::new(zone.x)
                            .add_attribute(Attribute::Bold)
                            .set_alignment(CellAlignment::Right),
                        Cell::new(zone.y)
                            .add_attribute(Attribute::Bold)
                            .set_alignment(CellAlignment::Right),
                        Cell::new(zone.width)
                            .add_attribute(Attribute::Bold)
                            .set_alignment(CellAlignment::Right),
                        Cell::new(zone.height)
                            .add_attribute(Attribute::Bold)
                            .set_alignment(CellAlignment::Right),
                        Cell::new(zone.enabled).add_attribute(Attribute::Bold),
                    ]);
                    table.add_row(row);
                } else {
                    println!("{}: <Unknown device>", format!("{index:02}").bold());
                }
            } else if let Some(device) = device_info {
                let row = Row::from(vec![
                    Cell::new(format!("{base_index:02}"))
                        .add_attribute(Attribute::Bold)
                        .set_alignment(CellAlignment::Right),
                    Cell::new(device.make).add_attribute(Attribute::Bold),
                    Cell::new(device.model).add_attribute(Attribute::Bold),
                    Cell::new("n.a.")
                        .add_attribute(Attribute::Italic)
                        .set_alignment(CellAlignment::Right),
                    Cell::new("n.a.")
                        .add_attribute(Attribute::Italic)
                        .set_alignment(CellAlignment::Right),
                    Cell::new("n.a.")
                        .add_attribute(Attribute::Italic)
                        .set_alignment(CellAlignment::Right),
                    Cell::new("n.a.")
                        .add_attribute(Attribute::Italic)
                        .set_alignment(CellAlignment::Right),
                    Cell::new("n.a.").add_attribute(Attribute::Bold),
                ]);
                table.add_row(row);
            } else {
                println!("{}: <Unknown device>", format!("{index:02}").bold());
            }

            base_index += 1;
        }
    }

    if misc.is_empty() {
        println!("{}", "<No supported devices detected>\n".italic());
    } else {
        let row = Row::from(vec![Cell::new(tr!("misc-devices"))]);
        table.add_row_if(|_, _| verbose > 0, row);

        for (index, dev) in misc.iter().enumerate() {
            let device_info = get_device_info(dev.0, dev.1);

            if let Some((_device, zone)) = zones.iter().find(|e| e.0 == base_index as u64) {
                if let Some(device) = device_info {
                    let row = Row::from(vec![
                        Cell::new(format!("{base_index:02}"))
                            .add_attribute(Attribute::Bold)
                            .set_alignment(CellAlignment::Right),
                        Cell::new(device.make).add_attribute(Attribute::Bold),
                        Cell::new(device.model).add_attribute(Attribute::Bold),
                        Cell::new(zone.x)
                            .add_attribute(Attribute::Bold)
                            .set_alignment(CellAlignment::Right),
                        Cell::new(zone.y)
                            .add_attribute(Attribute::Bold)
                            .set_alignment(CellAlignment::Right),
                        Cell::new(zone.width)
                            .add_attribute(Attribute::Bold)
                            .set_alignment(CellAlignment::Right),
                        Cell::new(zone.height)
                            .add_attribute(Attribute::Bold)
                            .set_alignment(CellAlignment::Right),
                        Cell::new(zone.enabled).add_attribute(Attribute::Bold),
                    ]);
                    table.add_row(row);
                } else {
                    println!("{}: <Unknown device>", format!("{index:02}").bold());
                }
            } else if let Some(device) = device_info {
                let row = Row::from(vec![
                    Cell::new(format!("{base_index:02}"))
                        .add_attribute(Attribute::Bold)
                        .set_alignment(CellAlignment::Right),
                    Cell::new(device.make).add_attribute(Attribute::Bold),
                    Cell::new(device.model).add_attribute(Attribute::Bold),
                    Cell::new("n.a.")
                        .add_attribute(Attribute::Italic)
                        .set_alignment(CellAlignment::Right),
                    Cell::new("n.a.")
                        .add_attribute(Attribute::Italic)
                        .set_alignment(CellAlignment::Right),
                    Cell::new("n.a.")
                        .add_attribute(Attribute::Italic)
                        .set_alignment(CellAlignment::Right),
                    Cell::new("n.a.")
                        .add_attribute(Attribute::Italic)
                        .set_alignment(CellAlignment::Right),
                    Cell::new("n.a.").add_attribute(Attribute::Bold),
                ]);
                table.add_row(row);
            } else {
                println!("{}: <Unknown device>", format!("{index:02}").bold());
            }

            base_index += 1;
        }
    }

    println!("{table}");

    Ok(())
}

async fn get_zone_command(device: &str) -> Result<()> {
    let device_index = device.parse::<u64>()?;

    // let verbose = crate::VERBOSE.load(Ordering::SeqCst);
    let mut base_index = 0;

    let mut found = false;

    let (keyboards, mice, misc) = get_devices()
        .await
        .wrap_err("Could not connect to the Eruption daemon")
        .suggestion("Please verify that the Eruption daemon is running")?;

    let zones = get_zones().await?;

    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic);

    let row = Row::from(vec![
        Cell::new("#").set_alignment(CellAlignment::Right),
        Cell::new("Make"),
        Cell::new("Model"),
        Cell::new("x").set_alignment(CellAlignment::Right),
        Cell::new("y").set_alignment(CellAlignment::Right),
        Cell::new("width").set_alignment(CellAlignment::Right),
        Cell::new("height").set_alignment(CellAlignment::Right),
        Cell::new("Enabled").set_alignment(CellAlignment::Right),
    ]);
    table.set_header(row);

    for (index, dev) in keyboards.iter().enumerate() {
        if base_index as u64 != device_index {
            base_index += 1;

            continue;
        }

        let device_info = get_device_info(dev.0, dev.1);

        if let Some((_device, zone)) = zones.iter().find(|e| e.0 == base_index as u64) {
            if let Some(device) = device_info {
                let row = Row::from(vec![
                    Cell::new(format!("{base_index:02}"))
                        .add_attribute(Attribute::Bold)
                        .set_alignment(CellAlignment::Right),
                    Cell::new(device.make).add_attribute(Attribute::Bold),
                    Cell::new(device.model).add_attribute(Attribute::Bold),
                    Cell::new(zone.x)
                        .add_attribute(Attribute::Bold)
                        .set_alignment(CellAlignment::Right),
                    Cell::new(zone.y)
                        .add_attribute(Attribute::Bold)
                        .set_alignment(CellAlignment::Right),
                    Cell::new(zone.width)
                        .add_attribute(Attribute::Bold)
                        .set_alignment(CellAlignment::Right),
                    Cell::new(zone.height)
                        .add_attribute(Attribute::Bold)
                        .set_alignment(CellAlignment::Right),
                    Cell::new(zone.enabled).add_attribute(Attribute::Bold),
                ]);
                table.add_row(row);

                found = true;
            } else {
                println!("{}: <Unknown device>", format!("{index:02}").bold());
            }
        } else if let Some(device) = device_info {
            let row = Row::from(vec![
                Cell::new(format!("{base_index:02}"))
                    .add_attribute(Attribute::Bold)
                    .set_alignment(CellAlignment::Right),
                Cell::new(device.make).add_attribute(Attribute::Bold),
                Cell::new(device.model).add_attribute(Attribute::Bold),
                Cell::new("n.a.")
                    .add_attribute(Attribute::Italic)
                    .set_alignment(CellAlignment::Right),
                Cell::new("n.a.")
                    .add_attribute(Attribute::Italic)
                    .set_alignment(CellAlignment::Right),
                Cell::new("n.a.")
                    .add_attribute(Attribute::Italic)
                    .set_alignment(CellAlignment::Right),
                Cell::new("n.a.")
                    .add_attribute(Attribute::Italic)
                    .set_alignment(CellAlignment::Right),
                Cell::new("n.a.").add_attribute(Attribute::Bold),
            ]);
            table.add_row(row);

            found = true;
        } else {
            println!("{}: <Unknown device>", format!("{index:02}").bold());
        }

        base_index += 1;
    }

    for (index, dev) in mice.iter().enumerate() {
        if base_index as u64 != device_index {
            base_index += 1;

            continue;
        }

        let device_info = get_device_info(dev.0, dev.1);

        if let Some((_device, zone)) = zones.iter().find(|e| e.0 == base_index as u64) {
            if let Some(device) = device_info {
                let row = Row::from(vec![
                    Cell::new(format!("{base_index:02}"))
                        .add_attribute(Attribute::Bold)
                        .set_alignment(CellAlignment::Right),
                    Cell::new(device.make).add_attribute(Attribute::Bold),
                    Cell::new(device.model).add_attribute(Attribute::Bold),
                    Cell::new(zone.x)
                        .add_attribute(Attribute::Bold)
                        .set_alignment(CellAlignment::Right),
                    Cell::new(zone.y)
                        .add_attribute(Attribute::Bold)
                        .set_alignment(CellAlignment::Right),
                    Cell::new(zone.width)
                        .add_attribute(Attribute::Bold)
                        .set_alignment(CellAlignment::Right),
                    Cell::new(zone.height)
                        .add_attribute(Attribute::Bold)
                        .set_alignment(CellAlignment::Right),
                    Cell::new(zone.enabled).add_attribute(Attribute::Bold),
                ]);
                table.add_row(row);

                found = true;
            } else {
                println!("{}: <Unknown device>", format!("{index:02}").bold());
            }
        } else if let Some(device) = device_info {
            let row = Row::from(vec![
                Cell::new(format!("{base_index:02}"))
                    .add_attribute(Attribute::Bold)
                    .set_alignment(CellAlignment::Right),
                Cell::new(device.make).add_attribute(Attribute::Bold),
                Cell::new(device.model).add_attribute(Attribute::Bold),
                Cell::new("n.a.")
                    .add_attribute(Attribute::Italic)
                    .set_alignment(CellAlignment::Right),
                Cell::new("n.a.")
                    .add_attribute(Attribute::Italic)
                    .set_alignment(CellAlignment::Right),
                Cell::new("n.a.")
                    .add_attribute(Attribute::Italic)
                    .set_alignment(CellAlignment::Right),
                Cell::new("n.a.")
                    .add_attribute(Attribute::Italic)
                    .set_alignment(CellAlignment::Right),
                Cell::new("n.a.").add_attribute(Attribute::Bold),
            ]);
            table.add_row(row);

            found = true;
        } else {
            println!("{}: <Unknown device>", format!("{index:02}").bold());
        }

        base_index += 1;
    }

    for (index, dev) in misc.iter().enumerate() {
        if base_index as u64 != device_index {
            base_index += 1;

            continue;
        }

        let device_info = get_device_info(dev.0, dev.1);

        if let Some((_device, zone)) = zones.iter().find(|e| e.0 == base_index as u64) {
            if let Some(device) = device_info {
                let row = Row::from(vec![
                    Cell::new(format!("{base_index:02}"))
                        .add_attribute(Attribute::Bold)
                        .set_alignment(CellAlignment::Right),
                    Cell::new(device.make).add_attribute(Attribute::Bold),
                    Cell::new(device.model).add_attribute(Attribute::Bold),
                    Cell::new(zone.x)
                        .add_attribute(Attribute::Bold)
                        .set_alignment(CellAlignment::Right),
                    Cell::new(zone.y)
                        .add_attribute(Attribute::Bold)
                        .set_alignment(CellAlignment::Right),
                    Cell::new(zone.width)
                        .add_attribute(Attribute::Bold)
                        .set_alignment(CellAlignment::Right),
                    Cell::new(zone.height)
                        .add_attribute(Attribute::Bold)
                        .set_alignment(CellAlignment::Right),
                    Cell::new(zone.enabled).add_attribute(Attribute::Bold),
                ]);
                table.add_row(row);

                found = true;
            } else {
                println!("{}: <Unknown device>", format!("{index:02}").bold());
            }
        } else if let Some(device) = device_info {
            let row = Row::from(vec![
                Cell::new(format!("{base_index:02}"))
                    .add_attribute(Attribute::Bold)
                    .set_alignment(CellAlignment::Right),
                Cell::new(device.make).add_attribute(Attribute::Bold),
                Cell::new(device.model).add_attribute(Attribute::Bold),
                Cell::new("n.a.")
                    .add_attribute(Attribute::Italic)
                    .set_alignment(CellAlignment::Right),
                Cell::new("n.a.")
                    .add_attribute(Attribute::Italic)
                    .set_alignment(CellAlignment::Right),
                Cell::new("n.a.")
                    .add_attribute(Attribute::Italic)
                    .set_alignment(CellAlignment::Right),
                Cell::new("n.a.")
                    .add_attribute(Attribute::Italic)
                    .set_alignment(CellAlignment::Right),
                Cell::new("n.a.").add_attribute(Attribute::Bold),
            ]);
            table.add_row(row);

            found = true;
        } else {
            println!("{}: <Unknown device>", format!("{index:02}").bold());
        }

        base_index += 1;
    }

    if found {
        println!("{table}");
    } else {
        eprintln!("Invalid device");
    }

    Ok(())
}

async fn set_zone_command(
    device: &str,
    x: Option<i32>,
    y: Option<i32>,
    width: Option<i32>,
    height: Option<i32>,
    enabled: Option<bool>,
) -> Result<()> {
    let device = device.parse::<u64>()?;

    let zones = get_devices_zone_allocations()?;

    if let Some(entry) = zones.iter().find(|e| e.0 == device) {
        let mut zone = entry.1;

        assign_conditionally!(x, zone.x);
        assign_conditionally!(y, zone.y);

        assign_conditionally!(width, zone.width);
        assign_conditionally!(height, zone.height);

        assign_conditionally!(enabled, zone.enabled);

        set_device_zone_allocation(device, &zone)?;
    } else {
        eprintln!("Invalid device");
    }

    Ok(())
}

async fn enable_zone_command(device: &str) -> Result<()> {
    let device = device.parse::<u64>()?;

    let zones = get_devices_zone_allocations()?;

    if let Some(entry) = zones.iter().find(|e| e.0 == device) {
        let mut zone = entry.1;
        zone.enabled = true;

        set_device_zone_allocation(device, &zone)?;
    } else {
        eprintln!("Invalid device");
    }

    Ok(())
}

async fn disable_zone_command(device: &str) -> Result<()> {
    let device = device.parse::<u64>()?;

    let zones = get_devices_zone_allocations()?;

    if let Some(entry) = zones.iter().find(|e| e.0 == device) {
        let mut zone = entry.1;
        zone.enabled = false;

        set_device_zone_allocation(device, &zone)?;
    } else {
        eprintln!("Invalid device");
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

/// Enumerate all allocated zones
async fn get_zones() -> Result<Vec<(u64, Zone)>> {
    get_devices_zone_allocations()
}

/// Fetches all allocated zones from the eruption daemon
fn get_devices_zone_allocations() -> Result<Vec<(u64, Zone)>> {
    use canvas::OrgEruptionCanvas;

    let conn = Connection::new_system()?;
    let status_proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/canvas",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let result = status_proxy.get_devices_zone_allocations()?;

    let result = result
        .iter()
        .map(|v| {
            (
                v.0,
                Zone {
                    x: v.1 .0,
                    y: v.1 .1,
                    width: v.1 .2,
                    height: v.1 .3,
                    enabled: v.1 .4,
                },
            )
        })
        .collect::<Vec<(u64, Zone)>>();

    Ok(result)
}

/// Update all allocated zones in the eruption daemon
#[allow(dead_code)]
fn set_devices_zone_allocations(zones: &[(u64, Zone)]) -> Result<()> {
    use canvas::OrgEruptionCanvas;

    let conn = Connection::new_system()?;
    let status_proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/canvas",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let zones = zones
        .iter()
        .map(|(device, zone)| {
            (
                *device,
                (zone.x, zone.y, zone.width, zone.height, zone.enabled),
            )
        })
        .collect::<Vec<(u64, (i32, i32, i32, i32, bool))>>();

    status_proxy.set_devices_zone_allocations(zones)?;

    Ok(())
}

/// Update all allocated zones in the eruption daemon
fn set_device_zone_allocation(device: u64, zone: &Zone) -> Result<()> {
    use canvas::OrgEruptionCanvas;

    let conn = Connection::new_system()?;
    let status_proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/canvas",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let zone = (zone.x, zone.y, zone.width, zone.height, zone.enabled);
    status_proxy.set_device_zone_allocation(device, zone)?;

    Ok(())
}
