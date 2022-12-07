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

use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Display;
use std::path::Path;
use std::path::PathBuf;

use chrono::{DateTime, Utc};
use color_eyre::owo_colors::OwoColorize;
use serde::Deserialize;
use serde::Serialize;
use serde_json_any_key::any_key_map;

use crate::util;

//pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct KeyMappingTable {
    pub metadata: TableMetadata,

    #[serde(with = "any_key_map")]
    pub mappings: BTreeMap<Source, Rule>,
}

#[allow(unused)]
impl KeyMappingTable {
    pub fn new() -> Self {
        Self {
            metadata: TableMetadata::default(),
            mappings: BTreeMap::new(),
        }
    }

    pub fn file_name(&self) -> &Path {
        &self.metadata.file_name
    }

    pub fn description(&self) -> &str {
        &self.metadata.description
    }

    pub fn set_file_name<P: AsRef<Path>>(&mut self, file_name: P) {
        self.metadata.file_name = file_name.as_ref().to_path_buf();
    }

    pub fn set_description(&mut self, description: &str) {
        self.metadata.description = description.to_string();
    }

    pub fn insert(&mut self, source: Source, rule: Rule) -> Option<Rule> {
        self.mappings.insert(source, rule)
    }

    pub fn remove(&mut self, source: &Source) -> Option<Rule> {
        self.mappings.remove(source)
    }

    pub fn metadata(&self) -> &TableMetadata {
        &self.metadata
    }

    pub fn mappings(&self) -> &BTreeMap<Source, Rule> {
        &self.mappings
    }

    pub fn metadata_mut(&mut self) -> &mut TableMetadata {
        &mut self.metadata
    }

