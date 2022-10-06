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

use std::fmt::Write;
use std::{fs, path::Path};

use chrono::Utc;
use colored::Colorize;

use crate::mapping::{Action, Event, KeyMappingTable};
use crate::{messages, tr};

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug)]
pub struct LuaBackend {}

impl LuaBackend {
    pub fn new() -> Self {
        Self {}
    }
}

impl super::Backend for LuaBackend {
    fn generate(&self, table: &KeyMappingTable) -> Result<String> {
        let mut text = String::new();

        write!(
            &mut text,
            r"
-- This file is part of Eruption.
--
-- Eruption is free software: you can redistribute it and/or modify
-- it under the terms of the GNU General Public License as published by
-- the Free Software Foundation, either version 3 of the License, or
-- (at your option) any later version.
--
-- Eruption is distributed in the hope that it will be useful,
-- but WITHOUT ANY WARRANTY without even the implied warranty of
-- MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
-- GNU General Public License for more details.
--
-- You should have received a copy of the GNU General Public License
-- along with Eruption.  If not, see <http://www.gnu.org/licenses/>.
--
-- Copyright (c) 2019-2022, The Eruption Development Team
--
-- AUTO GENERATED LUA SOURCE CODE FILE, DO NOT EDIT MANUALLY
--
-- Created by: `eruption-keymap compile --keymap {}`
-- Compiled at: {}

",
            table.file_name().display(),
            Utc::now()
        )?;

        for (index, (source, rule)) in table.mappings().iter().enumerate() {
            writeln!(&mut text, "-- {:?} -> {:?}", source, rule)?;

            match &rule.action {
                Action::Null => {
                    messages::info!(
                        "Rule: {}: {}",
                        &format!("#{:0>2}", index + 1),
                        tr!("action-disabled-null").white().bold()
                    );

                    writeln!(&mut text, "-- ACTION IS DISABLED")?;
                }

                //REMAPPING_TABLE = {} -- level 1 remapping table (No modifier keys applied)
                //MACRO_TABLE = {} -- level 1 macro table (No modifier keys applied)

                //MOUSE_HID_REMAPPING_TABLE = {} -- level 1 remapping table for mouse events (No modifier keys applied)

                //ACTIVE_EASY_SHIFT_LAYER = 1 -- level 4 supports up to 6 sub-layers
                //EASY_SHIFT_REMAPPING_TABLE = { -- level 4 remapping table (Easy Shift+ layer)
                //{}, {}, {}, {}, {}, {}
                //}
                //EASY_SHIFT_MACRO_TABLE = { -- level 4 macro table (Easy Shift+ layer)
                //{}, {}, {}, {}, {}, {}
                //}

                //EASY_SHIFT_MOUSE_DOWN_MACRO_TABLE =
                //{ -- macro tables for mouse button down events (Easy Shift+ layer)
                //{}, {}, {}, {}, {}, {}
                //}
                //EASY_SHIFT_MOUSE_UP_MACRO_TABLE =
                //{ -- macro tables for mouse button up events (Easy Shift+ layer)
                //{}, {}, {}, {}, {}, {}
                //}
                //EASY_SHIFT_MOUSE_HID_DOWN_MACRO_TABLE =
                //{ -- macro tables for mouse (HID) button down events (Easy Shift+ layer)
                //{}, {}, {}, {}, {}, {}
                //}
                //EASY_SHIFT_MOUSE_HID_UP_MACRO_TABLE =
                //{ -- macro tables for mouse (HID) button up events (Easy Shift+ layer)
                //{}, {}, {}, {}, {}, {}
                //}
                //EASY_SHIFT_MOUSE_WHEEL_MACRO_TABLE =
                //{ -- macro tables for mouse wheel events (Easy Shift+ layer)
                //{}, {}, {}, {}, {}, {}
                //}
                //EASY_SHIFT_MOUSE_DPI_MACRO_TABLE =
                //{ -- macro tables for mouse DPI change events (Easy Shift+ layer)
                //{}, {}, {}, {}, {}, {}
                //}
                //
                Action::InjectKey(dest) => match &source.event {
                    Event::Null => {
                        messages::info!(
                            "Rule: {}: {}",
                            &format!("#{:0>2}", index + 1),
                            tr!("action-disabled-null").white().bold()
                        );

                        writeln!(&mut text, "-- ACTION IS DISABLED")?;
                    }

                    Event::HidKeyDown(_key) => {
                        messages::warning!(
                            "Rule: {}: {}",
                            &format!("#{:0>2}", index + 1),
                            tr!("action-not-implemented").yellow()
                        );

                        writeln!(&mut text, "-- ACTION IS NOT IMPLEMENTED")?;
                    }

                    Event::HidKeyUp(_key) => {
                        messages::warning!(
                            "Rule: {}: {}",
                            &format!("#{:0>2}", index + 1),
                            tr!("action-not-implemented").yellow()
                        );

                        writeln!(&mut text, "-- ACTION IS NOT IMPLEMENTED")?;
                    }

                    Event::HidMouseDown(key) => {
                        if !rule.enabled {
                            write!(&mut text, "-- ACTION IS DISABLED: ")?;
                        }

                        writeln!(
                            &mut text,
                            "MOUSE_HID_REMAPPING_TABLE[{}] = {}",
                            key.key_index, dest.event
                        )?;
                    }

                    Event::HidMouseUp(key) => {
                        if !rule.enabled {
                            write!(&mut text, "-- ACTION IS DISABLED: ")?;
                        }

                        writeln!(
                            &mut text,
                            "MOUSE_HID_REMAPPING_TABLE[{}] = {}",
                            key.key_index, dest.event
                        )?;
                    }

                    Event::EasyShiftKeyDown(key) => {
                        for layer in &source.layers.0 {
                            if !rule.enabled {
                                write!(&mut text, "-- ACTION IS DISABLED: ")?;
                            }

                            writeln!(
                                &mut text,
                                "EASY_SHIFT_REMAPPING_TABLE[{}][{}] = {}",
                                layer, key.key_index, dest.event
                            )?;
                        }
                    }

                    Event::EasyShiftKeyUp(key) => {
                        for layer in &source.layers.0 {
                            if !rule.enabled {
                                write!(&mut text, "-- ACTION IS DISABLED: ")?;
                            }

                            writeln!(
                                &mut text,
                                "EASY_SHIFT_REMAPPING_TABLE[{}][{}] = {}",
                                layer, key.key_index, dest.event
                            )?;
                        }
                    }

                    Event::EasyShiftMouseDown(_key) => {
                        messages::warning!(
                            "Rule: {}: {}",
                            &format!("#{:0>2}", index + 1),
                            tr!("action-not-implemented").yellow()
                        );

                        writeln!(&mut text, "-- ACTION IS NOT IMPLEMENTED")?;
                    }

                    Event::EasyShiftMouseUp(_key) => {
                        messages::warning!(
                            "Rule: {}: {}",
                            &format!("#{:0>2}", index + 1),
                            tr!("action-not-implemented").yellow()
                        );

                        writeln!(&mut text, "-- ACTION IS NOT IMPLEMENTED")?;
                    }

                    Event::EasyShiftMouseWheel(_direction) => {
                        messages::warning!(
                            "Rule: {}: {}",
                            &format!("#{:0>2}", index + 1),
                            tr!("action-not-implemented").yellow()
                        );

                        writeln!(&mut text, "-- ACTION IS NOT IMPLEMENTED")?;
                    }

                    Event::EasyShiftMouseDpi(_direction) => {
                        messages::warning!(
                            "Rule: {}: {}",
                            &format!("#{:0>2}", index + 1),
                            tr!("action-not-implemented").yellow()
                        );

                        writeln!(&mut text, "-- ACTION IS NOT IMPLEMENTED")?;
                    }

                    Event::SimpleKeyDown(key) => {
                        if !rule.enabled {
                            write!(&mut text, "-- ACTION IS DISABLED: ")?;
                        }

                        writeln!(
                            &mut text,
                            "REMAPPING_TABLE[{}] = {}",
                            key.key_index, dest.event
                        )?;
                    }

                    Event::SimpleKeyUp(key) => {
                        if !rule.enabled {
                            write!(&mut text, "-- ACTION IS DISABLED: ")?;
                        }

                        writeln!(
                            &mut text,
                            "REMAPPING_TABLE[{}] = {}",
                            key.key_index, dest.event
                        )?;
                    }

                    Event::SimpleMouseDown(_key) => {
                        messages::warning!(
                            "Rule: {}: {}",
                            &format!("#{:0>2}", index + 1),
                            tr!("action-not-implemented").yellow()
                        );
                    }

                    Event::SimpleMouseUp(_key) => {
                        messages::warning!(
                            "Rule: {}: {}",
                            &format!("#{:0>2}", index + 1),
                            tr!("action-not-implemented").yellow()
                        );
                    }

                    Event::SimpleMouseWheel(_direction) => {
                        messages::warning!(
                            "Rule: {}: {}",
                            &format!("#{:0>2}", index + 1),
                            tr!("action-not-implemented").yellow()
                        );
                    }

                    Event::SimpleMouseDpi(_direction) => {
                        messages::warning!(
                            "Rule: {}: {}",
                            &format!("#{:0>2}", index + 1),
                            tr!("action-not-implemented").yellow()
                        );
                    }
                },

                Action::Call(call) => {
                    match &source.event {
                        Event::Null => {
                            messages::warning!(
                                "Rule: {}: {}",
                                &format!("#{:0>2}", index + 1),
                                tr!("action-disabled").yellow()
                            );

                            writeln!(&mut text, "-- ACTION IS DISABLED")?;
                        }

                        Event::HidKeyDown(_key) => {
                            messages::warning!(
                                "Rule: {}: {}",
                                &format!("#{:0>2}", index + 1),
                                tr!("action-not-implemented").yellow()
                            );

                            writeln!(&mut text, "-- ACTION IS NOT IMPLEMENTED")?;

                            //writeln!(&mut text, "{} {}", key.key_index, dest.key_index)?;
                        }

                        Event::HidKeyUp(_key) => {
                            messages::warning!(
                                "Rule: {}: {}",
                                &format!("#{:0>2}", index + 1),
                                tr!("action-disabled").yellow()
                            );

                            writeln!(&mut text, "-- ACTION IS DISABLED")?;
                        }

                        Event::HidMouseDown(key) => {
                            if !rule.enabled {
                                write!(&mut text, "-- ACTION IS DISABLED: ")?;
                            }

                            writeln!(
                                &mut text,
                                "MOUSE_HID_REMAPPING_TABLE[{}] = {}",
                                key.key_index, call.function_name
                            )?;
                        }

                        Event::HidMouseUp(_key) => {
                            messages::warning!(
                                "Rule: {}: {}",
                                &format!("#{:0>2}", index + 1),
                                tr!("action-not-implemented").yellow()
                            );

                            writeln!(&mut text, "-- ACTION IS NOT IMPLEMENTED")?;
                        }

                        Event::EasyShiftKeyDown(key) => {
                            for layer in &source.layers.0 {
                                if !rule.enabled {
                                    write!(&mut text, "-- ACTION IS DISABLED: ")?;
                                }

                                writeln!(
                                    &mut text,
                                    "EASY_SHIFT_MACRO_TABLE[{}][{}] = {}",
                                    layer, key.key_index, call.function_name
                                )?;
                            }
                        }

                        Event::EasyShiftKeyUp(key) => {
                            for layer in &source.layers.0 {
                                if !rule.enabled {
                                    write!(&mut text, "-- ACTION IS DISABLED: ")?;
                                }

                                writeln!(
                                    &mut text,
                                    "EASY_SHIFT_MACRO_TABLE[{}][{}] = {}",
                                    layer, key.key_index, call.function_name
                                )?;
                            }
                        }

                        Event::EasyShiftMouseDown(key) => {
                            for layer in &source.layers.0 {
                                if !rule.enabled {
                                    write!(&mut text, "-- ACTION IS DISABLED: ")?;
                                }

                                writeln!(
                                    &mut text,
                                    "EASY_SHIFT_MOUSE_DOWN_MACRO_TABLE[{}][{}] = {}",
                                    layer, key.key_index, call.function_name
                                )?;
                            }
                        }

                        Event::EasyShiftMouseUp(key) => {
                            for layer in &source.layers.0 {
                                if !rule.enabled {
                                    write!(&mut text, "-- ACTION IS DISABLED: ")?;
                                }

                                writeln!(
                                    &mut text,
                                    "EASY_SHIFT_MOUSE_UP_MACRO_TABLE[{}][{}] = {}",
                                    layer, key.key_index, call.function_name
                                )?;
                            }
                        }

                        Event::EasyShiftMouseWheel(direction) => {
                            for layer in &source.layers.0 {
                                if !rule.enabled {
                                    write!(&mut text, "-- ACTION IS DISABLED: ")?;
                                }

                                writeln!(
                                    &mut text,
                                    "EASY_SHIFT_MOUSE_WHEEL_MACRO_TABLE[{}][{}] = {}",
                                    layer,
                                    direction.as_int(),
                                    call.function_name
                                )?;
                            }
                        }

                        Event::EasyShiftMouseDpi(direction) => {
                            for layer in &source.layers.0 {
                                if !rule.enabled {
                                    write!(&mut text, "-- ACTION IS DISABLED: ")?;
                                }

                                writeln!(
                                    &mut text,
                                    "EASY_SHIFT_MOUSE_DPI_MACRO_TABLE[{}][{}] = {}",
                                    layer,
                                    direction.as_int(),
                                    call.function_name
                                )?;
                            }
                        }

                        Event::SimpleKeyDown(key) => {
                            if !rule.enabled {
                                write!(&mut text, "-- ACTION IS DISABLED: ")?;
                            }

                            writeln!(
                                &mut text,
                                "MACRO_TABLE[{}] = {}",
                                key.key_index, call.function_name
                            )?;
                        }

                        Event::SimpleKeyUp(key) => {
                            if !rule.enabled {
                                write!(&mut text, "-- ACTION IS DISABLED: ")?;
                            }

                            writeln!(
                                &mut text,
                                "MACRO_TABLE[{}] = {}",
                                key.key_index, call.function_name
                            )?;
                        }

                        Event::SimpleMouseDown(_key) => {
                            messages::warning!(
                                "Rule: {}: {}",
                                &format!("#{:0>2}", index + 1),
                                tr!("action-disabled").yellow()
                            );

                            writeln!(&mut text, "-- ACTION IS DISABLED")?;
                        }

                        Event::SimpleMouseUp(_key) => {
                            messages::warning!(
                                "Rule: {}: {}",
                                &format!("#{:0>2}", index + 1),
                                tr!("action-disabled").yellow()
                            );

                            writeln!(&mut text, "-- ACTION IS DISABLED")?;
                        }

                        Event::SimpleMouseWheel(_direction) => {
                            messages::warning!(
                                "Rule: {}: {}",
                                &format!("#{:0>2}", index + 1),
                                tr!("action-disabled").yellow()
                            );

                            writeln!(&mut text, "-- ACTION IS DISABLED")?;
                        }

                        Event::SimpleMouseDpi(_direction) => {
                            messages::warning!(
                                "Rule: {}: {}",
                                &format!("#{:0>2}", index + 1),
                                tr!("action-disabled").yellow()
                            );

                            writeln!(&mut text, "-- ACTION IS DISABLED")?;
                        }
                    }
                }
            }

            // insert a newline
            writeln!(&mut text)?;
        }

        Ok(text)
    }

    fn write_to_file<P: AsRef<Path>>(&self, path: P, table: &KeyMappingTable) -> Result<()> {
        let text = self.generate(table)?;

        fs::write(&path, &text)?;

        Ok(())
    }
}
