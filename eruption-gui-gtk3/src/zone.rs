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

use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Zone {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub enabled: bool,
}

#[allow(unused)]
impl Zone {
    #[inline]
    pub fn new(x: i32, y: i32, width: i32, height: i32, enabled: bool) -> Self {
        Self {
            x,
            y,
            width,
            height,
            enabled,
        }
    }

    #[inline]
    pub fn empty() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            enabled: false,
        }
    }

    #[inline]
    pub fn cell_count(&self) -> usize {
        (self.width * self.height).unsigned_abs() as usize
    }

    #[inline]
    pub fn x2(&self) -> i32 {
        self.x + self.width
    }

    #[inline]
    pub fn y2(&self) -> i32 {
        self.y + self.height
    }
}

impl Default for Zone {
    fn default() -> Self {
        Self::empty()
    }
}

impl Display for Zone {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}:{}x{}", self.x, self.y, self.width, self.height)
    }
}
