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

use lazy_static::lazy_static;
use log::*;
use mlua::prelude::*;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::any::Any;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use crate::constants;
use crate::plugins::{self, Plugin};

lazy_static! {
    /// A persistent key/value store that may be used by Lua scripts to store data across script reloads
    /// Will be stored to disk, and will survive a restart of the daemon
    pub static ref GLOBAL_STORE: Arc<RwLock<HashMap<String, StoreValue>>> = Arc::new(RwLock::new(HashMap::new()));

    /// An ephemeral key/value store that may be used by Lua scripts to store data across script reloads
    /// This is suitable only for transient data, since it will not survive a restart of the daemon
    pub static ref GLOBAL_EPHEMERAL_STORE: Arc<RwLock<HashMap<String, StoreValue>>> = Arc::new(RwLock::new(HashMap::new()));
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum StoreValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Color(u32),
    Array(HashMap<i32, String>),
    Hash(HashMap<String, String>),
}

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum PersistencePluginError {
    #[error("Invalid data type error: {description}")]
    TypeError { description: String },

    #[error("Non existent key: {description}")]
    KeyError { description: String },
}

pub struct PersistencePlugin {}

macro_rules! store_operation {
    ($t:ident, $tval:ty, $sval:ty) => {
        paste::item! {
            pub(crate) fn [<store_ $t>](key: String, value: $tval) -> Result<()> {
                GLOBAL_STORE
                    .write()
                    .insert(key, $sval(value));
                Ok(())
            }
        }
    };
}

macro_rules! load_operation {
    ($t:ident, $tval:ty, $sval:ty) => {
        paste::item! {
            pub(crate) fn [<load_ $t>](key: &str) -> Result<$tval> {
                match GLOBAL_STORE.read().get(key) {
                    Some(value) => {
                        if let $sval(val) = value {
                            Ok(val.clone())
                        } else {
                            Err(PersistencePluginError::TypeError {
                                description: key.to_owned(),
                            }.into())
                        }
                    }

                    None => Err(PersistencePluginError::KeyError {
                        description: key.to_owned(),
                    }.into()),
                }
            }
        }
    };
}

macro_rules! store_transient_operation {
    ($t:ident, $tval:ty, $sval:ty) => {
        paste::item! {
            pub(crate) fn [<store_ $t _transient>](key: String, value: $tval) -> Result<()> {
                GLOBAL_EPHEMERAL_STORE
                    .write()
                    .insert(key, $sval(value));
                Ok(())
            }
        }
    };
}

macro_rules! load_transient_operation {
    ($t:ident, $tval:ty, $sval:ty) => {
        paste::item! {
            pub(crate) fn [<load_ $t _transient>](key: &str) -> Result<$tval> {
                match GLOBAL_EPHEMERAL_STORE.read().get(key) {
                    Some(value) => {
                        if let $sval(val) = value {
                            Ok(val.clone())
                        } else {
                            Err(PersistencePluginError::TypeError {
                                description: key.to_owned(),
                            }.into())
                        }
                    }

                    None => Err(PersistencePluginError::KeyError {
                        description: key.to_owned(),
                    }.into()),
                }
            }
        }
    };
}

impl PersistencePlugin {
    pub fn new() -> Self {
        PersistencePlugin {}
    }

    /// Stores the state of the persistence layer to disk
    pub fn store_persistent_data() -> Result<()> {
        info!("Storing persistent state data to disk...");

        let json_string = serde_json::to_string_pretty(&*GLOBAL_STORE.read())?;

        let path = PathBuf::from(constants::STATE_DIR).join(&PathBuf::from("persistent.store"));

        fs::write(&path, json_string)?;

        Ok(())
    }

    /// Loads the state of the persistence layer from disk
    pub fn load_persistent_data() -> Result<()> {
        info!("Loading persistent state data from disk...");

        let path = PathBuf::from(constants::STATE_DIR).join(&PathBuf::from("persistent.store"));

        let json_string = fs::read_to_string(&path)?;

        let map: HashMap<String, StoreValue> = serde_json::from_str(&json_string)?;

        {
            *GLOBAL_STORE.write() = map;
        }

        Ok(())
    }

    // persistent data
    store_operation!(int, i64, StoreValue::Int);
    load_operation!(int, i64, StoreValue::Int);

    store_operation!(float, f64, StoreValue::Float);
    load_operation!(float, f64, StoreValue::Float);

    store_operation!(bool, bool, StoreValue::Bool);
    load_operation!(bool, bool, StoreValue::Bool);

    store_operation!(string, String, StoreValue::String);
    load_operation!(string, String, StoreValue::String);

    store_operation!(color, u32, StoreValue::Color);
    load_operation!(color, u32, StoreValue::Color);

    // experimental
    store_operation!(string_array, HashMap<i32, String>, StoreValue::Array);
    load_operation!(string_array, HashMap<i32, String>, StoreValue::Array);

    store_operation!(string_hash, HashMap<String, String>, StoreValue::Hash);
    load_operation!(string_hash, HashMap<String, String>, StoreValue::Hash);

