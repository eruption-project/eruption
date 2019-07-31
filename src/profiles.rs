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

use failure::Fail;
use log::*;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

pub type Result<T> = std::result::Result<T, ProfileError>;

#[derive(Debug, Fail)]
pub enum ProfileError {
    #[fail(display = "Could not open profile file for reading")]
    OpenError {},

    #[fail(display = "Could not parse profile file")]
    ParseError {},
    // #[fail(display = "Unknown error: {}", description)]
    // UnknownError { description: String },
}

fn default_id() -> usize {
    0
}

fn default_profile_file() -> PathBuf {
    "".into()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Profile {
    #[serde(default = "default_id")]
    pub id: usize,
    #[serde(default = "default_profile_file")]
    pub profile_file: PathBuf,

    pub name: String,
    pub description: String,
}

impl Profile {
    pub fn new(id: usize, profile_file: &Path) -> Result<Self> {
        // parse manifest
        match fs::read_to_string(profile_file) {
            Ok(toml) => {
                // parse manifest
                match toml::de::from_str::<Self>(&toml) {
                    Ok(mut result) => {
                        // fill in required fields
                        result.id = id;
                        result.profile_file = profile_file.to_path_buf();

                        Ok(result)
                    }

                    Err(_e) => Err(ProfileError::ParseError {}),
                }
            }

            Err(_e) => Err(ProfileError::OpenError {}),
        }
    }

    pub fn from(profile_file: &Path) -> Result<Self> {
        Self::new(default_id(), profile_file)
    }
}

pub fn get_profiles(profile_path: &Path) -> Result<Vec<Profile>> {
    let profile_files = get_profile_files(&profile_path).unwrap();

    let mut errors_present = false;
    let mut result: Vec<Profile> = vec![];

    for (id, profile_file) in profile_files.iter().enumerate() {
        match Profile::new(id, &profile_file) {
            Ok(profile) => {
                result.push(profile);
            }

            Err(e) => {
                errors_present = true;
                error!(
                    "Could not process profile '{}': {}",
                    profile_file.to_string_lossy(),
                    e
                );
            }
        }
    }

    if errors_present {
        warn!("An error occurred during processing of profiles");
    }

    Ok(result)
}

pub fn get_profile_files(profile_path: &Path) -> Result<Vec<PathBuf>> {
    let paths = fs::read_dir(&profile_path).unwrap();

    Ok(paths
        .map(|p| p.unwrap().path())
        .filter(|p| {
            if p.extension().is_some() {
                return p.extension().unwrap() == "profile";
            }

            false
        })
        .collect())
}
