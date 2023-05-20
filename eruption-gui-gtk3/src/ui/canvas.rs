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

use std::cell::RefCell;
use std::sync::atomic::Ordering;

use glib::clone;
use gtk::{
    prelude::*, Builder, ButtonsType, MessageDialog, MessageType, ResponseType, ScrolledWindow,
    ShadowType, Stack, StackSwitcher, TreeViewColumnSizing,
};
// use palette::{FromColor, Hsva, LinSrgba};

use crate::dbus_client::Zone;
use crate::timers::TimerMode;
use crate::{constants, dbus_client, notifications, timers, ConnectionState};
use crate::{events, util};

use super::hwdevices::keyboards::get_keyboard_device;
use super::hwdevices::mice::get_mouse_device;
use super::hwdevices::misc::get_misc_device;
use super::main_window::set_application_state;
use super::Pages;

const BORDER: (f64, f64) = (8.0, 8.0);

type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum CanvasError {
    // #[error("Unknown error")]
    // UnknownError,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum RenderMode {
    Preview,
    Zones,
}

#[derive(Debug, Clone)]
struct Rectangle {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
}

thread_local! {
    // Pango font description, used to render the captions on the visual representation of keyboard
    static FONT_DESC: RefCell<pango::FontDescription> = RefCell::new(pango::FontDescription::from_string("Roboto demibold 22"));
}

