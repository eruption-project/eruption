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

#![allow(dead_code)]

use crate::constants;
use indexmap::IndexMap;
use log::*;

use serde::{Deserialize, Serialize};
use std::default::Default;
use std::os::unix::prelude::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::{collections::BTreeMap, ffi::OsStr};
use std::{fs, io};
use uuid::Uuid;

use crate::scripting::manifest::Manifest;
use crate::scripting::parameters::{
    ProfileConfiguration, ProfileParameter, ProfileScriptParameters, TypedValue,
};

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum ProfileError {
    #[error("Could not open profile file for reading")]
    OpenError {},

    #[error("Could not parse profile file")]
    ParseError {},

    #[error("Could not save profile file: {msg}")]
    WriteError { msg: String },

    #[error("Could not find profile file from UUID")]
    FindError {},

    #[error("Could not enumerate profile files")]
    EnumError {},

    #[error("Could not set a config value in a profile: {msg}")]
    SetValueError { msg: String },

    #[error("Could not parse a param value")]
    ParseParamError {},
}

fn default_id() -> Uuid {
    Uuid::new_v4()
}

fn default_profile_file() -> PathBuf {
    "".into()
}

fn default_script_file() -> Vec<PathBuf> {
    vec![constants::DEFAULT_EFFECT_SCRIPT.into()]
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Profile {
    #[serde(default = "default_id")]
    pub id: Uuid,

    #[serde(default = "default_profile_file")]
    #[serde(skip)]
    pub profile_file: PathBuf,

    pub name: String,
    pub description: String,

    #[serde(default = "default_script_file")]
    pub active_scripts: Vec<PathBuf>,
    #[serde(default)]
    pub config: ProfileConfiguration,

    #[serde(skip)]
    pub manifests: IndexMap<String, Manifest>,
}

macro_rules! get_default_value {
    ($t:ident, $tval:ty, $rval:ty) => {
        paste::item! {
            pub fn [<get_default_ $t>] (&self, script_name: &str, parameter_name: &str) -> Option<$rval> {
                match self.get_parameter_default(script_name, parameter_name) {
                    Some($tval(value)) => Some(value.to_owned()),
                    _ => None,
                }
            }
        }
    };
}

macro_rules! get_config_value {
    ($t:ident, $pval:ty, $tval:ty) => {
        paste::item! {
            pub fn [<get_ $t _value>](&self, script_name: &str, parameter_name: &str) -> Option<&$tval> {
                match self.config.get_parameter(script_name, parameter_name) {
                    Some(ProfileParameter { value: $pval(value), .. } ) => Some(value),
                    None => {
                        debug!("Using default value for config param");
                        None
                    },
                    _ => {
                        debug!("Invalid data type");
                        None
                    }
                }
            }
        }
    };
}

macro_rules! set_config_value {
    ($t:ident, $pval:ty, $tval:ty) => {
        paste::item! {
            pub fn [<set_ $t _value>](&mut self, script_name: &str, parameter_name: &str, val: &$tval) -> Result<()> {
                match self.config.get_parameter_mut(script_name, parameter_name) {
                    Some(ref mut profile_parameter) => {
                        match profile_parameter.value {
                            $pval(_) => {
                                profile_parameter.value = $pval(val.to_owned());
                                Ok(())
                            }
                            _ => Err(ProfileError::SetValueError {
                                msg: "Invalid data type".into(),
                            }.into()),
                        }
                    }
                    _ => Err(ProfileError::SetValueError {
                        msg: "Could not get config".into(),
                    }.into())
                }
            }
        }
    };
}

impl Profile {
    /// Returns a failsafe profile that will work in almost all cases
    pub fn new_fail_safe() -> Self {
        let mut profile = Self {
            id: Uuid::new_v4(),
            name: "Failsafe mode".to_string(),
            description: "Failsafe mode virtual profile".to_string(),
            profile_file: PathBuf::from("failsafe.profile"),
            // force hardcoded directory for failsafe scripts
            active_scripts: vec![PathBuf::from(
                "/usr/share/eruption/scripts/lib/failsafe.lua",
            )],
            config: ProfileConfiguration::new(),
            manifests: IndexMap::new(),
        };

        if let Err(err) = profile.load_manifests() {
            error!("Could not open failsafe script. {}", err);
        } else {
            profile.merge_parameters();
        }

        profile
    }

    pub fn load_fully(profile_file: &Path) -> Result<Self> {
        // Deserialize the profile file
        let mut profile = Self::load_file_and_state_only(profile_file)?;

        // Load script manifests
        profile.load_manifests()?;

        // Apply manifest information to profile parameters
        profile.merge_parameters();

        Ok(profile)
    }

    pub fn load_file_and_state_only(profile_file: &Path) -> Result<Self> {
        // Deserialize the profile file
        let mut profile = Self::load_file_only(profile_file)?;

        // Load persisted profile state from disk, but ignore errors
        if let Err(e) = profile.load_params() {
            trace!("Error loading profile state from disk: {}", e);
        }

        Ok(profile)
    }

    // Just load the profile file itself, no state, no manifests.
    pub fn load_file_only(profile_file: &Path) -> Result<Self> {
        match fs::read_to_string(profile_file) {
            Ok(toml) => match toml::de::from_str::<Self>(&toml) {
                Ok(mut result) => {
                    result.profile_file = profile_file.to_path_buf();
                    Ok(result)
                }
                Err(e) => {
                    error!("Error parsing profile file. {}", e);
                    Err(ProfileError::ParseError {}.into())
                }
            },
            Err(e) => {
                error!("Error opening profile file. {}", e);
                Err(ProfileError::OpenError {}.into())
            }
        }
    }

    fn load_manifests(&mut self) -> Result<()> {
        for script_file in self.active_scripts.iter() {
            let manifest = Manifest::load(script_file);
            match manifest {
                Ok(manifest) => {
                    self.manifests.insert(manifest.name.to_owned(), manifest);
                }
                Err(err) => {
                    error!(
                        "Could not load script {} for profile {}: {}",
                        script_file.display(),
                        self.profile_file.display(),
                        err
                    );
                    return Err(ProfileError::OpenError {}.into());
                }
            }
        }
        Ok(())
    }

    fn merge_parameters(&mut self) {
        for manifest in self.manifests.values() {
            let profile_script_parameters = self.config.get_parameters_mut(&manifest.name);
            for manifest_parameter in manifest.config.iter() {
                let profile_parameter =
                    profile_script_parameters.get_parameter_mut(&manifest_parameter.name);
                if let Some(profile_parameter) = profile_parameter {
                    profile_parameter.manifest = Some(manifest_parameter.manifest.to_owned())
                }
            }
        }
    }

    pub fn find_by_uuid(uuid: Uuid) -> Result<Self> {
        let mut result = Err(ProfileError::FindError {}.into());

        if let Ok(profile_files) = get_profile_files() {
            'PROFILE_LOOP: for profile_file in profile_files.iter() {
                match Profile::load_file_and_state_only(profile_file) {
                    Ok(profile) => {
                        if profile.id == uuid {
                            result = Ok(profile);
                            break 'PROFILE_LOOP;
                        }
                    }

                    Err(e) => {
                        error!(
                            "Could not process profile {}: {}",
                            profile_file.display(),
                            e
                        );
                    }
                }
            }
        }

        result
    }

    pub fn save(&self) -> Result<()> {
        let toml = toml::ser::to_string_pretty(&self)?;

        fs::write(&self.profile_file, toml).map_err(|_| ProfileError::WriteError {
            msg: "Could not write file".into(),
        })?;

        Ok(())
    }

    pub fn load_params(&mut self) -> Result<()> {
        let path = self.profile_file.with_extension("profile.state");
        let file = fs::File::open(path)?;

        let map: BTreeMap<String, ProfileScriptParameters> =
            serde_json::from_reader(io::BufReader::new(file))?;

        self.config = ProfileConfiguration::from(map);

        Ok(())
    }

    pub fn save_params(&self) -> Result<()> {
        if !self.config.is_empty() {
            let state_path = self.profile_file.with_extension("profile.state");
            let profile_metadata = fs::metadata(&self.profile_file);

            let mut open_options = fs::OpenOptions::new();
            open_options.create(true).write(true);
            if let Ok(profile_metadata) = &profile_metadata {
                open_options.mode(profile_metadata.mode()); // (only takes effect if the file is new)
            }

            let file = open_options.open(&state_path)?;

            serde_json::to_writer_pretty(file, &self.config)?;

            // Try to give the state file the same permissions and ownership as the profile file.
            // This can be useful if the profile file is sitting under the user's home directory.
            // Otherwise, the state file is owned by the eruption user, which is root by default.
            if let Ok(profile_metadata) = &profile_metadata {
                let try_mode = fs::set_permissions(&state_path, profile_metadata.permissions());
                if try_mode.is_err() {
                    debug!("Could not set permissions on {}", &state_path.display());
                }

                let try_chown = nix::unistd::chown(
                    &state_path,
                    Some(profile_metadata.uid().into()),
                    Some(profile_metadata.gid().into()),
                );
                if try_chown.is_err() {
                    debug!("Could not change ownership of {}", &state_path.display());
                }
            }
        }

        Ok(())
    }

    fn get_parameter_default(&self, script_name: &str, parameter_name: &str) -> Option<TypedValue> {
        self.config
            .get_parameter(script_name, parameter_name)?
            .get_default()
    }

    get_default_value!(int, TypedValue::Int, i64);
    get_config_value!(int, TypedValue::Int, i64);
    set_config_value!(int, TypedValue::Int, i64);

    get_default_value!(float, TypedValue::Float, f64);
    get_config_value!(float, TypedValue::Float, f64);
    set_config_value!(float, TypedValue::Float, f64);

    get_default_value!(bool, TypedValue::Bool, bool);
    get_config_value!(bool, TypedValue::Bool, bool);
    set_config_value!(bool, TypedValue::Bool, bool);

    get_default_value!(string, TypedValue::String, String);
    get_config_value!(string, TypedValue::String, str);
    set_config_value!(string, TypedValue::String, str);

    get_default_value!(color, TypedValue::Color, u32);
    get_config_value!(color, TypedValue::Color, u32);
    set_config_value!(color, TypedValue::Color, u32);
}

