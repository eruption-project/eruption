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

use super::{Backend, BackendData};

type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, Clone)]
pub struct WaylandBackend {
    pub failed: bool,
}

impl WaylandBackend {
    pub fn new() -> Result<Self> {
        Ok(Self { failed: true })
    }
}

impl Backend for WaylandBackend {
    fn initialize(&mut self) -> Result<()> {
        // if we made it up to here, the initialization succeeded
        self.failed = false;

        Ok(())
    }

    fn get_id(&self) -> String {
        "wayland".to_string()
    }

    fn get_name(&self) -> String {
        "Wayland".to_string()
    }

    fn get_description(&self) -> String {
        "Capture the screen's content from a Wayland compositor".to_string()
    }

    fn is_failed(&self) -> bool {
        self.failed
    }

    fn set_failed(&mut self, failed: bool) {
        self.failed = failed;
    }

    fn poll(&mut self) -> Result<BackendData> {
        Ok("".to_string())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}
