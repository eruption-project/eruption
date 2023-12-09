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

mod about;
mod status;

use std::path::PathBuf;

use crate::translations::tr;

type Result<T> = std::result::Result<T, eyre::Error>;

// Sub-commands
#[derive(Debug, clap::Parser)]
pub enum Subcommands {
    #[clap(display_order = 0, about(tr!("search-about")))]
    Search { query: String },

    #[clap(display_order = 1, about(tr!("install-about")))]
    Install { packages: Vec<String> },

    #[clap(display_order = 2, about(tr!("update-about")))]
    Update { packages: Option<Vec<String>> },

    #[clap(display_order = 3, about(tr!("remove-about")))]
    Remove { packages: Vec<String> },

    #[clap(display_order = 4, about(tr!("download-about")))]
    Download { packages: Vec<String> },

    #[clap(display_order = 5, about(tr!("publish-about")))]
    Publish { directory: PathBuf },

    #[clap(display_order = 6, about(tr!("info-about")))]
    Info { packages: Vec<String> },

    #[clap(display_order = 7, about(tr!("status-about")))]
    Status {},

    #[clap(display_order = 8, hide = true, about(tr!("completions-about")))]
    Completions { shell: clap_complete::Shell },

    #[clap(display_order = 9, hide = true, about(tr!("manpages-about")))]
    Manpages {},
}

pub async fn handle_command(_subcommand: Subcommands) -> Result<()> {
    /* match subcommand {
        Subcommands::Search { query } => search::handle_command(query).await,
        Subcommands::Install { packages } => install::handle_command(packages).await,
        Subcommands::Update { packages } => update::handle_command(packages).await,
        Subcommands::Remove { packages } => remove::handle_command(packages).await,
        Subcommands::Download { packages } => download::handle_command(packages).await,
        Subcommands::Publish { directory } => publish::handle_command(directory).await,
        Subcommands::Info { packages } => info::handle_command(packages).await,
        Subcommands::Status {} => status::handle_command().await,
        Subcommands::Completions { shell } => completions::handle_command(shell).await,
        Subcommands::Manpages {} => manpages::handle_command().await,
    } */

    Ok(())
}
