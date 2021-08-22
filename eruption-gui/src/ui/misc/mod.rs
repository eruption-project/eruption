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
use gio::prelude::*;
use glib::clone;
use gtk::prelude::*;
use std::time::Duration;

mod hwdevices;

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum MiscError {
    #[error("Communication with the Eruption daemon failed")]
    CommunicationError,
    // #[error("Invalid layout type specified")]
    // InvalidLayout,
}

/// Initialize page "Misc devices"
pub fn initialize_misc_page(
    builder: &gtk::Builder,
    template: &gtk::Builder,
    device: u64,
) -> Result<gtk::Widget> {
    let misc_device = hwdevices::get_misc_devices(device)?;

    let misc_device_page = template.object("misc_device_template").unwrap();

    // let main_window: gtk::ApplicationWindow = builder.object("main_window").unwrap();

    let notification_box_global: gtk::Box = builder.object("notification_box_global").unwrap();

    let misc_name_label: gtk::Label = template.object("misc_device_name_label").unwrap();
    let drawing_area: gtk::DrawingArea = template.object("drawing_area_misc").unwrap();

    crate::dbus_client::ping().unwrap_or_else(|_e| {
        notification_box_global.show_now();
    });

    // device name and status
    let make_and_model = misc_device.get_make_and_model();
    misc_name_label.set_label(&format!("{} {}", make_and_model.0, make_and_model.1));

    // paint drawing area
    drawing_area.connect_draw(move |da: &gtk::DrawingArea, context: &cairo::Context| {
        if let Err(_e) = misc_device.draw(&da, &context) {
            notification_box_global.show();
        } else {
            notification_box_global.hide();
        }

        gtk::Inhibit(false)
    });

    glib::timeout_add_local(
        Duration::from_millis((1000 / constants::TARGET_FPS) / 2),
        clone!(@strong drawing_area => move || {
            drawing_area.queue_draw();
            Continue(true)
        }),
    );

    Ok(misc_device_page)
}
