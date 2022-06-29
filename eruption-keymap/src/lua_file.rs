/*
    This file is part of Eruption.

    Eruption is free software: you can redistribute it and/or &modify
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

use std::{
    path::{Path, PathBuf},
    sync::Arc,
};

use mlua::Lua;
use parking_lot::RwLock;

use crate::constants;

pub type Result<T> = std::result::Result<T, eyre::Error>;

/// Represents a Lua file
pub struct LuaFile {
    lua: Arc<RwLock<Lua>>,
    file_name: PathBuf,
}

impl LuaFile {
    pub fn new_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let lua_ctx = Arc::new(RwLock::new(unsafe {
            Lua::unsafe_new_with(mlua::StdLib::ALL, mlua::LuaOptions::default())
        }));

        let path_spec = format!(
            "package.path = package.path .. '{0}/lib/?;{0}/lib/?.lua;;{0}/lib/macros/?;{0}/lib/macros/?.lua;{0}/?.lua'",
            &constants::DEFAULT_SCRIPT_DIR
        );
        lua_ctx.write().load(&path_spec).exec().unwrap();

        let file_name = path.as_ref();

        {
            let lua = lua_ctx.write();

            if let Some(module) = file_name.to_str().to_owned() {
                let module = Path::new(&module.trim_end_matches(".lua"))
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .to_string();

                let forward_decls = r#"
                        -- initialize remapping tables
                        REMAPPING_TABLE = {} -- level 1 remapping table (No modifier keys applied)
                        MACRO_TABLE = {} -- level 1 macro table (No modifier keys applied)

                        MOUSE_HID_REMAPPING_TABLE = {} -- level 1 remapping table for mouse events (No modifier keys applied)

                        ACTIVE_EASY_SHIFT_LAYER = 1 -- level 4 supports up to 6 sub-layers

                        EASY_SHIFT_REMAPPING_TABLE = { -- level 4 remapping table (Easy Shift+ layer)
                            {}, {}, {}, {}, {}, {}
                        }

                        EASY_SHIFT_MACRO_TABLE = { -- level 4 macro table (Easy Shift+ layer)
                            {}, {}, {}, {}, {}, {}
                        }

                        EASY_SHIFT_MOUSE_DOWN_MACRO_TABLE =
                            { -- macro tables for mouse button down events (Easy Shift+ layer)
                                {}, {}, {}, {}, {}, {}
                            }

                        EASY_SHIFT_MOUSE_UP_MACRO_TABLE =
                            { -- macro tables for mouse button up events (Easy Shift+ layer)
                                {}, {}, {}, {}, {}, {}
                            }

                        EASY_SHIFT_MOUSE_HID_DOWN_MACRO_TABLE =
                            { -- macro tables for mouse (HID) button down events (Easy Shift+ layer)
                                {}, {}, {}, {}, {}, {}
                            }

                        EASY_SHIFT_MOUSE_HID_UP_MACRO_TABLE =
                            { -- macro tables for mouse (HID) button up events (Easy Shift+ layer)
                                {}, {}, {}, {}, {}, {}
                            }

                        EASY_SHIFT_MOUSE_WHEEL_MACRO_TABLE =
                            { -- macro tables for mouse wheel events (Easy Shift+ layer)
                                {}, {}, {}, {}, {}, {}
                            }

                        EASY_SHIFT_MOUSE_DPI_MACRO_TABLE =
                            { -- macro tables for mouse DPI change events (Easy Shift+ layer)
                                {}, {}, {}, {}, {}, {}
                            }
                "#;

                let code = format!(
                    r#"
                        {forward_decls}

                        local status, mymodule = pcall("require", "{module}")

                        print("" .. mymodule)

                        funcs = {{}}

                        if status then
                            for fname,obj in pairs(mymodule) do
                                if type(obj) == "function" then
                                    table.insert(funcs, fname)
                                end
                            end
                        end
                    "#
                );

                lua.load(&code).exec()?;
            }
        }

        let result = Self {
            lua: lua_ctx,
            file_name: file_name.to_owned(),
        };

        Ok(result)
    }

    pub fn functions(&self) -> LuaFunctionIter {
        let result = LuaFunctionIter {
            lua: Arc::clone(&self.lua),
            index: 0,
        };

        result
    }
}

/// Represents a Lua function inside a [LuaFile]
pub struct LuaFunction {
    pub name: String,
}

impl LuaFunction {
    pub fn name(&self) -> &str {
        &self.name
    }
}

/// An Iterator over all functions in a [LuaFile]
pub struct LuaFunctionIter {
    lua: Arc<RwLock<Lua>>,
    index: usize,
}

impl Iterator for LuaFunctionIter {
    type Item = LuaFunction;

    fn next(&mut self) -> Option<Self::Item> {
        let lua = self.lua.read();
        let globals = lua.globals();

        if let Ok(funcs) = globals.get::<_, Vec<String>>("funcs") {
            let result = funcs.get(self.index)?;

            self.index += 1;

            let result = LuaFunction {
                name: result.to_owned(),
            };

            Some(result)
        } else {
            None
        }
    }
}
