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
#![allow(unused_imports)]

// Linux specific configuration
mod linux {
    // ****************************************************************************
    // Default filesystem paths and names

    /// Default path of eruption master configuration file
    pub const DEFAULT_CONFIG_FILE: &str = "/etc/eruption/eruption.conf";

    /// Default path of the Magma TUI configuration file
    pub const DEFAULT_MAGMA_CONFIG_FILE: &str = "~/.config/eruption/magma.conf";

    /// Default effect script
    pub const DEFAULT_EFFECT_SCRIPT: &str = "solid.lua";

    /// Default profile directory
    pub const DEFAULT_PROFILE_DIR: &str = "/var/lib/eruption/profiles/";

    /// Default script directory
    pub const DEFAULT_SCRIPT_DIR: &str = "/usr/share/eruption/scripts/";

    /// Default script directory
    pub const DEFAULT_MACRO_DIR: &str = "/usr/share/eruption/scripts/lib/macros";

    /// Default script directory
    pub const DEFAULT_KEYMAP_DIR: &str = "/usr/share/eruption/scripts/lib/keymaps";

    /// Default AFK profile
    pub const DEFAULT_AFK_PROFILE: &str = "/var/lib/eruption/profiles/blackout.profile";

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

    // ****************************************************************************
}

use std::time::Duration;

#[cfg(not(target_os = "windows"))]
pub use linux::*;

// Windows specific configuration
mod windows {
    /// Default path of eruption master configuration file
    pub const DEFAULT_CONFIG_FILE: &str = "etc/eruption.conf";

    /// Default effect script
    pub const DEFAULT_EFFECT_SCRIPT: &str = "solid.lua";

    /// Default profile directory
    pub const DEFAULT_PROFILE_DIR: &str = "profiles/";

    /// Default script directory
    pub const DEFAULT_SCRIPT_DIR: &str = "scripts/";

    /// Default script directory
    pub const DEFAULT_MACRO_DIR: &str = "scripts/lib/macros";

    /// Default script directory
    pub const DEFAULT_KEYMAP_DIR: &str = "scripts/lib/keymaps";

    /// Default AFK profile
    pub const DEFAULT_AFK_PROFILE: &str = "profiles/blackout.profile";

    /// The `/run/eruption/` directory
    pub const RUN_ERUPTION_DIR: &str = "run/";

    /// State directory
    pub const STATE_DIR: &str = "run/";

    /// Eruption daemon PID file
    pub const PID_FILE: &str = "run/eruption.pid";

    /// Name of the Systemd unit file of eruption
    pub const UNIT_NAME_ERUPTION: &str = "eruption.service";

    /// Name of the Systemd unit file of the eruption process monitor
    pub const UNIT_NAME_PROCESS_MONITOR: &str = "eruption-process-monitor.service";

    /// Name of the Systemd unit file of the eruption audio proxy
    pub const UNIT_NAME_AUDIO_PROXY: &str = "eruption-audio-proxy.service";

    /// Name of the Systemd unit file of the eruption FX proxy
    pub const UNIT_NAME_FX_PROXY: &str = "eruption-fx-proxy.service";

    /// Eruption daemon control UNIX domain socket (SDK support)
    pub const CONTROL_PIPE_NAME: &str = "//./pipe/eruption-control";

    /// Eruption daemon audio data UNIX domain socket
    pub const AUDIO_PIPE_NAME: &str = "//./pipe/eruption-audio";
}

#[cfg(target_os = "windows")]
pub use windows::*;

// ****************************************************************************
// Eruption core daemon

/// Number of slots
pub const NUM_SLOTS: usize = 4;

/// Default AFK timeout
pub const AFK_TIMEOUT_SECS: u64 = 0;

/// Amount of time that has to pass before we retry sending a command to the LED/control USB sub-device
pub const DEVICE_SETTLE_MILLIS: u64 = 100;

/// Amount of time that we pause after sending a packet to the LED/control USB sub-device
pub const DEVICE_MICRO_DELAY: u64 = 1;

/// Amount of time that we pause after sending a command to the LED/control USB sub-device
pub const DEVICE_SHORT_DELAY: u64 = 25;

/// This helps slow USB HUBs and KVM switches to not fail to init the device
pub const DEVICE_SETTLE_DELAY: u64 = 250;

/// Update sensors every n seconds
/// It is recommended to use a prime number value here
pub const SENSOR_UPDATE_TICKS: u64 = TARGET_FPS_LIMIT / 2;

/// Generic timeout value for fast operations
pub const TIMEOUT_MILLIS_SHORT: u64 = 500;

/// Generic timeout value for slow operations
pub const TIMEOUT_MILLIS_LONG: u64 = 1000;

/// Timeout used for waiting for a device that is pending initialization
pub const TIMEOUT_PENDING_DEVICE_INIT: u64 = 4000;

