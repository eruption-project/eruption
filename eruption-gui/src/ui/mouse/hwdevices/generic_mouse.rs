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

use palette::{Hsva, Shade, Srgba};

use super::{Mouse, Rectangle};

#[derive(Debug)]
pub struct GenericMouse {}

impl GenericMouse {
    pub fn new() -> Self {
        GenericMouse {}
    }
}

impl Mouse for GenericMouse {
    fn draw_mouse(&self, _da: &gtk::DrawingArea, context: &cairo::Context) {
        // let da = da.as_ref();

        // let width = da.get_allocated_width();
        // let height = da.get_allocated_height();

        // let scale_factor = 1.5;

        // let pixbuf = Pixbuf::from_resource("/org/eruption/eruption-gui/img/mouse.png").unwrap();

        // paint the schematic drawing
        // context.scale(scale_factor, scale_factor);
        // context.set_source_pixbuf(&pixbuf, 0.0, 0.0);
        // context.paint();

        let led_colors = crate::dbus_client::get_led_colors().unwrap();

        // paint all cells in the "mouse zone" of the canvas
        for i in 144..(144 + 36) {
            self.paint_cell(i - 144, &led_colors[i], &context);
        }
    }

    fn paint_cell(&self, cell_index: usize, color: &crate::util::RGBA, cr: &cairo::Context) {
        let cell_def = Rectangle {
            x: (cell_index % 6 * 45) as f64,
            y: (cell_index / 6 * 45) as f64,
            width: 43.0,
            height: 43.0,
        };

        // compute scaling factor
        let factor = ((100.0 - crate::STATE.read().current_brightness.unwrap_or_else(|| 0) as f64)
            / 100.0)
            * 0.15;

        // post-process color
        let color = Srgba::new(
            color.r as f64 / 255.0,
            color.g as f64 / 255.0,
            color.b as f64 / 255.0,
            color.a as f64 / 255.0,
        );

        // saturate and lighten color somewhat
        let color = Hsva::from(color);
        let color = Srgba::from(
            color
                // .saturate(factor)
                .lighten(factor),
        )
        .into_components();

        cr.set_source_rgba(color.0, color.1, color.2, 1.0 - color.3);
        cr.rectangle(cell_def.x, cell_def.y, cell_def.width, cell_def.height);
        cr.fill();
    }
}
