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

use std::path::PathBuf;

use color_eyre::Help;
use colored::*;
use eyre::Context;

use crate::dbus_client::dbus_system_bus;
use crate::{constants, util};

type Result<T> = std::result::Result<T, eyre::Error>;

/// Sub-commands of the "switch" command
#[derive(Debug, clap::Parser)]
pub enum SwitchSubcommands {
    /// Switch profiles
    #[clap(display_order = 0)]
    Profile { profile_name: String },

    /// Switch slots
    #[clap(display_order = 1)]
    Slot { index: usize },
}

pub async fn handle_command(command: SwitchSubcommands) -> Result<()> {
    match command {
        SwitchSubcommands::Profile { profile_name } => profile_command(profile_name).await,
        SwitchSubcommands::Slot { index } => slot_command(index).await,
    }
}

async fn profile_command(profile_name: String) -> Result<()> {
    let profile_path = PathBuf::from(&profile_name);

    let profile_name = if profile_path.is_file() {
        Ok(profile_path.canonicalize()?)
        // use the absolute path, otherwise the pathname will be searched in the profile directory
    } else {
        util::match_profile_path(&profile_name)
    };

    match profile_name {
        Ok(profile_name) => {
            println!(
                "Switching to profile: {}",
                profile_name.display().to_string().bold()
            );
            switch_profile(&profile_name.to_string_lossy())
                .await
                .wrap_err("Could not connect to the Eruption daemon")
                .suggestion("Please verify that the Eruption daemon is running")?;
        }

        Err(_e) => {
            eprintln!("No matches found");
        }
    }

    Ok(())
}

async fn slot_command(index: usize) -> Result<()> {
    if !(1..=constants::NUM_SLOTS).contains(&index) {
        eprintln!(
            "Slot index out of bounds. Valid range is: {}-{}",
            1,
            constants::NUM_SLOTS
        );
    } else {
        println!("Switching to slot: {}", format!("{}", index).bold());
        let index = index - 1;
        switch_slot(index)
            .await
            .wrap_err("Could not connect to the Eruption daemon")
            .suggestion("Please verify that the Eruption daemon is running")?;
    }

    Ok(())
}

/// Switch the currently active profile
async fn switch_profile(name: &str) -> Result<()> {
    let file_name = name.to_owned();

    let (_result,): (bool,) = dbus_system_bus("/org/eruption/profile")
        .await?
        .method_call("org.eruption.Profile", "SwitchProfile", (file_name,))
        .await?;

    Ok(())
}

/// Switch the currently active slot
async fn switch_slot(index: usize) -> Result<()> {
    let (_result,): (bool,) = dbus_system_bus("/org/eruption/slot")
        .await?
        .method_call("org.eruption.Slot", "SwitchSlot", (index as u64,))
        .await?;

    Ok(())
}
