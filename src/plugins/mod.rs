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

use std::error;
use std::error::Error;
use std::fmt;

pub mod audio;
pub mod keyboard;
pub mod plugin;
pub mod sensors;
pub mod system;

pub use audio::AudioPlugin;
pub use keyboard::KeyboardPlugin;
pub use plugin::Plugin;
pub use sensors::SensorsPlugin;
pub use system::SystemPlugin;

use log::*;

use super::plugin_manager;

pub type Result<T> = std::result::Result<T, PluginError>;

#[derive(Debug, Clone)]
pub struct PluginError {
    code: u32,
}

impl error::Error for PluginError {
    fn description(&self) -> &str {
        match self.code {
            0 => "Could not register Lua extensions",
            _ => "Unknown error",
        }
    }

    fn cause(&self) -> Option<&dyn error::Error> {
        None
    }
}

impl fmt::Display for PluginError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// Register all available plugins
pub fn register_plugins() -> Result<()> {
    trace!("Registering all available plugins...");

    let mut plugin_manager = plugin_manager::PLUGIN_MANAGER.write().unwrap_or_else(|e| {
        error!("Could not lock a shared data structure: {}", e);
        panic!();
    });

    plugin_manager.register_plugin(Box::new(KeyboardPlugin::new()))?;
    plugin_manager.register_plugin(Box::new(SystemPlugin::new()))?;
    plugin_manager.register_plugin(Box::new(SensorsPlugin::new()))?;
    plugin_manager.register_plugin(Box::new(AudioPlugin::new()))?;

    trace!("Done registering all available plugins");

    Ok(())
}
