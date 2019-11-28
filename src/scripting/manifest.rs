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
use std::str::FromStr;

use crate::profiles;
use crate::util;

pub type Result<T> = std::result::Result<T, ManifestError>;

#[derive(Debug, Fail, Clone)]
pub enum ManifestError {
    #[fail(display = "Could not open file for reading")]
    OpenError {},

    #[fail(display = "Could not parse manifest file")]
    ParseError {},

    #[fail(display = "Could not enumerate script files")]
    ScriptEnumerationError {},

    #[fail(display = "Could not parse param value")]
    ParseParamError {},
    // #[fail(display = "Unknown error: {}", description)]
    // UnknownError { description: String },
}

fn default_id() -> usize {
    0
}

fn default_script_file() -> PathBuf {
    "".into()
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ConfigParam {
    Int {
        name: String,
        description: String,
        default: i64,
    },
    Float {
        name: String,
        description: String,
        default: f64,
    },
    Bool {
        name: String,
        description: String,
        default: bool,
    },
    String {
        name: String,
        description: String,
        default: String,
    },
    Color {
        name: String,
        description: String,
        default: u32,
    },
}

pub trait ParseConfig {
    fn parse_config_param(&self, param: &str, val: &str) -> Result<profiles::ConfigParam>;
}

impl ParseConfig for Vec<ConfigParam> {
    fn parse_config_param(&self, param: &str, val: &str) -> Result<profiles::ConfigParam> {
        for p in self.iter() {
            match &p {
                ConfigParam::Int { name, .. } => {
                    if name == param {
                        let value =
                            i64::from_str(&val).map_err(|_e| ManifestError::ParseParamError {})?;

                        return Ok(profiles::ConfigParam::Int {
                            name: name.to_string(),
                            value,
                        });
                    }
                }

                ConfigParam::Float { name, .. } => {
                    if name == param {
                        let value =
                            f64::from_str(&val).map_err(|_e| ManifestError::ParseParamError {})?;

                        return Ok(profiles::ConfigParam::Float {
                            name: name.to_string(),
                            value,
                        });
                    }
                }

                ConfigParam::Bool { name, .. } => {
                    if name == param {
                        let value =
                            bool::from_str(&val).map_err(|_e| ManifestError::ParseParamError {})?;

                        return Ok(profiles::ConfigParam::Bool {
                            name: name.to_string(),
                            value,
                        });
                    }
                }

                ConfigParam::String { name, .. } => {
                    if name == param {
                        let value = val.to_owned();

                        return Ok(profiles::ConfigParam::String {
                            name: name.to_string(),
                            value,
                        });
                    }
                }

                ConfigParam::Color { name, .. } => {
                    if name == param {
                        let value = u32::from_str_radix(&val[1..], 16)
                            .map_err(|_e| ManifestError::ParseParamError {})?;

                        return Ok(profiles::ConfigParam::Color {
                            name: name.to_string(),
                            value,
                        });
                    }
                }
            }
        }

        Err(ManifestError::ParseParamError {})
    }
}

pub trait GetAttr {
    fn get_name(&self) -> &String;
    fn get_default(&self) -> String;
}

impl GetAttr for ConfigParam {
    fn get_name(&self) -> &String {
        match self {
            ConfigParam::Int { ref name, .. } => name,

            ConfigParam::Float { ref name, .. } => name,

            ConfigParam::Bool { ref name, .. } => name,

            ConfigParam::String { ref name, .. } => name,

            ConfigParam::Color { ref name, .. } => name,
        }
    }

    fn get_default(&self) -> String {
        match self {
            ConfigParam::Int { ref default, .. } => format!("{}", default),

            ConfigParam::Float { ref default, .. } => format!("{}", default),

            ConfigParam::Bool { ref default, .. } => format!("{}", default),

            ConfigParam::String { ref default, .. } => default.to_owned(),

            ConfigParam::Color { ref default, .. } => format!("#{:06x}", default),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Manifest {
    #[serde(default = "default_id")]
    pub id: usize,
    #[serde(default = "default_script_file")]
    pub script_file: PathBuf,

    pub name: String,
    pub description: String,
    pub version: String,
    pub author: String,
    pub min_supported_version: String,
    pub config: Vec<ConfigParam>,
}

impl Manifest {
    pub fn new(id: usize, script: &Path) -> Result<Self> {
        // parse manifest
        match fs::read_to_string(util::get_manifest_for(script)) {
            Ok(toml) => {
                // parse manifest
                match toml::de::from_str::<Self>(&toml) {
                    Ok(mut result) => {
                        // fill in required fields, after parsing
                        result.id = id;
                        result.script_file = script.to_path_buf();

                        Ok(result)
                    }

                    Err(e) => {
                        error!("{}", e);
                        Err(ManifestError::ParseError {})
                    }
                }
            }

            Err(_e) => Err(ManifestError::OpenError {}),
        }
    }

    pub fn from(script: &Path) -> Result<Self> {
        Self::new(default_id(), script)
    }
}

/// Get a `Vec` of `PathBufs` of available script files in the directory `script_path`.
pub fn get_script_files(script_path: &Path) -> Result<Vec<PathBuf>> {
    match fs::read_dir(script_path) {
        Ok(paths) => Ok(paths
            .map(|p| p.unwrap().path())
            .filter(|p| {
                if p.extension().is_some() {
                    return p.extension().unwrap() == "lua";
                }

                false
            })
            .collect()),

        Err(_e) => Err(ManifestError::ScriptEnumerationError {}),
    }
}

pub fn get_scripts(script_path: &Path) -> Result<Vec<Manifest>> {
    let script_files = get_script_files(script_path).unwrap();

    let mut errors_present = false;
    let mut result: Vec<Manifest> = vec![];

    for (id, script_file) in script_files.iter().enumerate() {
        match Manifest::new(id, &script_file) {
            Ok(manifest) => {
                result.push(manifest);
            }

            Err(e) => {
                errors_present = true;
                error!(
                    "Could not process manifest file for script '{}': {}",
                    script_file.display(),
                    e
                );
            }
        }
    }

    if errors_present {
        warn!("An error occurred during processing of manifest files");
    }

    Ok(result)
}
