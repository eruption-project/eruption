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

    Copyright (c) 2019-2022, The Eruption Development Team
*/

use glib::clone;
use gtk::prelude::*;
// use palette::{FromColor, Hsva, Srgba};

use crate::{constants, dbus_client, timers};
use crate::{events, util};

use super::keyboards;
use super::mice;
use super::misc;

const BORDER: (f64, f64) = (8.0, 8.0);
const PIXEL_SIZE: usize = 4;

type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum CanvasError {
    // #[error("Unknown error")]
    // UnknownError,
}

struct Rectangle {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

/// Initialize page "Canvas"
pub fn initialize_canvas_page(builder: &gtk::Builder) -> Result<()> {
    // let application = application.as_ref();

    // let main_window: gtk::ApplicationWindow = builder.object("main_window").unwrap();

    let notification_box_global: gtk::Box = builder.object("notification_box_global").unwrap();

    let drawing_area: gtk::DrawingArea = builder.object("drawing_area_canvas").unwrap();

    let reset_postproc_button: gtk::Button = builder.object("reset_postproc_button").unwrap();

    let canvas_hue_scale: gtk::Scale = builder.object("canvas_hue_scale").unwrap();
    let canvas_saturation_scale: gtk::Scale = builder.object("canvas_saturation_scale").unwrap();
    let canvas_lightness_scale: gtk::Scale = builder.object("canvas_lightness_scale").unwrap();

    let devices_tree_view: gtk::TreeView = builder.object("devices_tree_view").unwrap();

    crate::dbus_client::ping().unwrap_or_else(|_e| {
        notification_box_global.show_now();

        // events::LOST_CONNECTION.store(true, Ordering::SeqCst);
    });

    reset_postproc_button.connect_clicked(
        clone!(@weak canvas_hue_scale, @weak canvas_saturation_scale, @weak canvas_lightness_scale  => move |_btn| {
            canvas_hue_scale.adjustment().set_value(0.0);
            canvas_saturation_scale.adjustment().set_value(0.0);
            canvas_lightness_scale.adjustment().set_value(0.0);
        }),
    );

    canvas_hue_scale.set_value(util::get_canvas_hue()?);
    canvas_saturation_scale.set_value(util::get_canvas_saturation()?);
    canvas_lightness_scale.set_value(util::get_canvas_lightness()?);

    canvas_hue_scale.connect_value_changed(move |s| {
        if !events::shall_ignore_pending_ui_event() {
            util::set_canvas_hue(s.value()).unwrap();
        }
    });

    canvas_saturation_scale.connect_value_changed(move |s| {
        if !events::shall_ignore_pending_ui_event() {
            util::set_canvas_saturation(s.value()).unwrap();
        }
    });

    canvas_lightness_scale.connect_value_changed(move |s| {
        if !events::shall_ignore_pending_ui_event() {
            util::set_canvas_lightness(s.value()).unwrap();
        }
    });

    // device selection
    devices_tree_view
        .selection()
        .connect_changed(move |_sel| {});

    // devices tree
    let devices_treestore = gtk::TreeStore::new(&[
        glib::Type::U64,
        // String::static_type(),
        String::static_type(),
        String::static_type(),
    ]);

    let devices = dbus_client::get_managed_devices()?;

    let mut index = 0_u32;

    // add keyboard devices
    for _device_ids in devices.0 {
        let device = keyboards::hwdevices::get_keyboard_device(index as u64)?;

        let make = device.get_make_and_model().0;
        let model = device.get_make_and_model().1;

        devices_treestore.insert_with_values(
            None,
            None,
            &[(0, &(index as u64)), (1, &(make)), (2, &(model))],
        );

        index += 1;
    }

    // add mouse devices
    for _device_ids in devices.1 {
        let device = mice::hwdevices::get_mouse_device(index as u64)?;

        let make = device.get_make_and_model().0;
        let model = device.get_make_and_model().1;

        devices_treestore.insert_with_values(
            None,
            None,
            &[(0, &(index as u64)), (1, &(make)), (2, &(model))],
        );

        index += 1;
    }

    // add misc devices
    for _device_ids in devices.2 {
        let device = misc::hwdevices::get_misc_device(index as u64)?;

        let make = device.get_make_and_model().0;
        let model = device.get_make_and_model().1;

        devices_treestore.insert_with_values(
            None,
            None,
            &[(0, &(index as u64)), (1, &(make)), (2, &(model))],
        );

        index += 1;
    }

    let column = gtk::TreeViewColumn::new();
    let cell = gtk::CellRendererText::new();

    gtk::prelude::CellLayoutExt::pack_start(&column, &cell, false);
    gtk::prelude::TreeViewColumnExt::add_attribute(&column, &cell, "text", 0);

    devices_tree_view.append_column(&column);

    let column = gtk::TreeViewColumn::new();
    let cell = gtk::CellRendererText::new();

    gtk::prelude::CellLayoutExt::pack_start(&column, &cell, true);
    gtk::prelude::TreeViewColumnExt::add_attribute(&column, &cell, "text", 1);

    devices_tree_view.append_column(&column);

    let column = gtk::TreeViewColumn::new();
    let cell = gtk::CellRendererText::new();

    gtk::prelude::CellLayoutExt::pack_start(&column, &cell, false);
    gtk::prelude::TreeViewColumnExt::add_attribute(&column, &cell, "text", 2);

    devices_tree_view.append_column(&column);

    devices_tree_view.set_model(Some(&devices_treestore));
    devices_tree_view.show_all();

    // drawing area
    drawing_area.connect_draw(move |da: &gtk::DrawingArea, context: &cairo::Context| {
        let hue = canvas_hue_scale.value();
        let saturation = canvas_saturation_scale.value();
        let lightness = canvas_lightness_scale.value();

        if let Err(_e) = render_canvas(da, (hue, saturation, lightness), context) {
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

    timers::register_timer(
        timers::CANVAS_RENDER_TIMER_ID,
        1000 / constants::TARGET_FPS,
        clone!(@weak drawing_area => @default-return Ok(()), move || {
            drawing_area.queue_draw();

            Ok(())
        }),
    )?;

    Ok(())
}

/// Update page "Canvas", e.g. after a hotplug event
pub fn update_canvas_page(builder: &gtk::Builder) -> Result<()> {
    // let application = application.as_ref();

    // let main_window: gtk::ApplicationWindow = builder.object("main_window").unwrap();

    let notification_box_global: gtk::Box = builder.object("notification_box_global").unwrap();

    let devices_tree_view: gtk::TreeView = builder.object("devices_tree_view").unwrap();

    crate::dbus_client::ping().unwrap_or_else(|_e| {
        notification_box_global.show_now();

        // events::LOST_CONNECTION.store(true, Ordering::SeqCst);
    });

    // devices tree
    let devices_treestore = gtk::TreeStore::new(&[
        glib::Type::U64,
        // String::static_type(),
        String::static_type(),
        String::static_type(),
    ]);

    let devices = dbus_client::get_managed_devices()?;

    let mut index = 0_u32;

    // add keyboard devices
    for _device_ids in devices.0 {
        let device = keyboards::hwdevices::get_keyboard_device(index as u64)?;

        let make = device.get_make_and_model().0;
        let model = device.get_make_and_model().1;

        devices_treestore.insert_with_values(
            None,
            None,
            &[(0, &(index as u64)), (1, &(make)), (2, &(model))],
        );

        index += 1;
    }

    // add mouse devices
    for _device_ids in devices.1 {
        let device = mice::hwdevices::get_mouse_device(index as u64)?;

        let make = device.get_make_and_model().0;
        let model = device.get_make_and_model().1;

        devices_treestore.insert_with_values(
            None,
            None,
            &[(0, &(index as u64)), (1, &(make)), (2, &(model))],
        );

        index += 1;
    }

    // add misc devices
    for _device_ids in devices.2 {
        let device = misc::hwdevices::get_misc_device(index as u64)?;

        let make = device.get_make_and_model().0;
        let model = device.get_make_and_model().1;

        devices_treestore.insert_with_values(
            None,
            None,
            &[(0, &(index as u64)), (1, &(make)), (2, &(model))],
        );

        index += 1;
    }

    devices_tree_view.set_model(Some(&devices_treestore));
    devices_tree_view.show_all();

    Ok(())
}

fn render_canvas(
    da: &gtk::DrawingArea,
    hsl: (f64, f64, f64),
    context: &cairo::Context,
) -> Result<()> {
    let width = da.allocated_width() as f64;
    let height = da.allocated_height() as f64;

    let scale_factor = height / (constants::CANVAS_HEIGHT as f64 * 4.15);

    let led_colors = crate::COLOR_MAP.lock();

    // paint all cells of the canvas
    for (i, color) in led_colors.iter().enumerate() {
        paint_cell(i, color, hsl, context, width, height, scale_factor)?;
    }

    Ok(())
}

fn paint_cell(
    cell_index: usize,
    color: &crate::util::RGBA,
    _hsl: (f64, f64, f64),
    cr: &cairo::Context,
    _width: f64,
    _height: f64,
    scale_factor: f64,
) -> Result<()> {
    let xval = (cell_index % constants::CANVAS_WIDTH * PIXEL_SIZE) as f64;
    let yval = (cell_index / constants::CANVAS_WIDTH * PIXEL_SIZE) as f64;

    let cell_def = Rectangle {
        x: BORDER.0 + xval * scale_factor,
        y: BORDER.1 + yval * scale_factor,
        width: PIXEL_SIZE as f64 * scale_factor,
        height: PIXEL_SIZE as f64 * scale_factor,
    };

    // post-process color
    // let color = Srgba::new(
    //     color.r as f64 / 255.0,
    //     color.g as f64 / 255.0,
    //     color.b as f64 / 255.0,
    //     color.a as f64 / 255.0,
    // );

    // let hue_value = hsl.0;
    // let saturation_value = hsl.1 / 100.0;
    // let lighten_value = hsl.2 / 100.0;

    // image post processing
    // let color = Hsva::from_color(color);
    // let color = Srgba::from_color(
    //     color  .shift_hue(hue_value)
    //            .saturate(saturation_value)
    //            .lighten(lighten_value),
    // )
    // .into_components();

    // cr.set_source_rgba(color.0, color.1, color.2, 1.0 - color.3);

    cr.set_source_rgba(
        color.r as f64 / 255.0,
        color.g as f64 / 255.0,
        color.b as f64 / 255.0,
        1.0, /* -  color.a as f64 / 255.0 */
    );
    cr.rectangle(cell_def.x, cell_def.y, cell_def.width, cell_def.height);
    cr.fill()?;

    Ok(())
}
