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

pub mod animal;
pub mod audio;
pub mod introspection;
pub mod keyboard;
pub mod macros;
pub mod mouse;
pub mod persistence;
pub mod plugin;
pub mod profiles;
pub mod sdk_support;
pub mod sensors;
pub mod system;
pub mod uleds;

pub use animal::AnimalPlugin;
pub use audio::AudioPlugin;
pub use introspection::IntrospectionPlugin;
pub use keyboard::KeyboardPlugin;
pub use macros::MacrosPlugin;
pub use mouse::MousePlugin;
pub use persistence::PersistencePlugin;
pub use plugin::Plugin;
pub use profiles::ProfilesPlugin;
pub use sdk_support::SdkSupportPlugin;
pub use sensors::SensorsPlugin;
pub use system::SystemPlugin;
pub use uleds::UledsPlugin;

use log::*;

use super::plugin_manager;

pub type Result<T> = std::result::Result<T, eyre::Error>;

// #[derive(Debug, Fail)]
// pub enum PluginError {
//     // #[error("Could not register Lua extensions")]
//     // LuaExtensionError {},

//     #[error("Unknown error: {}", description)]
//     UnknownError { description: String },
// }

/// Register all available plugins
pub fn register_plugins() -> Result<()> {
    trace!("Registering all available plugins...");

    let mut plugin_manager = plugin_manager::PLUGIN_MANAGER.write();

    // Base plugins
    let _ = plugin_manager
        .register_plugin(Box::new(KeyboardPlugin::new()))
        .map_err(|_e| error!("An error occurred during initialization of the plugin"));

    let _ = plugin_manager
        .register_plugin(Box::new(MousePlugin::new()))
        .map_err(|_e| error!("An error occurred during initialization of the plugin"));

    let _ = plugin_manager
        .register_plugin(Box::new(MacrosPlugin::new()))
        .map_err(|_e| error!("An error occurred during initialization of the plugin"));

    let _ = plugin_manager
        .register_plugin(Box::new(IntrospectionPlugin::new()))
        .map_err(|_e| error!("An error occurred during initialization of the plugin"));

    let _ = plugin_manager
        .register_plugin(Box::new(PersistencePlugin::new()))
        .map_err(|_e| error!("An error occurred during initialization of the plugin"));

    let _ = plugin_manager
        .register_plugin(Box::new(ProfilesPlugin::new()))
        .map_err(|_e| error!("An error occurred during initialization of the plugin"));

    let _ = plugin_manager
        .register_plugin(Box::new(SystemPlugin::new()))
        .map_err(|_e| error!("An error occurred during initialization of the plugin"));

    let _ = plugin_manager
        .register_plugin(Box::new(SensorsPlugin::new()))
        .map_err(|_e| error!("An error occurred during initialization of the plugin"));

    let _ = plugin_manager
        .register_plugin(Box::new(UledsPlugin::new()))
        .map_err(|_e| error!("An error occurred during initialization of the plugin"));

    let _ = plugin_manager
        .register_plugin(Box::new(SdkSupportPlugin::new()))
        .map_err(|_e| error!("An error occurred during initialization of the plugin"));

    let _ = plugin_manager
        .register_plugin(Box::new(AudioPlugin::new()))
        .map_err(|_e| error!("An error occurred during initialization of the plugin"));

    // Additional plugins
    let _ = plugin_manager
        .register_plugin(Box::new(AnimalPlugin::new()))
        .map_err(|_e| error!("An error occurred during initialization of the plugin"));

    trace!("Done registering all available plugins");

    Ok(())
}
