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

use crate::constants;
use crate::util;
use gio::prelude::*;
use glib::clone;
use gtk::prelude::*;
use std::time::Duration;

mod hwdevices;

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum KeyboardError {
    #[error("Communication with the Eruption daemon failed")]
    CommunicationError,
    // #[error("Invalid layout type specified")]
    // InvalidLayout,
}

/// Initialize page "Keyboard"
pub fn initialize_keyboard_page(
    builder: &gtk::Builder,
    template: &gtk::Builder,
    device: u64,
) -> Result<gtk::Widget> {
    let keyboard_device = hwdevices::get_keyboard_device(device)?;

    let keyboard_device_page = template.object("keyboard_device_template").unwrap();

    let notification_box_global: gtk::Box = builder.object("notification_box_global").unwrap();

    let keyboard_name_label: gtk::Label = template.object("keyboard_device_name_label").unwrap();
    let drawing_area: gtk::DrawingArea = template.object("drawing_area").unwrap();

    let device_brightness_scale: gtk::Scale = template.object("keyboard_brightness_scale").unwrap();

    crate::dbus_client::ping().unwrap_or_else(|_e| {
        notification_box_global.show_now();

        // events::LOST_CONNECTION.store(true, Ordering::SeqCst);
    });

    // device name and status
    let make_and_model = keyboard_device.get_make_and_model();
    keyboard_name_label.set_label(&format!("{} {}", make_and_model.0, make_and_model.1));

    let keyboard_device_handle = keyboard_device.get_device();

    let device_brightness = util::get_device_brightness(keyboard_device_handle)?;
    device_brightness_scale.set_value(device_brightness as f64);

    device_brightness_scale.connect_value_changed(move |s| {
        // if !events::shall_ignore_pending_ui_event() {
        util::set_device_brightness(keyboard_device_handle, s.value() as i64).unwrap();
        // }
    });

    // drawing area / keyboard indicator
    drawing_area.connect_draw(move |da: &gtk::DrawingArea, context: &cairo::Context| {
        if let Err(_e) = keyboard_device.draw_keyboard(&da, &context) {
            notification_box_global.show();

            // apparently we have lost the connection to the Eruption daemon
            // events::LOST_CONNECTION.store(true, Ordering::SeqCst);
        } else {
            notification_box_global.hide();

            // if events::LOST_CONNECTION.load(Ordering::SeqCst) {
            //     // we re-established the connection to the Eruption daemon,
            //     // update the GUI to show e.g. newly attached devices
            //     events::LOST_CONNECTION.store(false, Ordering::SeqCst);

            //     events::UPDATE_MAIN_WINDOW.store(true, Ordering::SeqCst);
            // }
        }

        gtk::Inhibit(false)
    });

    glib::timeout_add_local(
        Duration::from_millis(1000 / constants::TARGET_FPS),
        clone!(@weak drawing_area => @default-return Continue(true), move || {
            drawing_area.queue_draw();
            Continue(true)
        }),
    );

    Ok(keyboard_device_page)
}
