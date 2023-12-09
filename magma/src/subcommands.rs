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

mod completions;
mod manpages;

use crate::translations::tr;

type Result<T> = std::result::Result<T, eyre::Error>;

// Sub-commands
#[derive(Debug, clap::Parser)]
pub enum Subcommands {
    #[clap(display_order = 0, hide = true, about(tr!("completions-about")))]
    Completions { shell: clap_complete::Shell },

    #[clap(display_order = 1, hide = true, about(tr!("manpages-about")))]
    Manpages {},
}

pub async fn handle_command(subcommand: Subcommands) -> Result<()> {
    match subcommand {
        Subcommands::Completions { shell } => completions::handle_command(shell).await,
        Subcommands::Manpages {} => manpages::handle_command().await,
    }
}
