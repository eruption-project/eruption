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

#[derive(Debug, Default, Clone)]
pub struct Hardware {}

impl Hardware {
    pub fn new() -> Self {
        Self {}
    }
}

/*
impl Default for Hardware {
    fn default() -> Self {
        Self {}
    }
}
*/

use std::path::PathBuf;

use bincode::{Decode, Encode};

#[derive(Debug, Default, Clone, Encode, Decode)]
pub struct HotplugInfo {
    pub devpath: Option<PathBuf>,
    pub usb_vid: u16,
    pub usb_pid: u16,
}
