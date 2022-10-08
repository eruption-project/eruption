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

use log::*;
use same_file::is_same_file;
use std::{path::PathBuf, sync::atomic::Ordering};

use crate::{
    profiles::{self, FindConfig},
    script::{self},
    scripting::manifest::{self, ParseConfig},
    scripting::parameters::{PlainParameter, UntypedParameter},
};

pub type Result<T> = std::result::Result<T, eyre::Error>;

pub fn apply_parameters(
    profile_file: &str,
    script_file: &str,
    parameter_values: &[UntypedParameter],
) -> Result<()> {
    let profile_path = PathBuf::from(&profile_file);
    let script_path = PathBuf::from(&script_file);

    let manifest = manifest::Manifest::from(&script_path)?;
    let manifest_config = manifest.config.unwrap_or_default();

    // modify persistent profile state
    match profiles::Profile::from(&profile_path) {
        Ok(mut profile) => {
            assert!(profile.config.is_some());

            let profile_config = profile.config.as_mut().unwrap();
            let profile_config = profile_config
                .entry(manifest.name)
                .or_insert_with(std::vec::Vec::new);

            for parameter_value in parameter_values {
                let manifest_param = manifest_config
                    .parse_config_param(&parameter_value.name, &parameter_value.value)?;

                if let Some(param) = profile_config
                    .clone()
                    .find_config_param(&parameter_value.name)
                {
                    // param already exists, remove the existing one first
                    profile_config.retain(|elem| elem != param);
                }

                profile_config.push(manifest_param);
            }
            profile.save_params()?;
        }

        Err(e) => {
            error!("Could not update profile state: {}", e);
        }
    }

    let parameter_values: Vec<PlainParameter> = parameter_values
        .iter()
        .map(|pv| manifest_config.parse_config_param(&pv.name, &pv.value))
        .filter_map(|pv| match pv {
            Ok(pv) => Some(pv),
            Err(e) => {
                error!("Bad parameter: {}", e);
                None
            }
        })
        .map(|ppv| PlainParameter {
            name: ppv.name.to_owned(),
            value: ppv.value,
        })
        .collect();

    let mut need_to_reload_profile = true;
    {
        if let Some(active_profile) = &*crate::ACTIVE_PROFILE.lock() {
            if is_same_file(&active_profile.profile_file, &profile_path).unwrap_or(false) {
                let lua_txs = crate::LUA_TXS.read();
                let lua_tx = lua_txs
                    .iter()
                    .find(|&lua_tx| lua_tx.script_file == script_path);

                if let Some(lua_tx) = lua_tx {
                    let sent = lua_tx.send(script::Message::SetParameters { parameter_values });
                    match sent {
                        Ok(()) => need_to_reload_profile = false,
                        Err(_) => {
                            eprintln!("Could not update parameter from D-Bus request.");
                        }
                    }
                }
            }
        }
    }

    if need_to_reload_profile {
        crate::REQUEST_PROFILE_RELOAD.store(true, Ordering::SeqCst);
    }

    Ok(())
}
