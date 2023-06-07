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

use std::env;
use std::ffi::CString;

use crate::constants;
use async_trait::async_trait;
use byteorder::{ByteOrder, LittleEndian};
use parking_lot::Mutex;
use std::sync::Arc;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::x11_utils::TryParse;
use x11rb::xcb_ffi::XCBConnection;

use super::{Sensor, SensorConfiguration, SENSORS_CONFIGURATION};

type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum X11SensorError {
    // #[error("Unknown error: {description}")]
    // UnknownError { description: String },
    #[error("Sensor error: {description}")]
    SensorError { description: String },

    #[error("Sensor failed: {description}")]
    SensorFailed { description: String },
}

#[derive(Debug, Clone)]
pub struct X11SensorData {
    pub window_name: String,
    pub window_instance: String,
    pub window_class: String,
    pub pid: i32,
}

impl super::SensorData for X11SensorData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl super::WindowSensorData for X11SensorData {
    fn window_name(&self) -> Option<&str> {
        Some(&self.window_name)
    }

    fn window_instance(&self) -> Option<&str> {
        Some(&self.window_instance)
    }

    fn window_class(&self) -> Option<&str> {
        Some(&self.window_class)
    }
}

#[derive(Debug, Clone)]
pub struct X11Sensor {
    pub display: String,
    pub is_failed: bool,
    pub conn: Option<Arc<Mutex<XCBConnection>>>,
    pub screen: Option<usize>,
}

impl X11Sensor {
    pub fn new() -> Self {
        let display = crate::CONFIG
            .lock()
            .as_ref()
            .unwrap()
            .get_string("X11.display")
            .unwrap_or_else(|_| {
                env::var("DISPLAY").unwrap_or_else(|_| constants::DEFAULT_X11_DISPLAY.to_string())
            });

        X11Sensor {
            display,
            is_failed: false,
            conn: None,
            screen: None,
        }
    }
}

#[async_trait]
impl Sensor for X11Sensor {
    fn initialize(&mut self) -> Result<()> {
        let (conn, screen) = XCBConnection::connect(Some(&CString::new(self.display.clone())?))?;

        self.conn = Some(Arc::new(Mutex::new(conn)));
        self.screen = Some(screen);

        Ok(())
    }

    fn is_enabled(&self) -> bool {
        SENSORS_CONFIGURATION
            .read()
            .contains(&SensorConfiguration::EnableX11)
    }

    fn get_id(&self) -> String {
        "x11".to_string()
    }

    fn get_name(&self) -> String {
        "X11".to_string()
    }

    fn get_description(&self) -> String {
        "Watches the state of windows on the X window system server".to_string()
    }

    fn get_usage_example(&self) -> String {
        r#"
X11:
rules add window-[class|instance|name] <regex> [<profile-name.profile>|<slot number>]

rules add window-name '.*YouTube.*Mozilla Firefox' /var/lib/eruption/profiles/profile1.profile
rules add window-instance gnome-calculator 2

You may want to use the command line tool `xprop` to find the relevant information
"#
        .to_string()
    }

    fn is_pollable(&self) -> bool {
        true
    }

    fn is_failed(&self) -> bool {
        self.is_failed
    }

    fn set_failed(&mut self, failed: bool) {
        self.is_failed = failed;
    }

