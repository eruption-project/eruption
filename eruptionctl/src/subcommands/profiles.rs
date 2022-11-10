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

use clap;
use color_eyre::Help;
use colored::*;
use eyre::Context;

use crate::dbus_client::dbus_system_bus;
use crate::util;

type Result<T> = std::result::Result<T, eyre::Error>;

/// Sub-commands of the "profiles" command
#[derive(Debug, clap::Parser)]
pub enum ProfilesSubcommands {
    /// List all available profiles
    #[clap(display_order = 0)]
    List,

    /// Show information about a specific profile
    #[clap(display_order = 1)]
    Info { profile_name: String },

    /// Edit a profile
    #[clap(display_order = 2)]
    Edit { profile_name: String },
}

pub async fn handle_command(command: ProfilesSubcommands) -> Result<()> {
    match command {
        ProfilesSubcommands::Edit { profile_name } => edit_command(profile_name).await,
        ProfilesSubcommands::List => list_command().await,
        ProfilesSubcommands::Info { profile_name } => info_command(profile_name).await,
    }
}

async fn edit_command(profile_name: String) -> Result<()> {
    match util::match_profile_by_name(&profile_name) {
        Ok(profile) => util::edit_file(&profile.profile_file)?,
        Err(err) => eprintln!("{}", err),
    }

    Ok(())
}

async fn list_command() -> Result<()> {
    for p in get_profiles()
        .await
        .wrap_err("Could not connect to the Eruption daemon")
        .suggestion("Please verify that the Eruption daemon is running")?
    {
        println!("{}: {}", p.0.bold(), p.1);
    }

    Ok(())
}

async fn info_command(profile_name: String) -> Result<()> {
    match util::match_profile_by_name(&profile_name) {
        Ok(profile) => {
            println!(
                "Profile:\t{} ({})\nDescription:\t{}\nScripts:\t{:?}\n\n{:#?}",
                profile.name,
                profile.id,
                profile.description,
                profile.active_scripts,
                profile.config,
            );
        }
        Err(err) => eprintln!("{}", err),
    }

    Ok(())
}

/// Enumerate all available profiles
async fn get_profiles() -> Result<Vec<(String, String)>> {
    let (result,): (Vec<(String, String)>,) = dbus_system_bus("/org/eruption/profile")
        .await?
        .method_call("org.eruption.Profile", "EnumProfiles", ())
        .await?;

    Ok(result)
}
