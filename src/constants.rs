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

/// Default path of eruption master configuration file
pub const DEFAULT_CONFIG_FILE: &str = "/etc/eruption/eruption.conf";

/// Default profile directory
pub const DEFAULT_PROFILE_DIR: &str = "/var/lib/eruption/profiles/";

/// Default script directory
pub const DEFAULT_SCRIPT_DIR: &str = "/usr/lib/eruption/scripts/";

/// Default effect script
pub const DEFAULT_EFFECT_SCRIPT: &str = "shockwave.lua";

/// Target delay time of main loop iteration
pub const MAIN_LOOP_DELAY_MILLIS: u64 = (1000.0 / /* target FPS: */ 30.0) as u64;

/// Amount of time that has to pass before we can send another command to the LED control device
pub const DEVICE_SETTLE_MILLIS: u64 = 10;

/// Update sensors every other second
pub const SENSOR_UPDATE_TICKS: u64 = 60;

/// Timeout value to use for D-Bus connections
pub const DBUS_TIMEOUT_MILLIS: u32 = 10000;

/// Default listen address of the web frontend
pub const WEB_FRONTEND_LISTEN_ADDR: &str = "localhost";

/// Default port of the web frontend
pub const WEB_FRONTEND_PORT: u16 = 8059;

/// Default web frontend theme
pub const DEFAULT_FRONTEND_THEME: &str = "metal";
