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
use glib::clone;
use glib::Continue;
use gtk::prelude::*;
use std::time::Duration;

mod hwdevices;

pub type Result<T> = std::result::Result<T, eyre::Error>;

// #[derive(Debug, thiserror::Error)]
// pub enum MiscError {
//     #[error("Communication with the Eruption daemon failed")]
//     CommunicationError,
//     // #[error("Invalid layout type specified")]
//     // InvalidLayout,
// }

/// Initialize page "Misc devices"
pub fn initialize_misc_page(
    builder: &gtk::Builder,
    template: &gtk::Builder,
    device: u64,
) -> Result<gtk::Widget> {
    let misc_device = hwdevices::get_misc_devices(device)?;

    let misc_device_page = template.object("misc_device_template").unwrap();

    // let main_window: gtk::ApplicationWindow = builder.object("main_window").unwrap();

    let device_brightness_scale: gtk::Scale = template.object("misc_brightness_scale").unwrap();

    let misc_signal_label: gtk::Label = template.object("misc_signal_label").unwrap();
    let signal_strength_progress: gtk::ProgressBar =
        template.object("misc_signal_strength").unwrap();

    let misc_battery_level_label: gtk::Label = template.object("misc_battery_level_label").unwrap();
    let battery_level_progress: gtk::ProgressBar = template.object("misc_battery_level").unwrap();

    let notification_box_global: gtk::Box = builder.object("notification_box_global").unwrap();

    let misc_name_label: gtk::Label = template.object("misc_device_name_label").unwrap();
    let drawing_area: gtk::DrawingArea = template.object("drawing_area_misc").unwrap();

    crate::dbus_client::ping().unwrap_or_else(|_e| {
        notification_box_global.show_now();

        // events::LOST_CONNECTION.store(true, Ordering::SeqCst);
    });

    // device name and status
    let make_and_model = misc_device.get_make_and_model();
    misc_name_label.set_label(&format!("{} {}", make_and_model.0, make_and_model.1));

    let misc_device_handle = misc_device.get_device();

    let device_brightness = util::get_device_brightness(misc_device_handle)?;
    device_brightness_scale.set_value(device_brightness as f64);

    device_brightness_scale.connect_value_changed(move |s| {
        // if !events::shall_ignore_pending_ui_event() {
        util::set_device_brightness(misc_device_handle, s.value() as i64).unwrap();
        // }
    });

    // paint drawing area
    drawing_area.connect_draw(move |da: &gtk::DrawingArea, context: &cairo::Context| {
        if let Err(_e) = misc_device.draw(&da, &context) {
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

    // near realtime update path
    glib::timeout_add_local(
        Duration::from_millis(250),
        clone!(@weak signal_strength_progress, @weak battery_level_progress,
                    @weak misc_signal_label, @weak misc_battery_level_label =>
                    @default-return Continue(true), move || {

            // device status
            if let Ok(device_status) = util::get_device_status(misc_device_handle) {
                if let Some(signal_strength_percent) = device_status.get("signal-strength-percent") {
                    let value = signal_strength_percent.parse::<i32>().unwrap_or(0);

                    signal_strength_progress.set_fraction(value as f64 / 100.0);

                    misc_signal_label.show();
                    signal_strength_progress.show();
                } else {
                    misc_signal_label.hide();
                    signal_strength_progress.hide();
                }

                if let Some(battery_level_percent) = device_status.get("battery-level-percent") {
                    let value = battery_level_percent.parse::<i32>().unwrap_or(0);

                    battery_level_progress.set_fraction(value as f64 / 100.0);

                    misc_battery_level_label.show();
                    battery_level_progress.show();
                } else {
                    misc_battery_level_label.hide();
                    battery_level_progress.hide();
                }
            }

            Continue(true)
        }),
    );

    // // fast update path
    // glib::timeout_add_local(
    //     Duration::from_millis(1000),
    //     clone!(@weak device_brightness_scale => @default-return Continue(true), move || {
    //         if let Ok(device_brightness) = util::get_device_brightness(misc_device_handle) {
    //             device_brightness_scale.set_value(device_brightness as f64);
    //         }

    //         Continue(true)
    //     }),
    // );

    // slow update path
    // glib::timeout_add_local(
    //     Duration::from_millis(2500),
    //     clone!(@weak misc_firmware_label => @default-return Continue(true), move || {
    //         if let Ok(firmware) = util::get_firmware_revision(misc_device_handle) {
    //             misc_firmware_label.set_label(&firmware);
    //         }

    //         Continue(true)
    //     }),
    // );

    glib::timeout_add_local(
        Duration::from_millis(1000 / constants::TARGET_FPS),
        clone!(@weak drawing_area => @default-return Continue(true), move || {
            drawing_area.queue_draw();
            Continue(true)
        }),
    );

    Ok(misc_device_page)
}
