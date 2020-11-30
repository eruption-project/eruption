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

// use std::fs::File;
// use std::io::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};

// use log::*;

// pub type Result<T> = std::result::Result<T, eyre::Error>;

// #[derive(Debug, thiserror::Error)]
// pub enum UtilError {}

/// Returns the associated manifest path in `PathBuf` for the script `script_path`.
pub fn get_manifest_for(script_file: &Path) -> PathBuf {
    let mut manifest_path = script_file.to_path_buf();
    manifest_path.set_extension("lua.manifest");

    manifest_path
}

pub fn is_file_accessible<P: AsRef<Path>>(p: P) -> std::io::Result<String> {
    fs::read_to_string(p)
}

/// Checks whether a script file is readable
#[allow(dead_code)]
pub fn is_script_file_accessible(script_file: &Path) -> bool {
    is_file_accessible(script_file).is_ok()
}

/// Checks whether a script's manifest file is readable
#[allow(dead_code)]
pub fn is_manifest_file_accessible(script_file: &Path) -> bool {
    fs::read_to_string(get_manifest_for(script_file)).is_ok()
}
