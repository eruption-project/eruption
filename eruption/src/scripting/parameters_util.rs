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

use same_file;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::atomic::Ordering;
use tracing::*;

use crate::{
    profiles::Profile,
    script,
    scripting::manifest::Manifest,
    scripting::parameters::{
        ManifestParameter, ManifestValue, PlainParameter, ProfileParameter, ToPlainParameter,
        TypedValue, UntypedParameter,
    },
};

#[derive(Debug, thiserror::Error)]
pub enum ParametersUtilError {
    #[error("Could not open file for reading")]
    OpenError {},

    #[error("Could not parse parameter")]
    ParseParameterError {},

    #[error("Script manifest does not reference the parameter")]
    NoSuchParameter {},
}

pub type Result<T> = std::result::Result<T, eyre::Error>;

fn is_same_file(path1: &Path, path2: &Path) -> bool {
    same_file::is_same_file(path1, path2).unwrap_or(false)
}

pub fn apply_parameters(
    profile_file: &str,
    script_file: &str,
    parameter_values: &[UntypedParameter],
) -> Result<()> {
    let profile_path = PathBuf::from(&profile_file);
    let script_path = PathBuf::from(&script_file);

    // If the specified profile_file is for the active profile, update that directly.
    {
        let active_profile = &mut *crate::ACTIVE_PROFILE.write().unwrap();
        if let Some(active_profile) = active_profile.as_mut() {
            if is_same_file(&active_profile.profile_file, &profile_path) {
                let new_parameters =
                    update_profile_and_state_file(active_profile, &script_path, parameter_values)?;

                update_parameters_on_active_profile(&script_path, new_parameters)?;

                return Ok(());
            }
        }
    }

    // Otherwise, load the profile and manifest files and modify that
    let profile = Profile::load_file_and_state_only(&profile_path);
    let mut profile = match profile {
        Ok(profile) => profile,
        Err(e) => {
            error!("Could not open profile file: {}", e);
            return Err(ParametersUtilError::OpenError {}.into());
        }
    };

    let manifest = Manifest::load(&script_path);
    let manifest = match manifest {
        Ok(manifest) => manifest,
        Err(e) => {
            error!("Could not open manifest file: {}", e);
            return Err(ParametersUtilError::OpenError {}.into());
        }
    };

    profile.manifests.insert(manifest.name.to_owned(), manifest);

    update_profile_and_state_file(&mut profile, &script_path, parameter_values)?;
    Ok(())
}

fn update_profile_and_state_file(
    profile: &mut Profile,
    script_path: &Path,
    parameter_values: &[UntypedParameter],
) -> Result<Vec<PlainParameter>> {
    let manifest = profile
        .manifests
        .values()
        .find(|m| is_same_file(&m.script_file, script_path));
    let manifest = match manifest {
        Some(manifest) => manifest,
        None => {
            error!("Manifest is not found in profile");
            return Err(ParametersUtilError::OpenError {}.into());
        }
    };

    let mut new_parameters: Vec<PlainParameter> = vec![];

    // modify persistent profile state
    let profile_script_parameter = profile.config.get_parameters_mut(&manifest.name);

    for untyped_parameter in parameter_values {
        let manifest_param = manifest.config.get_parameter(&untyped_parameter.name);
        if let Some(manifest_param) = manifest_param {
            match parse_new_profile_parameter(manifest_param, &untyped_parameter.value) {
                Ok(profile_parameter) => {
                    new_parameters.push(profile_parameter.to_plain_parameter());
                    profile_script_parameter.set_parameter(profile_parameter);
                }
                Err(e) => {
                    error!(
                        "Could not parse {} value \"{}\": {}",
                        manifest_param.name, untyped_parameter.value, e
                    );
                    return Err(ParametersUtilError::ParseParameterError {}.into());
                }
            }
        } else {
            error!(
                "Unknown configuration parameter \"{}\"",
                untyped_parameter.name
            );
            return Err(ParametersUtilError::NoSuchParameter {}.into());
        }
    }

    profile.save_params()?;

    Ok(new_parameters)
}

fn update_parameters_on_active_profile(
    script_path: &Path,
    parameter_values: Vec<PlainParameter>,
) -> Result<()> {
    let lua_txs = crate::LUA_TXS.read().unwrap();
    let lua_tx = lua_txs
        .iter()
        .find(|&lua_tx| is_same_file(&lua_tx.script_file, script_path));

    if let Some(lua_tx) = lua_tx {
        let sent = lua_tx.send(script::Message::SetParameters { parameter_values });
        if let Err(e) = sent {
            eprintln!("Could not update parameter from D-Bus request. {e}");
            crate::REQUEST_PROFILE_RELOAD.store(true, Ordering::SeqCst);
        }
    }

    Ok(())
}

fn parse_new_profile_parameter(
    manifest_parameter: &ManifestParameter,
    val: &str,
) -> std::result::Result<ProfileParameter, Box<dyn std::error::Error>> {
    let typed_value = match &manifest_parameter.manifest {
        ManifestValue::Int { .. } => TypedValue::Int(i64::from_str(val)?),
        ManifestValue::Float { .. } => TypedValue::Float(f64::from_str(val)?),
        ManifestValue::Bool { .. } => {
            TypedValue::Bool(bool::from_str(&val.to_string().to_lowercase())?)
        }
        ManifestValue::String { .. } => TypedValue::String(val.to_owned()),
        ManifestValue::Color { .. } => {
            if &val[0..1] == "#" {
                TypedValue::Color(u32::from_str_radix(&val[1..], 16)?)
            } else {
                TypedValue::Color(u32::from_str(val)?)
            }
        }
    };

    Ok(ProfileParameter {
        name: manifest_parameter.name.to_owned(),
        value: typed_value,
        manifest: Some(manifest_parameter.manifest.to_owned()),
    })
}
