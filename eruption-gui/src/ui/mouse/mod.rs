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
use gtk::prelude::*;

use crate::constants;

mod hwdevices;

type Result<T> = std::result::Result<T, eyre::Error>;

/// Initialize page "Mouse"
pub fn initialize_mouse_page(builder: &gtk::Builder) -> Result<()> {
    let mouse_device = hwdevices::get_mouse_device();
    let drawing_area: gtk::DrawingArea = builder.get_object("drawing_area_mouse").unwrap();

    // drawing area / mouse indicator
    drawing_area.connect_draw(move |da: &gtk::DrawingArea, context: &cairo::Context| {
        mouse_device.draw_mouse(&da, &context);

        gtk::Inhibit(false)
    });

    glib::timeout_add_local(
        1000 / (constants::TARGET_FPS as u32 * 5),
        clone!(@strong drawing_area => move || {
            drawing_area.queue_draw();
            Continue(true)
        }),
    );

    Ok(())
}