/// Timeout value to use for the D-Bus event loop
pub const DBUS_WAIT_MILLIS: u32 = 250;

/// Timeout value to use for D-Bus connections
pub const DBUS_TIMEOUT_MILLIS: u32 = 4000;

/// Timeout value to use for D-Bus connections
/// that may involve interactivity like e.g.: PolicyKit authentication
pub const DBUS_TIMEOUT_MILLIS_INTERACTIVE: u32 = 30000;

/// Wait n milliseconds before sending the LED "off pattern" on shutdown
pub const SHUTDOWN_TIMEOUT_MILLIS: u32 = DEVICE_SETTLE_MILLIS as u32;

/// Timer interval in milliseconds for the device config and status poll timer
/// It is recommended to use a prime number value here
pub const POLL_TIMER_INTERVAL_MILLIS: u64 = 499;

/// Max. supported number of keys on a keyboard
pub const MAX_KEYS: usize = 256;

/// Max. supported number of mouse buttons
pub const MAX_MOUSE_BUTTONS: usize = 32;

/// Limit event handler upcalls to 1 every `EVENTS_UPCALL_RATE_LIMIT_MILLIS` milliseconds
pub const EVENTS_UPCALL_RATE_LIMIT_MILLIS: u64 = 16;

/// Limit framerate to n frames per second
pub const TARGET_FPS_LIMIT: u64 = 60;

/// Timer tick events per second (timer resolution)
pub const TIMER_TPS: u64 = 60;

/// The width of the canvas (max. reasonable value approx. 128)
/// NOTE: Values considerably larger than 128 currently lead to stuttering in the Eruption GUI
pub const CANVAS_WIDTH: usize = 64; // 86;

/// The height of the canvas (max. reasonable value approx. 128)
/// NOTE: Values considerably larger than 128 currently lead to stuttering in the Eruption GUI
pub const CANVAS_HEIGHT: usize = 48; // 64;

/// The number of "pixels" on the canvas
pub const CANVAS_SIZE: usize = CANVAS_WIDTH * CANVAS_HEIGHT;

/// Timeout for waiting on condition variables of Lua upcalls
pub const TIMEOUT_UPCALL_MILLIS: u64 = 250;

/// Timeout for waiting on ready condition after the RealizeColorMap upcall
pub const TIMEOUT_REALIZE_COLOR_MAP_MILLIS: u64 = 40;

/// Max number of events that will be processed in each iteration of the main loop
pub const MAX_EVENTS_PER_ITERATION: u64 = 1024;

/// Fade in on profile switch for n milliseconds
pub const FADE_MILLIS: u64 = 1333;

/// Fade-in effect duration on startup
pub const STARTUP_FADE_IN_MILLIS: u64 = 2000;

// ****************************************************************************

// ****************************************************************************
// Networking related

/// The capacity of the buffer used for exchanging control messages
pub const NET_BUFFER_CAPACITY: usize = CANVAS_SIZE * 4 + 128;

/// The capacity of the buffer used for receiving audio samples as well as control messages
pub const NET_AUDIO_BUFFER_CAPACITY: usize = 4096;

/// Default X11 display used by the X11 sensor plugin of the eruption-fx-proxy daemon
pub const DEFAULT_X11_DISPLAY: &str = ":0";

/// Default host name for the eruption-netfx utility
pub const DEFAULT_HOST: &str = "localhost";

/// Default port number for the eruption-netfx utility
pub const DEFAULT_PORT: u16 = 2359;

// ****************************************************************************

// ****************************************************************************
// Session daemons and UIs

/// Audio proxy loop sleep time/timeout for poll(2)
pub const SLEEP_TIME_TIMEOUT: u64 = 2000;

/// Default delay between images, used for animation mode of the eruption-fx-proxy daemon
pub const DEFAULT_ANIMATION_DELAY_MILLIS: u64 = 83;

/// Default delay between screenshots, used for ambient mode of the eruption-fx-proxy daemon
pub const DEFAULT_FRAME_DELAY_MILLIS: u64 = 37;

/// Main loop delay of the eruption-fx-proxy daemon
pub const MAIN_LOOP_SLEEP_MILLIS: u64 = 199;

/// The time for which a notification shall be shown
pub const NOTIFICATION_TIME_MILLIS: u64 = 4000;

/// Notify the software watchdog every n milliseconds
pub const WATCHDOG_NOTIFY_MILLIS: u64 = 1499;

/// eruption-gui: The time to wait before an external process is spawned, after the profile has been switched
pub const PROCESS_SPAWN_WAIT_MILLIS: u64 = 800;

/// The number of lines to scroll on cursor up or down
pub const SCROLL_LINES: usize = 1;

/// The number of lines to scroll on page up or page down
pub const PAGE_SCROLL_LINES: usize = 10;

// ****************************************************************************