/// Initialize page "Canvas"
pub fn initialize_canvas_page(builder: &gtk::Builder) -> Result<()> {
    // let application = application.as_ref();

    // let main_window: gtk::ApplicationWindow = builder.object("main_window").unwrap();

    let notification_box_global: gtk::Box = builder.object("notification_box_global").unwrap();

    let canvas_stack: Stack = builder.object("canvas_stack").unwrap();
    let canvas_switcher: StackSwitcher = builder.object("canvas_switcher").unwrap();

    let drawing_area_preview: gtk::DrawingArea =
        builder.object("drawing_area_canvas_preview").unwrap();
    let drawing_area_zones: gtk::DrawingArea = builder.object("drawing_area_canvas_zones").unwrap();

    let reset_postproc_button: gtk::Button = builder.object("reset_postproc_button").unwrap();

    let canvas_hue_scale: gtk::Scale = builder.object("canvas_hue_scale").unwrap();
    let canvas_saturation_scale: gtk::Scale = builder.object("canvas_saturation_scale").unwrap();
    let canvas_lightness_scale: gtk::Scale = builder.object("canvas_lightness_scale").unwrap();

    let devices_treeview: gtk::TreeView = builder.object("devices_tree_view").unwrap();

    if let Err(e) = crate::dbus_client::ping() {
        tracing::error!("Lost connection to the Eruption daemon: {e}");
        set_application_state(ConnectionState::Disconnected, builder)?;
    } else {
        tracing::info!("Connected to the Eruption daemon");
        set_application_state(ConnectionState::Connected, builder)?;
    };

    reset_postproc_button.connect_clicked(
        clone!(@weak canvas_hue_scale, @weak canvas_saturation_scale, @weak canvas_lightness_scale => move |_btn| {
            let message =
            "Reset all image post-processing filters to their respective default values?".to_string();

            let message_dialog = MessageDialog::builder()
            .destroy_with_parent(true)
            .modal(true)
            .message_type(MessageType::Question)
            .icon_name("dialog-question")
            // .title("Reset image post-processing filters")
            .text(message)
            .secondary_text("Hue, saturation and lightness will be reset to their respective default values")
            .buttons(ButtonsType::YesNo)
            .build();

            let result = message_dialog.run();
            message_dialog.hide();

            if result  == ResponseType::Yes {
                canvas_hue_scale.adjustment().set_value(0.0);
                canvas_saturation_scale.adjustment().set_value(0.0);
                canvas_lightness_scale.adjustment().set_value(0.0);
            }
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
    devices_treeview.selection().connect_changed(
        clone!(@weak canvas_stack, @weak canvas_switcher  => move |_sel| {
            canvas_stack.set_visible_child_name("page1");
        }),
    );

    // devices tree
    let devices_treestore = gtk::TreeStore::new(&[
        glib::Type::BOOL,
        glib::Type::U64,
        String::static_type(),
        String::static_type(),
        String::static_type(),
    ]);

    let devices = dbus_client::get_managed_devices()?;

    let mut index = 0_u32;

    // add keyboard devices
    for _device_ids in devices.0 {
        let device = get_keyboard_device(index as u64)?;

        let make = device.get_make_and_model().0;
        let model = device.get_make_and_model().1;

        let device_type = String::from("Keyboard");

        let enabled = dbus_client::is_device_enabled(index as u64)?;

        devices_treestore.insert_with_values(
            None,
            None,
            &[
                (0, &(enabled)),
                (1, &(index as u64)),
                (2, &(device_type)),
                (3, &(make)),
                (4, &(model)),
            ],
        );

        // populate_canvas_stack_widget_for_device(builder, &format!("Zone {index}"))?;

        index += 1;
    }

    // add mouse devices
    for _device_ids in devices.1 {
        let device = get_mouse_device(index as u64)?;

        let make = device.get_make_and_model().0;
        let model = device.get_make_and_model().1;

        let device_type = String::from("Mouse");

        let enabled = dbus_client::is_device_enabled(index as u64)?;

        devices_treestore.insert_with_values(
            None,
            None,
            &[
                (0, &(enabled)),
                (1, &(index as u64)),
                (2, &(device_type)),
                (3, &(make)),
                (4, &(model)),
            ],
        );

        // populate_canvas_stack_widget_for_device(builder, &format!("Zone {index}"))?;

        index += 1;
    }

    // add misc devices
    for _device_ids in devices.2 {
        let device = get_misc_device(index as u64)?;

        let make = device.get_make_and_model().0;
        let model = device.get_make_and_model().1;

        let device_type = String::from("Misc");

        let enabled = dbus_client::is_device_enabled(index as u64)?;

        devices_treestore.insert_with_values(
            None,
            None,
            &[
                (0, &(enabled)),
                (1, &(index as u64)),
                (2, &(device_type)),
                (3, &(make)),
                (4, &(model)),
            ],
        );

        // populate_canvas_stack_widget_for_device(builder, &format!("Zone {index}"))?;

        index += 1;
    }

    let column = gtk::TreeViewColumn::builder()
        .sizing(TreeViewColumnSizing::Autosize)
        .alignment(0.5)
        // .visible(false)
        .build();

    let cell = gtk::CellRendererToggle::builder().build();

    cell.connect_toggled(clone!(@weak devices_treeview => move |_f, p| {
        let devices_treestore: gtk::TreeStore = devices_treeview
            .model()
            .unwrap()
            .downcast::<gtk::TreeStore>()
            .unwrap();

        let device_index = devices_treestore
            .value(&devices_treestore.iter(&p).unwrap(), 1)
            .get::<u64>()
            .unwrap();

        let value = devices_treestore
            .value(&devices_treestore.iter(&p).unwrap(), 0)
            .get::<bool>()
            .unwrap();
        devices_treestore.set_value(
            &devices_treestore.iter(&p).unwrap(),
            0,
            &(!value).to_value(),
        );

        dbus_client::set_device_enabled(device_index, !value).unwrap_or_else(|e| {
            tracing::error!("Could not set device status: {e}");
            notifications::error(&format!("Could not set device status: {e}"));
        });
    }));

    gtk::prelude::CellLayoutExt::pack_start(&column, &cell, false);
    gtk::prelude::TreeViewColumnExt::add_attribute(&column, &cell, "active", 0);

    devices_treeview.append_column(&column);

    let column = gtk::TreeViewColumn::builder()
        .title("ID")
        .sizing(TreeViewColumnSizing::Autosize)
        // .visible(false)
        .build();

    let cell = gtk::CellRendererText::new();

    gtk::prelude::CellLayoutExt::pack_start(&column, &cell, false);
    gtk::prelude::TreeViewColumnExt::add_attribute(&column, &cell, "text", 1);

    devices_treeview.append_column(&column);

    let column = gtk::TreeViewColumn::builder()
        .title("Type")
        .sizing(TreeViewColumnSizing::Autosize)
        .build();

    let cell = gtk::CellRendererText::new();

    gtk::prelude::CellLayoutExt::pack_start(&column, &cell, false);
    gtk::prelude::TreeViewColumnExt::add_attribute(&column, &cell, "text", 2);

    devices_treeview.append_column(&column);

    let column = gtk::TreeViewColumn::builder()
        .title("Make")
        .sizing(TreeViewColumnSizing::Autosize)
        .build();

    let cell = gtk::CellRendererText::new();

    gtk::prelude::CellLayoutExt::pack_start(&column, &cell, false);
    gtk::prelude::TreeViewColumnExt::add_attribute(&column, &cell, "text", 3);

    devices_treeview.append_column(&column);

    let column = gtk::TreeViewColumn::builder()
        .title("Model")
        .sizing(TreeViewColumnSizing::Autosize)
        .build();

    let cell = gtk::CellRendererText::new();

    gtk::prelude::CellLayoutExt::pack_start(&column, &cell, true);
    gtk::prelude::TreeViewColumnExt::add_attribute(&column, &cell, "text", 4);

    devices_treeview.append_column(&column);

    devices_treeview.set_model(Some(&devices_treestore));
    devices_treeview.show_all();

    let hue = canvas_hue_scale.value();
    let saturation = canvas_saturation_scale.value();
    let lightness = canvas_lightness_scale.value();

    // drawing areas
    drawing_area_preview.connect_draw(clone!(@weak notification_box_global => @default-return gtk::Inhibit(true), move |da: &gtk::DrawingArea, context: &cairo::Context| {
        if let Err(_e) = render_canvas(
            RenderMode::Preview,
            da,
            (hue, saturation, lightness),
            context,
        ) {
            notification_box_global.show();

            // apparently we have lost the connection to the Eruption daemon
            events::LOST_CONNECTION.store(true, Ordering::SeqCst);
        } else {
            notification_box_global.hide();

            if events::LOST_CONNECTION.load(Ordering::SeqCst) {
                // we re-established the connection to the Eruption daemon,
                // update the GUI to show e.g. newly attached devices
                events::LOST_CONNECTION.store(false, Ordering::SeqCst);

                events::UPDATE_MAIN_WINDOW.store(true, Ordering::SeqCst);
            }
        }

        gtk::Inhibit(false)
    }));

    drawing_area_zones.connect_draw(clone!(@weak notification_box_global => @default-return gtk::Inhibit(true), move |da: &gtk::DrawingArea, context: &cairo::Context| {
        if let Err(_e) = render_canvas(RenderMode::Zones, da, (hue, saturation, lightness), context)
        {
            notification_box_global.show();

            // apparently we have lost the connection to the Eruption daemon
            events::LOST_CONNECTION.store(true, Ordering::SeqCst);
        } else {
            notification_box_global.hide();

            if events::LOST_CONNECTION.load(Ordering::SeqCst) {
                // we re-established the connection to the Eruption daemon,
                // update the GUI to show e.g. newly attached devices
                events::LOST_CONNECTION.store(false, Ordering::SeqCst);

                events::UPDATE_MAIN_WINDOW.store(true, Ordering::SeqCst);
            }
        }

        gtk::Inhibit(false)
    }));

    // update the global LED color map vector
    timers::register_timer(
        timers::CANVAS_RENDER_TIMER_ID,
        TimerMode::ActiveStackPage(Pages::Canvas as u8),
        1000 / (crate::constants::TARGET_FPS * 2),
        clone!(@weak drawing_area_preview, @weak drawing_area_zones, @weak canvas_stack => @default-return Ok(()), move || {
            let page = crate::ACTIVE_PAGE.load(Ordering::SeqCst);
            if page == Pages::Canvas as usize {
                if canvas_stack.visible_child_name().unwrap() == "page0" {
                    drawing_area_preview.queue_draw();
                } else if canvas_stack.visible_child_name().unwrap() == "page1" {
                    drawing_area_zones.queue_draw();
                }
            }

            Ok(())
        }),
    )?;

    // update device zone allocation information
    timers::register_timer(
        timers::CANVAS_ZONES_TIMER_ID,
        TimerMode::ActiveStackPage(Pages::Canvas as u8),
        500,
        clone!(@weak builder => @default-return Ok(()), move || {
            let page = crate::ACTIVE_PAGE.load(Ordering::SeqCst);
            if page == Pages::Canvas as usize

            {
                let _result = update_allocated_zones(&builder).map_err(|e| tracing::error!("{e}"));
            }

            Ok(())
        }),
    )?;

    canvas_stack.set_visible_child_name("page0");

    Ok(())
}

pub fn update_allocated_zones(_builder: &gtk::Builder) -> Result<()> {
    let zones = crate::dbus_client::get_allocated_zones()?;
    *crate::ZONES.lock() = zones;

    Ok(())
}

/// Update page "Canvas", e.g. after a hotplug event
pub fn update_canvas_page(builder: &gtk::Builder) -> Result<()> {
    // let application = application.as_ref();

    // let main_window: gtk::ApplicationWindow = builder.object("main_window").unwrap();

    // let notification_box_global: gtk::Box = builder.object("notification_box_global").unwrap();

    let devices_tree_view: gtk::TreeView = builder.object("devices_tree_view").unwrap();

    // devices tree
    let devices_treestore = gtk::TreeStore::new(&[
        glib::Type::BOOL,
        glib::Type::U64,
        String::static_type(),
        String::static_type(),
        String::static_type(),
    ]);

    let devices = dbus_client::get_managed_devices()?;

    let mut index = 0_u32;

    // add keyboard devices
    for _device_ids in devices.0 {
        let device = get_keyboard_device(index as u64)?;

        let make = device.get_make_and_model().0;
        let model = device.get_make_and_model().1;

        let device_type = String::from("Keyboard");

        let enabled = dbus_client::is_device_enabled(index as u64)?;

        devices_treestore.insert_with_values(
            None,
            None,
            &[
                (0, &(enabled)),
                (1, &(index as u64)),
                (2, &(device_type)),
                (3, &(make)),
                (4, &(model)),
            ],
        );

        // populate_canvas_stack_widget_for_device(builder, &format!("Zone {index}"))?;

        index += 1;
    }

    // add mouse devices
    for _device_ids in devices.1 {
        let device = get_mouse_device(index as u64)?;

        let make = device.get_make_and_model().0;
        let model = device.get_make_and_model().1;

        let device_type = String::from("Mouse");

        let enabled = dbus_client::is_device_enabled(index as u64)?;

        devices_treestore.insert_with_values(
            None,
            None,
            &[
                (0, &(enabled)),
                (1, &(index as u64)),
                (2, &(device_type)),
                (3, &(make)),
                (4, &(model)),
            ],
        );

        // populate_canvas_stack_widget_for_device(builder, &format!("Zone {index}"))?;

        index += 1;
    }

    // add misc devices
    for _device_ids in devices.2 {
        let device = get_misc_device(index as u64)?;

        let make = device.get_make_and_model().0;
        let model = device.get_make_and_model().1;

        let device_type = String::from("Misc");

        let enabled = dbus_client::is_device_enabled(index as u64)?;

        devices_treestore.insert_with_values(
            None,
            None,
            &[
                (0, &(enabled)),
                (1, &(index as u64)),
                (2, &(device_type)),
                (3, &(make)),
                (4, &(model)),
            ],
        );

        // populate_canvas_stack_widget_for_device(builder, &format!("Zone {index}"))?;

        index += 1;
    }

    devices_tree_view.set_model(Some(&devices_treestore));
    devices_tree_view.show_all();

    Ok(())
}

#[allow(dead_code)]
fn populate_canvas_stack_widget_for_device(builder: &Builder, title: &str) -> Result<()> {
    let stack_widget: Stack = builder.object("canvas_stack").unwrap();
    let stack_switcher: StackSwitcher = builder.object("canvas_switcher").unwrap();

    let context = stack_switcher.style_context();
    context.add_class("small-font");

    let scrolled_window = ScrolledWindow::builder()
        .shadow_type(ShadowType::None)
        .build();
    // scrolled_window.add(&sourceview);

    scrolled_window.show_all();

    stack_widget.add_titled(&scrolled_window, "Zone", title);

    scrolled_window.show_all();

    Ok(())
}

fn render_canvas(
    mode: RenderMode,
    da: &gtk::DrawingArea,
    hsl: (f64, f64, f64),
    context: &cairo::Context,
) -> Result<()> {
    let width = da.allocated_width() as f64 - 400.0;
    let height = da.allocated_height() as f64;

    let scale_factor = 1.0; // width / (constants::CANVAS_WIDTH as f64 * 15.0);

    let led_colors = crate::COLOR_MAP.lock();

    // paint all cells of the canvas
    for (i, color) in led_colors.iter().enumerate() {
        paint_cell(i, color, hsl, context, width, height, scale_factor)?;
    }

    if mode == RenderMode::Zones {
        let layout = pangocairo::create_layout(context);
        FONT_DESC.with(|f| -> Result<()> {
            let desc = f.borrow();
            layout.set_font_description(Some(&desc));

            // draw allocated zones
            for (device, zone) in crate::ZONES.lock().iter() {
                paint_zone(context, width, height, &layout, *device, zone, scale_factor)?;
            }

            Ok(())
        })?;
    }

    Ok(())
}

#[allow(dead_code)]
pub fn rounded_rectangle(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: f64,
    color: &(f64, f64, f64, f64),
    color2: &(f64, f64, f64, f64),
) -> Result<()> {
    let aspect = 1.0; // aspect ratio
    let corner_radius = height / radius; // corner curvature radius

    let radius = corner_radius / aspect;
    let degrees = std::f64::consts::PI / 180.0;

    cr.new_sub_path();
    cr.arc(
        x + width - radius,
        y + radius,
        radius,
        -90.0 * degrees,
        0.0 * degrees,
    );
    cr.arc(
        x + width - radius,
        y + height - radius,
        radius,
        0.0 * degrees,
        90.0 * degrees,
    );
    cr.arc(
        x + radius,
        y + height - radius,
        radius,
        90.0 * degrees,
        180.0 * degrees,
    );
    cr.arc(
        x + radius,
        y + radius,
        radius,
        180.0 * degrees,
        270.0 * degrees,
    );
    cr.close_path();

    cr.set_source_rgba(color2.0, color2.1, color2.2, 1.0 - color2.3);
    cr.fill_preserve()?;

    cr.set_source_rgba(color.0, color.1, color.2, 1.0 - color.3);
    cr.set_line_width(1.85);
    cr.stroke()?;

    Ok(())
}

fn paint_zone(
    cr: &cairo::Context,
    width: f64,
    height: f64,
    layout: &pango::Layout,
    device: u64,
    zone: &Zone,
    scale_factor: f64,
) -> Result<()> {
    let pixel_width: f64 = width / constants::CANVAS_WIDTH as f64;
    let pixel_height: f64 = height / constants::CANVAS_HEIGHT as f64;

    for y in zone.y..zone.y2() {
        for x in zone.x..zone.x2() {
            let cell_def = Rectangle {
                x: BORDER.0 + (x as f64 * pixel_width) * scale_factor,
                y: BORDER.1 + (y as f64 * pixel_height) * scale_factor,
                width: pixel_width * scale_factor,
                height: pixel_height * scale_factor,
            };

            // use a translucent color to paint the zone
            let color = (1.0, 1.0, 1.0, 0.55);
            cr.set_source_rgba(color.0, color.1, color.2, color.3);
            cr.rectangle(
                cell_def.x - 0.5,
                cell_def.y - 0.5,
                cell_def.width + 0.5,
                cell_def.height + 0.5,
            );
            cr.fill()?;

            let color = (1.0, 1.0, 1.0, 0.65);
            cr.set_source_rgba(color.0, color.1, color.2, color.3);
            cr.stroke()?;
        }
    }

    // draw caption background
    cr.set_source_rgba(0.21, 0.21, 0.21, 0.85);
    cr.rectangle(
        BORDER.0 + (zone.x as f64 * pixel_width * scale_factor),
        BORDER.1 + (zone.y as f64 * pixel_height * scale_factor),
        pixel_width * scale_factor * 2.0,
        pixel_height * scale_factor * 2.0,
    );
    cr.fill()?;

    // draw caption
    cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
    cr.move_to(
        BORDER.0 + (zone.x as f64 * pixel_width * scale_factor) + 14.5,
        BORDER.1 + (zone.y as f64 * pixel_height * scale_factor) + 2.0,
    );
    layout.set_text(&format!("{}", device));
    pangocairo::show_layout(cr, layout);

    Ok(())
}

fn paint_cell(
    cell_index: usize,
    color: &crate::util::RGBA,
    _hsl: (f64, f64, f64),
    cr: &cairo::Context,
    width: f64,
    height: f64,
    scale_factor: f64,
) -> Result<()> {
    let pixel_width: f64 = width / constants::CANVAS_WIDTH as f64;
    let pixel_height: f64 = height / constants::CANVAS_HEIGHT as f64;

    let xval = (cell_index % constants::CANVAS_WIDTH) as f64 * pixel_width;
    let yval = (cell_index / constants::CANVAS_WIDTH) as f64 * pixel_height;

    let cell_def = Rectangle {
        x: BORDER.0 + xval * scale_factor,
        y: BORDER.1 + yval * scale_factor,
        width: pixel_width * scale_factor,
        height: pixel_height * scale_factor,
    };

    // post-process color
    // let color = LinSrgba::new(
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
    // let color = LinSrgba::from_color(
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
        1.0 - color.a as f64 / 255.0,
    );
    cr.rectangle(
        cell_def.x - 0.5,
        cell_def.y - 0.5,
        cell_def.width + 0.5,
        cell_def.height + 0.5,
    );
    cr.fill()?;

    cr.set_source_rgba(
        color.r as f64 / 255.0,
        color.g as f64 / 255.0,
        color.b as f64 / 255.0,
        1.0 - color.a as f64 / 255.0,
    );
    cr.stroke()?;

    Ok(())
}
