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
use std::sync::atomic::Ordering;
use std::sync::Arc;

use gdk::ModifierType;
use glib::clone;
use gtk::{
    prelude::*, Builder, ButtonsType, MessageDialog, MessageType, ResponseType, ScrolledWindow,
    ShadowType, Stack, StackSwitcher, TreeViewColumnSizing,
};
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

const BORDER: (f64, f64) = (8.0, 8.0);

type Result<T> = std::result::Result<T, eyre::Error>;

lazy_static! {
    /// Per-device allocated zones on the unified canvas
    pub static ref ZONES: Arc<RwLock<Vec<(u64, Zone)>>> = Arc::new(RwLock::new(vec![]));

    /// The zone that the cursor is hovering over
    pub static ref HOVER_ZONE: Arc<RwLock<Option<Zone>>> = Arc::new(RwLock::new(None));

    /// The zone that is currently selected
    pub static ref SELECTED_ZONE: Arc<RwLock<Option<Zone>>> = Arc::new(RwLock::new(None));
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
                    if let Some(e) = ZONES.read().iter().find(|(index, _zone)| index == &device) {
                        *SELECTED_ZONE.write() = Some(e.1);
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
            .find(|&v| v.0 == index as u64)
            .map(|v| v.1.enabled)
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
            .find(|&v| v.0 == index as u64)
            .map(|v| v.1.enabled)
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
            .find(|&v| v.0 == index as u64)
            .map(|v| v.1.enabled)
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

        let mut zones = ZONES.write();
        let zone = zones.iter_mut().find(|v| v.0 == device_index);

        match zone {
            Some(zone) => {
                let mut zone = zone.1;
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

    // We need to handle some events to support manipulation of per-device zones
    drawing_area_zones.connect_button_press_event(
        clone!(@weak builder => @default-return gtk::Inhibit(true), move |da, event| drawing_area_button_press(da, event, &builder)),
    );
    drawing_area_zones.connect_button_release_event(drawing_area_button_release);

    drawing_area_zones.connect_motion_notify_event(drawing_area_motion_notify);

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

    fetch_allocated_zones(builder)?;

    canvas_stack.set_visible_child_name("page0");

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
            .find(|&v| v.0 == index as u64)
            .map(|v| v.1.enabled)
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
            .find(|&v| v.0 == index as u64)
            .map(|v| v.1.enabled)
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
            .find(|&v| v.0 == index as u64)
            .map(|v| v.1.enabled)
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
) -> gtk::Inhibit {
    da.queue_draw();

    match event.button() {
        gdk::BUTTON_PRIMARY => {
            // primary button is down, check whether we need to select a zone...
            match event.coords() {
                Some((x, y)) => {
                    let mut hovering_over_a_zone = false;

                    for (device, zone) in ZONES.read().iter() {
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
                        if cell_x >= zone.x
                            && cell_x <= zone.x2()
                            && cell_y >= zone.y
                            && cell_y <= zone.y2()
                        {
                            // we are hovering over this zone
                            *HOVER_ZONE.write() = Some(*zone);
                            hovering_over_a_zone = true;

                            // since the primary button is down, save the current zone as the currently selected one
                            *SELECTED_ZONE.write() = Some(*zone);

                            let devices_treeview: gtk::TreeView =
                                builder.object("devices_tree_view").unwrap();
                            select_row_by_device(&devices_treeview, *device);

                            break;
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

                    gtk::Inhibit(true)
                }

                _ => gtk::Inhibit(false),
            }
        }

        _ => gtk::Inhibit(false),
    }
}

fn drawing_area_button_release(da: &gtk::DrawingArea, event: &gdk::EventButton) -> gtk::Inhibit {
    da.queue_draw();

    match event.button() {
        gdk::BUTTON_PRIMARY => gtk::Inhibit(true),

        _ => gtk::Inhibit(false),
    }
}

fn drawing_area_motion_notify(da: &gtk::DrawingArea, event: &gdk::EventMotion) -> gtk::Inhibit {
    fn set_cursor(cursor: Option<gdk::CursorType>) {
        *CURSOR_TYPE.write() = cursor;
    }

    match event.coords() {
        Some((x, y)) => {
            let mut hovering_over_a_zone = false;

            for (device, ref mut zone) in ZONES.write().iter_mut() {
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
                if cell_x >= zone.x
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
                        } else if (zone.x2() - 2..=zone.x2() + 2).contains(&cell_x)
                            && (zone.y - 2..=zone.y + 2).contains(&cell_y)
                        {
                            set_cursor(Some(gdk::CursorType::TopRightCorner));
                        } else if cell_x == zone.x2() {
                            set_cursor(Some(gdk::CursorType::RightSide));
                        } else if (zone.x2() - 2..=zone.x2() + 2).contains(&cell_x)
                            && (zone.y2() - 2..=zone.y2() + 2).contains(&cell_y)
                        {
                            set_cursor(Some(gdk::CursorType::BottomRightCorner));
                        } else if cell_y == zone.y2() {
                            set_cursor(Some(gdk::CursorType::BottomSide));
                        } else if (zone.x - 2..=zone.x + 2).contains(&cell_x)
                            && (zone.y2() - 2..=zone.y2() + 2).contains(&cell_y)
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
                        // we are not hovering over a sensitive border, but inside the zone
                        set_cursor(Some(gdk::CursorType::Fleur));

                        if event.state() & ModifierType::BUTTON1_MASK == ModifierType::BUTTON1_MASK
                        {
                            // primary button is down, move the zone around or resize

                            // let offset_x = cell_x - zone.x;
                            // let offset_y = cell_y - zone.y;

                            zone.x = cell_x - zone.width / 2;
                            zone.y = cell_y - zone.height / 2;

                            if zone.x < 0 {
                                zone.x = 0;
                            } else if zone.x > constants::CANVAS_WIDTH as i32 {
                                zone.x = constants::CANVAS_WIDTH as i32;
                            } else if zone.y < 0 {
                                zone.y = 0;
                            }
                            if zone.y > constants::CANVAS_HEIGHT as i32 {
                                zone.y = constants::CANVAS_HEIGHT as i32;
                            }
                            if zone.width < 1 {
                                zone.width = 1;
                            } else if zone.height < 1 {
                                zone.height = 1;
                            } else if zone.x2() > constants::CANVAS_WIDTH as i32 {
                                zone.x -= zone.x2() - (constants::CANVAS_WIDTH as i32 + 1);
                            } else if zone.y2() > constants::CANVAS_HEIGHT as i32 {
                                zone.y -= zone.y2() - (constants::CANVAS_HEIGHT as i32 + 1);
                            }

                            let _ = dbus_client::set_device_zone_allocation(*device, zone).map_err(
                                |e| {
                                    notifications::error(&format!("Could not update zone: {e}"));
                                    tracing::error!("Could not update zone: {e}");
                                },
                            );
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

                if event.state() & ModifierType::BUTTON1_MASK == ModifierType::BUTTON1_MASK {
                    // primary button is down, move the zone around or resize
                    *SELECTED_ZONE.write() = None;
                }
            }

            gtk::Inhibit(true)
        }

        _ => gtk::Inhibit(false),
    }
}

fn render_canvas(
    mode: RenderMode,
    da: &gtk::DrawingArea,
    mut hsl: (f64, f64, f64),
    context: &cairo::Context,
) -> Result<()> {
    let width = da.allocated_width() as f64 - 400.0;
    let height = da.allocated_height() as f64;

    let scale_factor = 1.0; // width / (constants::CANVAS_WIDTH as f64 * 15.0);

    let led_colors = crate::COLOR_MAP.lock();

    if mode == RenderMode::Zones {
        // dim lightness in zone allocation mode
        hsl.2 = -0.5;
    }

    // paint all cells of the canvas
    for (i, color) in led_colors.iter().enumerate() {
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
            for (device, zone) in ZONES.read().iter() {
                let state;

                if selected_zone.is_some() && *zone == selected_zone.unwrap() {
                    if hover_zone.is_some() && *zone == hover_zone.unwrap() {
                        state = ZoneDrawState::SelectedHover;
                    } else {
                        state = ZoneDrawState::Selected;
                    }
                } else if hover_zone.is_some() && *zone == hover_zone.unwrap() {
                    state = ZoneDrawState::Hover;
                } else {
                    state = ZoneDrawState::Normal;
                }

                paint_zone(
                    context,
                    width,
                    height,
                    &layout,
                    *device,
                    zone,
                    state,
                    scale_factor,
                )?;
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

    match state {
        ZoneDrawState::Normal => {
            let x = zone.x as f64 * pixel_width;
            let y = zone.y as f64 * pixel_height;
            let width = zone.width as f64 * pixel_width;
            let height = zone.height as f64 * pixel_height;

            let color = (0.8, 0.8, 0.8, 0.3);
            let color2 = (0.8, 0.8, 0.8, 0.3);
            rounded_rectangle(cr, x, y, width, height, 90.0, &color, &color2)?;
        }

        ZoneDrawState::Hover => {
            let x = zone.x as f64 * pixel_width;
            let y = zone.y as f64 * pixel_height;
            let width = zone.width as f64 * pixel_width;
            let height = zone.height as f64 * pixel_height;

            let color = (0.8, 0.8, 0.8, 0.5);
            let color2 = (0.8, 0.8, 0.8, 0.5);
            rounded_rectangle(cr, x, y, width, height, 90.0, &color, &color2)?;
        }

        ZoneDrawState::Selected => {
            let x = zone.x as f64 * pixel_width;
            let y = zone.y as f64 * pixel_height;
            let width = zone.width as f64 * pixel_width;
            let height = zone.height as f64 * pixel_height;

            let color = (0.4, 0.4, 0.85, 0.75);
            let color2 = (0.4, 0.4, 0.85, 0.75);
            rounded_rectangle(cr, x, y, width, height, 90.0, &color, &color2)?;
        }

        ZoneDrawState::SelectedHover => {
            let x = zone.x as f64 * pixel_width;
            let y = zone.y as f64 * pixel_height;
            let width = zone.width as f64 * pixel_width;
            let height = zone.height as f64 * pixel_height;

            let color = (0.4, 0.4, 0.85, 0.8);
            let color2 = (0.4, 0.4, 0.85, 0.8);
            rounded_rectangle(cr, x, y, width, height, 90.0, &color, &color2)?;
        }
    }

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
    let color = LinSrgba::new(
        color.r as f64 / 255.0,
        color.g as f64 / 255.0,
        color.b as f64 / 255.0,
        color.a as f64 / 255.0,
    );

    // let hue_value = hsl.0;
    // let saturation_value = hsl.1;
    let lighten_value = hsl.2;

    // image post processing
    let color = LinSrgba::from_color(color.lighten(lighten_value)).into_components();

    cr.set_source_rgba(color.0, color.1, color.2, 1.0);

    cr.set_source_rgba(color.0, color.1, color.2, 1.0);
    cr.rectangle(
        cell_def.x - 0.5,
        cell_def.y - 0.5,
        cell_def.width + 0.5,
        cell_def.height + 0.5,
    );
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
