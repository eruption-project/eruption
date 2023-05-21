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
use std::{env, path::Path};

use crate::Options;

type Result<T> = std::result::Result<T, eyre::Error>;

pub async fn handle_command() -> Result<()> {
    const BIN_NAME: &str = env!("CARGO_PKG_NAME");

    let command = Options::command();

    let man = clap_mangen::Man::new(command);
    let mut buffer: Vec<u8> = Default::default();

    man.render(&mut buffer)?;

    std::fs::write(
        Path::new(&env::var("MANPAGES_OUTPUT_DIR")?.to_string()).join(&format!("{BIN_NAME}.1")),
        buffer,
    )?;

    Ok(())
}
