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

use lazy_static::lazy_static;
use log::*;
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

use crate::plugins::{Plugin, Result};

lazy_static! {
    pub static ref PLUGIN_MANAGER: Arc<RwLock<PluginManager>> =
        Arc::new(RwLock::new(PluginManager::new()));
}

type PluginType = dyn Plugin + Sync + Send;

/// Plugin manager
/// Keeps track of registered plugins
pub struct PluginManager {
    registered_plugins: HashMap<String, Box<PluginType>>,
}

#[allow(dead_code)]
impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        PluginManager {
            registered_plugins: HashMap::new(),
        }
    }

    /// Register a plugin with the system
    pub fn register_plugin(&mut self, mut plugin: Box<PluginType>) -> Result<()> {
        info!(
            "Registering plugin: {} - {}",
            plugin.get_name(),
            plugin.get_description()
        );

        plugin.initialize()?;

        self.registered_plugins.insert(plugin.get_name(), plugin);

        Ok(())
    }

    pub fn get_plugins(&self) -> Vec<&PluginType> {
        self.registered_plugins
            .values()
            .map(AsRef::as_ref)
            .collect()
    }

    pub fn get_plugins_mut(&mut self) -> Vec<&mut Box<PluginType>> {
        self.registered_plugins.values_mut().collect()
    }

    pub fn find_plugin_by_name(&self, name: String) -> Option<&PluginType> {
        self.registered_plugins.get(&name).map(AsRef::as_ref)
    }

    pub fn find_plugin_by_name_mut(&mut self, name: String) -> Option<&mut PluginType> {
        self.registered_plugins.get_mut(&name).map(AsMut::as_mut)
    }
}
