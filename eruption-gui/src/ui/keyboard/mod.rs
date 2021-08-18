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
pub fn initialize_keyboard_page(builder: &gtk::Builder) -> Result<()> {
    let keyboard_device = hwdevices::get_keyboard_device()?;

    let main_window: gtk::ApplicationWindow = builder.object("main_window").unwrap();

    let notification_box_global: gtk::Box = builder.object("notification_box_global").unwrap();

    let keyboard_name_label: gtk::Label = builder.object("keyboard_device_name_label").unwrap();
    let drawing_area: gtk::DrawingArea = builder.object("drawing_area").unwrap();

    let device_brightness_scale: gtk::Scale = builder.object("keyboard_brightness_scale").unwrap();

    let networkfx_ambient_switch: gtk::Switch = builder.object("networkfx_ambient_switch").unwrap();
    let soundfx_switch: gtk::Switch = builder.object("soundfx_switch").unwrap();

    crate::dbus_client::ping().unwrap_or_else(|_e| {
        notification_box_global.show_now();
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

    // special options
    networkfx_ambient_switch.connect_state_set(move |_sw, enabled| {
        if enabled {
            crate::STATE.write().saved_profile = util::get_active_profile().ok();

            util::toggle_netfx_ambient(true).unwrap_or_else(|e| {
                let message = "Could not toggle Network FX".to_string();
                let secondary = format!("{}", e);

                let message_dialog = gtk::MessageDialogBuilder::new()
                    .parent(&main_window)
                    .destroy_with_parent(true)
                    .decorated(true)
                    .message_type(gtk::MessageType::Error)
                    .text(&message)
                    .secondary_text(&secondary)
                    .title("Error")
                    .buttons(gtk::ButtonsType::Ok)
                    .build();

                message_dialog.run();
                message_dialog.hide();
            });
        } else {
            util::toggle_netfx_ambient(false).unwrap_or_else(|e| {
                let message = "Could not toggle Network FX".to_string();
                let secondary = format!("{}", e);

                let message_dialog = gtk::MessageDialogBuilder::new()
                    .parent(&main_window)
                    .destroy_with_parent(true)
                    .decorated(true)
                    .message_type(gtk::MessageType::Error)
                    .text(&message)
                    .secondary_text(&secondary)
                    .title("Error")
                    .buttons(gtk::ButtonsType::Ok)
                    .build();

                message_dialog.run();
                message_dialog.hide();
            });

            if let Some(saved_profile) = &crate::STATE.read().saved_profile {
                let _result = util::switch_profile(&saved_profile);
            }
        }

        gtk::Inhibit(false)
    });

    soundfx_switch.set_state(util::get_sound_fx().unwrap_or(false));

    soundfx_switch.connect_state_set(move |_sw, enabled| {
        util::set_sound_fx(enabled).unwrap();

        gtk::Inhibit(false)
    });

    Ok(())
}
