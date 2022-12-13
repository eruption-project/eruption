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

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::constants;
use crate::profiles;
use crate::profiles::Profile;

type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum UtilError {
    #[error("File not found: {description}")]
    FileNotFound { description: String },

    #[error("Read failed: {description}")]
    FileReadError {
        #[source]
        source: std::io::Error,
        description: String,
    },

    #[error("Not a file")]
    NotAFile {},

    #[error("Profile error: {err}")]
    ProfileError { err: eyre::Error },
}

pub fn get_profile_dirs() -> Vec<PathBuf> {
    get_dirs(
        "global.profile_dirs",
        constants::DEFAULT_PROFILE_DIR,
        "profile",
    )
}

pub fn get_script_dirs() -> Vec<PathBuf> {
    get_dirs(
        "global.script_dirs",
        constants::DEFAULT_SCRIPT_DIR,
        "script",
    )
}

fn get_dirs(config_key: &str, fallback_dir: &str, fallback_description: &str) -> Vec<PathBuf> {
    let config = crate::CONFIG.lock();

    let script_dirs = config
        .as_ref()
        .unwrap()
        .get::<Vec<String>>(config_key)
        .unwrap_or_else(|_| vec![]);

    let mut result = script_dirs
        .iter()
        .map(PathBuf::from)
        .collect::<Vec<PathBuf>>();

    // if we could not determine a valid set of paths, use a hard coded fallback instead
    if result.is_empty() {
        tracing::warn!("Using default fallback {} directory", fallback_description);

        let path = PathBuf::from(fallback_dir);
        result.push(path);
    }

    result
}

/// Returns the associated manifest path in `PathBuf` for the script `script_path`.
pub fn get_manifest_for(script_file: &Path) -> PathBuf {
    let mut manifest_path = script_file.to_path_buf();
    manifest_path.set_extension("lua.manifest");

    manifest_path
}

pub fn demand_file_is_accessible<P: AsRef<Path>>(p: P) -> Result<()> {
    // Does the path exist?
    let path = match fs::canonicalize(p) {
        Ok(path) => path,
        Err(e) => {
            return Err(UtilError::FileReadError {
                source: e,
                description: "Could not find file".to_owned(),
            }
            .into())
        }
    };

    // Is the metadata readable?
    let metadata = match fs::metadata(&path) {
        Ok(metadata) => metadata,
        Err(e) => {
            return Err(UtilError::FileReadError {
                source: e,
                description: "Could not read metadata".to_owned(),
            }
            .into())
        }
    };

    // Is the path a regular file?  (Symlinks will have been canonicalized to regular files.)
    if !metadata.is_file() {
        return Err(UtilError::NotAFile {}.into());
    }

    // Is the file readable?
    match fs::File::open(&path) {
        Err(e) => {
            return Err(UtilError::FileReadError {
                source: e,
                description: "Could not open file".to_owned(),
            }
            .into())
        }
        _ => {}
    };

    Ok(())
}

pub fn edit_file<P: AsRef<Path>>(file_name: P) -> Result<()> {
    println!("Editing: {}", &file_name.as_ref().to_string_lossy());

    Command::new(std::env::var("EDITOR").unwrap_or_else(|_| "/usr/bin/nano".to_string()))
        .args(&[file_name.as_ref().to_string_lossy().to_string()])
        .status()?;

    Ok(())
}

pub fn match_profile_by_name(profile_name: &str) -> Result<Profile> {
    let profile_path = PathBuf::from(&profile_name);
    if profile_path.is_file() {
        match Profile::load_file_and_state_only(&profile_path) {
            Ok(profile) => Ok(profile),
            Err(err) => Err(UtilError::ProfileError { err }.into()),
        }
    } else {
        let profiles = profiles::get_profiles().unwrap_or_else(|_| vec![]);
        let profile = profiles
            .into_iter()
            .find(|p| p.profile_file.to_string_lossy() == profile_name);
        match profile {
            Some(profile) => Ok(profile),
            None => Err(UtilError::FileNotFound {
                description: profile_name.into(),
            }
            .into()),
        }
    }
}

pub fn match_profile_path<P: AsRef<Path>>(profile_file: &P) -> Result<PathBuf> {
    match_path(get_profile_dirs(), profile_file)
}

pub fn match_script_path<P: AsRef<Path>>(script_file: &P) -> Result<PathBuf> {
    match_path(get_script_dirs(), script_file)
}

fn match_path<P: AsRef<Path>>(dirs: Vec<PathBuf>, file: &P) -> Result<PathBuf> {
    let file = file.as_ref();

    for dir in dirs.iter() {
        let profile_path = dir.join(file);

        if let Ok(metadata) = fs::metadata(&profile_path) {
            if metadata.is_file() {
                return Ok(profile_path);
            }
        }
    }

    Err(UtilError::FileNotFound {
        description: format!("Could not find file in search path(s): {}", &file.display()),
    }
    .into())
}
