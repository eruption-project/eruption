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

#![allow(dead_code)]

/// Default path of eruption-process-monitor configuration file
pub const PROCESS_MONITOR_CONFIG_FILE: &str = "/etc/eruption/process-monitor.conf";

/// State directory
pub const STATE_DIR: &str = "~/.local/share/eruption-process-monitor/";

/// Main loop delay
pub const MAIN_LOOP_SLEEP_MILLIS: u64 = 250;

/// Timeout of D-Bus operations
pub const DBUS_TIMEOUT_MILLIS: u64 = 5000;

/// Timeout value to use for D-Bus connections
/// that may involve interactivity like e.g.:
/// PolicyKit authentication
pub const DBUS_TIMEOUT_MILLIS_INTERACTIVE: u32 = 30000;

/// Default X11 display used by the X11 sensor plugin
pub const DEFAULT_X11_DISPLAY: &str = ":1";

/// The default profile to use
pub const DEFAULT_PROFILE: &str = "default.profile";
