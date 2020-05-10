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

use failure::{Error, Fail};

use lazy_static::lazy_static;
use log::*;
use parking_lot::RwLock;
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::constants;
use crate::plugins::audio;

pub type Result<T> = std::result::Result<T, StateError>;

#[derive(Debug, Fail)]
pub enum StateError {
    #[fail(display = "Could not load global runtime state: {}", error)]
    StateLoadError { error: Error },

    #[fail(display = "Could not save global runtime state: {}", error)]
    StateWriteError { error: Error },
}

lazy_static! {
    /// Global state
    pub static ref STATE: Arc<RwLock<Option<config::Config>>> = Arc::new(RwLock::new(None));
}

#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
struct State {
    active_slot: usize,
    profiles: Vec<PathBuf>,
    enable_sfx: bool,
    brightness: i64,
}

pub fn init_global_runtime_state() -> Result<()> {
    // initialize runtime state to sane defaults
    let mut profiles = crate::SLOT_PROFILES.lock();
    profiles.replace(vec![
        PathBuf::from("profile1.profile"),
        PathBuf::from("profile2.profile"),
        PathBuf::from("profile3.profile"),
        PathBuf::from("profile4.profile"),
    ]);

    // load state file
    let state_path = PathBuf::from(constants::STATE_DIR).join("eruption.state");

    let state = config::Config::default();
    *STATE.write() = Some(state);

    STATE
        .write()
        .as_mut()
        .unwrap()
        .set_default("active_slot", 0)
        .unwrap();

    STATE
        .write()
        .as_mut()
        .unwrap()
        .set_default("enable_sfx", false)
        .unwrap();

    STATE
        .write()
        .as_mut()
        .unwrap()
        .set_default("brightness", 100)
        .unwrap();

    STATE
        .write()
        .as_mut()
        .unwrap()
        .merge(config::File::new(
            &state_path.to_str().unwrap(),
            config::FileFormat::Toml,
        ))
        .map_err(|e| StateError::StateLoadError { error: e.into() })?;

    audio::ENABLE_SFX.store(
        STATE
            .read()
            .as_ref()
            .unwrap()
            .get_bool("enable_sfx")
            .unwrap(),
        Ordering::SeqCst,
    );

    STATE
        .read()
        .as_ref()
        .unwrap()
        .get("profiles")
        .and_then(|p| {
            profiles.replace(p);
            Ok(())
        })
        .unwrap_or_else(|_| warn!("Invalid saved state: profiles"));

    crate::ACTIVE_SLOT.store(
        STATE
            .read()
            .as_ref()
            .unwrap()
            .get::<usize>("active_slot")
            .unwrap() as usize,
        Ordering::SeqCst,
    );

    crate::BRIGHTNESS.store(
        STATE
            .read()
            .as_ref()
            .unwrap()
            .get::<i64>("brightness")
            .unwrap() as isize,
        Ordering::SeqCst,
    );

    Ok(())
}

pub fn save_runtime_state() -> Result<()> {
    let state_path = PathBuf::from(constants::STATE_DIR).join("eruption.state");

    let config = State {
        active_slot: crate::ACTIVE_SLOT.load(Ordering::SeqCst),
        profiles: crate::SLOT_PROFILES.lock().as_ref().unwrap().clone(),
        enable_sfx: audio::ENABLE_SFX.load(Ordering::SeqCst),
        brightness: crate::BRIGHTNESS.load(Ordering::SeqCst) as i64,
    };

    let toml = toml::ser::to_string_pretty(&config)
        .map_err(|e| StateError::StateWriteError { error: e.into() })?;

    fs::write(&state_path, &toml).map_err(|e| StateError::StateWriteError { error: e.into() })?;

    Ok(())
}
