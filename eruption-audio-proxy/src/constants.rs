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

/// Eruption daemon audio data UNIX domain socket
pub const AUDIO_SOCKET_NAME: &str = "/run/eruption/audio.sock";

/// The capacity of the sample buffer
pub const AUDIO_BUFFER_SIZE: usize = 4096 - 16;

/// The capacity of the buffer used for sending audio samples/commands over a socket
pub const NET_BUFFER_CAPACITY: usize = 4096;

// /// Timeout of D-Bus operations
// pub const DBUS_TIMEOUT_MILLIS: u64 = 5000;

/// Time in milliseconds that has to pass before we query PipeWire/PulseAudio for
/// the master volume and audio muted state of the device again
pub const DEVICE_POLL_INTERVAL: u64 = 100;

/// Main loop sleep time/timeout for poll(2)
pub const SLEEP_TIME_TIMEOUT: u64 = 2000;

/// Main loop sleep time, when we are disconnected from Eruption
pub const SLEEP_TIME_WHILE_DISCONNECTED: u64 = 1000;
