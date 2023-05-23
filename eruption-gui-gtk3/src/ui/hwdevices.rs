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

use std::collections::HashMap;

pub mod keyboards;
pub mod mice;
pub mod misc;

/// Generic Device status information, like e.g.: 'signal strength' or 'battery level'
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DeviceStatus(pub HashMap<String, String>);

impl std::ops::Deref for DeviceStatus {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for DeviceStatus {
    fn default() -> Self {
        let map = HashMap::new();

        // fill in default values
        // map.insert("connected".to_owned(), format!("{}", true));

        Self(map)
    }
}
