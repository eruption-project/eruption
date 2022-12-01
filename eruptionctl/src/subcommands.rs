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

mod color_schemes;
mod completions;
mod config;
mod devices;
mod effects;
mod names;
mod param;
mod profiles;
mod rules;
mod scripts;
mod status;
mod switch;

use crate::translations::tr;

type Result<T> = std::result::Result<T, eyre::Error>;

// Sub-commands
#[derive(Debug, clap::Parser)]
pub enum Subcommands {
    #[clap(display_order = 0, about(tr!("status-about")))]
    Status {
        #[clap(subcommand)]
        command: status::StatusSubcommands,
    },

    #[clap(display_order = 1, about(tr!("switch-about")))]
    Switch {
        #[clap(subcommand)]
        command: switch::SwitchSubcommands,
    },

    #[clap(display_order = 2, about(tr!("config-about")))]
    Config {
        #[clap(subcommand)]
        command: config::ConfigSubcommands,
    },

    #[clap(display_order = 3, about(tr!("devices-about")))]
    Devices {
        #[clap(subcommand)]
        command: devices::DevicesSubcommands,
    },

    #[clap(display_order = 4, about(tr!("profiles-about")))]
    Profiles {
        #[clap(subcommand)]
        command: profiles::ProfilesSubcommands,
    },

    #[clap(display_order = 5, about(tr!("scripts-about")))]
    Scripts {
        #[clap(subcommand)]
        command: scripts::ScriptsSubcommands,
    },

    #[clap(display_order = 6, about(tr!("color-scheme-about")))]
    ColorSchemes {
        #[clap(subcommand)]
        command: color_schemes::ColorSchemesSubcommands,
    },

    #[clap(display_order = 7, about(tr!("param-about")))]
    Param {
        script: Option<String>,
        parameter: Option<String>,
        value: Option<String>,
    },

    #[clap(display_order = 8, about(tr!("names-about")))]
    Names {
        #[clap(subcommand)]
        command: names::NamesSubcommands,
    },

    #[clap(display_order = 9, about(tr!("effects-about")))]
    Effects {
        #[clap(subcommand)]
        command: effects::EffectsSubcommands,
    },

    #[clap(display_order = 10, about(tr!("rules-about")))]
    Rules {
        #[clap(subcommand)]
        command: rules::RulesSubcommands,
    },

    #[clap(display_order = 11, hide = true, about(tr!("completions-about")))]
    Completions { shell: clap_complete::Shell },
}

pub async fn handle_command(subcommand: Subcommands) -> Result<()> {
    match subcommand {
        Subcommands::Status { command } => status::handle_command(command).await,
        Subcommands::Switch { command } => switch::handle_command(command).await,
        Subcommands::Config { command } => config::handle_command(command).await,
        Subcommands::Devices { command } => devices::handle_command(command).await,
        Subcommands::Profiles { command } => profiles::handle_command(command).await,
        Subcommands::Scripts { command } => scripts::handle_command(command).await,
        Subcommands::ColorSchemes { command } => color_schemes::handle_command(command).await,
        Subcommands::Param {
            script,
            parameter,
            value,
        } => param::handle_command(script, parameter, value).await,
        Subcommands::Names { command } => names::handle_command(command).await,
        Subcommands::Effects { command } => effects::handle_command(command).await,
        Subcommands::Rules { command } => rules::handle_command(command).await,
        Subcommands::Completions { shell } => completions::handle_command(shell).await,
    }
}
