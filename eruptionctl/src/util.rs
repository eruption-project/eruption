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

use crate::manifest::{self, Manifest};
use crate::profiles;

type Result<T> = std::result::Result<T, eyre::Error>;

pub fn enumerate_scripts<P: AsRef<Path>>(path: P) -> Result<Vec<Manifest>> {
    manifest::get_scripts(&path.as_ref())
}

pub fn enumerate_profiles<P: AsRef<Path>>(path: P) -> Result<Vec<profiles::Profile>> {
    profiles::get_profiles(&path.as_ref())
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

    Command::new(env!("EDITOR"))
        .args(&[file_name.as_ref().to_string_lossy().to_string()])
        .status()?;

    Ok(())
}
