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


use color_eyre::Help;
use colored::*;
use dbus::nonblock::stdintf::org_freedesktop_dbus::Properties;
use eyre::Context;

use crate::constants;
use crate::dbus_client::dbus_system_bus;

type Result<T> = std::result::Result<T, eyre::Error>;

/// Subcommands of the "names" command
#[derive(Debug, clap::Parser)]
pub enum NamesSubcommands {
    /// List slot names
    List,

    /// Set the name of a single profile slot
    Set { slot_index: usize, name: String },

    /// Set all the profile slot names at once
    SetAll { names: Vec<String> },
}

pub async fn handle_command(command: NamesSubcommands) -> Result<()> {
    match command {
        NamesSubcommands::List => list_command().await,
        NamesSubcommands::Set { slot_index, name } => set_command(slot_index, name).await,
        NamesSubcommands::SetAll { names } => set_all_command(names).await,
    }
}

async fn list_command() -> Result<()> {
    let slot_names = get_slot_names()
        .await
        .wrap_err("Could not connect to the Eruption daemon")
        .suggestion("Please verify that the Eruption daemon is running")?;

    for (index, name) in slot_names.iter().enumerate() {
        let s = format!("{}", index + 1);
        println!("{}: {}", s.bold(), name);
    }

    Ok(())
}

async fn set_command(slot_index: usize, name: String) -> Result<()> {
    if slot_index > 0 && slot_index <= constants::NUM_SLOTS {
        set_slot_name(slot_index - 1, name)
            .await
            .wrap_err("Could not connect to the Eruption daemon")
            .suggestion("Please verify that the Eruption daemon is running")?;
    } else {
        eprintln!("Slot index out of bounds");
    }

    Ok(())
}

async fn set_all_command(names: Vec<String>) -> Result<()> {
    if names.len() == constants::NUM_SLOTS {
        set_slot_names(&names)
            .await
            .wrap_err("Could not connect to the Eruption daemon")
            .suggestion("Please verify that the Eruption daemon is running")?;
    } else {
        eprintln!("Elements do not match number of slots");
    }

    Ok(())
}

/// Get the names of the profile slots
async fn get_slot_names() -> Result<Vec<String>> {
    let result: Vec<String> = dbus_system_bus("/org/eruption/slot")
        .await?
        .get("org.eruption.Slot", "SlotNames")
        .await?;

    Ok(result)
}

/// Set the name of a single profile slot
async fn set_slot_name(slot_index: usize, name: String) -> Result<()> {
    let mut result = get_slot_names().await?;

    result[slot_index] = name;
    set_slot_names(&result).await?;

    Ok(())
}

/// Set the names of the profile slots
async fn set_slot_names(names: &[String]) -> Result<()> {
    let arg = Box::new(names);

    dbus_system_bus("/org/eruption/slot")
        .await?
        .set("org.eruption.Slot", "SlotNames", arg)
        .await?;

    Ok(())
}
