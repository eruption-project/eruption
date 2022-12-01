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

use colored::*;
use comfy_table::modifiers::UTF8_ROUND_CORNERS;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Attribute, Cell, ContentArrangement, Table};
use dbus::nonblock::stdintf::org_freedesktop_dbus::Properties;
use same_file::is_same_file;
use std::sync::atomic::Ordering;

use crate::dbus_client;
use crate::dbus_client::dbus_system_bus;
use crate::profiles::Profile;
use crate::scripting::manifest::Manifest;
use crate::scripting::parameters::{
    ManifestParameter, ManifestValue, ProfileParameter, TypedValue,
};

type Result<T> = std::result::Result<T, eyre::Error>;

pub async fn handle_command(
    script_name: Option<String>,
    parameter_name: Option<String>,
    value: Option<String>,
) -> Result<()> {
    let profile_name = get_active_profile().await.map_err(|e| {
        eprintln!(
            "Could not determine the currently active profile! Is the Eruption daemon running?"
        );
        e
    })?;

    let profile = Profile::load_fully(&PathBuf::from(&profile_name));
    let profile = match profile {
        Ok(profile) => profile,
        Err(err) => {
            eprintln!("Could not load the current profile ({})", profile_name);
            eprintln!("{}", err);
            return Ok(());
        }
    };

    if let Some(script_name) = script_name {
        let manifest = match find_manifest(&profile, script_name) {
            Some(manifest) => manifest,
            None => {
                println!("Script not found.");
                return Ok(());
            }
        };

        if let Some(parameter_name) = parameter_name {
            if let Some(value) = value {
                set_parameter(&profile, manifest, parameter_name, value)?;
            } else {
                list_specified_parameter(&profile, manifest, parameter_name);
            }
        } else {
            list_script_parameters(&profile, manifest);
        }
    } else {
        list_all_parameters(&profile);
    }

    Ok(())
}

fn find_manifest(profile: &Profile, script_name: String) -> Option<&Manifest> {
    // Get manifest by either its declared name or its script file
    profile.manifests.get(&script_name).or_else(|| {
        let script_path = PathBuf::from(&script_name);
        profile
            .manifests
            .values()
            .find(|m| is_same_file(&m.script_file, &script_path).unwrap_or(false))
    })
}

/// List parameters from all scripts in the currently active profile
fn list_all_parameters(profile: &Profile) {
    print_profile_header(profile);

    if crate::VERBOSE.load(Ordering::SeqCst) == 0 {
        // dump parameters set in .profile file
        println!("Profile parameters:\n");
        let mut table = create_table();
        for manifest in profile.manifests.values() {
            add_script_parameters(&mut table, profile, manifest, true);
        }
        println!("{table}");
    } else {
        // dump all available parameters that could be set in the .profile file
        println!("Available parameters:");
        let mut table = create_table();
        for manifest in profile.manifests.values() {
            add_script_parameters(&mut table, profile, manifest, false);
        }
        println!("{table}");
    }
}

/// List parameters from the specified script
fn list_script_parameters(profile: &Profile, manifest: &Manifest) {
    println!("Listing all parameters from the specified script:");
    let mut table = create_table();
    add_script_parameters(&mut table, profile, manifest, false);
    println!("{table}");
}

/// List the specified parameter from the specified script
fn list_specified_parameter(profile: &Profile, manifest: &Manifest, parameter: String) {
    let warning_about_hash_because_i_always_forget =
        "\nNote: Remember to quote the value when changing color parameters. Unquoted, the '#' mark signifies a comment.";

    let profile_parameter = profile.config.get_parameter(&manifest.name, &parameter);
    if let Some(profile_parameter) = profile_parameter {
        print_profile_header(profile);
        let mut table = create_table();
        table.add_row(profile_parameter_row(&manifest.name, profile_parameter));
        println!("{table}");

        if let TypedValue::Color(_) = profile_parameter.value {
            println!("{}", warning_about_hash_because_i_always_forget);
        }
    } else {
        // Not all script manifest parameters need be listed in the profile
        match manifest.config.get_parameter(&parameter) {
            Some(manifest_param) => {
                let mut table = create_table();
                table.add_row(manifest_parameter_row(&manifest.name, manifest_param));
                println!("{table}");

                if let ManifestValue::Color { .. } = manifest_param.manifest {
                    println!("{}", warning_about_hash_because_i_always_forget);
                }
            }
            None => println!("No parameter found."),
        }
    }
}

