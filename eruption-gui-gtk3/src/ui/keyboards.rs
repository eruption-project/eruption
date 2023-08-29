/*  SPDX-License-Identifier: GPL-3.0-or-later  */

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

    Copyright (c) 2019-2023, The Eruption Development Team
*/

use std::sync::atomic::Ordering;

use crate::timers;
use crate::timers::TimerMode;
use crate::util;
use glib_macros::clone;
use gtk::glib;
use gtk::prelude::{BuilderExtManual, LabelExt, RangeExt, WidgetExt};

use super::hwdevices::keyboards::get_keyboard_device;
use super::Pages;

pub type Result<T> = std::result::Result<T, eyre::Error>;

// #[derive(Debug, thiserror::Error)]
// pub enum KeyboardError {
//     #[error("Communication with the Eruption daemon failed")]
//     CommunicationError,
//     // #[error("Invalid layout type specified")]
//     // InvalidLayout,
// }

/// Initialize page "Keyboard"
pub fn initialize_keyboard_page(
    builder: &gtk::Builder,
    template: &gtk::Builder,
    device: u64,
) -> Result<gtk::Widget> {
    let keyboard_device = get_keyboard_device(device)?;

    let keyboard_device_page = template.object("keyboard_device_template").unwrap();

    let notification_box_global: gtk::Box = builder.object("notification_box_global").unwrap();

    let keyboard_name_label: gtk::Label = template.object("keyboard_device_name_label").unwrap();
    let drawing_area: gtk::DrawingArea = template.object("drawing_area").unwrap();

    let device_brightness_scale: gtk::Scale = template.object("keyboard_brightness_scale").unwrap();

    // device name and status
    let make_and_model = keyboard_device.get_make_and_model();
    keyboard_name_label.set_label(&format!("{} {}", make_and_model.0, make_and_model.1));

    // let signal_strength_indicator: gtk::LevelBar =
    //     template.object("keyboard_signal_strength").unwrap();
    // let battery_level_indicator: gtk::LevelBar = template.object("keyboard_battery_level").unwrap();

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
        if let Err(_e) = keyboard_device.draw_keyboard(da, context) {
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

        false.into()
    });

    // // near realtime update path
    // timers::register_timer(
    //     timers::KEYBOARD_TIMER_ID + device as usize,
    //     TimerMode::ActiveStackPage(Pages::Keyboards as u8),
    //     139,
    //     clone!(@weak signal_strength_indicator, @weak battery_level_indicator =>
    //                 @default-return Ok(()), move || {

    //         // device status
    //         if let Ok(device_status) = util::get_device_status(keyboard_device_handle) {
    //             if let Some(signal_strength_percent) = device_status.get("signal-strength-percent") {
    //                 let value = signal_strength_percent.parse::<i32>().unwrap_or(0);

    //                 signal_strength_indicator.set_value(value as f64 / 100.0);
    //                 signal_strength_indicator.show();
    //             } else {
    //                 signal_strength_indicator.hide();
    //             }

    //             if let Some(battery_level_percent) = device_status.get("battery-level-percent") {
    //                 let value = battery_level_percent.parse::<i32>().unwrap_or(0);

    //                 battery_level_indicator.set_value(value as f64 / 100.0);
    //                 battery_level_indicator.show();
    //             } else {
    //                 battery_level_indicator.hide();
    //             }
    //         } else {
    //             signal_strength_indicator.hide();
    //             battery_level_indicator.hide();
    //         }

    //         Ok(())
    //     }),
    // )?;

    timers::register_timer(
        timers::KEYBOARD_RENDER_TIMER_ID + device as usize,
        TimerMode::ActiveStackPage(Pages::Keyboards as u8),
        1000 / (crate::constants::TARGET_FPS * 2),
        clone!(@weak drawing_area => @default-return Ok(()), move || {
                if crate::ACTIVE_PAGE.load(Ordering::SeqCst) == Pages::Keyboards as usize {
                            drawing_area.queue_draw();
                }

                Ok(())
            }
        ),
    )?;

    Ok(keyboard_device_page)
}