impl Default for Profile {
    fn default() -> Self {
        let profile_file =
            Path::new(constants::DEFAULT_PROFILE_DIR).join(Path::new("default.profile"));

        Self {
            id: default_id(),
            profile_file,
            name: "Default".into(),
            description: "Auto-generated profile".into(),
            active_scripts: vec![PathBuf::from(constants::DEFAULT_EFFECT_SCRIPT)],
            config: ProfileConfiguration::new(),
            manifests: IndexMap::new(),
        }
    }
}

pub fn get_profile_dirs() -> Vec<PathBuf> {
    let mut result = vec![];

    let config = crate::CONFIG.lock();

    let profile_dirs = config
        .as_ref()
        .unwrap()
        .get::<Vec<String>>("global.profile_dirs")
        .unwrap_or_else(|_| vec![]);

    let mut profile_dirs = profile_dirs
        .iter()
        .map(PathBuf::from)
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

pub fn get_profiles() -> Result<Vec<Profile>> {
    get_profiles_from(&get_profile_dirs())
}

pub fn get_profiles_from(profile_dirs: &[PathBuf]) -> Result<Vec<Profile>> {
    let mut result: Vec<Profile> = vec![];
    let mut errors_present = false;

    let profile_files = get_profile_files_from(profile_dirs).unwrap_or_else(|e| {
        log::warn!("Could not enumerate profiles: {}", &e);
        vec![]
    });

    for profile_file in profile_files.iter() {
        match Profile::load_file_and_state_only(profile_file) {
            Ok(profile) => {
                result.push(profile);
            }

            Err(e) => {
                errors_present = true;
                error!(
                    "Could not process profile {}: {}",
                    profile_file.display(),
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

pub fn get_profile_files() -> Result<Vec<PathBuf>> {
    get_profile_files_from(&get_profile_dirs())
}

pub fn get_profile_files_from(profile_dirs: &[PathBuf]) -> Result<Vec<PathBuf>> {
    let mut result = vec![];

    for profile_path in profile_dirs {
        if let Ok(paths) = fs::read_dir(profile_path) {
            let mut profile_paths = paths
                .map(|p| p.unwrap().path())
                .filter(|p| {
                    if p.extension().is_some() {
                        return p.extension().unwrap_or_else(|| OsStr::new("")) == "profile";
                    }

                    false
                })
                .collect::<Vec<PathBuf>>();

            result.append(&mut profile_paths);
        }
    }

    Ok(result)
}

pub fn find_path_by_uuid(uuid: Uuid) -> Option<PathBuf> {
    find_path_by_uuid_from(uuid, &get_profile_dirs())
}

pub fn find_path_by_uuid_from(uuid: Uuid, profile_dirs: &Vec<PathBuf>) -> Option<PathBuf> {
    let profile_files = get_profile_files_from(profile_dirs).unwrap_or_else(|_| vec![]);

    let mut errors_present = false;
    let mut result = None;

    'PROFILE_LOOP: for profile_file in profile_files.iter() {
        match Profile::load_file_and_state_only(profile_file) {
            Ok(profile) => {
                if profile.id == uuid {
                    result = Some(profile_file.to_path_buf());
                    break 'PROFILE_LOOP;
                }
            }

            Err(e) => {
                errors_present = true;
                error!(
                    "Could not process profile {}: {}",
                    profile_file.display(),
                    e
                );
            }
        }
    }

    if errors_present {
        warn!("An error occurred during processing of profiles");
    }

    result
}

pub fn get_fail_safe_profile() -> Profile {
    Profile::new_fail_safe()
}

#[cfg(test)]
mod tests {
    use std::{path::PathBuf, str::FromStr};

    use indexmap::IndexMap;
    use uuid::Uuid;

    use crate::scripting::parameters::{ManifestValue, ProfileParameter, TypedValue};

    use super::Profile;

    #[test]
    fn enum_profile_files() -> super::Result<()> {
        let path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let files = super::get_profile_files_from(&[path.join("../support/tests/assets/")])?;

        assert!(
            files.contains(&path.join("../support/tests/assets/default.profile")),
            "Missing default.profile: {:#?}",
            files
        );

        Ok(())
    }

    #[test]
    fn enum_profiles() -> super::Result<()> {
        let path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let profiles = super::get_profiles_from(&[path.join("../support/tests/assets/")])?;

        assert!(
            profiles
                .iter()
                .map(|p| p.name.as_ref())
                .collect::<Vec<&str>>()
                .contains(&"Organic FX"),
            "Missing profile 'Organic FX' in profiles: {:#?}",
            profiles
        );

        Ok(())
    }

    #[test]
    fn find_profile_path_by_uuid() -> super::Result<()> {
        let uuid = Uuid::from_str("5dc62fa6-e965-45cb-a0da-e87d29713093").unwrap();

        let path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let profile_path =
            super::find_path_by_uuid_from(uuid, &vec![path.join("../support/tests/assets/")])
                .unwrap();

        assert_eq!(
            profile_path,
            path.join("../support/tests/assets/default.profile"),
            "Invalid path {:#?}",
            profile_path
        );

        Ok(())
    }

    #[test]
    fn load_profile_by_path() -> super::Result<()> {
        let uuid = Uuid::from_str("5dc62fa6-e965-45cb-a0da-e87d29713093").unwrap();

        let path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let profile_path =
            super::find_path_by_uuid_from(uuid, &vec![path.join("../support/tests/assets/")])
                .unwrap();

        let profile = super::Profile::load_file_only(&profile_path)?;

        assert_eq!(profile.id, uuid);
        assert_eq!(profile.name, "Organic FX");

        Ok(())
    }

    #[test]
    fn load_profile_with_no_parameters() -> super::Result<()> {
        let path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let profile_path = path.join("../support/tests/assets/test2.profile");

        let profile = super::Profile::load_file_only(&profile_path)?;

        assert_eq!(profile.name, "Test 2");
        assert!(profile.config.is_empty());

        Ok(())
    }

    #[test]
    fn load_profile_with_parameters() -> super::Result<()> {
        let path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let profile_path = path.join("../support/tests/assets/test3.profile");

        let profile = super::Profile::load_file_only(&profile_path)?;

        assert_eq!(profile.name, "Test 3");
        assert!(!profile.config.is_empty());

        let opacity_param = profile.config.get_parameter("Raindrops", "opacity");
        assert!(opacity_param.is_some());

        let opacity_param = opacity_param.unwrap();
        assert_eq!(opacity_param.name, "opacity");
        assert_eq!(opacity_param.value, TypedValue::Float(0.75));

        Ok(())
    }

    #[test]
    fn load_profile_with_state() -> super::Result<()> {
        let path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let profile_path = path.join("../support/tests/assets/with_state.profile");

        let profile = super::Profile::load_file_and_state_only(&profile_path)?;

        assert_eq!(profile.name, "With State");
        assert!(!profile.config.is_empty());

        let color_background = profile
            .config
            .get_parameter("Solid Color", "color_background");
        assert!(color_background.is_some());

        let color_background = color_background.unwrap();
        assert_eq!(color_background.value, TypedValue::Color(0xff654321_u32));

        Ok(())
    }

    #[test]
    fn load_profile_with_manifest() -> super::Result<()> {
        let assets_path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap())
            .join("../support/tests/assets");

        let profile_path = assets_path.join("manifest_test.profile").canonicalize()?;

        let config = config::Config::builder()
            .set_override(
                "global.script_dirs",
                vec![assets_path.to_string_lossy().to_string()],
            )?
            .build()
            .unwrap();

        *crate::CONFIG.lock() = Some(config);

        let profile = super::Profile::load_fully(&profile_path);
        let profile = match profile {
            Ok(profile) => profile,
            Err(err) => {
                assert!(
                    false,
                    "Profile {} was not loaded: {}",
                    profile_path.display(),
                    err
                );
                return Err(err);
            }
        };

        assert_eq!(profile.name, "Manifest Test");
        assert!(
            !profile.config.is_empty(),
            "Profile config should not be empty"
        );

        let some_integer = profile
            .config
            .get_parameter("Manifest Test", "some_integer");
        assert!(some_integer.is_some(), "some_integer should not be None");

        let some_integer = some_integer.unwrap();
        assert_eq!(
            some_integer.value,
            TypedValue::Int(9876),
            "some_integer.value should be 9876 instead of {:?}",
            some_integer.value
        );

        let default = some_integer.get_default();
        assert!(default.is_some(), "default should not be None");

        let default = default.unwrap();
        assert_eq!(default, TypedValue::Int(7));

        let manifest_value = some_integer.manifest.as_ref();
        assert!(
            manifest_value.is_some(),
            "manifest_value should not be None"
        );

        let manifest_value = manifest_value.unwrap();
        match manifest_value {
            ManifestValue::Int {
                default: 7,
                min: Some(-1),
                max: Some(9999),
            } => {}
            _ => assert!(false, "Wrong manifest_value: {:?}", manifest_value),
        }

        Ok(())
    }

    #[test]
    fn test_profile_parameters() -> super::Result<()> {
        let uuid = Uuid::from_str("5dc62fa6-e965-45cb-a0da-e87d29713093").unwrap();

        let path = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
        let profile_path =
            super::find_path_by_uuid_from(uuid, &vec![path.join("../support/tests/assets/")])
                .unwrap();

        let profile = super::Profile::load_file_only(&profile_path)?;

        assert_eq!(profile.id, uuid);
        assert_eq!(profile.name, "Organic FX");

        let config = profile.config;

        let param = config
            .get_parameter("Shockwave", "color_step_shockwave")
            .unwrap();

        assert_eq!(
            *param,
            ProfileParameter {
                name: String::from("color_step_shockwave"),
                value: TypedValue::Color(0x05010000),
                manifest: None
            }
        );

        let param = config.get_parameter("Shockwave", "mouse_events").unwrap();

        assert_eq!(
            *param,
            ProfileParameter {
                name: String::from("mouse_events"),
                value: TypedValue::Bool(true),
                manifest: None
            }
        );

        Ok(())
    }

    #[test]
    pub fn verify_deserialization_and_serialization() -> super::Result<()> {
        let lit_profile = Profile {
            id: Uuid::from_str("9030f2e0-489d-11ed-b7bd-a306df98fead")?,
            profile_file: PathBuf::from("test.profile"),
            name: "Test profile".to_string(),
            description: "Testing serialization".to_string(),
            active_scripts: vec![
                PathBuf::from("xyz"),
                PathBuf::from("def"),
                PathBuf::from("abc"),
                PathBuf::from("123"),
                PathBuf::from("ghi"),
                PathBuf::from("789"),
                PathBuf::from("jkl"),
                PathBuf::from("456"),
                PathBuf::from("mno"),
                PathBuf::from("pqr"),
            ],
            config: [
                (
                    "123".to_string(),
                    [ProfileParameter {
                        name: "123_param".to_string(),
                        value: TypedValue::Int(1),
                        manifest: None,
                    }]
                    .into(),
                ),
                (
                    "Xyz".to_string(),
                    [ProfileParameter {
                        name: "xyz_param".to_string(),
                        value: TypedValue::Bool(true),
                        manifest: None,
                    }]
                    .into(),
                ),
                (
                    "Abc".to_string(),
                    [ProfileParameter {
                        name: "abc_param".to_string(),
                        value: TypedValue::Int(3),
                        manifest: None,
                    }]
                    .into(),
                ),
            ]
            .into(),
            manifests: IndexMap::new(),
        };

        let lit_toml = r#"
id = '9030f2e0-489d-11ed-b7bd-a306df98fead'
name = 'Test profile'
description = 'Testing serialization'
active_scripts = [
    'xyz',
    'def',
    'abc',
    '123',
    'ghi',
    '789',
    'jkl',
    '456',
    'mno',
    'pqr',
]
[[config.123]]
name = '123_param'
type = 'int'
value = 1

[[config.Abc]]
name = 'abc_param'
type = 'int'
value = 3

[[config.Xyz]]
name = 'xyz_param'
type = 'bool'
value = true
        "#;

        let mut de_profile = toml::de::from_str::<Profile>(lit_toml)?;
        de_profile.profile_file = PathBuf::from("test.profile");

        assert_eq!(lit_profile, de_profile);

        let ser_toml = toml::ser::to_string_pretty(&lit_profile)?;

        assert_eq!(lit_toml.trim(), ser_toml.trim());

        Ok(())
    }
}
