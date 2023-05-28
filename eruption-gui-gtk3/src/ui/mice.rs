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

use std::collections::HashMap;
use std::sync::atomic::Ordering;

use glib::clone;
use gtk::glib;
use gtk::prelude::*;

use crate::notifications;
use crate::timers;
use crate::timers::TimerMode;
use crate::util;

use super::hwdevices::mice::get_mouse_device;
use super::hwdevices::DeviceStatus;
use super::Pages;

type Result<T> = std::result::Result<T, eyre::Error>;

// #[derive(Debug, thiserror::Error)]
// pub enum MouseError {
//     #[error("Communication with the Eruption daemon failed")]
//     CommunicationError,
//     // #[error("Invalid layout type specified")]
//     // InvalidLayout,
// }

/// Initialize page "Mouse"
pub fn initialize_mouse_page(
    builder: &gtk::Builder,
    template: &gtk::Builder,
    device: u64,
) -> Result<gtk::Widget> {
    let mouse_device = get_mouse_device(device)?;

    let mouse_device_page = template.object("mouse_device_template").unwrap();

    let notification_box_global: gtk::Box = builder.object("notification_box_global").unwrap();

    let mouse_name_label: gtk::Label = template.object("mouse_device_name_label").unwrap();
    let drawing_area: gtk::DrawingArea = template.object("drawing_area_mouse").unwrap();

    let device_brightness_scale: gtk::Scale = template.object("mouse_brightness_scale").unwrap();

    let mouse_firmware_label: gtk::Label = template.object("mouse_firmware_label").unwrap();
    let mouse_rate_label: gtk::Label = template.object("mouse_rate_label").unwrap();
    let mouse_dpi_label: gtk::Label = template.object("mouse_dpi_label").unwrap();
    let mouse_profile_label: gtk::Label = template.object("mouse_profile_label").unwrap();

    // let signal_strength_indicator: gtk::LevelBar =
    //     template.object("mouse_signal_strength").unwrap();
    // let battery_level_indicator: gtk::LevelBar = template.object("mouse_battery_level").unwrap();

    let debounce_switch: gtk::Switch = template.object("debounce_switch").unwrap();
    let angle_snapping_switch: gtk::Switch = template.object("angle_snapping_switch").unwrap();

    // device name and status
    let make_and_model = mouse_device.get_make_and_model();
    mouse_name_label.set_label(&format!("{} {}", make_and_model.0, make_and_model.1));

    let mouse_device_handle = mouse_device.get_device();

    let device_brightness = util::get_device_brightness(mouse_device_handle)?;
    device_brightness_scale.set_value(device_brightness as f64);

    device_brightness_scale.connect_value_changed(move |s| {
        // if !events::shall_ignore_pending_ui_event() {
        util::set_device_brightness(mouse_device_handle, s.value() as i64).unwrap();
        // }
    });

    debounce_switch.connect_state_set(move |_s, state| {
        // if !events::shall_ignore_pending_ui_event() {
        util::set_debounce(mouse_device_handle, state).unwrap();
        // }

        gtk::Inhibit(false)
    });

    angle_snapping_switch.connect_state_set(move |_s, state| {
        // if !events::shall_ignore_pending_ui_event() {
        util::set_angle_snapping(mouse_device_handle, state).unwrap();
        // }

        gtk::Inhibit(false)
    });

    // drawing area / mouse indicator
    drawing_area.connect_draw(move |da: &gtk::DrawingArea, context: &cairo::Context| {
        if let Err(_e) = mouse_device.draw_mouse(da, context) {
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
    timers::register_timer(
        timers::MOUSE_TIMER_ID + device as usize,
        TimerMode::ActiveStackPage(Pages::Mice as u8),
        151,
        clone!(@strong template => move || {
            let _ = update_levels(&template, device).map_err(|e| tracing::error!("{e}") );

            Ok(())
        }),
    )?;

    // fast update path
    timers::register_timer(
        timers::MOUSE_FAST_TIMER_ID,
        TimerMode::ActiveStackPage(Pages::Mice as u8),
        1051,
        clone!(@weak device_brightness_scale, @weak mouse_dpi_label,
                    @weak mouse_profile_label, @weak debounce_switch,
                    @weak angle_snapping_switch => @default-return Ok(()), move || {

            if let Ok(device_brightness) = util::get_device_brightness(mouse_device_handle) {
                device_brightness_scale.set_value(device_brightness as f64);
            }

            if let Ok(dpi) = util::get_dpi_slot(mouse_device_handle) {
                mouse_dpi_label.set_label(&format!("{dpi}"));
            }

            if let Ok(hardware_profile) = util::get_hardware_profile(mouse_device_handle) {
                mouse_profile_label.set_label(&format!("{hardware_profile}"));
            }

            if let Ok(debounce) = util::get_debounce(mouse_device_handle) {
                debounce_switch.set_active(debounce);
            }

            if let Ok(angle_snapping) = util::get_angle_snapping(mouse_device_handle) {
                angle_snapping_switch.set_active(angle_snapping);
            }

            Ok(())
        }),
    )?;

    // slow update path
    timers::register_timer(
        timers::MOUSE_SLOW_TIMER_ID + device as usize,
        TimerMode::ActiveStackPage(Pages::Mice as u8),
        3023,
        clone!(@weak mouse_firmware_label, @weak mouse_rate_label => @default-return Ok(()), move || {
            if let Ok(firmware) = util::get_firmware_revision(mouse_device_handle) {
                mouse_firmware_label.set_label(&firmware);
            }

            if let Ok(poll_rate) = util::get_poll_rate(mouse_device_handle) {
                mouse_rate_label.set_label(&format!("{poll_rate}"));
            }

            Ok(())
        }),
    )?;

    timers::register_timer(
        timers::MOUSE_RENDER_TIMER_ID + device as usize,
        TimerMode::ActiveStackPage(Pages::Mice as u8),
        1000 / (crate::constants::TARGET_FPS * 2),
        clone!(@weak drawing_area => @default-return Ok(()), move || {
            if crate::ACTIVE_PAGE.load(Ordering::SeqCst) == Pages::Mice as usize {
                drawing_area.queue_draw();
            }

            Ok(())
        }),
    )?;

    Ok(mouse_device_page)
}

pub fn update_levels(template: &gtk::Builder, device: u64) -> Result<()> {
    let mouse_signal_strength_indicator: gtk::LevelBar =
        template.object("mouse_signal_strength").unwrap();
    let mouse_battery_level_indicator: gtk::LevelBar =
        template.object("mouse_battery_level").unwrap();

    let mouse_signal_label: gtk::Label = template.object("mouse_signal_label").unwrap();
    let mouse_battery_level_label: gtk::Label =
        template.object("mouse_battery_level_label").unwrap();

    let mouse_signal_indicator_label: gtk::Label =
        template.object("mouse_signal_indicator_label").unwrap();
    let mouse_battery_indicator_label: gtk::Label =
        template.object("mouse_battery_indicator_label").unwrap();

    let mut errors_present = false;

    // device status
    if let Some(device_status) = crate::DEVICE_STATUS.read().get(&device) {
        if let Some(signal_strength_percent) = device_status.get("signal-strength-percent") {
            let value = signal_strength_percent.parse::<i32>().unwrap_or(0);

            tracing::debug!("{device}: signal: {}", value as f64 / 100.0);

            mouse_signal_label.show();
            mouse_signal_indicator_label.show();

            mouse_signal_indicator_label.set_text(&format!("{value}%"));
            mouse_signal_strength_indicator.set_value(value as f64 / 100.0);

            mouse_signal_strength_indicator.show();
        } else {
            mouse_signal_label.hide();
            mouse_signal_indicator_label.hide();

            mouse_signal_strength_indicator.hide();

            // notifications::warn("Signal strength not currently available");
        }

        if let Some(battery_level_percent) = device_status.get("battery-level-percent") {
            let value = battery_level_percent.parse::<i32>().unwrap_or(0);

            tracing::debug!("{device}: battery: {}", value as f64 / 100.0);

            mouse_battery_level_label.show();
            mouse_battery_indicator_label.show();

            mouse_battery_indicator_label.set_text(&format!("{value}%"));
            mouse_battery_level_indicator.set_value(value as f64 / 100.0);

            mouse_battery_level_indicator.show();
        } else {
            mouse_battery_level_label.hide();
            mouse_battery_indicator_label.hide();

            mouse_battery_level_indicator.hide();

            // notifications::warn("Battery level not currently available");
        }
    } else {
        errors_present = true;
    }

    if errors_present {
        // try harder to get the devices' status
        let status = util::get_device_status(device).unwrap_or_else(|_| HashMap::new());
        crate::DEVICE_STATUS
            .write()
            .insert(device, DeviceStatus(status));

        mouse_signal_label.hide();
        mouse_signal_indicator_label.hide();

        mouse_signal_strength_indicator.hide();

        mouse_battery_level_label.hide();
        mouse_battery_indicator_label.hide();

        mouse_battery_level_indicator.hide();

        notifications::error("Could not query the device status");
    }

    Ok(())
}
