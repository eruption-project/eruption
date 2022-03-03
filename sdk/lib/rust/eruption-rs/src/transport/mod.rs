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

    Copyright (c) 2019-2022, The Eruption Development Team
*/

use crate::canvas::Canvas;
use crate::Result;
use crate::hardware::HotplugInfo;

mod local;
pub use local::*;

pub trait Transport {
    fn connect(&mut self) -> Result<()>;
    fn disconnect(&mut self) -> Result<()>;

    fn get_server_status(&self) -> Result<ServerStatus>;
    fn submit_canvas(&self, canvas: &Canvas) -> Result<()>;

    fn notify_device_hotplug(&self, hotplug_info: &HotplugInfo) -> Result<()>;
}

#[derive(Debug, Default, Clone)]
pub struct ServerStatus {
    pub server: String,
}
