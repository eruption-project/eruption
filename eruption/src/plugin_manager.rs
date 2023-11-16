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

use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::*;
use tracing_mutex::stdsync::RwLock;

use crate::plugins::{Plugin, Result};

lazy_static! {
    pub static ref PLUGIN_MANAGER: Arc<RwLock<PluginManager>> =
        Arc::new(RwLock::new(PluginManager::new()));
}

type PluginType = (dyn Plugin + Sync + Send + 'static);

/// Plugin manager
/// Keeps track of registered plugins
pub struct PluginManager {
    registered_plugins: HashMap<String, Arc<Box<PluginType>>>,
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

        plugin.initialize().map_err(|e| {
            error!(
                "Initialization failed for plugin '{}': {}",
                plugin.get_name(),
                e
            );
            e
        })?;

        self.registered_plugins
            .insert(plugin.get_name(), Arc::new(plugin));

        Ok(())
    }

    pub fn get_plugins(&self) -> Vec<Arc<Box<PluginType>>> {
        self.registered_plugins
            .values()
            .cloned()
            .collect::<Vec<_>>()
    }
}
