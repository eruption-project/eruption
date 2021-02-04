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
*/

use crate::util::RGBA;

mod generic_mouse;

pub struct Rectangle {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

pub trait Mouse {
    /// Draw an animated mouse with live action colors
    fn draw_mouse(&self, _da: &gtk::DrawingArea, context: &cairo::Context);

    /// Paint a cell on the Mouse widget
    fn paint_cell(&self, cell_index: usize, color: &RGBA, cr: &cairo::Context);
}

pub fn get_mouse_device() -> Box<dyn Mouse> {
    // TODO: Implement this
    Box::new(generic_mouse::GenericMouse::new())
}
