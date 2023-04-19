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

use color_eyre::{owo_colors::OwoColorize, Help};
use colored::*;
use dbus::{blocking::Connection, nonblock::stdintf::org_freedesktop_dbus::Properties};
use eyre::Context;
use std::{fmt::Display, time::Duration};

use crate::{
    constants,
    dbus_client::{
        config::OrgEruptionConfig, dbus_system_bus, profile::OrgEruptionProfile,
        slot::OrgEruptionSlot,
    },
};

type Result<T> = std::result::Result<T, eyre::Error>;

/// Sub-commands of the "status" command
#[derive(Debug, clap::Parser)]
pub enum StatusSubcommands {
    /// Shows some status informations about Eruption
    #[clap(display_order = 0)]
    Daemon,

    /// Shows the currently active profile
    #[clap(display_order = 1)]
    Profile,

    /// Shows the currently active slot
    #[clap(display_order = 2)]
    Slot,
}

pub async fn handle_command(command: StatusSubcommands) -> Result<()> {
    match command {
        StatusSubcommands::Daemon => daemon_command().await,
        StatusSubcommands::Profile => profile_command().await,
        StatusSubcommands::Slot => slot_command().await,
    }
}

async fn daemon_command() -> Result<()> {
    let daemon_status = get_daemon_status().await?;
    println!("{}\n\n{daemon_status}", "Eruption status".bold());

    Ok(())
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
    println!("Current slot: {}", format!("{index}").bold());

    Ok(())
}

/// Status information about the running Eruption daemons
struct DaemonStatus {
    /// Status of the Eruption daemon
    pub status: String,

    /// List of slot names
    pub slot_names: Vec<String>,

    /// List of associated profiles for each slot
    pub slot_profiles: Vec<String>,

    /// List of available profiles
    pub profiles: Vec<(String, String)>,

    /// Currently active profile
    pub active_profile: String,

    /// Currently active slot
    pub active_slot: usize,
}

impl Display for DaemonStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Eruption daemon: {}\nLoaded Profiles: {}\n",
            self.status.bold(),
            self.profiles.len()
        )?;

        write!(
            f,
            "Active Profile: {}\nActive Slot: {}\n\nSlots:\n",
            self.active_profile.bold(),
            (self.active_slot + 1).bold()
        )?;

        for (index, (slot_name, profile)) in self
            .slot_names
            .iter()
            .zip(self.slot_profiles.iter())
            .enumerate()
        {
            writeln!(
                f,
                "{}: {}: {}",
                (index + 1).dimmed(),
                slot_name.bold(),
                profile
            )?;
        }

        Ok(())
    }
}

/// Returns a few stats about the running Eruption daemons
async fn get_daemon_status() -> Result<DaemonStatus> {
    let conn = Connection::new_system()?;

    let config_proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/config",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let profiles_proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/profile",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let slot_proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/slot",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    // let status_proxy = conn.with_proxy(
    //     "org.eruption",
    //     "/org/eruption/status",
    //     Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    // );

    let status = if config_proxy.ping()? {
        "OK"
    } else {
        "<no response>"
    }
    .to_string();

    let slot_names = slot_proxy.slot_names()?;
    let slot_profiles = slot_proxy.get_slot_profiles()?;
    let profiles = profiles_proxy.enum_profiles()?;

    let active_profile = get_active_profile().await?;
    let active_slot = get_active_slot().await?;

    Ok(DaemonStatus {
        status,
        slot_names,
        slot_profiles,
        profiles,
        active_profile,
        active_slot,
    })
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