    fn poll(&mut self) -> Result<Box<dyn super::SensorData>> {
        if let Some(conn) = &self.conn {
            let conn = conn.lock();

            let screen = self.screen.unwrap_or(0);
            let root = conn.setup().roots[screen].root;

            let net_active_window = conn.intern_atom(false, b"_NET_ACTIVE_WINDOW")?.reply()?;
            let net_wm_name = conn.intern_atom(false, b"_NET_WM_NAME")?.reply()?;
            let net_wm_pid = conn.intern_atom(false, b"_NET_WM_PID")?.reply()?;
            let utf8_string = conn.intern_atom(false, b"UTF8_STRING")?.reply()?;
            let cardinal = conn.intern_atom(false, b"CARDINAL")?.reply()?;

            let focus = find_active_window(&*conn, root, net_active_window)?;

            if focus == 0 {
                // found the root window

                let result = self::X11SensorData {
                    window_name: "".to_string(),
                    window_instance: "".to_string(),
                    window_class: "".to_string(),
                    pid: 0,
                };

                Ok(Box::from(result))
            } else {
                // any other window

                // collect the replies to the atoms
                let (net_wm_name, net_wm_pid, utf8_string, cardinal) = (
                    net_wm_name.atom,
                    net_wm_pid.atom,
                    utf8_string.atom,
                    cardinal.atom,
                );
                let (wm_class, string) = (
                    conn.intern_atom(false, b"WM_CLASS")?.reply()?.atom,
                    conn.intern_atom(false, b"STRING")?.reply()?.atom,
                );

                // get window properties
                let name = conn
                    .get_property(false, focus, net_wm_name, utf8_string, 0, u32::max_value())
                    .ok();
                let class = conn
                    .get_property(false, focus, wm_class, string, 0, u32::max_value())
                    .ok();
                let pid = conn
                    .get_property(false, focus, net_wm_pid, cardinal, 0, u32::max_value())
                    .ok();

                let name = name.and_then(|name| name.reply().ok());
                let class = class.and_then(|class| class.reply().ok());
                let pid = pid.and_then(|pid| pid.reply().ok());

                let instance_and_class = class.and_then(|class| parse_wm_class(&class));

                let pid = pid.and_then(|pid| Some(parse_pid(&pid)));

                let result = self::X11SensorData {
                    window_name: name
                        .and_then(|name| Some(parse_string_property(&name).to_owned()))
                        .unwrap_or_else(|| "".to_string())
                        .to_string(),
                    window_instance: instance_and_class
                        .clone()
                        .and_then(|(ref instance, _)| Some(instance.to_owned()))
                        .unwrap_or_else(|| "".to_string())
                        .to_string()
                        .clone(),
                    window_class: instance_and_class
                        .and_then(|(_, ref class)| Some(class.to_owned()))
                        .unwrap_or_else(|| "".to_string())
                        .to_string()
                        .clone(),
                    pid: pid.unwrap_or(0),
                };

                if result.window_name.is_empty()
                    && result.window_instance.is_empty()
                    && result.window_class.is_empty()
                {
                    Err(X11SensorError::SensorFailed {
                        description: "Empty sensor data".to_owned(),
                    }
                    .into())
                } else {
                    Ok(Box::from(result))
                }
            }
        } else {
            Err(X11SensorError::SensorError {
                description: "Could not connect to the X server".to_string(),
            }
            .into())
        }
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

fn find_active_window(
    conn: &impl Connection,
    root: Window,
    net_active_window: InternAtomReply,
) -> Result<Window> {
    let window = conn.intern_atom(false, b"WINDOW")?.reply()?.atom;
    let active_window = conn
        .get_property(false, root, net_active_window.atom, window, 0, 1)?
        .reply()?;

    if active_window.format == 32 && active_window.length == 1 {
        // Things will be so much easier with the next release:
        // This does active_window.value32().next().unwrap()
        Ok(u32::try_parse(&active_window.value)?.0)
    } else {
        // Query the input focus
        Ok(conn.get_input_focus()?.reply()?.focus)
    }
}

fn parse_string_property(property: &GetPropertyReply) -> &str {
    std::str::from_utf8(&property.value).unwrap_or_default()
}

fn parse_wm_class(property: &GetPropertyReply) -> Option<(String, String)> {
    if property.format != 8 {
        return None;
    }

    let value = &property.value;

    // The property should contain two null-terminated strings. Find them.
    if let Some(middle) = value.iter().position(|&b| b == 0) {
        let (instance, class) = value.split_at(middle);

        // Skip the null byte at the beginning
        let mut class = &class[1..];

        // Remove the last null byte from the class, if it is there.
        if class.last() == Some(&0) {
            class = &class[..class.len() - 1];
        }

        let instance = std::str::from_utf8(instance);
        let class = std::str::from_utf8(class);

        if instance.is_ok() && class.is_ok() {
            Some((
                instance.unwrap_or("").to_owned(),
                class.unwrap_or("").to_owned(),
            ))
        } else {
            None
        }
    } else {
        None
    }
}

fn parse_pid(property: &GetPropertyReply) -> i32 {
    if property.value_len < 4 {
        0
    } else {
        let value = &property.value;
        LittleEndian::read_u32(value) as i32
    }
}
