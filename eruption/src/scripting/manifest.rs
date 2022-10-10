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

#![allow(dead_code)]

use log::*;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::fs;
use std::path::{Path, PathBuf};

use crate::profiles::Profile;
use crate::scripting::parameters::{ManifestConfiguration, PlainParameter, ToPlainParameter};
use crate::util;
use crate::util::get_script_dirs;

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, Clone, thiserror::Error)]
pub enum ManifestError {
    #[error("Could not open file for reading")]
    OpenError,

    #[error("Could not parse manifest file")]
    ParseError,

    #[error("Could not enumerate script files")]
    ScriptEnumerationError,

    #[error("Could not parse a param value")]
    ParseParamError,
}

fn default_script_file() -> PathBuf {
    "".into()
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Manifest {
    #[serde(default = "default_script_file")]
    pub script_file: PathBuf,

    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub min_supported_version: String,
    pub tags: Option<Vec<ScriptTag>>,
    #[serde(default)]
    pub config: ManifestConfiguration,
}

impl std::cmp::PartialOrd for Manifest {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.name.cmp(&other.name))
    }
}

impl Manifest {
    pub fn load(script_path: &Path) -> Result<Self> {
        let (script_path, manifest_path) = verify_script_and_manifest_paths(script_path)?;

        match fs::read_to_string(&manifest_path) {
            Ok(toml) => {
                // parse manifest
                match toml::de::from_str::<Self>(&toml) {
                    Ok(mut result) => {
                        // fill in required fields, after parsing
                        result.script_file = script_path;

                        Ok(result)
                    }

                    Err(e) => {
                        error!("{}", e);
                        println!("{}", e);
                        Err(ManifestError::ParseError {}.into())
                    }
                }
            }

            Err(_e) => Err(ManifestError::OpenError {}.into()),
        }
    }

    pub fn get_merged_parameters(&self, profile: &Profile) -> Vec<PlainParameter> {
        let profile_script_parameters = profile.config.get_parameters(&self.name);
        if let Some(profile_script_parameters) = profile_script_parameters {
            self.config
                .iter()
                .map(|manifest_parameter| {
                    match profile_script_parameters.get_parameter(&manifest_parameter.name) {
                        Some(profile_parameter) => profile_parameter.to_plain_parameter(),
                        None => {
                            debug!(
                                "Parameter {} is undefined. Using defaults from script manifest.",
                                manifest_parameter.name
                            );
                            manifest_parameter.to_plain_parameter()
                        }
                    }
                })
                .collect()
        } else {
            debug!("Active profile does not have {} config. Using config parameters from script manifest.", &self.name);
            self.config.iter().map(|p| p.to_plain_parameter()).collect()
        }
    }
}

fn verify_script_and_manifest_paths(script_path: &Path) -> Result<(PathBuf, PathBuf)> {
    let script_path = if script_path.exists() {
        script_path.to_owned()
    } else {
        match util::match_script_path(&script_path) {
            Ok(script_path) => script_path,
            Err(error) => {
                error!(
                    "Script file {} cannot be found: {}",
                    script_path.display(),
                    error
                );
                return Err(ManifestError::OpenError {}.into());
            }
        }
    };

    let manifest_path = util::get_manifest_for(&script_path);

    let script_path = match script_path.canonicalize() {
        Ok(script_path) => script_path,
        Err(err) => {
            error!(
                "Script file path {} could not be canonicalized: {}",
                script_path.display(),
                err
            );
            return Err(ManifestError::OpenError {}.into());
        }
    };

    let manifest_path = match manifest_path.canonicalize() {
        Ok(manifest_path) => manifest_path,
        Err(err) => {
            error!(
                "Manifest file path {} could not be canonicalized: {}",
                manifest_path.display(),
                err
            );
            return Err(ManifestError::OpenError {}.into());
        }
    };

    if let Err(err) = util::demand_file_is_accessible(&script_path) {
        error!(
            "Script file {} is not accessible: {}",
            script_path.display(),
            err
        );

        return Err(ManifestError::OpenError {}.into());
    }

    if let Err(err) = util::demand_file_is_accessible(&manifest_path) {
        error!(
            "Manifest file for script {} is not accessible: {}",
            script_path.display(),
            err
        );

        return Err(ManifestError::OpenError {}.into());
    }

    Ok((script_path, manifest_path))
}

/// Get a `Vec` of `PathBufs` of available script files in the directory `script_path`.
#[allow(dead_code)]
pub fn get_script_files() -> Result<Vec<PathBuf>> {
    get_script_files_from(&get_script_dirs())
}

