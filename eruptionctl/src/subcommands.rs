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

mod canvas;
mod color_schemes;
mod completions;
mod config;
mod devices;
mod names;
mod param;
mod profiles;
mod scripts;
mod status;
mod switch;

use crate::translations::tr;

type Result<T> = std::result::Result<T, eyre::Error>;

// Sub-commands
#[derive(Debug, clap::Parser)]
pub enum Subcommands {
    #[clap(about(tr!("canvas-about")))]
    Canvas {
        #[clap(subcommand)]
        command: canvas::CanvasSubcommands,
    },

    #[clap(about(tr!("config-about")))]
    Config {
        #[clap(subcommand)]
        command: config::ConfigSubcommands,
    },

    #[clap(about(tr!("color-scheme-about")))]
    ColorSchemes {
        #[clap(subcommand)]
        command: color_schemes::ColorSchemesSubcommands,
    },

    #[clap(about(tr!("devices-about")))]
    Devices {
        #[clap(subcommand)]
        command: devices::DevicesSubcommands,
    },

    #[clap(about(tr!("status-about")))]
    Status {
        #[clap(subcommand)]
        command: status::StatusSubcommands,
    },

    #[clap(about(tr!("switch-about")))]
    Switch {
        #[clap(subcommand)]
        command: switch::SwitchSubcommands,
    },

    #[clap(about(tr!("profiles-about")))]
    Profiles {
        #[clap(subcommand)]
        command: profiles::ProfilesSubcommands,
    },

    #[clap(about(tr!("names-about")))]
    Names {
        #[clap(subcommand)]
        command: names::NamesSubcommands,
    },

    #[clap(about(tr!("scripts-about")))]
    Scripts {
        #[clap(subcommand)]
        command: scripts::ScriptsSubcommands,
    },

    #[clap(about(tr!("param-about")))]
    Param {
        script: Option<String>,
        parameter: Option<String>,
        value: Option<String>,
    },

    #[clap(about(tr!("completions-about")))]
    Completions { shell: clap_complete::Shell },
}

pub async fn handle_command(subcommand: Subcommands) -> Result<()> {
    match subcommand {
        Subcommands::Canvas { command } => canvas::handle_command(command).await,
        Subcommands::Config { command } => config::handle_command(command).await,
        Subcommands::ColorSchemes { command } => color_schemes::handle_command(command).await,
        Subcommands::Devices { command } => devices::handle_command(command).await,
        Subcommands::Status { command } => status::handle_command(command).await,
        Subcommands::Switch { command } => switch::handle_command(command).await,
        Subcommands::Profiles { command } => profiles::handle_command(command).await,
        Subcommands::Names { command } => names::handle_command(command).await,
        Subcommands::Scripts { command } => scripts::handle_command(command).await,
        Subcommands::Param {
            script,
            parameter,
            value,
        } => param::handle_command(script, parameter, value).await,
        Subcommands::Completions { shell } => completions::handle_command(shell).await,
    }
}
