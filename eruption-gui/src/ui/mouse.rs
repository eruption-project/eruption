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
use glib::clone;
use glib::IsA;
use gtk::prelude::*;
use palette::{Hsva, Shade, Srgba};

use crate::{constants, util::RGBA};

type Result<T> = std::result::Result<T, eyre::Error>;

struct Rectangle {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

/// Initialize page "Mouse"
pub fn initialize_mouse_page(builder: &gtk::Builder) -> Result<()> {
    let drawing_area: gtk::DrawingArea = builder.get_object("drawing_area_mouse").unwrap();

    // drawing area / keyboard indicator
    drawing_area.connect_draw(&draw_mouse);

    glib::timeout_add_local(
        1000 / (constants::TARGET_FPS as u32 * 4),
        clone!(@strong drawing_area => move || {
            drawing_area.queue_draw();
            Continue(true)
        }),
    );

    Ok(())
}

/// Paint a key on the keyboard widget
fn paint_cell(cell_index: usize, color: &RGBA, cr: &cairo::Context) {
    let cell_def = Rectangle {
        x: (cell_index % 6 * 45) as f64,
        y: (cell_index / 6 * 45) as f64,
        width: 42.0,
        height: 42.0,
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

/// Draw an animated mouse with live action colors
pub fn draw_mouse<D: IsA<gtk::DrawingArea>>(_da: &D, context: &cairo::Context) -> gtk::Inhibit {
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
        paint_cell(i - 144, &led_colors[i], &context);
    }

    gtk::Inhibit(false)
}
