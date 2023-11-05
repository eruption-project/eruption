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

    Copyright (c) 2019-2023, The Eruption Development Team
*/

use crate::canvas::Canvas;
use crate::hardware::HotplugInfo;
use crate::transport::{LocalTransport, ServerStatus, Transport};
use crate::Result;
use std::sync::Arc;
use tracing_mutex::stdsync::RwLock;

#[derive(Clone)]
pub struct Connection {
    con: Arc<RwLock<Box<dyn Transport + Sync + Send>>>,
}

impl Connection {
    pub fn new(connection_type: ConnectionType) -> Result<Self> {
        Ok(Self {
            con: Arc::new(RwLock::new(make_transport(&connection_type)?)),
        })
    }

    pub fn connect(&self) -> Result<()> {
        self.con.write().unwrap().connect()
    }

    pub fn disconnect(&self) -> Result<()> {
        self.con.write().unwrap().disconnect()
    }

    pub fn submit_canvas(&self, canvas: &Canvas) -> Result<()> {
        self.con.read().unwrap().submit_canvas(canvas)
    }

    pub fn get_canvas(&self) -> Result<Canvas> {
        self.con.read().unwrap().get_canvas()
    }

    pub fn get_server_status(&self) -> Result<ServerStatus> {
        self.con.read().unwrap().get_server_status()
    }

    pub fn notify_device_hotplug(&self, hotplug_info: &HotplugInfo) -> Result<()> {
        self.con.read().unwrap().notify_device_hotplug(hotplug_info)
    }

    pub fn notify_resume_from_suspend(&self) -> Result<()> {
        self.con.read().unwrap().notify_resume_from_suspend()
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

fn make_transport(_connection_type: &ConnectionType) -> Result<Box<impl Transport>> {
    Ok(Box::new(LocalTransport::new()?))
}
