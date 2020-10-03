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

/// Target "Frames per Second"
pub const TARGET_FPS: u64 = 24;

/// Target delay time of main loop iteration
pub const MAIN_LOOP_DELAY_MILLIS: u64 = 1000 / TARGET_FPS;

/// How often should we read out the USB hardware state?
pub const MAIN_LOOP_HW_READ_DIVISOR: u64 = 1; // 1 == do not skip iterations

/// Timeout for waiting on condition variables of Lua upcalls
pub const TIMEOUT_CONDITION_MILLIS: u64 = 100;

/// Max number of events that will be processed in each iteration of the main loop
pub const MAX_EVENTS_PER_ITERATION: u64 = 250;

/// Limit event handler upcalls to 1 per `EVENTS_UPCALL_RATE_LIMIT_MILLIS` milliseconds
pub const EVENTS_UPCALL_RATE_LIMIT_MILLIS: u64 = 10;

/// Amount of time that has to pass before we retry sending a command to the LED control device
pub const DEVICE_SETTLE_MILLIS: u64 = 5;

/// Amount of time that has to pass before we retry to open a failed hardware device
pub const DEVICE_RETRY_MILLIS: u64 = 5000;

/// Update sensors every n seconds
pub const SENSOR_UPDATE_TICKS: u64 = TARGET_FPS /* * 1 */;

/// Timeout value to use for D-Bus connections
pub const DBUS_TIMEOUT_MILLIS: u32 = 250;
