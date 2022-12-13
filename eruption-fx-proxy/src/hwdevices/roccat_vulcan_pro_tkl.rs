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

use super::Keyboard;

#[derive(Debug, Clone)]
pub struct RoccatVulcanProTKL {}

impl RoccatVulcanProTKL {
    pub fn new() -> Self {
        Self {}
    }
}

impl Keyboard for RoccatVulcanProTKL {
    fn get_num_keys(&self) -> usize {
        NUM_KEYS
    }

    fn get_num_rows(&self) -> usize {
        NUM_ROWS
    }

    fn get_num_cols(&self) -> usize {
        NUM_COLS
    }

    fn get_rows_topology(&self) -> &'static [u8] {
        &ROWS_TOPOLOGY
    }
}

pub const NUM_KEYS: usize = 96;
pub const NUM_ROWS: usize = 6;
pub const NUM_COLS: usize = 16;

// const ARRAY_OFFSET: usize = 102;

#[rustfmt::skip]
pub const ROWS_TOPOLOGY: [u8; 102] = [

    // ISO model
    0x02, 0x0d, 0x14, 0x19, 0x1e, 0x28, 0x2f, 0x35, 0x3b, 0x41, 0x47, 0x4d, 0x4f, 0x5c, 0xff, 0xff, 0xff,
	0x03, 0x08, 0x0e, 0x15, 0x1a, 0x1f, 0x24, 0x29, 0x30, 0x36, 0x3c, 0x42, 0x48, 0x50, 0x54, 0x58, 0x5d,
	0x04, 0x09, 0x0f, 0x16, 0x1b, 0x20, 0x25, 0x2a, 0x31, 0x37, 0x3d, 0x43, 0x49, 0x52, 0x55, 0x59, 0x5e,
	0x05, 0x0a, 0x10, 0x17, 0x1c, 0x21, 0x26, 0x2b, 0x32, 0x38, 0x3e, 0x44, 0x4a, 0xff, 0xff, 0xff, 0xff,
	0x00, 0x06, 0x0b, 0x11, 0x18, 0x1d, 0x22, 0x27, 0x2c, 0x33, 0x39, 0x3f, 0x4b, 0xff, 0x5a, 0xff, 0xff,
    0x01, 0x07, 0x0c, 0x23, 0x3a, 0x40, 0x46, 0x4c, 0x56, 0x5b, 0x5f, 0x40, 0xff, 0xff, 0xff, 0xff, 0xff,

    // ANSI model
    // TODO: Implement this

    ];
