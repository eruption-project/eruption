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

use color_eyre::Help;
use colored::*;
use dbus::nonblock::stdintf::org_freedesktop_dbus::Properties;
use eyre::Context;

use crate::dbus_client::dbus_system_bus;

type Result<T> = std::result::Result<T, eyre::Error>;

/// Sub-commands of the "config" command
#[derive(Debug, clap::Parser)]
pub enum ConfigSubcommands {
    /// Get or set the global brightness of the LEDs
    #[clap(display_order = 0)]
    Brightness { brightness: Option<i64> },

    /// Get or set the state of SoundFX
    #[clap(display_order = 1)]
    Soundfx { enable: Option<bool> },
}

pub async fn handle_command(command: ConfigSubcommands) -> Result<()> {
    match command {
        ConfigSubcommands::Brightness { brightness } => brightness_command(brightness).await,
        ConfigSubcommands::Soundfx { enable } => sound_fx_command(enable).await,
    }
}

async fn brightness_command(brightness: Option<i64>) -> Result<()> {
    if let Some(brightness) = brightness {
        set_brightness(brightness)
            .await
            .wrap_err("Could not connect to the Eruption daemon")
            .suggestion("Please verify that the Eruption daemon is running")?;
    } else {
        let result = get_brightness()
            .await
            .wrap_err("Could not connect to the Eruption daemon")
            .suggestion("Please verify that the Eruption daemon is running")?;
        println!(
            "{}",
            format!("Global brightness: {}", format!("{}%", result).bold())
        );
    }

    Ok(())
}

async fn sound_fx_command(enable: Option<bool>) -> Result<()> {
    if let Some(enable) = enable {
        set_sound_fx(enable)
            .await
            .wrap_err("Could not connect to the Eruption daemon")
            .suggestion("Please verify that the Eruption daemon is running")?;
    } else {
        let result = get_sound_fx()
            .await
            .wrap_err("Could not connect to the Eruption daemon")
            .suggestion("Please verify that the Eruption daemon is running")?;
        println!(
            "{}",
            format!("SoundFX enabled: {}", format!("{}", result).bold())
        );
    }

    Ok(())
}

/// Get the current brightness value
async fn get_brightness() -> Result<i64> {
    let result = dbus_system_bus("/org/eruption/config")
        .await?
        .get("org.eruption.Config", "Brightness")
        .await?;

    Ok(result)
}

/// Set the current brightness value
async fn set_brightness(brightness: i64) -> Result<()> {
    let arg = Box::new(brightness);

    dbus_system_bus("/org/eruption/config")
        .await?
        .set("org.eruption.Config", "Brightness", arg)
        .await?;

    Ok(())
}

/// Returns true when SoundFX is enabled
async fn get_sound_fx() -> Result<bool> {
    let result = dbus_system_bus("/org/eruption/config")
        .await?
        .get("org.eruption.Config", "EnableSfx")
        .await?;

    Ok(result)
}

/// Set SoundFX state to `enabled`
async fn set_sound_fx(enabled: bool) -> Result<()> {
    let arg = Box::new(enabled);

    dbus_system_bus("/org/eruption/config")
        .await?
        .set("org.eruption.Config", "EnableSfx", arg)
        .await?;

    Ok(())
}
