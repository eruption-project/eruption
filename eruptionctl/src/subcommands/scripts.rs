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


use colored::*;
use same_file::is_same_file;

use crate::scripting::manifest::{self, Manifest};
use crate::util;

type Result<T> = std::result::Result<T, eyre::Error>;

/// Subcommands of the "scripts" command
#[derive(Debug, clap::Parser)]
pub enum ScriptsSubcommands {
    /// Show info about a script
    Info { script_name: String },

    /// Edit a script
    Edit { script_name: String },

    /// List available scripts
    List,
}

pub async fn handle_command(command: ScriptsSubcommands) -> Result<()> {
    match command {
        ScriptsSubcommands::Edit { script_name } => edit_command(script_name).await,
        ScriptsSubcommands::List => list_command().await,
        ScriptsSubcommands::Info { script_name } => info_command(script_name).await,
    }
}

async fn edit_command(script_name: String) -> Result<()> {
    match find_script_by_name(&script_name) {
        Some(manifest) => util::edit_file(&manifest.script_file)?,
        None => eprintln!("Script not found."),
    }

    Ok(())
}

async fn list_command() -> Result<()> {
    for s in get_script_list()? {
        println!("{}: {}", s.0.bold(), s.1);
    }

    Ok(())
}

async fn info_command(script_name: String) -> Result<()> {
    match find_script_by_name(&script_name) {
        Some(script) => {
            let empty = vec![];
            println!(
                "Lua script:\t{} ({})\nDaemon version:\t{}\nAuthor:\t\t{}\nDescription:\t{}\nTags:\t\t{:?}",
                script.name,
                script.version,
                script.min_supported_version,
                script.author,
                script.description,
                script.tags.as_ref().unwrap_or(&empty),
            );
        }
        None => eprintln!("Script not found."),
    }

    Ok(())
}

/// Enumerate all available scripts
fn get_script_list() -> Result<Vec<(String, String)>> {
    let scripts = manifest::get_scripts()?;

    let result = scripts
        .iter()
        .map(|s| {
            (
                format!("{} - {}", s.name.clone(), s.description.clone()),
                s.script_file.to_string_lossy().to_string(),
            )
        })
        .collect();

    Ok(result)
}

fn find_script_by_name(script_name: &str) -> Option<Manifest> {
    // Find the script specified, either by script name or filename.
    let script_path = PathBuf::from(script_name);
    let script_path = if script_path.is_file() {
        if let Ok(script) = Manifest::load(&script_path) {
            return Some(script);
        } else {
            Some(script_path)
        }
    } else {
        None
    };

    let scripts = manifest::get_scripts();
    let scripts = match scripts {
        Ok(scripts) => scripts,
        Err(err) => {
            println!("{}", err);
            return None;
        }
    };

    scripts
        .into_iter()
        .find(|script| match (script.name == script_name, &script_path) {
            (true, _) => true,
            (_, None) => script.script_file.file_name().unwrap().to_string_lossy() == script_name,
            (_, Some(script_path)) => {
                is_same_file(&script.script_file, script_path).unwrap_or(false)
            }
        })
}
