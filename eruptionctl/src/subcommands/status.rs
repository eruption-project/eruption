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
use std::{
    fmt::Display,
    io::Read,
    process::{Command, Stdio},
    time::Duration,
};

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
    /// Shows status information about Eruption
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
    print_header();

    let daemon_status = get_systems_status().await?;
    println!("{}\n\n{daemon_status}", "Eruption Status".bold());

    Ok(())
}

async fn profile_command() -> Result<()> {
    let profile_name = get_active_profile()
        .await
        .wrap_err("Could not connect to the Eruption daemon")
        .suggestion("Please verify that the Eruption daemon is running")?;
    println!("Profile: {}", profile_name.bold());

    Ok(())
}

async fn slot_command() -> Result<()> {
    let index = get_active_slot()
        .await
        .wrap_err("Could not connect to the Eruption daemon")
        .suggestion("Please verify that the Eruption daemon is running")?
        + 1;
    println!("Slot: {}", format!("{index}").bold());

    Ok(())
}

fn print_header() {
    println!(
        r"
     ********                          **   **
     /**/////                 ******   /**  //
     /**       ****** **   **/**///** ****** **  ******  *******
     /******* //**//*/**  /**/**  /**///**/ /** **////**//**///**
     /**////   /** / /**  /**/******   /**  /**/**   /** /**  /**
     /**       /**   /**  /**/**///    /**  /**/**   /** /**  /**
     /********/***   //******/**       //** /**//******  ***  /**
     //////// ///     ////// //         //  //  //////  ///   //
    "
    );
}

/// Status information about the running Eruption daemons
struct SystemsStatus {
    /// Status of the Eruption daemon
    pub eruption_status: ServiceStatus,

    /// Eruption process-monitor daemon status
    pub eruption_process_monitor_status: ServiceStatus,

    /// Eruption audio-proxy daemon status
    pub eruption_audio_proxy_status: ServiceStatus,

    /// Eruption fx-proxy daemon status
    pub eruption_fx_proxy_status: ServiceStatus,

    /// Eruption daemon ping call reply
    pub eruption_ping: String,

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

impl Display for SystemsStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Eruption core: {}\nPing: {}\nLoaded Profiles: {}\n",
            self.eruption_status.bold(),
            self.eruption_ping.bold(),
            self.profiles.len()
        )?;

        write!(
            f,
            "Active Profile: {}\nActive Slot: {}\n\n",
            self.active_profile.bold(),
            (self.active_slot + 1).bold()
        )?;

        write!(f, "{}\n\n", "Session Daemons".bold())?;

        write!(
            f,
            "Process Monitor: {}\nAudio Proxy: {}\nEffects Proxy: {}\n\n",
            self.eruption_process_monitor_status.bold(),
            self.eruption_audio_proxy_status.bold(),
            self.eruption_fx_proxy_status.bold(),
        )?;

        write!(f, "{}\n\n", "Slots".bold())?;

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
                profile.italic(),
            )?;
        }

        Ok(())
    }
}

