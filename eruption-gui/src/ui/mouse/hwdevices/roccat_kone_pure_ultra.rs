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

use gdk::prelude::GdkContextExt;
use gdk_pixbuf::Pixbuf;
use gtk::WidgetExt;
use palette::{Hsva, Shade, Srgba};

use super::{Mouse, Rectangle};

const BORDER: (f64, f64) = (32.0, 32.0);

// pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug)]
pub struct RoccatKonePureUltra {}

impl RoccatKonePureUltra {
    pub fn new() -> Self {
        RoccatKonePureUltra {}
    }
}

impl Mouse for RoccatKonePureUltra {
    fn get_make_and_model(&self) -> (&'static str, &'static str) {
        ("ROCCAT", "Kone Pure Ultra")
    }

    fn draw_mouse(&self, da: &gtk::DrawingArea, context: &cairo::Context) {
        let width = da.get_allocated_width() as f64;
        // let height = da.get_allocated_height() as f64;

        let led_colors = crate::dbus_client::get_led_colors().unwrap();

        // paint all cells in the "mouse zone" of the canvas
        for i in 144..(144 + 1) {
            self.paint_cell(i - 144, &led_colors[i], &context, width);
        }

        let pixbuf =
            Pixbuf::from_resource("/org/eruption/eruption-gui/img/roccat-kone-pure-ultra.png")
                .unwrap();

        // let scale_factor = (height / pixbuf.get_height() as f64) * 0.75;
        let scale_factor = 0.715;

        // paint the image
        context.scale(scale_factor, scale_factor);
        context.set_source_pixbuf(&pixbuf, width / 2.0 + BORDER.0, BORDER.1);
        context.paint();
    }

    fn paint_cell(
        &self,
        _cell_index: usize,
        color: &crate::util::RGBA,
        cr: &cairo::Context,
        width: f64,
    ) {
        let scale_factor = 0.715;

        let cell_def = Rectangle {
            x: ((width / 2.0) + 95.0 + BORDER.0) * scale_factor,
            y: (BORDER.1 + 470.0) * scale_factor,
            width: 80.0,
            height: 70.0,
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
