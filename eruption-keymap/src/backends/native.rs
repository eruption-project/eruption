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

    Copyright (c) 2019-2022, The Eruption Development Team
*/

use std::{fs, path::Path};

use crate::mapping::KeyMappingTable;

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug)]
pub struct NativeBackend {}

impl NativeBackend {
    pub fn new() -> Self {
        Self {}
    }

    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<KeyMappingTable> {
        let path = path.as_ref();

        let data = fs::read_to_string(path)?;
        let mappings = serde_json::from_str(&data)?;

        Ok(mappings)
    }
}

impl super::Backend for NativeBackend {
    fn generate(&self, table: &KeyMappingTable) -> Result<String> {
        let result = serde_json::to_string(&table)?;

        Ok(result)
    }

    fn write_to_file<P: AsRef<Path>>(&self, path: P, table: &KeyMappingTable) -> Result<()> {
        let path = path.as_ref();

        let data = serde_json::to_string(&table)?;
        fs::write(path, data)?;

        Ok(())
    }
}