/// Returns a few stats about the running Eruption daemons
async fn get_systems_status() -> Result<SystemsStatus> {
    let conn_system_bus = Connection::new_system()?;
    // let conn_session_bus = Connection::new_session()?;

    let config_proxy = conn_system_bus.with_proxy(
        "org.eruption",
        "/org/eruption/config",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let profiles_proxy = conn_system_bus.with_proxy(
        "org.eruption",
        "/org/eruption/profile",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let slot_proxy = conn_system_bus.with_proxy(
        "org.eruption",
        "/org/eruption/slot",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    // let status_proxy = conn.with_proxy(
    //     "org.eruption",
    //     "/org/eruption/status",
    //     Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    // );

    let eruption_ping = if config_proxy.ping()? {
        "OK"
    } else {
        "<Unknown>"
    }
    .to_string();

    let slot_names = slot_proxy.slot_names()?;
    let slot_profiles = slot_proxy.get_slot_profiles()?;
    let profiles = profiles_proxy.enum_profiles()?;

    let active_profile = get_active_profile().await?;
    let active_slot = get_active_slot().await?;

    Ok(SystemsStatus {
        eruption_status: get_daemon_status(Daemon::Eruption)?,
        eruption_process_monitor_status: get_daemon_status(Daemon::ProcessMonitor)?,
        eruption_audio_proxy_status: get_daemon_status(Daemon::AudioProxy)?,
        eruption_fx_proxy_status: get_daemon_status(Daemon::FxProxy)?,
        eruption_ping: eruption_ping,
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

#[derive(Debug, thiserror::Error)]
pub enum ServiceError {
    #[error("Daemon action failed")]
    ActionFailed,
    // #[error("Unknown error")]
    // UnknownError,
}

#[derive(Debug)]
pub enum Daemon {
    Eruption,
    ProcessMonitor,
    AudioProxy,
    FxProxy,
}

// pub fn set_daemon_status(daemon: Daemon, running: bool) -> Result<()> {
//     let unit_file = match daemon {
//         Daemon::Eruption => constants::UNIT_NAME_ERUPTION,
//         Daemon::ProcessMonitor => constants::UNIT_NAME_PROCESS_MONITOR,
//         Daemon::AudioProxy => constants::UNIT_NAME_AUDIO_PROXY,
//         Daemon::FxProxy => constants::UNIT_NAME_FX_PROXY,
//     };

//     let user_or_system = match daemon {
//         Daemon::Eruption => "--system",
//         Daemon::ProcessMonitor => "--user",
//         Daemon::AudioProxy => "--user",
//         Daemon::FxProxy => "--user",
//     };

//     let action = if running { "start" } else { "stop" };

//     let status = Command::new("/usr/bin/systemctl")
//         // .stdout(Stdio::null())
//         .arg(user_or_system)
//         .arg(action)
//         .arg(unit_file)
//         .status()?;

//     let exit_code = status.code().unwrap_or(0);

//     if exit_code != 0 {
//         Err(ServiceError::ActionFailed {}.into())
//     } else {
//         Ok(())
//     }
// }

pub enum ServiceStatus {
    Unknown,
    Active,
    Inactive,
    Failed,
}

impl Display for ServiceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            ServiceStatus::Unknown => write!(f, "Unknown"),
            ServiceStatus::Active => write!(f, "Active"),
            ServiceStatus::Inactive => write!(f, "Inactive"),
            ServiceStatus::Failed => write!(f, "Failed"),
        }?;

        Ok(())
    }
}

pub fn get_daemon_status(daemon: Daemon) -> Result<ServiceStatus> {
    let unit_file = match daemon {
        Daemon::Eruption => constants::UNIT_NAME_ERUPTION,
        Daemon::ProcessMonitor => constants::UNIT_NAME_PROCESS_MONITOR,
        Daemon::AudioProxy => constants::UNIT_NAME_AUDIO_PROXY,
        Daemon::FxProxy => constants::UNIT_NAME_FX_PROXY,
    };

    let user_or_system = match daemon {
        Daemon::Eruption => "--system",
        Daemon::ProcessMonitor => "--user",
        Daemon::AudioProxy => "--user",
        Daemon::FxProxy => "--user",
    };

    let mut status = Command::new("/usr/bin/systemctl")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .arg(user_or_system)
        .arg("is-failed")
        .arg(unit_file)
        .spawn()?;

    let _status = status.wait()?;

    match status.stdout {
        Some(ref mut out) => {
            let mut output = String::new();
            out.read_to_string(&mut output)?;

            match output.trim() {
                "failed" => Ok(ServiceStatus::Failed),
                "active" => Ok(ServiceStatus::Active),
                "inactive" => Ok(ServiceStatus::Inactive),

                _ => Ok(ServiceStatus::Unknown),
            }
        }

        None => Err(ServiceError::ActionFailed {}.into()),
    }
}
