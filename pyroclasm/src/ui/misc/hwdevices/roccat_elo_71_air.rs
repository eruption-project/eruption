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

use super::{MiscDevice, Rectangle};
use crate::constants;
use palette::{FromColor, Hsva, Shade, Srgba};

const BORDER: (f64, f64) = (16.0, 16.0);

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug)]
pub struct RoccatElo71Air {
    pub device: u64,
}

impl RoccatElo71Air {
    pub fn new(device: u64) -> Self {
        RoccatElo71Air { device }
    }
}

impl MiscDevice for RoccatElo71Air {
    fn get_device(&self) -> u64 {
        self.device
    }

    fn get_make_and_model(&self) -> (&'static str, &'static str) {
        ("ROCCAT/Turtle Beach", "Elo 7.1 Air")
    }

    fn draw(&self) -> super::Result<()> {
        Ok(())
    }
}
