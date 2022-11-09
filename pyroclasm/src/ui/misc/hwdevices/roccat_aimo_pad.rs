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

use super::MiscDevice;
use crate::constants;
use crate::ui::misc::hwdevices::Rectangle;
use palette::{FromColor, Hsva, Shade, Srgba};

// canvas to LED index mapping
const LED_0: usize = constants::CANVAS_SIZE - 36;
const LED_1: usize = constants::CANVAS_SIZE - 1;

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug)]
pub struct RoccatAimoPad {
    pub device: u64,
}

impl RoccatAimoPad {
    pub fn new(device: u64) -> Self {
        RoccatAimoPad { device }
    }
}

impl MiscDevice for RoccatAimoPad {
    fn get_device(&self) -> u64 {
        self.device
    }

    fn get_make_and_model(&self) -> (&'static str, &'static str) {
        ("ROCCAT", "Sense AIMO XXL")
    }

    fn draw(&self) -> super::Result<()> {
        Ok(())
    }
}
