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

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use gdk::ModifierType;
use glib::clone;
use gtk::{
    prelude::*, Builder, ButtonsType, MessageDialog, MessageType, ResponseType, ScrolledWindow,
    ShadowType, Stack, StackSwitcher, TreeViewColumnSizing,
};
use ndarray::ArrayView2;
use palette::{FromColor, Lighten, LinSrgba};
use parking_lot::RwLock;

use crate::timers::TimerMode;
use crate::zone::Zone;
use crate::{constants, dbus_client, notifications, timers};
use crate::{events, util};
use lazy_static::lazy_static;

use super::hwdevices::keyboards::get_keyboard_device;
use super::hwdevices::mice::get_mouse_device;
use super::hwdevices::misc::get_misc_device;
use super::main_window::CURSOR_TYPE;
use super::Pages;

const BORDER: (f64, f64) = (0.0, 0.0);

type Result<T> = std::result::Result<T, eyre::Error>;

lazy_static! {
    /// Managed devices
    pub static ref DEVICE_INFO: Arc<RwLock<HashMap<u64, (String, String)>>> = Arc::new(RwLock::new(HashMap::new()));

    /// Per-device allocated zones on the unified canvas
    pub static ref ZONES: Arc<RwLock<Vec<Zone>>> = Arc::new(RwLock::new(vec![]));

    /// The zone that the cursor is hovering over
    pub static ref HOVER_ZONE: Arc<RwLock<Option<Zone>>> = Arc::new(RwLock::new(None));

    /// The zone that is currently selected
    pub static ref SELECTED_ZONE: Arc<RwLock<Option<Zone>>> = Arc::new(RwLock::new(None));

    /// Offset ccordinates of the current selection
    pub static ref OFFSET_COORDINATES: Arc<RwLock<Option<(i32, i32)>>> = Arc::new(RwLock::new(None));
}

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
    static FONT_DESC: RefCell<pango::FontDescription> = RefCell::new(pango::FontDescription::from_string("Roboto demibold 12"));
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
        clone!(@weak canvas_stack, @weak canvas_switcher  => move |sel| {
            canvas_stack.set_visible_child_name("page1");

            if let Some((model, selection)) = sel.selected() {
                if let Ok(device) = model.value(&selection, 1).get::<u64>() {
                    if let Some(e) = ZONES.read().iter().find(|zone| zone.device == Some(device)) {
                        *SELECTED_ZONE.write() = Some(*e);
                    } else {
                        *SELECTED_ZONE.write() = None;
                    }
                }
            }
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
    let zones = dbus_client::get_devices_zone_allocations()?;

    let mut index = 0_u32;

    // add keyboard devices
    for _device_ids in devices.0 {
        let device = get_keyboard_device(index as u64)?;

        let make = device.get_make_and_model().0;
        let model = device.get_make_and_model().1;

        let device_type = String::from("Keyboard");

        let enabled = zones
            .iter()
            .find(|&v| v.device == Some(index as u64))
            .map(|v| v.enabled)
            .unwrap_or(false);

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

        let enabled = zones
            .iter()
            .find(|&v| v.device == Some(index as u64))
            .map(|v| v.enabled)
            .unwrap_or(false);

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

        let enabled = zones
            .iter()
            .find(|&v| v.device == Some(index as u64))
            .map(|v| v.enabled)
            .unwrap_or(false);

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

    cell.connect_toggled(clone!(@weak drawing_area_zones, @weak devices_treeview => move |_f, p| {
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

        let mut zones = ZONES.write();
        let zone = zones.iter_mut().find(|v| v.device == Some(device_index));

        match zone {
            Some(zone) => {
                zone.enabled = !value;

                dbus_client::set_device_zone_allocation(device_index, &zone).unwrap_or_else(|e| {
                    tracing::error!("Could not set device zone status: {e}");
                    notifications::error(&format!("Could not set device zone status: {e}"));
                });
            }

            None => {
                tracing::error!("Could not find the devices zone");
                notifications::error(&format!("Could not find the devices zone"));
            }
        }

        drawing_area_zones.queue_draw();
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
    drawing_area_preview.connect_draw(clone!(@weak notification_box_global => @default-return glib::Propagation::Proceed, move |da: &gtk::DrawingArea, context: &cairo::Context| {
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

        false.into()
    }));

    drawing_area_zones.connect_draw(clone!(@weak notification_box_global => @default-return glib::Propagation::Proceed, move |da: &gtk::DrawingArea, context: &cairo::Context| {
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

        false.into()
    }));

    // We need to handle some events to support manipulation of per-device zones
    drawing_area_zones.connect_button_press_event(
        clone!(@weak builder => @default-return glib::Propagation::Proceed, move |da, event| drawing_area_button_press(da, event, &builder)),
    );
    drawing_area_zones.connect_button_release_event(drawing_area_button_release);

    drawing_area_zones.connect_motion_notify_event(drawing_area_motion_notify);

    drawing_area_zones.connect_leave_notify_event(drawing_area_leave_notify);

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

    // // update device zone allocation information
    // timers::register_timer(
    //     timers::CANVAS_ZONES_TIMER_ID,
    //     TimerMode::ActiveStackPage(Pages::Canvas as u8),
    //     500,
    //     clone!(@weak builder => @default-return Ok(()), move || {
    //         let page = crate::ACTIVE_PAGE.load(Ordering::SeqCst);
    //         if page == Pages::Canvas as usize

    //         {
    //             let _result = update_allocated_zones(&builder).map_err(|e| tracing::error!("{e}"));
    //         }

    //         Ok(())
    //     }),
    // )?;

    fetch_device_info(builder)?;
    fetch_allocated_zones(builder)?;

    canvas_stack.set_visible_child_name("page0");

    Ok(())
}

pub fn fetch_device_info(_builder: &gtk::Builder) -> Result<()> {
    let mut device_info = DEVICE_INFO.write();

    let devices = dbus_client::get_managed_devices()?;

    let mut index = 0;

    // add keyboard devices
    for _device_ids in devices.0 {
        let device = get_keyboard_device(index)?;

        let make = device.get_make_and_model().0;
        let model = device.get_make_and_model().1;

        device_info.insert(index, (make.to_string(), model.to_string()));

        index += 1;
    }

    // add mouse devices
    for _device_ids in devices.1 {
        let device = get_mouse_device(index)?;

        let make = device.get_make_and_model().0;
        let model = device.get_make_and_model().1;

        device_info.insert(index, (make.to_string(), model.to_string()));

        index += 1;
    }

    // add misc devices
    for _device_ids in devices.2 {
        let device = get_misc_device(index)?;

        let make = device.get_make_and_model().0;
        let model = device.get_make_and_model().1;

        device_info.insert(index, (make.to_string(), model.to_string()));

        index += 1;
    }

    Ok(())
}

pub fn fetch_allocated_zones(_builder: &gtk::Builder) -> Result<()> {
    let zones = crate::dbus_client::get_devices_zone_allocations()?;
    *ZONES.write() = zones;

    Ok(())
}

// pub fn update_allocated_zones(_builder: &gtk::Builder) -> Result<()> {
//     let zones = crate::dbus_client::get_allocated_zones()?;
//     *ZONES.write() = zones;

//     Ok(())
// }

/// Update page "Canvas", e.g. after a hotplug event
pub fn update_canvas_page(builder: &gtk::Builder) -> Result<()> {
    // let application = application.as_ref();

    // let main_window: gtk::ApplicationWindow = builder.object("main_window").unwrap();
    // let notification_box_global: gtk::Box = builder.object("notification_box_global").unwrap();

    let canvas_stack: Stack = builder.object("canvas_stack").unwrap();

    let devices_tree_view: gtk::TreeView = builder.object("devices_tree_view").unwrap();

    // devices tree
    let devices_treestore = gtk::TreeStore::new(&[
        glib::Type::BOOL,
        glib::Type::U64,
        String::static_type(),
        String::static_type(),
        String::static_type(),
    ]);

    // canvas_stack.set_visible_child_name("page0");

    let devices = dbus_client::get_managed_devices()?;
    let zones = dbus_client::get_devices_zone_allocations()?;

    let mut index = 0_u32;

    // add keyboard devices
    for _device_ids in devices.0 {
        let device = get_keyboard_device(index as u64)?;

        let make = device.get_make_and_model().0;
        let model = device.get_make_and_model().1;

        let device_type = String::from("Keyboard");

        let enabled = zones
            .iter()
            .find(|&v| v.device == Some(index as u64))
            .map(|v| v.enabled)
            .unwrap_or(false);

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

        let enabled = zones
            .iter()
            .find(|&v| v.device == Some(index as u64))
            .map(|v| v.enabled)
            .unwrap_or(false);

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

        let enabled = zones
            .iter()
            .find(|&v| v.device == Some(index as u64))
            .map(|v| v.enabled)
            .unwrap_or(false);

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

    canvas_stack.set_visible_child_name("page0");

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

fn drawing_area_button_press(
    da: &gtk::DrawingArea,
    event: &gdk::EventButton,
    builder: &gtk::Builder,
) -> glib::Propagation {
    da.queue_draw();

    match event.button() {
        gdk::BUTTON_PRIMARY => {
            // primary button is down, check whether we need to select a zone...
            match event.coords() {
                Some((x, y)) => {
                    let mut hovering_over_a_zone = false;

                    for zone in ZONES.write().iter_mut() {
                        let pixel_width: f64 =
                            (da.allocated_width() as f64 - 400.0) / constants::CANVAS_WIDTH as f64;
                        let pixel_height: f64 =
                            da.allocated_height() as f64 / constants::CANVAS_HEIGHT as f64;

                        let cell_x = (((da.allocated_width() as f64 - 400.0) / pixel_width)
                            / (constants::CANVAS_WIDTH as f64 / x))
                            / pixel_width;
                        let cell_y = ((da.allocated_height() as f64 / pixel_height)
                            / (constants::CANVAS_HEIGHT as f64 / y))
                            / pixel_height;

                        // println!("{cell_x}:{cell_y}");

                        let cell_x = cell_x.floor() as i32;
                        let cell_y = cell_y.floor() as i32;

                        // check whether we are hovering the cursor over the current zone
                        if zone.enabled
                            && cell_x >= zone.x
                            && cell_x <= zone.x2()
                            && cell_y >= zone.y
                            && cell_y <= zone.y2()
                        {
                            // compute and store offsets from the top left corner of the zone
                            let offset_x = cell_x - zone.x;
                            let offset_y = cell_y - zone.y;

                            *OFFSET_COORDINATES.write() = Some((offset_x, offset_y));

                            // we are hovering over this zone
                            *HOVER_ZONE.write() = Some(*zone);
                            hovering_over_a_zone = true;

                            // since the primary button is down, save the current zone as the currently selected one
                            *SELECTED_ZONE.write() = Some(*zone);

                            let devices_treeview: gtk::TreeView =
                                builder.object("devices_tree_view").unwrap();
                            select_row_by_device(&devices_treeview, zone.device.unwrap());

                            if event.click_count().unwrap_or(1) >= 2 {
                                zone.x = 0;
                                zone.y = 0;
                                zone.width = constants::CANVAS_WIDTH as i32;
                                zone.height = constants::CANVAS_HEIGHT as i32;

                                let _ = dbus_client::set_device_zone_allocation(
                                    zone.device.unwrap(),
                                    &zone,
                                )
                                .map_err(|e| {
                                    notifications::error(&format!("Could not update zone: {e}"));
                                    tracing::error!("Could not update zone: {e}");
                                });
                            }

                            // grab pointer to receive all motion events
                            da.grab_add();
                        }
                    }

                    if !hovering_over_a_zone {
                        *HOVER_ZONE.write() = None;

                        if event.state() & ModifierType::BUTTON1_MASK == ModifierType::BUTTON1_MASK
                        {
                            // primary button is down, move the zone around or resize
                            *SELECTED_ZONE.write() = None;
                        }
                    }

                    glib::Propagation::Stop
                }

                _ => glib::Propagation::Stop,
            }
        }
        _ => glib::Propagation::Stop,
    }
}

fn drawing_area_button_release(
    da: &gtk::DrawingArea,
    event: &gdk::EventButton,
) -> glib::Propagation {
    da.queue_draw();

    match event.button() {
        gdk::BUTTON_PRIMARY => {
            da.grab_remove();

            glib::Propagation::Stop
        }

        _ => glib::Propagation::Stop,
    }
}

fn drawing_area_leave_notify(
    _da: &gtk::DrawingArea,
    _event: &gdk::EventCrossing,
) -> glib::Propagation {
    *CURSOR_TYPE.write() = None;

    glib::Propagation::Stop
}

fn drawing_area_motion_notify(
    da: &gtk::DrawingArea,
    event: &gdk::EventMotion,
) -> glib::Propagation {
    fn set_cursor(cursor: Option<gdk::CursorType>) {
        *CURSOR_TYPE.write() = cursor;
    }

    match event.coords() {
        Some((x, y)) => {
            let mut hovering_over_a_zone = false;

            for zone in ZONES.write().iter_mut() {
                let pixel_width: f64 =
                    (da.allocated_width() as f64 - 400.0) / constants::CANVAS_WIDTH as f64;
                let pixel_height: f64 =
                    da.allocated_height() as f64 / constants::CANVAS_HEIGHT as f64;

                let cell_x = (((da.allocated_width() as f64 - 400.0) / pixel_width)
                    / (constants::CANVAS_WIDTH as f64 / x))
                    / pixel_width;
                let cell_y = ((da.allocated_height() as f64 / pixel_height)
                    / (constants::CANVAS_HEIGHT as f64 / y))
                    / pixel_height;

                // println!("{cell_x}:{cell_y}");

                let cell_x = cell_x.floor() as i32;
                let cell_y = cell_y.floor() as i32;

                // check whether we are hovering the cursor over the current zone
                if zone.enabled
                    && cell_x >= zone.x
                    && cell_x <= zone.x2()
                    && cell_y >= zone.y
                    && cell_y <= zone.y2()
                {
                    // we are hovering over this zone
                    *HOVER_ZONE.write() = Some(*zone);
                    hovering_over_a_zone = true;

                    // print!("Hovering over: {}", _device);
                    // println!("{zone:#?}");

                    // check whether we are hovering over the 1px wide border
                    if cell_x == zone.x
                        || cell_x == zone.x2()
                        || cell_y == zone.y
                        || cell_y == zone.y2()
                    {
                        // yes, we are above the border, so we need to determine which border...
                        if cell_x == zone.x && cell_y == zone.y {
                            set_cursor(Some(gdk::CursorType::TopLeftCorner));
                        } else if cell_y == zone.y {
                            set_cursor(Some(gdk::CursorType::TopSide));
                        } else if (zone.x2() - 1..=zone.x2() + 1).contains(&cell_x)
                            && (zone.y - 1..=zone.y + 1).contains(&cell_y)
                        {
                            set_cursor(Some(gdk::CursorType::TopRightCorner));
                        } else if cell_x == zone.x2() {
                            set_cursor(Some(gdk::CursorType::RightSide));
                        } else if (zone.x2() - 1..=zone.x2() + 1).contains(&cell_x)
                            && (zone.y2() - 1..=zone.y2() + 1).contains(&cell_y)
                        {
                            set_cursor(Some(gdk::CursorType::BottomRightCorner));
                        } else if cell_y == zone.y2() {
                            set_cursor(Some(gdk::CursorType::BottomSide));
                        } else if (zone.x - 1..=zone.x + 1).contains(&cell_x)
                            && (zone.y2() - 1..=zone.y2() + 1).contains(&cell_y)
                        {
                            set_cursor(Some(gdk::CursorType::BottomLeftCorner));
                        } else if cell_x == zone.x {
                            set_cursor(Some(gdk::CursorType::LeftSide));
                        } else {
                            // invalid state
                            tracing::error!("invalid input state");

                            set_cursor(None);
                        }
                    } else {
                        if event.state() & ModifierType::BUTTON1_MASK == ModifierType::BUTTON1_MASK
                        {
                            // primary button is down, move the zone around or resize

                            // we are not hovering over a sensitive border, but inside the zone
                            set_cursor(Some(gdk::CursorType::Fleur));

                            let offset_x = OFFSET_COORDINATES.read().unwrap_or((0, 0)).0;
                            let offset_y = OFFSET_COORDINATES.read().unwrap_or((0, 0)).1;

                            zone.x = cell_x - offset_x;
                            zone.y = cell_y - offset_y;

                            if zone.x < 0 {
                                zone.x = 0;
                            }

                            if zone.y < 0 {
                                zone.y = 0;
                            }

                            if zone.x2() > constants::CANVAS_WIDTH as i32 {
                                zone.x = constants::CANVAS_WIDTH as i32 - zone.width;
                            }

                            if zone.y2() > constants::CANVAS_HEIGHT as i32 {
                                zone.y = constants::CANVAS_HEIGHT as i32 - zone.height;
                            }

                            if zone.width < 1 {
                                zone.width = 1;
                            }

                            if zone.height < 1 {
                                zone.height = 1;
                            }

                            let _ =
                                dbus_client::set_device_zone_allocation(zone.device.unwrap(), zone)
                                    .map_err(|e| {
                                        notifications::error(&format!(
                                            "Could not update zone: {e}"
                                        ));
                                        tracing::error!("Could not update zone: {e}");
                                    });
                        } else {
                            // we are not hovering over a sensitive border, but inside the zone
                            set_cursor(Some(gdk::CursorType::Arrow));
                        }
                    }

                    break;
                }
            }

            // update state
            let zones = crate::dbus_client::get_devices_zone_allocations().unwrap();
            *ZONES.write() = zones;

            da.queue_draw();

            if !hovering_over_a_zone {
                *HOVER_ZONE.write() = None;
                set_cursor(None);

                // if event.state() & modifiertype::button1_mask == modifiertype::button1_mask {
                //     // primary button is down, move the zone around or resize
                //     *selected_zone.write() = none;
                // }
            }

            glib::Propagation::Stop
        }

        _ => {
            // *HOVER_ZONE.write() = None;
            // set_cursor(None);

            // if event.state() & ModifierType::BUTTON1_MASK == ModifierType::BUTTON1_MASK {
            //     // primary button is down, move the zone around or resize
            //     *SELECTED_ZONE.write() = None;
            // }

            glib::Propagation::Stop
        }
    }
}

fn render_canvas(
    mode: RenderMode,
    da: &gtk::DrawingArea,
    mut hsl: (f64, f64, f64),
    context: &cairo::Context,
) -> Result<()> {
    let width = da.allocated_width() as f64 - 390.0;
    let height = da.allocated_height() as f64;

    let scale_factor = 1.0; // width / (constants::CANVAS_WIDTH as f64 * 15.0);

    let led_map = crate::COLOR_MAP.lock();

    let canvas = ArrayView2::from_shape(
        (constants::CANVAS_HEIGHT, constants::CANVAS_WIDTH),
        &*led_map,
    )?;

    if mode == RenderMode::Zones {
        // dim lightness in zone allocation mode
        hsl.2 = -0.55;
    }

    // paint all cells of the canvas
    for (i, color) in canvas.iter().enumerate() {
        paint_cell(i, color, hsl, context, width, height, scale_factor)?;
    }

    if mode == RenderMode::Zones {
        let hover_zone = *HOVER_ZONE.read();
        let selected_zone = *SELECTED_ZONE.read();

        let layout = pangocairo::create_layout(context);
        FONT_DESC.with(|f| -> Result<()> {
            let desc = f.borrow();
            layout.set_font_description(Some(&desc));

            // draw allocated zones
            for zone in ZONES.read().iter() {
                let state;

                if selected_zone.is_some() && zone.device == selected_zone.unwrap().device {
                    if hover_zone.is_some() && zone.device == hover_zone.unwrap().device {
                        state = ZoneDrawState::SelectedHover;
                    } else {
                        state = ZoneDrawState::Selected;
                    }
                } else if hover_zone.is_some() && zone.device == hover_zone.unwrap().device {
                    state = ZoneDrawState::Hover;
                } else {
                    state = ZoneDrawState::Normal;
                }

                if zone.enabled {
                    paint_zone(
                        context,
                        width,
                        height,
                        &layout,
                        zone.device.unwrap_or_default(),
                        zone,
                        state,
                        scale_factor,
                    )?;
                }
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

    cr.set_source_rgba(color2.0, color2.1, color2.2, color2.3);
    cr.fill_preserve()?;

    cr.set_source_rgba(color.0, color.1, color.2, color.3);
    cr.set_line_width(4.0);
    cr.stroke()?;

    Ok(())
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum ZoneDrawState {
    Normal,
    Hover,
    Selected,
    SelectedHover,
}

#[inline]
fn paint_zone(
    cr: &cairo::Context,
    width: f64,
    height: f64,
    layout: &pango::Layout,
    device: u64,
    zone: &Zone,
    state: ZoneDrawState,
    scale_factor: f64,
) -> Result<()> {
    let pixel_width: f64 = width / constants::CANVAS_WIDTH as f64;
    let pixel_height: f64 = height / constants::CANVAS_HEIGHT as f64;

    let zones = ZONES.read();
    let Zone { enabled, .. } = zones.get(device as usize).map(|v| *v).unwrap_or_default();

    if enabled {
        let device_info = DEVICE_INFO.read();
        let make_and_model = device_info.get(&device);

        let make = make_and_model
            .map(|v| v.0.clone())
            .unwrap_or_else(|| "<unknown>".to_string());

        let model = make_and_model
            .map(|v| v.1.clone())
            .unwrap_or_else(|| "<unknown>".to_string());

        match state {
            ZoneDrawState::Normal => {
                let x = BORDER.0 + zone.x as f64 * pixel_width;
                let y = BORDER.1 + zone.y as f64 * pixel_height;
                let width = zone.width as f64 * pixel_width;
                let height = zone.height as f64 * pixel_height;

                let color = (0.8, 0.8, 0.8, 0.2);
                let color2 = (0.8, 0.8, 0.8, 0.2);
                rounded_rectangle(cr, x, y, width, height, 90.0, &color, &color2)?;

                // draw caption
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.2);
                cr.move_to(
                    BORDER.0 + (zone.x as f64 * pixel_width * scale_factor) + 14.5,
                    BORDER.1 + (zone.y as f64 * pixel_height * scale_factor) + 12.0,
                );
                layout.set_text(&format!("{}: {} {}", device, make, model));
                pangocairo::show_layout(cr, layout);
            }

            ZoneDrawState::Hover => {
                let x = BORDER.0 + zone.x as f64 * pixel_width;
                let y = BORDER.1 + zone.y as f64 * pixel_height;
                let width = zone.width as f64 * pixel_width;
                let height = zone.height as f64 * pixel_height;

                let color = (0.8, 0.8, 0.8, 0.5);
                let color2 = (0.8, 0.8, 0.8, 0.5);
                rounded_rectangle(cr, x, y, width, height, 90.0, &color, &color2)?;

                // draw caption
                cr.set_source_rgba(1.0, 1.0, 1.0, 0.8);
                cr.move_to(
                    BORDER.0 + (zone.x as f64 * pixel_width * scale_factor) + 14.5,
                    BORDER.1 + (zone.y as f64 * pixel_height * scale_factor) + 12.0,
                );
                layout.set_text(&format!("{}: {} {}", device, make, model));
                pangocairo::show_layout(cr, layout);
            }

            ZoneDrawState::Selected => {
                let x = BORDER.0 + zone.x as f64 * pixel_width;
                let y = BORDER.1 + zone.y as f64 * pixel_height;
                let width = zone.width as f64 * pixel_width;
                let height = zone.height as f64 * pixel_height;

                let color = (0.4, 0.4, 0.85, 0.8);
                let color2 = (0.4, 0.4, 0.85, 0.8);
                rounded_rectangle(cr, x, y, width, height, 90.0, &color, &color2)?;

                // draw caption
                cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                cr.move_to(
                    BORDER.0 + (zone.x as f64 * pixel_width * scale_factor) + 14.5,
                    BORDER.1 + (zone.y as f64 * pixel_height * scale_factor) + 12.0,
                );
                layout.set_text(&format!("{}: {} {}", device, make, model));
                pangocairo::show_layout(cr, layout);
            }

            ZoneDrawState::SelectedHover => {
                let x = BORDER.0 + zone.x as f64 * pixel_width;
                let y = BORDER.1 + zone.y as f64 * pixel_height;
                let width = zone.width as f64 * pixel_width;
                let height = zone.height as f64 * pixel_height;

                let color = (0.4, 0.4, 0.85, 0.8);
                let color2 = (0.4, 0.4, 0.85, 0.8);
                rounded_rectangle(cr, x, y, width, height, 90.0, &color, &color2)?;

                // draw caption
                cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
                cr.move_to(
                    BORDER.0 + (zone.x as f64 * pixel_width * scale_factor) + 14.5,
                    BORDER.1 + (zone.y as f64 * pixel_height * scale_factor) + 12.0,
                );
                layout.set_text(&format!("{}: {} {}", device, make, model));
                pangocairo::show_layout(cr, layout);
            }
        }
    }

    Ok(())
}

#[inline]
fn paint_cell(
    cell_index: usize,
    color: &crate::util::RGBA,
    hsl: (f64, f64, f64),
    cr: &cairo::Context,
    width: f64,
    height: f64,
    scale_factor: f64,
) -> Result<()> {
    // spacing of pixels on the grid, negative values cause overlap
    const GRID_SPACING_X: f64 = -0.8;
    const GRID_SPACING_Y: f64 = -0.8;

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
    let source_color = LinSrgba::new(
        color.r as f64 / 255.0,
        color.g as f64 / 255.0,
        color.b as f64 / 255.0,
        color.a as f64 / 255.0,
    );

    // let hue_value = hsl.0;
    // let saturation_value = hsl.1;
    let lighten_value = hsl.2;

    // image post processing
    let fill_color = LinSrgba::from_color(source_color.lighten(lighten_value)).into_components();
    // let outline_color = fill_color;

    // cr.set_line_width(6.0);

    cr.rectangle(
        cell_def.x,
        cell_def.y,
        cell_def.width - GRID_SPACING_X,
        cell_def.height - GRID_SPACING_Y,
    );

    // cr.set_source_rgba(outline_color.0, outline_color.1, outline_color.2, 0.8);
    // cr.stroke_preserve()?;

    cr.set_source_rgba(fill_color.0, fill_color.1, fill_color.2, 1.0);
    cr.fill()?;

    Ok(())
}

fn select_row_by_device(tree_view: &gtk::TreeView, device: u64) {
    let selection = tree_view.selection();
    let model = tree_view.model().unwrap();

    let iter = model.iter_first();
    while let Some(ref row) = iter {
        let row_value = model.value(row, 1).get::<u64>().ok();
        if let Some(row_value) = row_value {
            if row_value == device {
                selection.select_iter(row);
                return;
            }
        }

        if !model.iter_next(row) {
            break;
        }
    }
}

// fn select_row_with_value(tree_view: &gtk::TreeView, column_index: i32, value: &str) {
//     let selection = tree_view.selection();
//     let model = tree_view.model().unwrap();

//     let iter = model.iter_first();
//     while let Some(ref row) = iter {
//         let row_value = model.value(row, column_index).get::<String>().ok();
//         if let Some(row_value) = row_value {
//             if row_value == value {
//                 selection.select_iter(row);
//                 return;
//             }
//         }

//         if !model.iter_next(&row) {
//             break;
//         }
//     }
// }
