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

use crate::{constants, util};
use gio::prelude::*;
use glib::clone;
use gtk::prelude::*;

mod hwdevices;

type Result<T> = std::result::Result<T, eyre::Error>;

// #[derive(Debug, thiserror::Error)]
// pub enum KeyboardError {
//     #[error("Invalid layout type specified")]
//     InvalidLayout,
// }

/// Initialize page "Keyboard"
pub fn initialize_keyboard_page(builder: &gtk::Builder) -> Result<()> {
    let keyboard_device = hwdevices::get_keyboard_device().unwrap();

    let keyboard_name_label: gtk::Label = builder.get_object("keyboard_device_name_label").unwrap();
    let drawing_area: gtk::DrawingArea = builder.get_object("drawing_area").unwrap();

    let networkfx_ambient_switch: gtk::Switch =
        builder.get_object("networkfx_ambient_switch").unwrap();
    let soundfx_switch: gtk::Switch = builder.get_object("soundfx_switch").unwrap();

    // device name and status
    let make_and_model = keyboard_device.get_make_and_model();
    keyboard_name_label.set_label(&format!("{} {}", make_and_model.0, make_and_model.1));

    // drawing area / keyboard indicator
    drawing_area.connect_draw(move |da: &gtk::DrawingArea, context: &cairo::Context| {
        keyboard_device.draw_keyboard(&da, &context);

        gtk::Inhibit(false)
    });

    glib::timeout_add_local(
        1000 / (constants::TARGET_FPS as u32 * 5),
        clone!(@strong drawing_area => move || {
            drawing_area.queue_draw();
            Continue(true)
        }),
    );

    // special options
    networkfx_ambient_switch.connect_state_set(move |_sw, enabled| {
        util::toggle_netfx_ambient(enabled).unwrap_or_else(|e| {
            let message = "Could not toggle Network FX".to_string();
            let secondary = format!("{}", e);

            let message_dialog = gtk::MessageDialogBuilder::new()
                .destroy_with_parent(true)
                .decorated(true)
                .message_type(gtk::MessageType::Error)
                .text(&message)
                .secondary_text(&secondary)
                .title("Error")
                .buttons(gtk::ButtonsType::Ok)
                .build();

            message_dialog.run();
        });

        gtk::Inhibit(false)
    });

    soundfx_switch.set_state(util::get_sound_fx()?);

    soundfx_switch.connect_state_set(move |_sw, enabled| {
        util::set_sound_fx(enabled).unwrap();

        gtk::Inhibit(false)
    });

    Ok(())
}
