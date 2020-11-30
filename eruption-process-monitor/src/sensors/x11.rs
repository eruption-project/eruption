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

use crate::constants;
use byteorder::{ByteOrder, LittleEndian};
use x11rb::protocol::xproto::*;
use x11rb::x11_utils::TryParse;
use x11rb::{connection::Connection, rust_connection::RustConnection};

use super::Sensor;

type Result<T> = std::result::Result<T, eyre::Error>;

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

#[derive(Debug, Clone)]
pub struct X11Sensor {
    pub display: String,
}

impl X11Sensor {
    pub fn new() -> Self {
        let display = crate::CONFIG
            .lock()
            .as_ref()
            .unwrap()
            .get_str("X11.display")
            .unwrap_or_else(|_| constants::DEFAULT_X11_DISPLAY.to_string());

        X11Sensor { display }
    }
}

impl Sensor for X11Sensor {
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

rules add window-name '.*YouTube.*Mozilla Firefox' profile3.profile
rules add window-instance gnome-calculator 2

You may want to use the command line tool `xprop` to find the relevant information
"#
        .to_string()
    }

    fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    fn is_pollable(&self) -> bool {
        true
    }

    fn poll(&mut self) -> Result<Box<dyn super::SensorData>> {
        // set up our state
        let (conn, screen) = RustConnection::connect(Some(self.display.as_str()))?;
        let root = conn.setup().roots[screen].root;

        let net_active_window = conn.intern_atom(false, b"_NET_ACTIVE_WINDOW")?.reply()?;
        let net_wm_name = conn.intern_atom(false, b"_NET_WM_NAME")?.reply()?;
        let net_wm_pid = conn.intern_atom(false, b"_NET_WM_PID")?.reply()?;
        let utf8_string = conn.intern_atom(false, b"UTF8_STRING")?.reply()?;
        let cardinal = conn.intern_atom(false, b"CARDINAL")?.reply()?;

        let focus = find_active_window(&conn, root, net_active_window)?;

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
        let name =
            conn.get_property(false, focus, net_wm_name, utf8_string, 0, u32::max_value())?;
        let class = conn.get_property(false, focus, wm_class, string, 0, u32::max_value())?;
        let pid = conn.get_property(false, focus, net_wm_pid, cardinal, 0, u32::max_value())?;
        let (name, class, pid) = (name.reply()?, class.reply()?, pid.reply()?);

        let (instance, class) = parse_wm_class(&class);

        let pid = parse_pid(&pid);

        let result = self::X11SensorData {
            window_name: parse_string_property(&name).to_string(),
            window_instance: instance.to_string(),
            window_class: class.to_string(),
            pid,
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
    std::str::from_utf8(&property.value).unwrap_or("Invalid utf8")
}

fn parse_wm_class(property: &GetPropertyReply) -> (&str, &str) {
    if property.format != 8 {
        return (
            "Malformed property: wrong format",
            "Malformed property: wrong format",
        );
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
        (
            instance.unwrap_or("Invalid utf8"),
            class.unwrap_or("Invalid utf8"),
        )
    } else {
        ("Missing null byte", "Missing null byte")
    }
}

fn parse_pid(property: &GetPropertyReply) -> i32 {
    let value = &property.value;
    LittleEndian::read_u32(&value) as i32
}
