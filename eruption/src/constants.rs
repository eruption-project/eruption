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

#![allow(dead_code)]

/// Default path of eruption master configuration file
pub const DEFAULT_CONFIG_FILE: &str = "/etc/eruption/eruption.conf";

/// Default profile directory
pub const DEFAULT_PROFILE_DIR: &str = "/var/lib/eruption/profiles/";

/// Default script directory
pub const DEFAULT_SCRIPT_DIR: &str = "/usr/share/eruption/scripts/";

/// Default script directory
pub const DEFAULT_MACRO_DIR: &str = "/usr/share/eruption/scripts/lib/macros";

/// Default script directory
pub const DEFAULT_KEYMAP_DIR: &str = "/usr/share/eruption/scripts/lib/keymaps";

/// The `/run/eruption/` directory
pub const RUN_ERUPTION_DIR: &str = "/run/eruption/";

/// State directory
pub const STATE_DIR: &str = "/var/lib/eruption/";

/// Eruption daemon PID file
pub const PID_FILE: &str = "/run/eruption/eruption.pid";

/// Name of the Systemd unit file of eruption
pub const UNIT_NAME_ERUPTION: &str = "eruption.service";

/// Name of the Systemd unit file of the eruption process monitor
pub const UNIT_NAME_PROCESS_MONITOR: &str = "eruption-process-monitor.service";

/// Name of the Systemd unit file of the eruption audio proxy
pub const UNIT_NAME_AUDIO_PROXY: &str = "eruption-audio-proxy.service";

/// Name of the Systemd unit file of the eruption FX proxy
pub const UNIT_NAME_FX_PROXY: &str = "eruption-fx-proxy.service";

/// Eruption daemon control UNIX domain socket (SDK support)
pub const CONTROL_SOCKET_NAME: &str = "/run/eruption/control.sock";

/// Eruption daemon audio data UNIX domain socket
pub const AUDIO_SOCKET_NAME: &str = "/run/eruption/audio.sock";

/// Number of slots
pub const NUM_SLOTS: usize = 4;

/// Default effect script
pub const DEFAULT_EFFECT_SCRIPT: &str = "organic.lua";

/// Default AFK timeout
pub const AFK_TIMEOUT_SECS: u64 = 0;

/// Default AFK profile
pub const DEFAULT_AFK_PROFILE: &str = "/var/lib/eruption/profiles/rainbow-wave.profile";

/// Notify the software watchdog every n milliseconds
pub const WATCHDOG_NOTIFY_MILLIS: u64 = 1499;

/// eruption-gui: The time to wait before an external process is spawned, after the profile has been switched
pub const PROCESS_SPAWN_WAIT_MILLIS: u64 = 800;

/// Target frames per second
pub const TARGET_FPS: u64 = 19;

/// Target timer tick events per second
pub const TICK_FPS: u64 = 19;

/// Fade in on profile switch for n milliseconds
pub const FADE_MILLIS: u64 = 1333;

/// The number of "pixels" on the canvas
pub const CANVAS_SIZE: usize = 144 + 36;

/// The width of the canvas
pub const CANVAS_WIDTH: usize = 22 + 8;

/// The height of the canvas
pub const CANVAS_HEIGHT: usize = 6;

/// The capacity of the buffer used for receiving audio samples
pub const NET_BUFFER_CAPACITY: usize = 4096;

/// Timeout for waiting on condition variables of Lua upcalls
pub const TIMEOUT_CONDITION_MILLIS: u64 = 25;

/// Max number of events that will be processed in each iteration of the main loop
pub const MAX_EVENTS_PER_ITERATION: u64 = 128;

/// Limit event handler upcalls to 1 per `EVENTS_UPCALL_RATE_LIMIT_MILLIS` milliseconds
pub const EVENTS_UPCALL_RATE_LIMIT_MILLIS: u64 = 25;

/// Amount of time that has to pass before we retry sending a command to the LED control device
pub const DEVICE_SETTLE_MILLIS: u64 = 25;

/// Update sensors every n seconds
/// It is recommended to use a prime number value here
pub const SENSOR_UPDATE_TICKS: u64 = 19; // TARGET_FPS /* * 1 */;

/// Timeout value to use for D-Bus connections
pub const DBUS_TIMEOUT_MILLIS: u32 = 250;

/// Timeout value to use for D-Bus connections
/// that may involve interactivity like e.g.: PolicyKit authentication
pub const DBUS_TIMEOUT_MILLIS_INTERACTIVE: u32 = 30000;

/// Wait n seconds before sending the LED "off pattern" on shutdown
pub const SHUTDOWN_TIMEOUT_MILLIS: u32 = DEVICE_SETTLE_MILLIS as u32;

/// Timer interval in milliseconds for the device config and status poll timer
/// It is recommended to use a prime number value here
pub const POLL_TIMER_INTERVAL_MILLIS: u64 = 499;

/// Audio proxy loop sleep time/timeout for poll(2)
pub const SLEEP_TIME_TIMEOUT: u64 = 2000;

/// Max. supported number of keys on a keyboard
pub const MAX_KEYS: usize = 144;

/// Max. supported number of mouse buttons
pub const MAX_MOUSE_BUTTONS: usize = 32;