    pub fn mappings_mut(&mut self) -> &mut BTreeMap<Source, Rule> {
        &mut self.mappings
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub struct TableMetadata {
    pub file_name: PathBuf,
    pub description: String,
    pub creation_date: DateTime<Utc>,
}

#[allow(unused)]
impl TableMetadata {
    pub fn new(file_name: PathBuf, description: String, creation_date: DateTime<Utc>) -> Self {
        TableMetadata {
            file_name,
            description,
            creation_date,
        }
    }
}

impl Default for TableMetadata {
    fn default() -> TableMetadata {
        Self {
            file_name: PathBuf::from("default.keymap"),
            description: "<no description specified>".to_string(),
            creation_date: Utc::now(),
        }
    }
}

impl Display for TableMetadata {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "File: {}\nDescription: {}\nCreation date: {}",
            self.file_name.display(),
            self.description,
            self.creation_date
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub struct Source {
    pub event: Event,
    pub layers: LayerSet<usize>,
}

impl Source {
    pub fn new(event: Event) -> Self {
        let mut layers = BTreeSet::new();

        layers.insert(1);

        Self {
            event,
            layers: LayerSet(layers),
        }
    }

    #[allow(unused)]
    pub fn new_with_layers(event: Event, active_layers: &[usize]) -> Self {
        let mut layers = BTreeSet::new();

        layers.extend(active_layers.iter());

        Self {
            event,
            layers: LayerSet(layers),
        }
    }

    pub fn get_layers_mut(&mut self) -> &mut BTreeSet<usize> {
        &mut self.layers.0
    }
}

impl Display for Source {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let has_layers = match self.event {
            Event::Null => false,
            Event::HidKeyDown(_) => false,
            Event::HidKeyUp(_) => false,
            Event::HidMouseDown(_) => false,
            Event::HidMouseUp(_) => false,
            Event::EasyShiftKeyDown(_) => true,
            Event::EasyShiftKeyUp(_) => true,
            Event::EasyShiftMouseDown(_) => true,
            Event::EasyShiftMouseUp(_) => true,
            Event::EasyShiftMouseWheel(_) => true,
            Event::EasyShiftMouseDpi(_) => true,
            Event::SimpleKeyDown(_) => false,
            Event::SimpleKeyUp(_) => false,
            Event::SimpleMouseDown(_) => false,
            Event::SimpleMouseUp(_) => false,
            Event::SimpleMouseWheel(_) => false,
            Event::SimpleMouseDpi(_) => false,
        };

        if has_layers {
            f.write_str(&format!(
                "Source: {} on layers: [ {}]",
                self.event, self.layers
            ))
        } else {
            f.write_str(&format!("Source: {}", self.event))
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub struct LayerSet<T>(pub BTreeSet<T>)
where
    T: Ord + PartialOrd;

impl<T> Display for LayerSet<T>
where
    T: Display + Ord + PartialOrd,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for value in self.0.iter() {
            f.write_str(&format!("{value} "))?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub struct Rule {
    pub description: String,
    pub enabled: bool,
    pub action: Action,
}

impl Rule {
    pub fn new(action: Action, description: &str, enabled: bool) -> Self {
        Self {
            description: description.to_owned(),
            enabled,
            action,
        }
    }
}

impl Display for Rule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("Action: {}", self.action))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum Action {
    Null,
    InjectKey(EvdevEvent),
    Call(Macro),
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Action::Null => f.write_str("<No action>"),

            Action::InjectKey(key) => f.write_str(&format!("Event: {key}")),
            Action::Call(call) => f.write_str(&format!("Call: {call}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub struct Key {
    pub key_index: usize,

    /// The USB vendor ID of the device
    pub usb_vid: u16,

    /// The USB product ID of the device
    pub usb_pid: u16,
}

impl Key {
    pub fn new(key_index: usize, (usb_vid, usb_pid): (u16, u16)) -> Self {
        Self {
            key_index,
            usb_vid,
            usb_pid,
        }
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!(
            "{} ({})",
            util::key_index_to_symbol(self.key_index + 1, (self.usb_vid, self.usb_pid))
                .unwrap_or_else(|| "<invalid symbol>".italic().to_string()),
            self.key_index
        ))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub struct EvdevEvent {
    pub event: u32,
}

impl Display for EvdevEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        //f.write_str(&format!("Event:{}", self.key_index))
        f.write_str(&util::evdev_event_code_to_string(self.event))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub struct Macro {
    pub function_name: String,
}

impl Display for Macro {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("function {}", self.function_name))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Eq, PartialEq, Hash, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum Event {
    Null,

    HidKeyDown(Key),
    HidKeyUp(Key),
    HidMouseDown(Key),
    HidMouseUp(Key),

    EasyShiftKeyDown(Key),
    EasyShiftKeyUp(Key),
    EasyShiftMouseDown(Key),
    EasyShiftMouseUp(Key),
    EasyShiftMouseWheel(Direction),
    EasyShiftMouseDpi(Direction),

    SimpleKeyDown(Key),
    SimpleKeyUp(Key),
    SimpleMouseDown(Key),
    SimpleMouseUp(Key),
    SimpleMouseWheel(Direction),
    SimpleMouseDpi(Direction),
}

impl Display for Event {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Event::Null => f.write_str("<No action>"),

            Event::HidKeyDown(key) => f.write_str(&format!("hid+key-down: {key}")),
            Event::HidKeyUp(key) => f.write_str(&format!("hid+key-up: {key}")),
            Event::HidMouseDown(key) => f.write_str(&format!("hid+mouse-down: {key}")),
            Event::HidMouseUp(key) => f.write_str(&format!("hid+mouse-up: {key}")),

            Event::EasyShiftKeyDown(key) => f.write_str(&format!("es+key-down: {key}")),
            Event::EasyShiftKeyUp(key) => f.write_str(&format!("es+key-up: {key}")),
            Event::EasyShiftMouseDown(key) => f.write_str(&format!("es+mouse-down: {key}")),
            Event::EasyShiftMouseUp(key) => f.write_str(&format!("es+mouse-up: {key}")),
            Event::EasyShiftMouseWheel(direction) => {
                f.write_str(&format!("es+mouse-wheel: {direction}"))
            }
            Event::EasyShiftMouseDpi(direction) => {
                f.write_str(&format!("es+mouse-dpi: {direction}"))
            }

            Event::SimpleKeyDown(key) => f.write_str(&format!("key-down: {key}")),
            Event::SimpleKeyUp(key) => f.write_str(&format!("key-up: {key}")),
            Event::SimpleMouseDown(key) => f.write_str(&format!("mouse-down: {key}")),
            Event::SimpleMouseUp(key) => f.write_str(&format!("mouse-up: {key}")),
            Event::SimpleMouseWheel(direction) => {
                f.write_str(&format!("mouse-wheel: {direction}"))
            }
            Event::SimpleMouseDpi(direction) => f.write_str(&format!("mouse-dpi: {direction}")),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq, Ord, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    pub fn as_int(&self) -> i32 {
        match self {
            Direction::Up => 1,
            Direction::Down => 2,
            Direction::Left => 3,
            Direction::Right => 4,
        }
    }
}

impl Display for Direction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Direction::Up => f.write_str("Up"),
            Direction::Down => f.write_str("Down"),
            Direction::Left => f.write_str("Left"),
            Direction::Right => f.write_str("Right"),
        }
    }
}
