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
*/

#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::constants;
use crate::manifest::{self, Manifest};
use crate::profiles;

type Result<T> = std::result::Result<T, eyre::Error>;

pub fn enumerate_scripts<P: AsRef<Path>>(path: P) -> Result<Vec<Manifest>> {
    manifest::get_scripts(&path.as_ref())
}

pub fn get_profile_dirs() -> Vec<PathBuf> {
    // process configuration file
    let config_file = constants::DEFAULT_CONFIG_FILE;

    let mut config = config::Config::default();
    if let Err(e) = config.merge(config::File::new(&config_file, config::FileFormat::Toml)) {
        log::error!("Could not parse configuration file: {}", e);
    }

    let mut result = vec![];

    let profile_dirs = config
        .get::<Vec<String>>("global.profile_dirs")
        .unwrap_or_else(|_| vec![]);

    let mut profile_dirs = profile_dirs
        .iter()
        .map(|e| PathBuf::from(e))
        .collect::<Vec<PathBuf>>();

    result.append(&mut profile_dirs);

    // if we could not determine a valid set of paths, use a hard coded fallback instead
    if result.is_empty() {
        log::warn!("Using default fallback profile directory");

        let path = PathBuf::from(constants::DEFAULT_PROFILE_DIR);
        result.push(path);
    }

    result
}

pub fn enumerate_profiles() -> Result<Vec<profiles::Profile>> {
    let mut result = profiles::get_profiles()?;

    // sort profiles by their name
    result.sort_by(|lhs, rhs| lhs.name.cmp(&rhs.name));

    Ok(result)
}

/// Returns the associated manifest path in `PathBuf` for the script `script_path`.
pub fn get_manifest_for(script_file: &Path) -> PathBuf {
    let mut manifest_path = script_file.to_path_buf();
    manifest_path.set_extension("lua.manifest");

    manifest_path
}

pub fn is_file_accessible<P: AsRef<Path>>(p: P) -> std::io::Result<String> {
    fs::read_to_string(p)
}

pub fn edit_file<P: AsRef<Path>>(file_name: P) -> Result<()> {
    println!("Editing: {}", &file_name.as_ref().to_string_lossy());

    Command::new(std::env::var("EDITOR").unwrap_or_else(|_| "/usr/bin/nano".to_string()))
        .args(&[file_name.as_ref().to_string_lossy().to_string()])
        .status()?;

    Ok(())
}