    // transient data
    store_transient_operation!(int, i64, StoreValue::Int);
    load_transient_operation!(int, i64, StoreValue::Int);

    store_transient_operation!(float, f64, StoreValue::Float);
    load_transient_operation!(float, f64, StoreValue::Float);

    store_transient_operation!(bool, bool, StoreValue::Bool);
    load_transient_operation!(bool, bool, StoreValue::Bool);

    store_transient_operation!(string, String, StoreValue::String);
    load_transient_operation!(string, String, StoreValue::String);

    store_transient_operation!(color, u32, StoreValue::Color);
    load_transient_operation!(color, u32, StoreValue::Color);

    // experimental
    store_transient_operation!(string_array, HashMap<i32, String>, StoreValue::Array);
    load_transient_operation!(string_array, HashMap<i32, String>, StoreValue::Array);

    store_transient_operation!(string_hash, HashMap<String, String>, StoreValue::Hash);
    load_transient_operation!(string_hash, HashMap<String, String>, StoreValue::Hash);
}

#[async_trait::async_trait]
impl Plugin for PersistencePlugin {
    fn get_name(&self) -> String {
        "Persistence".to_string()
    }

    fn get_description(&self) -> String {
        "A storage and persistence layer for Lua Scripts".to_string()
    }

    async fn initialize(&mut self) -> plugins::Result<()> {
        Ok(())
    }

