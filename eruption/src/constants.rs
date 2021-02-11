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

/// Default profile directory
pub const DEFAULT_PROFILE_DIR: &str = "/var/lib/eruption/profiles/";

/// Default script directory
pub const DEFAULT_SCRIPT_DIR: &str = "/usr/share/eruption/scripts/";

/// State directory
pub const STATE_DIR: &str = "/var/lib/eruption/";

/// Number of slots
pub const NUM_SLOTS: usize = 4;

/// Default effect script
pub const DEFAULT_EFFECT_SCRIPT: &str = "organic.lua";

/// Default AFK timeout
pub const AFK_TIMEOUT_SECS: u64 = 0;

/// Default AFK profile
pub const DEFAULT_AFK_PROFILE: &str = "rainbow-wave.profile";

/// eruption-gui: The time to wait before an external process is spawned, after the profile has been switched
pub const PROCESS_SPAWN_WAIT_MILLIS: u64 = 800;

/// Target frames per second
pub const TARGET_FPS: u64 = 20;

/// The number of "pixels" on the canvas
pub const CANVAS_SIZE: usize = 144 + 36;

/// The width of the canvas
pub const CANVAS_WIDTH: usize = 22 + 8;

/// The height of the canvas
pub const CANVAS_HEIGHT: usize = 6;

/// Timeout for waiting on condition variables of Lua upcalls
pub const TIMEOUT_CONDITION_MILLIS: u64 = 100;

/// Max number of events that will be processed in each iteration of the main loop
pub const MAX_EVENTS_PER_ITERATION: u64 = 32;

/// Limit event handler upcalls to 1 per `EVENTS_UPCALL_RATE_LIMIT_MILLIS` milliseconds
pub const EVENTS_UPCALL_RATE_LIMIT_MILLIS: u64 = 25;

/// Amount of time that has to pass before we retry sending a command to the LED control device
pub const DEVICE_SETTLE_MILLIS: u64 = 50;

/// Update sensors every n seconds
pub const SENSOR_UPDATE_TICKS: u64 = TARGET_FPS /* * 1 */;

/// Timeout value to use for D-Bus connections
pub const DBUS_TIMEOUT_MILLIS: u32 = 250;

/// Timeout value to use for D-Bus connections
/// that may involve interactivity like e.g.: PolicyKit authentication
pub const DBUS_TIMEOUT_MILLIS_INTERACTIVE: u32 = 30000;

// Wait n seconds before sending the LED "off pattern" on shutdown
pub const SHUTDOWN_TIMEOUT_MILLIS: u32 = 150;

// Max. supported number of keys on a keyboard
pub const MAX_KEYS: usize = 144;

// Max. supported number of mouse buttons
pub const MAX_MOUSE_BUTTONS: usize = 32;
