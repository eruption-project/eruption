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


use color_eyre::Help;
use colored::*;
use dbus::nonblock::stdintf::org_freedesktop_dbus::Properties;
use eyre::Context;

use crate::dbus_client::dbus_system_bus;

type Result<T> = std::result::Result<T, eyre::Error>;

/// Sub-commands of the "status" command
#[derive(Debug, clap::Parser)]
pub enum StatusSubcommands {
    /// Shows the currently active profile
    #[clap(display_order = 0)]
    Profile,

    /// Shows the currently active slot
    #[clap(display_order = 1)]
    Slot,
}

pub async fn handle_command(command: StatusSubcommands) -> Result<()> {
    match command {
        StatusSubcommands::Profile => profile_command().await,
        StatusSubcommands::Slot => slot_command().await,
    }
}

async fn profile_command() -> Result<()> {
    let profile_name = get_active_profile()
        .await
        .wrap_err("Could not connect to the Eruption daemon")
        .suggestion("Please verify that the Eruption daemon is running")?;
    println!("Current profile: {}", profile_name.bold());

    Ok(())
}

async fn slot_command() -> Result<()> {
    let index = get_active_slot()
        .await
        .wrap_err("Could not connect to the Eruption daemon")
        .suggestion("Please verify that the Eruption daemon is running")?
        + 1;
    println!("Current slot: {}", format!("{}", index).bold());

    Ok(())
}

/// Get the name of the currently active profile
async fn get_active_profile() -> Result<String> {
    let result: String = dbus_system_bus("/org/eruption/profile")
        .await?
        .get("org.eruption.Profile", "ActiveProfile")
        .await?;

    Ok(result)
}

/// Get the index of the currently active slot
async fn get_active_slot() -> Result<usize> {
    let result: u64 = dbus_system_bus("/org/eruption/slot")
        .await?
        .get("org.eruption.Slot", "ActiveSlot")
        .await?;

    Ok(result as usize)
}
