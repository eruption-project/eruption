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

use super::Mouse;

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug)]
pub struct NullMouse {}

impl NullMouse {
    pub fn new() -> Self {
        NullMouse {}
    }
}

impl Mouse for NullMouse {
    fn get_device(&self) -> u64 {
        0
    }

    fn get_make_and_model(&self) -> (&'static str, &'static str) {
        ("Unknown", "Unknown")
    }

    fn draw_mouse(&self, _da: &gtk::DrawingArea, _context: &cairo::Context) -> super::Result<()> {
        Ok(())
    }

    fn paint_cell(
        &self,
        _cell_index: usize,
        _color: &crate::util::RGBA,
        _cr: &cairo::Context,
        _width: f64,
        _height: f64,
        _scale_factor: f64,
    ) -> Result<()> {
        Ok(())
    }
}
