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

use clap::CommandFactory;
use std::env;

use crate::Options;

type Result<T> = std::result::Result<T, eyre::Error>;

pub async fn handle_command(shell: clap_complete::Shell) -> Result<()> {
    const BIN_NAME: &str = env!("CARGO_PKG_NAME");

    let mut command = Options::command();
    let mut fd = std::io::stdout();

    clap_complete::generate(shell, &mut command, BIN_NAME.to_string(), &mut fd);

    Ok(())
}