#[allow(dead_code)]
pub fn get_script_files_from(script_dirs: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut result = vec![];

    for script_path in script_dirs {
        if let Ok(paths) = fs::read_dir(script_path) {
            let mut script_paths = paths
                .map(|p| p.unwrap().path())
                .filter(|p| {
                    if p.extension().is_some() {
                        return p.extension().unwrap_or_else(|| OsStr::new("")) == "lua";
                    }

                    false
                })
                .collect::<Vec<PathBuf>>();

            result.append(&mut script_paths);
        }
    }

    Ok(result)
}

#[allow(dead_code)]
pub fn get_scripts() -> Result<Vec<Manifest>> {
    let script_files = get_script_files().unwrap();

    let mut errors_present = false;
    let mut result: Vec<Manifest> = vec![];

    for script_file in &script_files {
        match Manifest::load(script_file) {
            Ok(manifest) => {
                result.push(manifest);
            }

            Err(e) => {
                errors_present = true;
                error!(
                    "Could not process manifest file for script {}: {}",
                    script_file.display(),
                    e
                );
            }
        }
    }

    if errors_present {
        warn!("An error occurred during processing of manifest files");
    }

    // sort and group by tags, then by name
    result.sort_by(|a, b| {
        let empty_vec = vec![];

        let tags_a = a.tags.as_ref().unwrap_or(&empty_vec);
        let tags_b = b.tags.as_ref().unwrap_or(&empty_vec);

        let result = tags_a.cmp(tags_b);
        if result == std::cmp::Ordering::Equal {
            a.name.cmp(&b.name)
        } else {
            result
        }
    });

    Ok(result)
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, PartialOrd, Eq, Ord)]
pub enum ScriptTag {
    // Script "classes"
    /// Script may be categorized as a "background"
    Background,

    /// Script filters data, input/processing/output
    Filter,

    /// Script may be categorized as an "effect"
    Effect,

    /// Script may be categorized as a collection of macros
    Macros,

    // other effects
    /// Script is of pre-release quality
    BetaVersion,

    /// Script is vendor-supplied
    Vendor,

    /// Script is from a 3rd party
    ThirdParty,

    /// Some kind of noise function
    Noise,

    /// Some kind of gradient
    Gradient,

    /// Script visualizes audio in some way
    AudioVisualization,

    /// Script should be considered a technology demo
    Demo,
}

impl ScriptTag {
    pub fn get_description(&self) -> String {
        match *self {
            // "classes"
            ScriptTag::Background => "Background".into(),
            ScriptTag::Filter => "Filter".into(),
            ScriptTag::Effect => "Effect".into(),
            ScriptTag::Macros => "Macros".into(),

            // other tags
            ScriptTag::BetaVersion => "Beta version".into(),
            ScriptTag::Demo => "Technology demo".into(),
            ScriptTag::Vendor => "Vendor supplied".into(),
            ScriptTag::ThirdParty => "3rd party script".into(),
            ScriptTag::Noise => "Some random noise function".into(),
            ScriptTag::Gradient => "Some kind of gradient".into(),
            ScriptTag::AudioVisualization => "Script visualizes audio in some way".into(),
        }
    }

    pub fn get_css_class(&self) -> String {
        match *self {
            // "classes"
            ScriptTag::Background => "class-background".into(),
            ScriptTag::Filter => "class-filter".into(),
            ScriptTag::Effect => "class-effect".into(),
            ScriptTag::Macros => "class-macros".into(),

            // other tags
            ScriptTag::BetaVersion => "beta-version".into(),
            ScriptTag::Demo => "demo".into(),
            ScriptTag::Vendor => "vendor".into(),
            ScriptTag::ThirdParty => "3rd-party".into(),
            ScriptTag::Noise => "noise".into(),
            ScriptTag::Gradient => "gradient".into(),
            ScriptTag::AudioVisualization => "audio-visualization".into(),
        }
    }

    pub fn get_badge_css_class(&self) -> String {
        match *self {
            // "classes"
            ScriptTag::Background => "badge-background".into(),
            ScriptTag::Filter => "badge-filter".into(),
            ScriptTag::Effect => "badge-effect".into(),
            ScriptTag::Macros => "badge-macros".into(),

            // other tags
            ScriptTag::BetaVersion => "badge-beta-version".into(),
            ScriptTag::Demo => "badge-demo".into(),
            ScriptTag::Vendor => "badge-vendor".into(),
            ScriptTag::ThirdParty => "badge-3rd-party".into(),
            ScriptTag::Noise => "badge-noise".into(),
            ScriptTag::Gradient => "badge-gradient".into(),
            ScriptTag::AudioVisualization => "badge-audio-visualization".into(),
        }
    }
}
