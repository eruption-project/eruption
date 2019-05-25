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

// use log::*;
use rlua;
use rlua::Context;
use std::any::Any;
use std::error;
use std::error::Error;
use std::fmt;

use crate::plugins::{Plugin, Result};

// pub type Result<T> = std::result::Result<T, AudioPluginError>;

#[derive(Debug, Clone)]
pub struct AudioPluginError {
    code: u32,
}

impl error::Error for AudioPluginError {
    fn description(&self) -> &str {
        match self.code {
            _ => "Unknown error",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}

impl fmt::Display for AudioPluginError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

pub struct AudioPlugin {}

/// A plugin that performs audio-related tasks like playing or capturing sounds
impl AudioPlugin {
    pub fn new() -> Self {
        AudioPlugin {}
    }
}

impl Plugin for AudioPlugin {
    fn get_name(&self) -> String {
        "Audio".to_string()
    }

    fn get_description(&self) -> String {
        "Audio related functions".to_string()
    }

    fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    fn register_lua_funcs(&self, lua_ctx: Context) -> rlua::Result<()> {
        let _globals = lua_ctx.globals();

        // let get_package_temp =
        //     lua_ctx.create_function(move |_, ()| Ok(AudioPlugin::get_package_temp()))?;
        // globals.set("get_package_temp", get_package_temp)?;

        Ok(())
    }

    fn main_loop_hook(&self, _ticks: u64) {}

    fn as_any(&self) -> &Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut Any {
        self
    }
}
