/*
    This file is part of Eruption.

    Eruption is free software: you can redistribute it and/or modify
    it under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 2 of the License, or
    (at your option) any later version.

    Eruption is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the GNU General Public License
    along with Eruption.  If not, see <http://www.gnu.org/licenses/>.

    Copyright (c) 2019-2022, The Eruption Development Team
*/

// SPDX-License-Identifier: GPL-2.0

//! Eruption blackout

use kernel::prelude::*;
use kernel::Result;

pub mod hwdevices;
pub use hwdevices::*;

module! {
    type: EruptionBlackout,
    name: b"eruption_blackout",
    author: b"X3n0m0rph59 <x3n0m0rph59@gmail.com>",
    description: b"Turn off the LEDs of supported devices, and wait for the Eruption daemon to take over from userspace",
    license: b"GPL v2",
}

struct EruptionBlackout {}

impl KernelModule for EruptionBlackout {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("Eruption blackout (init)\n");

        Ok(Self {})
    }
}

impl Drop for EruptionBlackout {
    fn drop(&mut self) {
        pr_info!("Eruption blackout (exit)\n");
    }
}
