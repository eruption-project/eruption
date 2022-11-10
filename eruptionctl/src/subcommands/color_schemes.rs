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

use std::{env, fs, path::PathBuf};

use clap;
use colored::*;

use crate::color_scheme::{ColorScheme, PywalColorScheme};
use crate::dbus_client;

type Result<T> = std::result::Result<T, eyre::Error>;

/// Sub-commands of the "color-schemes" command
#[derive(Debug, clap::Parser)]
pub enum ColorSchemesSubcommands {
    /// List all color schemes known to Eruption
    #[clap(display_order = 0)]
    List {},

    /// Add a new named color scheme
    #[clap(display_order = 1)]
    Add { name: String, colors: Vec<String> },

    /// Remove a color scheme by name
    #[clap(display_order = 2)]
    Remove { name: String },

    /// Import a color scheme from a file, e.g.: like the Pywal configuration
    #[clap(display_order = 3)]
    Import {
        #[clap(subcommand)]
        command: ColorSchemeImportSubcommands,
    },
}

/// Sub-commands of the "colorscheme" command
#[derive(Debug, clap::Parser)]
pub enum ColorSchemeImportSubcommands {
    /// Import an existing Pywal color scheme
    Pywal {
        /// Optionally specify the file name to the pywal color scheme
        file_name: Option<PathBuf>,

        /// Optimize palette
        #[clap(required = false, short, long, default_value = "false")]
        optimize: bool,
    },
}

pub async fn handle_command(command: ColorSchemesSubcommands) -> Result<()> {
    match command {
        ColorSchemesSubcommands::List {} => list_command().await,
        ColorSchemesSubcommands::Add { name, colors } => add_command(name, colors).await,
        ColorSchemesSubcommands::Remove { name } => remove_command(name).await,
        ColorSchemesSubcommands::Import { command } => import_command(command).await,
    }
}

async fn list_command() -> Result<()> {
    let color_schemes = dbus_client::get_color_schemes()?;

    println!("Color schemes:\n");

    for color_scheme in color_schemes {
        println!("{}", color_scheme.bold());
    }

    println!("\nStock gradients:\n");

    println!("system");
    println!("rainbow-smooth");
    println!("sinebow-smooth");
    println!("spectral-smooth");
    println!("rainbow-sharp");
    println!("sinebow-sharp");
    println!("spectral-sharp");

    Ok(())
}

async fn add_command(name: String, colors: Vec<String>) -> Result<()> {
    println!("Importing color scheme from commandline");

    if colors.len() % 4 != 0 {
        eprintln!("Invalid number of parameters specified, please use the 'RGBA' format");
    } else {
        let color_scheme = ColorScheme::try_from(colors)?;

        dbus_client::set_color_scheme(&name, &color_scheme)?;
    }

    Ok(())
}

async fn remove_command(name: String) -> Result<()> {
    println!("Removing color scheme: {}", name.bold());

    let result = dbus_client::remove_color_scheme(&name)?;

    if !result {
        eprintln!("The specified color scheme does not exist");
    }

    Ok(())
}

async fn import_command(command: ColorSchemeImportSubcommands) -> Result<()> {
    match command {
        ColorSchemeImportSubcommands::Pywal {
            file_name,
            optimize,
        } => import_pywal(file_name, optimize).await,
    }
}

async fn import_pywal(file_name: Option<PathBuf>, optimize: bool) -> Result<()> {
    let file_name = if let Some(path) = file_name {
        path
    } else {
        PathBuf::from(env::var("HOME")?).join(".cache/wal/colors.json")
    };

    println!(
        "Importing Pywal color scheme from: {}",
        file_name.display().to_string().bold()
    );

    let json_data = fs::read_to_string(&file_name)?;
    let mut pywal_color_scheme: PywalColorScheme = serde_json::from_str(&json_data)?;

    if optimize {
        pywal_color_scheme.optimize();
    }

    let color_scheme = ColorScheme::try_from(pywal_color_scheme)?;

    dbus_client::set_color_scheme("system", &color_scheme)?;

    Ok(())
}
