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

use mlua::prelude::*;
use std::any::Any;
use std::sync::atomic::Ordering;

use crate::plugins;
use crate::plugins::Plugin;

//pub type Result<T> = std::result::Result<T, eyre::Error>;

//#[derive(Debug, Fail)]
//pub enum ProfilesPluginError {
////#[error("Unknown error: {}", description)]
////UnknownError { description: String },
//}

/// A plugin that enables Eruption to switch profiles, based on the current system state
pub struct ProfilesPlugin {}

impl ProfilesPlugin {
    pub fn new() -> Self {
        ProfilesPlugin {}
    }

    pub(crate) fn get_current_slot() -> usize {
        crate::ACTIVE_SLOT.load(Ordering::SeqCst)
    }

    pub(crate) fn switch_to_slot(index: usize) {
        // the main loop will switch the active profile when it
        // detects, that ACTIVE_SLOT has been changed
        crate::ACTIVE_SLOT.store(index, Ordering::SeqCst);
    }

    pub(crate) fn get_current_profile() -> Option<String> {
        (*crate::ACTIVE_PROFILE.lock())
            .as_ref()
            .map(|profile| (*profile.profile_file.to_string_lossy()).to_string())
    }

    pub(crate) fn switch_to_profile(profile: String) {
        // the main loop will switch the active profile when it
        // detects, that ACTIVE_PROFILE_NAME has been changed
        *crate::ACTIVE_PROFILE_NAME.lock() = Some(profile);
    }
}

#[async_trait::async_trait]
impl Plugin for ProfilesPlugin {
    fn get_name(&self) -> String {
        "Profiles".to_string()
    }

    fn get_description(&self) -> String {
        "Switch profiles based on system state".to_string()
    }

    fn initialize(&mut self) -> plugins::Result<()> {
        Ok(())
    }

    fn register_lua_funcs(&self, lua_ctx: &Lua) -> mlua::Result<()> {
        let globals = lua_ctx.globals();

        let get_current_slot =
            lua_ctx.create_function(move |_, ()| Ok(ProfilesPlugin::get_current_slot()))?;
        globals.set("get_current_slot", get_current_slot)?;

        let switch_to_slot = lua_ctx.create_function(move |_, index: usize| {
            ProfilesPlugin::switch_to_slot(index);
            Ok(())
        })?;
        globals.set("switch_to_slot", switch_to_slot)?;

        let get_current_profile = lua_ctx.create_function(move |_, ()| {
            Ok(ProfilesPlugin::get_current_profile().unwrap_or_default())
        })?;
        globals.set("get_current_profile", get_current_profile)?;

        let switch_to_profile = lua_ctx.create_function(move |_, profile: String| {
            ProfilesPlugin::switch_to_profile(profile);
            Ok(())
        })?;
        globals.set("switch_to_profile", switch_to_profile)?;

        Ok(())
    }

    async fn main_loop_hook(&self, _ticks: u64) {}

    fn sync_main_loop_hook(&self, _ticks: u64) {}

    fn as_any(&self) -> &dyn Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
