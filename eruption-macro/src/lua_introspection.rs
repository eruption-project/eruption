/*  SPDX-License-Identifier: GPL-3.0-or-later  */

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

#![allow(unused)]

use std::path::{Path, PathBuf};

use crate::constants;

#[allow(unused)]
pub type LuaSyntaxIntrospection = LuaIntrospection<syntax::LuaFile>;

#[allow(unused)]
pub type LuaInterpreterIntrospection = LuaIntrospection<interpreter::LuaFile>;

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug)]
pub struct LuaIntrospection<Handler> {
    pub file_name: PathBuf,
    pub handler: Handler,
}

impl LuaIntrospection<syntax::LuaFile> {
    #[allow(unused)]
    pub fn new_from_file<P: AsRef<Path>>(lua_file: P) -> Result<Self> {
        let path = lua_file.as_ref();

        let handler = syntax::LuaFile::new_from_file(&lua_file)?;

        Ok(Self {
            file_name: path.to_owned(),
            handler,
        })
    }
}

impl LuaIntrospection<interpreter::LuaFile> {
    pub fn new_from_file<P: AsRef<Path>>(lua_file: P) -> Result<Self> {
        let path = lua_file.as_ref();

        let handler = interpreter::LuaFile::new_from_file(&lua_file)?;

        Ok(Self {
            file_name: path.to_owned(),
            handler,
        })
    }
}

impl<Handler> LuaIntrospection<Handler>
where
    Handler: Introspection,
{
    pub fn functions(&self) -> <Handler as Introspection>::IteratorItem {
        self.handler.functions()
    }
}

pub trait Introspection {
    type IteratorItem;

    fn functions(&self) -> Self::IteratorItem;
}

/// Represents a Lua function inside a Lua script file
#[derive(Debug, Clone)]
pub struct LuaFunction {
    pub name: String,
}

impl LuaFunction {
    pub fn name(&self) -> &str {
        &self.name
    }
}

pub mod syntax {
    use std::fs;
    use std::path::Path;
    use std::path::PathBuf;

    use regex::RegexBuilder;

    use super::LuaFunction;
    use super::Result;

    #[derive(Debug)]
    pub struct LuaFile {
        file_name: PathBuf,
        functions: Vec<LuaFunction>,
    }

    impl LuaFile {
        pub fn new_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
            let file_name = path.as_ref();

            let regex = RegexBuilder::new(r"function\s+(?P<fn>.*)\(\w*\)")
                .case_insensitive(true)
                .multi_line(true)
                .build()?;

            let text = fs::read_to_string(file_name)?;
            let mut functions = Vec::new();

            let matches = regex.find_iter(&text);

            for loc in matches {
                let lua_function = LuaFunction {
                    name: loc.as_str().trim().to_owned(),
                };

                functions.push(lua_function);
            }

            Ok(Self {
                file_name: file_name.to_path_buf(),
                functions,
            })
        }
    }

    impl super::Introspection for LuaFile {
        type IteratorItem = LuaFunctionIter;

        fn functions(&self) -> LuaFunctionIter {
            LuaFunctionIter {
                index: 0,
                functions: self.functions.clone(),
            }
        }
    }

    /// An Iterator over all functions in [LuaRegex]
    #[derive(Debug)]
    pub struct LuaFunctionIter {
        index: usize,
        functions: Vec<LuaFunction>,
    }

    impl Iterator for LuaFunctionIter {
        type Item = LuaFunction;

        fn next(&mut self) -> Option<Self::Item> {
            if self.index >= self.functions.len() {
                None
            } else {
                let result = self.functions[self.index].clone();
                self.index += 1;

                Some(result)
            }
        }
    }
}

pub mod interpreter {
    use parking_lot::RwLock;
    use std::{path::Path, sync::Arc};

    use mlua::Lua;

    use super::LuaFunction;
    use super::Result;

    use super::constants;

    /// Represents a Lua file
    #[derive(Debug)]
    pub struct LuaFile {
        lua: Arc<RwLock<Lua>>,
        // file_name: PathBuf,
    }

    impl LuaFile {
        pub fn new_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
            let lua_ctx = Arc::new(RwLock::new(unsafe {
                Lua::unsafe_new_with(mlua::StdLib::ALL, mlua::LuaOptions::default())
            }));

            let path_spec = format!(
                "package.path = package.path .. '{0}/lib/?;{0}/lib/?.lua;{0}/?.lua'",
                &constants::DEFAULT_SCRIPT_DIR
            );
            lua_ctx.write().load(&path_spec).exec()?;

            let file_name = path.as_ref();

            {
                let lua = lua_ctx.write();

                import_stubs(&lua)?;

                if let Some(module) = file_name.to_str().to_owned() {
                    let module = Path::new(&module.trim_end_matches(".lua"))
                        .file_name()
                        .unwrap()
                        .to_string_lossy()
                        .to_string();

                    let code = format!(
                        r#"
                        require("{module}")

                        funcs = {{}}

                        for fname,obj in pairs(mymodule) do
                            if type(obj) == "function" then
                                table.insert(funcs, fname)
                            end
                        end
                    "#
                    );

                    if let Err(e) = lua.load(&code).exec() {
                        eprintln!("{e}\n\nPlease consider reporting a bug!");
                    }
                }
            }

            let result = Self {
                lua: lua_ctx,
                // file_name: file_name.to_owned(),
            };

            Ok(result)
        }
    }

    impl super::Introspection for LuaFile {
        type IteratorItem = LuaFunctionIter;

        fn functions(&self) -> LuaFunctionIter {
            LuaFunctionIter {
                lua: Arc::clone(&self.lua),
                index: 0,
            }
        }
    }

    /// An Iterator over all functions in a [LuaFile]
    #[derive(Debug)]
    pub struct LuaFunctionIter {
        pub(crate) lua: Arc<RwLock<Lua>>,
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

    /// Import stubs and forward declarations of the `Eruption Support Library`
    fn import_stubs(lua: &Lua) -> super::Result<()> {
        let file_name = "/usr/share/eruption/scripts/macros.lua";

        let module = Path::new(&file_name.trim_end_matches(".lua"))
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string();

        let function_stubs = r#"
                function get_target_fps()
                    return 24
                end

                function get_canvas_size()
                    return 16*16
                end

                function get_canvas_width()
                    return 16
                end

                function get_canvas_height()
                    return 16
                end

                function get_num_keys()
                    return 144
                end
            "#;

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
                {function_stubs}

                {forward_decls}

                require("{module}")

                funcs = {{}}

                for fname,obj in pairs(mymodule) do
                    if type(obj) == "function" then
                        table.insert(funcs, fname)
                    end
                end
            "#
        );

        lua.load(&code).exec()?;

        Ok(())
    }
}
