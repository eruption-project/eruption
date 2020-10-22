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

/// Default path of eruption master configuration file
pub const DEFAULT_CONFIG_FILE: &str = "/etc/eruption/eruption.conf";

pub const PROCESS_MONITOR_CONFIG_FILE: &str = "/etc/eruption/process-monitor.conf";

/// Default profile directory
pub const DEFAULT_PROFILE_DIR: &str = "/var/lib/eruption/profiles/";

/// Default script directory
pub const DEFAULT_SCRIPT_DIR: &str = "/usr/share/eruption/scripts/";

/// State directory
pub const STATE_DIR: &str = "~/.local/share/eruption-process-monitor/";

/// Default manifest directory
pub const DEFAULT_MANIFEST_DIR: &str = "/usr/share/eruption-process-monitor/manifests";

/// Number of slots
pub const NUM_SLOTS: usize = 4;

/// Main loop delay
pub const MAIN_LOOP_SLEEP_MILLIS: u64 = 250;

/// Default host name
pub const DEFAULT_HOST: &str = "localhost";

/// Default port number
pub const DEFAULT_PORT: u16 = 2359;

/// Timeout of D-Bus operations
pub const DBUS_TIMEOUT_MILLIS: u64 = 5000;

/// Default delay between images, used for animation mode
pub const DEFAULT_ANIMATION_DELAY_MILLIS: u64 = 83;

/// Default delay between screenshots, used for ambient mode
pub const DEFAULT_FRAME_DELAY_MILLIS: u64 = 37;

/// Default X11 display used by the X11 sensor plugin
pub const DEFAULT_X11_DISPLAY: &str = ":1";

/// The default profile to use
pub const DEFAULT_PROFILE: &str = "default.profile";