/// Set a parameter from the specified script in the currently active profile
fn set_parameter(
    profile: &Profile,
    manifest: &Manifest,
    parameter_name: String,
    value: String,
) -> Result<()> {
    print_profile_header(profile);

    // set param value
    dbus_client::set_parameter(
        &profile.profile_file.to_string_lossy(),
        &manifest.script_file.to_string_lossy(),
        &parameter_name,
        &value,
    )?;

    let default = manifest
        .config
        .get_parameter(&parameter_name)
        .map(|p| p.get_default());
    let mut table = create_table();
    table.add_row(vec![
        Cell::new(&manifest.name),
        Cell::new(&parameter_name),
        match default {
            Some(default) => get_value_cell(&default),
            None => Cell::new(""),
        },
        Cell::new(&value).add_attribute(Attribute::Bold),
    ]);
    println!("{table}");

    Ok(())
}

/// Get the name of the currently active profile
async fn get_active_profile() -> Result<String> {
    let result: String = dbus_system_bus("/org/eruption/profile")
        .await?
        .get("org.eruption.Profile", "ActiveProfile")
        .await?;

    Ok(result)
}

fn print_profile_header(profile: &Profile) {
    println!(
        "{}:\t{} ({})\n{}:\t{}\n{}:\t{:?}\n",
        "Profile".bold(),
        profile.name,
        profile.id,
        "Description".bold(),
        profile.description,
        "Scripts".bold(),
        profile.active_scripts,
    );
}

fn create_table() -> Table {
    let mut table = Table::new();
    table
        .load_preset(UTF8_FULL)
        .apply_modifier(UTF8_ROUND_CORNERS)
        .set_content_arrangement(ContentArrangement::Dynamic)
        .set_header(vec!["Script", "Parameter", "Default", "Profile Value"]);
    table
}

fn add_script_parameters(
    table: &mut Table,
    profile: &Profile,
    manifest: &Manifest,
    profile_parameters_only: bool,
) {
    let profile_script_parameters = profile.config.get_parameters(&manifest.name);
    if profile_parameters_only && profile_script_parameters.is_none() {
        return;
    }

    for manifest_parameter in manifest.config.iter() {
        let profile_parameter =
            profile_script_parameters.and_then(|p| p.get_parameter(&manifest_parameter.name));
        match profile_parameter {
            Some(profile_parameter) => {
                table.add_row(profile_parameter_row(&manifest.name, profile_parameter));
            }
            None => {
                if profile_parameters_only {
                    continue;
                }
                table.add_row(manifest_parameter_row(&manifest.name, manifest_parameter));
            }
        }
    }
}

fn manifest_parameter_row(script_name: &str, parameter: &ManifestParameter) -> Vec<Cell> {
    vec![
        Cell::new(script_name),
        Cell::new(&parameter.name),
        get_value_cell(&parameter.get_default()),
        Cell::new("(use default)").add_attribute(Attribute::Italic),
    ]
}

fn profile_parameter_row(script_name: &str, parameter: &ProfileParameter) -> Vec<Cell> {
    let default_value = parameter.get_default();
    if let Some(default_value) = default_value {
        let value_attribute = if parameter.value == default_value {
            Attribute::NoBold
        } else {
            Attribute::Bold
        };
        vec![
            Cell::new(script_name),
            Cell::new(&parameter.name),
            get_value_cell(&default_value),
            get_value_cell(&parameter.value).add_attribute(value_attribute),
        ]
    } else {
        vec![
            Cell::new(script_name),
            Cell::new(&parameter.name),
            Cell::new(""),
            get_value_cell(&parameter.value),
        ]
    }
}

fn get_value_cell(value: &TypedValue) -> Cell {
    let mut cell = Cell::new(value);
    if let TypedValue::Color(rgb) = value {
        cell = apply_rgb(cell, *rgb);
    }
    cell
}

fn apply_rgb(cell: Cell, rgb: u32) -> Cell {
    let (r, g, b) = ((rgb >> 0o20 & 0xff), (rgb >> 0o10 & 0xff), (rgb & 0xff));
    // Magic numbers from https://stackoverflow.com/a/3943023/1991305
    let fg = if (r * 299 + g * 587 + b * 114) > 128000 {
        comfy_table::Color::Black
    } else {
        comfy_table::Color::White
    };
    let bg = comfy_table::Color::Rgb {
        r: r as u8,
        g: g as u8,
        b: b as u8,
    };

    cell.bg(bg).fg(fg)
}
