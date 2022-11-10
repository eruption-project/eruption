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

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::canvas::Canvas;
use crate::hardware::HotplugInfo;
use crate::Result;

mod local;
pub use local::*;

pub trait Transport {
    fn connect(&mut self) -> Result<()>;
    fn disconnect(&mut self) -> Result<()>;

    fn get_server_status(&self) -> Result<ServerStatus>;
    fn get_active_profile(&self) -> Result<PathBuf>;
    fn switch_profile(&self, profile_file: &Path) -> Result<bool>;
    fn set_parameters(
        &self,
        profile_file: &Path,
        script_file: &Path,
        parameter_values: HashMap<String, String>,
    ) -> Result<()>;
    fn submit_canvas(&self, canvas: &Canvas) -> Result<()>;

    fn notify_device_hotplug(&self, hotplug_info: &HotplugInfo) -> Result<()>;
}

#[derive(Debug, Default, Clone)]
pub struct ServerStatus {
    pub server: String,
}
