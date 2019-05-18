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

mod audio;
mod keyboard;
mod plugin;
mod sensors;
mod system;

pub use audio::*;
pub use keyboard::*;
pub use plugin::*;
pub use sensors::*;
pub use system::*;

use log::*;

use super::plugin_manager;

pub type Result<T> = std::result::Result<T, PluginError>;

pub struct PluginError {}

/// Register all available plugins
pub fn register_plugins() -> Result<()> {
    trace!("Registering all available plugins...");

    let mut plugin_manager = plugin_manager::PLUGIN_MANAGER.write().unwrap();

    plugin_manager.register_plugin(Box::new(KeyboardPlugin::new()));
    plugin_manager.register_plugin(Box::new(SystemPlugin::new()));
    // plugin_manager.register_plugin(Box::new(AudioPlugin::new()));
    // plugin_manager.register_plugin(Box::new(SensorsPlugin::new()));

    Ok(())
}
