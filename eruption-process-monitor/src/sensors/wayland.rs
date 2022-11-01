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

// use crate::constants;
// use byteorder::{ByteOrder, LittleEndian};
use async_trait::async_trait;
use std::sync::atomic::Ordering;

use super::Sensor;

type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, Clone)]
pub struct WaylandSensorData {
    pub window_title: String,
    pub window_instance: String,
    pub window_class: String,
}

impl super::SensorData for WaylandSensorData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl super::WindowSensorData for WaylandSensorData {
    fn window_name(&self) -> Option<&str> {
        Some(&self.window_title)
    }

    fn window_instance(&self) -> Option<&str> {
        Some(&self.window_instance)
    }

    fn window_class(&self) -> Option<&str> {
        Some(&self.window_class)
    }
}

#[derive(Debug, Clone)]
pub struct WaylandSensor {}

impl WaylandSensor {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Sensor for WaylandSensor {
    fn get_id(&self) -> String {
        "wayland".to_string()
    }

    fn get_name(&self) -> String {
        "Wayland".to_string()
    }

    fn get_description(&self) -> String {
        "Watches the state of windows on your Wayland based environment".to_string()
    }

    fn get_usage_example(&self) -> String {
        r#"
Wayland:
rules add [window-class|window-class-instance] <regex> [<profile-name.profile>|<slot number>]

rules add window-class '.*YouTube.*Mozilla Firefox' /var/lib/eruption/profiles/profile1.profile
rules add window-instance gnome-calculator 2
"#
        .to_string()
    }

    fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    fn is_pollable(&self) -> bool {
        // HACK: This is an ugly hack, but it improves performance and lowers CPU load
        !crate::X11_POLL_SUCCEEDED.load(Ordering::SeqCst)
    }

    fn is_failed(&self) -> bool {
        false
    }

    fn set_failed(&mut self, _failed: bool) {
        // no op
    }

    fn poll(&mut self) -> Result<Box<dyn super::SensorData>> {
        let result = self::WaylandSensorData {
            window_title: "".to_string(),
            window_instance: "".to_string(),
            window_class: "".to_string(),
        };

        Ok(Box::from(result))
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