    fn register_lua_funcs(&self, lua_ctx: &Lua) -> mlua::Result<()> {
        let globals = lua_ctx.globals();

        // persistent data
        let store_int = lua_ctx.create_function(|_, (key, value): (String, i64)| {
            PersistencePlugin::store_int(key, value).unwrap();
            Ok(())
        })?;
        globals.set("store_int", store_int)?;

        let load_int = lua_ctx.create_function(|_, (key, default): (String, i64)| {
            match PersistencePlugin::load_int(&key) {
                Ok(result) => Ok(result),
                Err(_e) => Ok(default),
            }
        })?;
        globals.set("load_int", load_int)?;

        let store_float = lua_ctx.create_function(|_, (key, value): (String, f64)| {
            PersistencePlugin::store_float(key, value).unwrap();
            Ok(())
        })?;
        globals.set("store_float", store_float)?;

        let load_float = lua_ctx.create_function(|_, (key, default): (String, f64)| {
            match PersistencePlugin::load_float(&key) {
                Ok(result) => Ok(result),
                Err(_e) => Ok(default),
            }
        })?;
        globals.set("load_float", load_float)?;

        let store_bool = lua_ctx.create_function(|_, (key, value): (String, bool)| {
            PersistencePlugin::store_bool(key, value).unwrap();
            Ok(())
        })?;
        globals.set("store_bool", store_bool)?;

        let load_bool = lua_ctx.create_function(|_, (key, default): (String, bool)| {
            match PersistencePlugin::load_bool(&key) {
                Ok(result) => Ok(result),
                Err(_e) => Ok(default),
            }
        })?;
        globals.set("load_bool", load_bool)?;

        let store_string = lua_ctx.create_function(|_, (key, value): (String, String)| {
            PersistencePlugin::store_string(key, value).unwrap();
            Ok(())
        })?;
        globals.set("store_string", store_string)?;

        let load_string = lua_ctx.create_function(|_, (key, default): (String, String)| {
            match PersistencePlugin::load_string(&key) {
                Ok(result) => Ok(result),
                Err(_e) => Ok(default),
            }
        })?;
        globals.set("load_string", load_string)?;

        let store_color = lua_ctx.create_function(|_, (key, value): (String, u32)| {
            PersistencePlugin::store_color(key, value).unwrap();
            Ok(())
        })?;
        globals.set("store_color", store_color)?;

        let load_color = lua_ctx.create_function(|_, (key, default): (String, u32)| {
            match PersistencePlugin::load_color(&key) {
                Ok(result) => Ok(result),
                Err(_e) => Ok(default),
            }
        })?;
        globals.set("load_color", load_color)?;

        let store_string_array =
            lua_ctx.create_function(|_, (key, value): (String, HashMap<i32, String>)| {
                PersistencePlugin::store_string_array(key, value).unwrap();
                Ok(())
            })?;
        globals.set("store_string_array", store_string_array)?;

        let load_string_array =
            lua_ctx.create_function(|_, (key, default): (String, HashMap<i32, String>)| {
                match PersistencePlugin::load_string_array(&key) {
                    Ok(result) => Ok(result),
                    Err(_e) => Ok(default),
                }
            })?;
        globals.set("load_string_array", load_string_array)?;

        let store_string_hash =
            lua_ctx.create_function(|_, (key, value): (String, HashMap<String, String>)| {
                PersistencePlugin::store_string_hash(key, value).unwrap();
                Ok(())
            })?;
        globals.set("store_string_hash", store_string_hash)?;

        let load_string_hash =
            lua_ctx.create_function(|_, (key, default): (String, HashMap<String, String>)| {
                match PersistencePlugin::load_string_hash(&key) {
                    Ok(result) => Ok(result),
                    Err(_e) => Ok(default),
                }
            })?;
        globals.set("load_string_hash", load_string_hash)?;

        // transient data
        let store_int_transient = lua_ctx.create_function(|_, (key, value): (String, i64)| {
            PersistencePlugin::store_int_transient(key, value).unwrap();
            Ok(())
        })?;
        globals.set("store_int_transient", store_int_transient)?;

        let load_int_transient = lua_ctx.create_function(|_, (key, default): (String, i64)| {
            match PersistencePlugin::load_int_transient(&key) {
                Ok(result) => Ok(result),
                Err(_e) => Ok(default),
            }
        })?;
        globals.set("load_int_transient", load_int_transient)?;

        let store_float_transient = lua_ctx.create_function(|_, (key, value): (String, f64)| {
            PersistencePlugin::store_float_transient(key, value).unwrap();
            Ok(())
        })?;
        globals.set("store_float_transient", store_float_transient)?;

        let load_float_transient =
            lua_ctx.create_function(|_, (key, default): (String, f64)| {
                match PersistencePlugin::load_float_transient(&key) {
                    Ok(result) => Ok(result),
                    Err(_e) => Ok(default),
                }
            })?;
        globals.set("load_float_transient", load_float_transient)?;

        let store_bool_transient = lua_ctx.create_function(|_, (key, value): (String, bool)| {
            PersistencePlugin::store_bool_transient(key, value).unwrap();
            Ok(())
        })?;
        globals.set("store_bool_transient", store_bool_transient)?;

        let load_bool_transient =
            lua_ctx.create_function(|_, (key, default): (String, bool)| {
                match PersistencePlugin::load_bool_transient(&key) {
                    Ok(result) => Ok(result),
                    Err(_e) => Ok(default),
                }
            })?;
        globals.set("load_bool_transient", load_bool_transient)?;

        let store_string_transient =
            lua_ctx.create_function(|_, (key, value): (String, String)| {
                PersistencePlugin::store_string_transient(key, value).unwrap();
                Ok(())
            })?;
        globals.set("store_string_transient", store_string_transient)?;

        let load_string_transient =
            lua_ctx.create_function(|_, (key, default): (String, String)| {
                match PersistencePlugin::load_string_transient(&key) {
                    Ok(result) => Ok(result),
                    Err(_e) => Ok(default),
                }
            })?;
        globals.set("load_string_transient", load_string_transient)?;

        let store_color_transient = lua_ctx.create_function(|_, (key, value): (String, u32)| {
            PersistencePlugin::store_color_transient(key, value).unwrap();
            Ok(())
        })?;
        globals.set("store_color_transient", store_color_transient)?;

        let load_color_transient =
            lua_ctx.create_function(|_, (key, default): (String, u32)| {
                match PersistencePlugin::load_color_transient(&key) {
                    Ok(result) => Ok(result),
                    Err(_e) => Ok(default),
                }
            })?;
        globals.set("load_color_transient", load_color_transient)?;

        let store_string_array_transient =
            lua_ctx.create_function(|_, (key, value): (String, HashMap<i32, String>)| {
                PersistencePlugin::store_string_array_transient(key, value).unwrap();
                Ok(())
            })?;
        globals.set("store_string_array_transient", store_string_array_transient)?;

        let load_string_array_transient =
            lua_ctx.create_function(|_, (key, default): (String, HashMap<i32, String>)| {
                match PersistencePlugin::load_string_array_transient(&key) {
                    Ok(result) => Ok(result),
                    Err(_e) => Ok(default),
                }
            })?;
        globals.set("load_string_array_transient", load_string_array_transient)?;

        let store_string_array_transient =
            lua_ctx.create_function(|_, (key, value): (String, HashMap<i32, String>)| {
                PersistencePlugin::store_string_array_transient(key, value).unwrap();
                Ok(())
            })?;
        globals.set("store_string_array_transient", store_string_array_transient)?;

        let load_string_array_transient =
            lua_ctx.create_function(|_, (key, default): (String, HashMap<i32, String>)| {
                match PersistencePlugin::load_string_array_transient(&key) {
                    Ok(result) => Ok(result),
                    Err(_e) => Ok(default),
                }
            })?;
        globals.set("load_string_array_transient", load_string_array_transient)?;

        let store_string_hash_transient =
            lua_ctx.create_function(|_, (key, value): (String, HashMap<String, String>)| {
                PersistencePlugin::store_string_hash_transient(key, value).unwrap();
                Ok(())
            })?;
        globals.set("store_string_hash_transient", store_string_hash_transient)?;

        let load_string_hash_transient =
            lua_ctx.create_function(|_, (key, default): (String, HashMap<String, String>)| {
                match PersistencePlugin::load_string_hash_transient(&key) {
                    Ok(result) => Ok(result),
                    Err(_e) => Ok(default),
                }
            })?;
        globals.set("load_string_hash_transient", load_string_hash_transient)?;

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
