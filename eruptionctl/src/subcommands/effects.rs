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

use std::path::PathBuf;

use crate::dbus_client;

type Result<T> = std::result::Result<T, eyre::Error>;

/// Sub-commands of the "effects" command
#[derive(Debug, clap::Parser)]
pub enum EffectsSubcommands {
    /// Enable an effect
    Enable {
        #[clap(subcommand)]
        command: EnableSubcommands,
    },

    /// Disable an active effect
    Disable {
        #[clap(subcommand)]
        command: DisableSubcommands,
    },

    /// Show the status of the Eruption effects subsystem
    Status,

    /// Configure an active effect
    Config {
        #[clap(subcommand)]
        command: ConfigSubcommands,
    },
}

/// Sub-commands of the "effects enable" command
#[derive(Debug, clap::Parser)]
pub enum EnableSubcommands {
    /// Load an image file and display it on the connected devices
    Image { filename: PathBuf },

    /// Load image files from a directory and display each one on the connected devices
    Animation {
        directory_name: PathBuf,
        frame_delay: Option<u64>,
    },

    /// Make the LEDs of connected devices reflect what is shown on the screen
    Ambient { frame_delay: Option<u64> },
}

/// Sub-commands of the "effects config" command
#[derive(Debug, clap::Parser)]
pub enum ConfigSubcommands {
    /// Load an image file and display it on the connected devices
    Image { filename: PathBuf },

    /// Load image files from a directory and display each one on the connected devices
    Animation {
        directory_name: PathBuf,
        frame_delay: Option<u64>,
    },

    /// Make the LEDs of connected devices reflect what is shown on the screen
    Ambient { frame_delay: Option<u64> },
}

/// Sub-commands of the "effects disable" command
#[derive(Debug, clap::Parser)]
pub enum DisableSubcommands {
    /// Disable the image effect
    Image,

    /// Disable the animation effect
    Animation,

    /// Disable the ambient effect
    Ambient,
}

pub async fn handle_command(command: EffectsSubcommands) -> Result<()> {
    match command {
        EffectsSubcommands::Enable { command } => match command {
            EnableSubcommands::Image { filename: _ } => todo!(),
            EnableSubcommands::Animation {
                directory_name: _,
                frame_delay: _,
            } => todo!(),
            EnableSubcommands::Ambient { frame_delay: _ } => dbus_client::enable_ambient_effect(),
        },

        EffectsSubcommands::Config { command } => match command {
            ConfigSubcommands::Image { filename: _ } => todo!(),
            ConfigSubcommands::Animation {
                directory_name: _,
                frame_delay: _,
            } => todo!(),
            ConfigSubcommands::Ambient { frame_delay: _ } => dbus_client::enable_ambient_effect(),
        },

        EffectsSubcommands::Disable { command } => match command {
            DisableSubcommands::Image => todo!(),
            DisableSubcommands::Animation => todo!(),
            DisableSubcommands::Ambient => dbus_client::disable_ambient_effect(),
        },

        EffectsSubcommands::Status => todo!(),
    }
}
