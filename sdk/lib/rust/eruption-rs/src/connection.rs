/*  SPDX-License-Identifier: LGPL-3.0-or-later  */

/*
    This file is part of the Eruption SDK.

    The Eruption SDK is free software: you can redistribute it and/or modify
    it under the terms of the GNU Lesser General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    The Eruption SDK is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU Lesser General Public License for more details.

    You should have received a copy of the GNU Lesser General Public License
    along with the Eruption SDK.  If not, see <http://www.gnu.org/licenses/>.

    Copyright (c) 2019-2022, The Eruption Development Team
*/

use crate::canvas::Canvas;
use crate::hardware::HotplugInfo;
use crate::transport::{LocalTransport, ServerStatus, Transport};
use crate::Result;
use parking_lot::Mutex;
use std::sync::Arc;

#[derive(Clone)]
pub struct Connection {
    con: Arc<Mutex<dyn Transport>>,
}

impl Connection {
    pub fn new(connection_type: ConnectionType) -> Result<Self> {
        Ok(Self {
            con: Arc::new(Mutex::new(make_transport(&connection_type)?)),
        })
    }

    pub fn connect(&self) -> Result<()> {
        self.con.lock().connect()
    }

    pub fn disconnect(&self) -> Result<()> {
        self.con.lock().disconnect()
    }

    pub fn submit_canvas(&self, canvas: &Canvas) -> Result<()> {
        self.con.lock().submit_canvas(canvas)
    }

    pub fn get_server_status(&self) -> Result<ServerStatus> {
        self.con.lock().get_server_status()
    }

    pub fn notify_device_hotplug(&self, hotplug_info: &HotplugInfo) -> Result<()> {
        self.con.lock().notify_device_hotplug(hotplug_info)
    }
}

impl Drop for Connection {
    fn drop(&mut self) {
        let _ = self.disconnect();
    }
}

/// The type of the connection
#[derive(Debug, Clone)]
pub enum ConnectionType {
    /// Unknown connection type
    Unknown,

    /// Local transport
    Local,

    /// Type REMOTE is currently not implemented
    Remote,
}

fn make_transport(_connection_type: &ConnectionType) -> Result<impl Transport> {
    LocalTransport::new()
}
