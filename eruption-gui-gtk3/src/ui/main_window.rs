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

use eruption_sdk::connection::Connection;
use eruption_sdk::connection::ConnectionType;
use gdk::CursorType;
use gio::prelude::*;
use glib::clone;
use glib::IsA;
use gtk::glib;
use gtk::prelude::*;
use gtk::Justification;
use parking_lot::RwLock;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::dbus_client;
use crate::device;
use crate::events;
use crate::notifications;
use crate::timers;
use crate::timers::TimerMode;
use crate::ui;
use crate::update_ui_state;
use crate::util;
use crate::util::ratelimited;
use crate::ConnectionState;
use crate::CssProviderExt;
use crate::STATE;
use crate::{switch_to_slot, switch_to_slot_and_profile};
use lazy_static::lazy_static;

use super::mice::initialize_mouse_page;
use super::Pages;

type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum MainError {
    #[error("Unknown profile: {description}")]
    UnknownProfileError { description: String },
}

lazy_static! {
    pub static ref SUBPAGES_INITIALIZED: AtomicBool = AtomicBool::new(false);

    /// The cursor type that will be set during the next iteration of the main loop
    pub static ref CURSOR_TYPE: Arc<RwLock<Option<CursorType>>> = Arc::new(RwLock::new(None));
}

/// Search the list of available profiles and return the
/// index of the profile associated with slot `slot_index`
fn find_profile_index(slot_index: usize, treestore: &gtk::TreeStore) -> Result<u32> {
    // fetch associated profiles for all slots
    let slot_profiles = util::get_slot_profiles()?;
    let slot_profile_path = PathBuf::from(&slot_profiles[slot_index]);

    let mut index = 0;
    let mut found = false;

    treestore.foreach(|model, _path, iter| {
        let file = model.value(iter, 2).to_value().get::<String>().unwrap();
        let path = PathBuf::from(&file);

        if slot_profile_path == path {
            found = true;
            return true;
        }

        index += 1;

        false
    });

    if found {
        Ok(index)
    } else {
        Err(MainError::UnknownProfileError {
            description: slot_profile_path.to_string_lossy().to_string(),
        }
        .into())
    }
}

/// Set the global state of the Eruption GUI application
/// e.g.: "Connected" or "Disconnected"
pub fn set_application_state(state: ConnectionState, builder: &gtk::Builder) -> Result<()> {
    // let notification_box_global: gtk::Box = builder.object("notification_box_global").unwrap();

    // let main_window: gtk::ApplicationWindow = builder.object("main_window").unwrap();

    // let outer_stack: gtk::Stack = builder.object("outer_stack").unwrap();
    // let main_stack: gtk::Stack = builder.object("main_stack").unwrap();

    // let restart_eruption_daemon_button: gtk::Button =
    //     builder.object("restart_eruption_button_global").unwrap();

    // let header_bar: gtk::HeaderBar = builder.object("header_bar").unwrap();

    // let switch_bar: gtk::Frame = builder.object("switch_bar").unwrap();
    // let slot_bar: gtk::Box = builder.object("slot_bar").unwrap();

    match state {
        ConnectionState::Initializing => {
            *crate::CONNECTION_STATE.write() = ConnectionState::Initializing;

            update_main_window(builder)?;
        }

        ConnectionState::Disconnected => {
            *crate::CONNECTION_STATE.write() = ConnectionState::Disconnected;

            events::LOST_CONNECTION.store(true, Ordering::SeqCst);
            events::GAINED_CONNECTION.store(false, Ordering::SeqCst);

            update_main_window(builder)?;
        }

        ConnectionState::Connected => {
            *crate::CONNECTION_STATE.write() = ConnectionState::Connected;

            events::LOST_CONNECTION.store(false, Ordering::SeqCst);
            events::GAINED_CONNECTION.store(true, Ordering::SeqCst);

            update_main_window(builder)?;
        }
    }

    Ok(())
}

/// Initialize the slot bar on the bottom of the main window
fn initialize_slot_bar(builder: &gtk::Builder) -> Result<()> {
    let window: gtk::Window = builder.object("main_window").unwrap();

    let slot1_frame: gtk::Frame = builder.object("slot1_frame").unwrap();
    let slot2_frame: gtk::Frame = builder.object("slot2_frame").unwrap();
    let slot3_frame: gtk::Frame = builder.object("slot3_frame").unwrap();
    let slot4_frame: gtk::Frame = builder.object("slot4_frame").unwrap();

    let slot1_radio_button: gtk::RadioButton = builder.object("slot1_radio_button").unwrap();
    let slot2_radio_button: gtk::RadioButton = builder.object("slot2_radio_button").unwrap();
    let slot3_radio_button: gtk::RadioButton = builder.object("slot3_radio_button").unwrap();
    let slot4_radio_button: gtk::RadioButton = builder.object("slot4_radio_button").unwrap();

    let slot1_entry: gtk::Entry = builder.object("slot1_entry").unwrap();
    let slot2_entry: gtk::Entry = builder.object("slot2_entry").unwrap();
    let slot3_entry: gtk::Entry = builder.object("slot3_entry").unwrap();
    let slot4_entry: gtk::Entry = builder.object("slot4_entry").unwrap();

    let edit_slot1_button: gtk::Button = builder.object("edit_slot1_button").unwrap();
    let edit_slot2_button: gtk::Button = builder.object("edit_slot2_button").unwrap();
    let edit_slot3_button: gtk::Button = builder.object("edit_slot3_button").unwrap();
    let edit_slot4_button: gtk::Button = builder.object("edit_slot4_button").unwrap();

    let _slot1_color_button: gtk::ColorButton = builder.object("slot1_color_button").unwrap();
    let _slot2_color_button: gtk::ColorButton = builder.object("slot1_color_button").unwrap();
    let _slot3_color_button: gtk::ColorButton = builder.object("slot1_color_button").unwrap();
    let _slot4_color_button: gtk::ColorButton = builder.object("slot1_color_button").unwrap();

    let slot1_combo: gtk::ComboBox = builder.object("slot1_combo").unwrap();
    let slot2_combo: gtk::ComboBox = builder.object("slot2_combo").unwrap();
    let slot3_combo: gtk::ComboBox = builder.object("slot3_combo").unwrap();
    let slot4_combo: gtk::ComboBox = builder.object("slot4_combo").unwrap();

    slot1_radio_button.set_action_name(Some("app.switch-to-slot-1"));
    slot2_radio_button.set_action_name(Some("app.switch-to-slot-2"));
    slot3_radio_button.set_action_name(Some("app.switch-to-slot-3"));
    slot4_radio_button.set_action_name(Some("app.switch-to-slot-4"));

    // slot names
    let names = util::get_slot_names()?;

    slot1_entry.set_text(names.first().unwrap_or(&"Profile Slot 1".to_string()));
    slot2_entry.set_text(names.get(1).unwrap_or(&"Profile Slot 2".to_string()));
    slot3_entry.set_text(names.get(2).unwrap_or(&"Profile Slot 3".to_string()));
    slot4_entry.set_text(names.get(3).unwrap_or(&"Profile Slot 4".to_string()));

    edit_slot1_button.connect_clicked(clone!(@weak window, @weak slot1_entry => move |_btn| {
            window.set_focus(Some(&slot1_entry));
    }));

    edit_slot2_button.connect_clicked(clone!(@weak window, @weak slot2_entry => move |_btn| {
            window.set_focus(Some(&slot2_entry));
    }));

    edit_slot3_button.connect_clicked(clone!(@weak window, @weak slot3_entry => move |_btn| {
        window.set_focus(Some(&slot3_entry));
    }));

    edit_slot4_button.connect_clicked(clone!(@weak window, @weak slot4_entry => move |_btn| {
        window.set_focus(Some(&slot4_entry));
    }));

    // slot1_color_button.connect_color_set(clone!(@weak window => move |btn| {
    //     let _color = btn.rgba();

    // }));

    // slot2_color_button.connect_color_set(clone!(@weak window => move |btn| {
    //     let _color = btn.rgba();

    // }));

    // slot3_color_button.connect_color_set(clone!(@weak window => move |btn| {
    //     let _color = btn.rgba();

    // }));

    // slot4_color_button.connect_color_set(clone!(@weak window => move |btn| {
    //     let _color = btn.rgba();

    // }));

    slot1_entry.connect_focus_out_event(|edit, _event| {
        let slot_name = edit.text().to_string();
        util::set_slot_name(0, &slot_name).unwrap_or_else(|e| tracing::error!("{}", e));

        false.into()
    });

    slot2_entry.connect_focus_out_event(|edit, _event| {
        let slot_name = edit.text().to_string();
        util::set_slot_name(1, &slot_name).unwrap_or_else(|e| tracing::error!("{}", e));

        false.into()
    });

    slot3_entry.connect_focus_out_event(|edit, _event| {
        let slot_name = edit.text().to_string();
        util::set_slot_name(2, &slot_name).unwrap_or_else(|e| tracing::error!("{}", e));

        false.into()
    });

    slot4_entry.connect_focus_out_event(|edit, _event| {
        let slot_name = edit.text().to_string();
        util::set_slot_name(3, &slot_name).unwrap_or_else(|e| tracing::error!("{}", e));

        false.into()
    });

    // profiles list
    let profiles_treestore = gtk::TreeStore::new(&[
        glib::Type::U64,
        String::static_type(),
        String::static_type(),
    ]);

    for profile in util::enumerate_profiles()? {
        let name = profile.name;
        let filename = profile
            .profile_file
            .to_string_lossy()
            .to_owned()
            .to_string();

        profiles_treestore.insert_with_values(None, None, &[(1, &name), (2, &filename)]);
    }

    let cell_renderer_id = gtk::CellRendererText::new();
    let cell_renderer_name = gtk::CellRendererText::new();
    let cell_renderer_filename = gtk::CellRendererText::new();

    slot1_combo.pack_start(&cell_renderer_name, true);

    slot1_combo.add_attribute(&cell_renderer_id, "text", 0);
    slot1_combo.add_attribute(&cell_renderer_name, "text", 1);
    slot1_combo.add_attribute(&cell_renderer_filename, "text", 2);

    slot1_combo.set_model(Some(&profiles_treestore));
    slot1_combo.show_all();
    slot1_combo.set_active(find_profile_index(0, &profiles_treestore).ok());

    slot1_combo.connect_changed(clone!(@weak profiles_treestore => move |cb| {
        let id = cb.active().unwrap();
        let entry = cb
            .model()
            .unwrap()
            .iter_nth_child(None, id as i32)
            .unwrap();

        let file = profiles_treestore
            .value(&entry, 2)
            .to_value().get::<String>()
            .unwrap();

        switch_to_slot_and_profile(0, file).unwrap();
    }));

    slot2_combo.pack_start(&cell_renderer_name, true);

    slot2_combo.add_attribute(&cell_renderer_id, "text", 0);
    slot2_combo.add_attribute(&cell_renderer_name, "text", 1);
    slot2_combo.add_attribute(&cell_renderer_filename, "text", 2);

    slot2_combo.set_model(Some(&profiles_treestore));
    slot2_combo.show_all();
    slot2_combo.set_active(find_profile_index(1, &profiles_treestore).ok());

    slot2_combo.connect_changed(clone!(@weak profiles_treestore => move |cb| {
        let id = cb.active().unwrap();
        let entry = cb
            .model()
            .unwrap()
            .iter_nth_child(None, id as i32)
            .unwrap();

        let file = profiles_treestore
            .value(&entry, 2)
            .to_value().get::<String>()
            .unwrap();

        switch_to_slot_and_profile(1, file).unwrap();
    }));

    slot3_combo.pack_start(&cell_renderer_name, true);

    slot3_combo.add_attribute(&cell_renderer_id, "text", 0);
    slot3_combo.add_attribute(&cell_renderer_name, "text", 1);
    slot3_combo.add_attribute(&cell_renderer_filename, "text", 2);

    slot3_combo.set_model(Some(&profiles_treestore));
    slot3_combo.show_all();
    slot3_combo.set_active(find_profile_index(2, &profiles_treestore).ok());

    slot3_combo.connect_changed(clone!(@weak profiles_treestore => move |cb| {
        let id = cb.active().unwrap();
        let entry = cb
            .model()
            .unwrap()
            .iter_nth_child(None, id as i32)
            .unwrap();

        let file = profiles_treestore
            .value(&entry, 2)
            .to_value().get::<String>()
            .unwrap();

        switch_to_slot_and_profile(2, file).unwrap();
    }));

    slot4_combo.pack_start(&cell_renderer_name, true);

    slot4_combo.add_attribute(&cell_renderer_id, "text", 0);
    slot4_combo.add_attribute(&cell_renderer_name, "text", 1);
    slot4_combo.add_attribute(&cell_renderer_filename, "text", 2);

    slot4_combo.set_model(Some(&profiles_treestore));
    slot4_combo.show_all();
    slot4_combo.set_active(find_profile_index(3, &profiles_treestore).ok());

    slot4_combo.connect_changed(clone!(@weak profiles_treestore => move |cb| {
        let id = cb.active().unwrap();
        let entry = cb
            .model()
            .unwrap()
            .iter_nth_child(None, id as i32)
            .unwrap();

        let file = profiles_treestore
            .value(&entry, 2)
            .to_value().get::<String>()
            .unwrap();

        switch_to_slot_and_profile(3, file).unwrap();
    }));

    events::ignore_next_ui_events(1);
    let active_slot = STATE.read().active_slot.unwrap();

    match active_slot {
        0 => {
            slot1_radio_button.set_active(true);

            let context = slot1_frame.style_context();
            context.add_class("active");

            let context = slot2_frame.style_context();
            context.remove_class("active");

            let context = slot3_frame.style_context();
            context.remove_class("active");

            let context = slot4_frame.style_context();
            context.remove_class("active");
        }

        1 => {
            slot2_radio_button.set_active(true);

            let context = slot1_frame.style_context();
            context.remove_class("active");

            let context = slot2_frame.style_context();
            context.add_class("active");

            let context = slot3_frame.style_context();
            context.remove_class("active");

            let context = slot4_frame.style_context();
            context.remove_class("active");
        }

        2 => {
            slot3_radio_button.set_active(true);

            let context = slot1_frame.style_context();
            context.remove_class("active");

            let context = slot2_frame.style_context();
            context.remove_class("active");

            let context = slot3_frame.style_context();
            context.add_class("active");

            let context = slot4_frame.style_context();
            context.remove_class("active");
        }

        3 => {
            slot4_radio_button.set_active(true);

            let context = slot1_frame.style_context();
            context.remove_class("active");

            let context = slot2_frame.style_context();
            context.remove_class("active");

            let context = slot3_frame.style_context();
            context.remove_class("active");

            let context = slot4_frame.style_context();
            context.add_class("active");
        }

        _ => panic!("Invalid slot index"),
    };

    events::reenable_ui_events();

    Ok(())
}

fn update_slot_indicator_state(builder: &gtk::Builder, active_slot: usize) {
    let slot1_frame: gtk::Frame = builder.object("slot1_frame").unwrap();
    let slot2_frame: gtk::Frame = builder.object("slot2_frame").unwrap();
    let slot3_frame: gtk::Frame = builder.object("slot3_frame").unwrap();
    let slot4_frame: gtk::Frame = builder.object("slot4_frame").unwrap();

    // let slot1_radio_button: gtk::RadioButton = builder.object("slot1_radio_button").unwrap();
    // let slot2_radio_button: gtk::RadioButton = builder.object("slot2_radio_button").unwrap();
    // let slot3_radio_button: gtk::RadioButton = builder.object("slot3_radio_button").unwrap();
    // let slot4_radio_button: gtk::RadioButton = builder.object("slot4_radio_button").unwrap();

    // events::ignore_next_ui_events(1);
    // let active_slot = STATE.read().active_slot.unwrap();

    match active_slot {
        0 => {
            // slot1_radio_button.set_active(true);

            let context = slot1_frame.style_context();
            context.add_class("active");

            let context = slot2_frame.style_context();
            context.remove_class("active");

            let context = slot3_frame.style_context();
            context.remove_class("active");

            let context = slot4_frame.style_context();
            context.remove_class("active");
        }

        1 => {
            // slot2_radio_button.set_active(true);

            let context = slot1_frame.style_context();
            context.remove_class("active");

            let context = slot2_frame.style_context();
            context.add_class("active");

            let context = slot3_frame.style_context();
            context.remove_class("active");

            let context = slot4_frame.style_context();
            context.remove_class("active");
        }

        2 => {
            // slot3_radio_button.set_active(true);

            let context = slot1_frame.style_context();
            context.remove_class("active");

            let context = slot2_frame.style_context();
            context.remove_class("active");

            let context = slot3_frame.style_context();
            context.add_class("active");

            let context = slot4_frame.style_context();
            context.remove_class("active");
        }

        3 => {
            // slot4_radio_button.set_active(true);

            let context = slot1_frame.style_context();
            context.remove_class("active");

            let context = slot2_frame.style_context();
            context.remove_class("active");

            let context = slot3_frame.style_context();
            context.remove_class("active");

            let context = slot4_frame.style_context();
            context.add_class("active");
        }

        _ => panic!("Invalid slot index"),
    };

    // events::reenable_ui_events();
}

pub fn switch_main_stack_page(to: Pages, builder: &gtk::Builder) {
    let main_stack: gtk::Stack = builder.object("main_stack").unwrap();

    match to {
        Pages::Canvas => main_stack.set_visible_child_name("page0"),
        Pages::Keyboards => main_stack.set_visible_child_name("page1"),
        Pages::Mice => main_stack.set_visible_child_name("page2"),
        Pages::Misc => main_stack.set_visible_child_name("page3"),
        Pages::ColorSchemes => main_stack.set_visible_child_name("page4"),
        Pages::AutomationRules => main_stack.set_visible_child_name("page5"),
        Pages::Profiles => main_stack.set_visible_child_name("page6"),
        Pages::Macros => main_stack.set_visible_child_name("page7"),
        Pages::Keymaps => main_stack.set_visible_child_name("page8"),
        Pages::Settings => main_stack.set_visible_child_name("page9"),
    }
}

/// Register global actions and keyboard accelerators
fn register_actions<A: IsA<gtk::Application>>(
    application: &A,
    builder: &gtk::Builder,
) -> Result<()> {
    let application = application.as_ref();

    let _main_window: gtk::ApplicationWindow = builder.object("main_window").unwrap();

    // let stack_switcher: gtk::StackSwitcher = builder.object("stack_switcher").unwrap();
    let _main_stack: gtk::Stack = builder.object("main_stack").unwrap();

    // switching between stack pages
    let switch_to_page1 = gio::SimpleAction::new("switch-to-page-1", None);
    switch_to_page1.connect_activate(clone!(@weak builder => move |_, _| {
        switch_main_stack_page(Pages::Canvas, &builder);
    }));

    application.add_action(&switch_to_page1);
    application.set_accels_for_action("app.switch-to-page-1", &["<alt>1"]);

    let switch_to_page2 = gio::SimpleAction::new("switch-to-page-2", None);
    switch_to_page2.connect_activate(clone!(@weak builder => move |_, _| {
        switch_main_stack_page(Pages::Keyboards, &builder);
    }));

    application.add_action(&switch_to_page2);
    application.set_accels_for_action("app.switch-to-page-2", &["<alt>2"]);

    let switch_to_page3 = gio::SimpleAction::new("switch-to-page-3", None);
    switch_to_page3.connect_activate(clone!(@weak builder => move |_, _| {
        switch_main_stack_page(Pages::Mice, &builder);

    }));

    application.add_action(&switch_to_page3);
    application.set_accels_for_action("app.switch-to-page-3", &["<alt>3"]);

    let switch_to_page4 = gio::SimpleAction::new("switch-to-page-4", None);
    switch_to_page4.connect_activate(clone!(@weak builder => move |_, _| {
        switch_main_stack_page(Pages::Misc, &builder);

    }));

    application.add_action(&switch_to_page4);
    application.set_accels_for_action("app.switch-to-page-4", &["<alt>4"]);

    let switch_to_page5 = gio::SimpleAction::new("switch-to-page-5", None);
    switch_to_page5.connect_activate(clone!(@weak builder => move |_, _| {
        switch_main_stack_page(Pages::ColorSchemes, &builder);

    }));

    application.add_action(&switch_to_page5);
    application.set_accels_for_action("app.switch-to-page-5", &["<alt>5"]);

    let switch_to_page6 = gio::SimpleAction::new("switch-to-page-6", None);
    switch_to_page6.connect_activate(clone!(@weak builder => move |_, _| {
        switch_main_stack_page(Pages::AutomationRules, &builder);

    }));

    application.add_action(&switch_to_page6);
    application.set_accels_for_action("app.switch-to-page-6", &["<alt>6"]);

    let switch_to_page7 = gio::SimpleAction::new("switch-to-page-7", None);
    switch_to_page7.connect_activate(clone!(@weak builder => move |_, _| {
        switch_main_stack_page(Pages::Profiles, &builder);

    }));

    application.add_action(&switch_to_page7);
    application.set_accels_for_action("app.switch-to-page-7", &["<alt>7"]);

    let switch_to_page8 = gio::SimpleAction::new("switch-to-page-8", None);
    switch_to_page8.connect_activate(clone!(@weak builder => move |_, _| {
        switch_main_stack_page(Pages::Macros, &builder);


    }));

    application.add_action(&switch_to_page8);
    application.set_accels_for_action("app.switch-to-page-8", &["<alt>8"]);

    let switch_to_page9 = gio::SimpleAction::new("switch-to-page-9", None);
    switch_to_page9.connect_activate(clone!(@weak builder => move |_, _| {
        switch_main_stack_page(Pages::Keymaps, &builder);

    }));

    application.add_action(&switch_to_page9);
    application.set_accels_for_action("app.switch-to-page-9", &["<alt>9"]);

    let switch_to_page10 = gio::SimpleAction::new("switch-to-page-10", None);
    switch_to_page10.connect_activate(clone!(@weak builder => move |_, _| {
        switch_main_stack_page(Pages::Settings, &builder);

    }));

    application.add_action(&switch_to_page10);
    application.set_accels_for_action("app.switch-to-page-10", &["<alt>0"]);

    // switching between slots
    let switch_to_slot1 = gio::SimpleAction::new("switch-to-slot-1", None);
    switch_to_slot1.connect_activate(clone!(@weak builder => move |_, _| {
        if !events::shall_ignore_pending_ui_event() {
            match switch_to_slot(0) {
                Ok(()) => update_slot_indicator_state(&builder, 0),
                Err(e) => tracing::error!("Could not switch slots: {e}"),
            }
        }
    }));

    application.add_action(&switch_to_slot1);
    // application.set_accels_for_action("app.switch-to-slot-1", &["<Primary>1"]);

    let switch_to_slot2 = gio::SimpleAction::new("switch-to-slot-2", None);
    switch_to_slot2.connect_activate(clone!(@weak builder => move |_, _| {
        if !events::shall_ignore_pending_ui_event() {
            match switch_to_slot(1) {
                Ok(()) => update_slot_indicator_state(&builder, 1),
                Err(e) => tracing::error!("Could not switch slots: {e}"),
            }
        }
    }));

    application.add_action(&switch_to_slot2);
    // application.set_accels_for_action("app.switch-to-slot-2", &["<Primary>2"]);

    let switch_to_slot3 = gio::SimpleAction::new("switch-to-slot-3", None);
    switch_to_slot3.connect_activate(clone!(@weak builder => move |_, _| {
        if !events::shall_ignore_pending_ui_event() {
            match switch_to_slot(2) {
                Ok(()) => update_slot_indicator_state(&builder, 2),
                Err(e) => tracing::error!("Could not switch slots: {e}"),
            }
        }
    }));

    application.add_action(&switch_to_slot3);
    // application.set_accels_for_action("app.switch-to-slot-3", &["<Primary>3"]);

    let switch_to_slot4 = gio::SimpleAction::new("switch-to-slot-4", None);
    switch_to_slot4.connect_activate(clone!(@weak builder => move |_, _| {
        if !events::shall_ignore_pending_ui_event() {
            match switch_to_slot(3) {
                Ok(()) => update_slot_indicator_state(&builder, 3),
                Err(e) => tracing::error!("Could not switch slots: {e}"),
            }
        }
    }));

    application.add_action(&switch_to_slot4);
    // application.set_accels_for_action("app.switch-to-slot-4", &["<Primary>4"]);

    let quit = gio::SimpleAction::new("quit", None);
    quit.connect_activate(clone!(@weak application => move |_, _| {
        application.quit();
    }));

    application.add_action(&quit);
    application.set_accels_for_action("app.quit", &["<Primary>Q"]);

    // let action_group = gio::SimpleActionGroup::new();

    // action_group.add_action(&switch_to_page1);
    // action_group.add_action(&switch_to_page2);
    // action_group.add_action(&switch_to_page3);
    // action_group.add_action(&switch_to_page4);
    // action_group.add_action(&switch_to_page5);
    // action_group.add_action(&switch_to_page6);
    // action_group.add_action(&switch_to_page7);
    // action_group.add_action(&switch_to_page8);
    // action_group.add_action(&switch_to_page9);

    // action_group.add_action(&switch_to_slot1);
    // action_group.add_action(&switch_to_slot2);
    // action_group.add_action(&switch_to_slot3);
    // action_group.add_action(&switch_to_slot4);

    // application.set_action_group(Some(&action_group));

    Ok(())
}

fn initialize_sub_pages_and_spawn_dbus_threads(
    application: &gtk::Application,
    builder: &gtk::Builder,
) {
    // let _ = update_main_window(builder).map_err(|e| {
    //     tracing::error!("Error updating the main window: {e:?}");
    //     e
    // });

    let _ = ui::canvas::initialize_canvas_page(builder).map_err(|e| {
        tracing::error!("Error updating the canvas page: {e:?}");
        e
    });

    // let _ = ui::keyboards::initialize_keyboard_page(builder).map_err(|e| {
    //     tracing::error!("Error updating the keyboard devices page: {e:?}");
    //     e
    // });

    // let _ = ui::mice::initialize_mouse_page(builder).map_err(|e| {
    //     tracing::error!("Error updating the mouse devices page: {e:?}");
    //     e
    // });

    // let _ = ui::misc::initialize_misc_page(builder).map_err(|e| {
    //     tracing::error!("Error updating the misc devices page: {e:?}");
    //     e
    // });

    let _ = ui::color_schemes::initialize_color_schemes_page(&application.clone(), builder)
        .map_err(|e| {
            tracing::error!("Error updating the color schemes page: {e:?}");
            e
        });

    let _ = ui::automation_rules::initialize_automation_rules_page(&application.clone(), builder)
        .map_err(|e| {
            tracing::error!("Error updating the color schemes page: {e:?}");
            e
        });

    let _ = ui::profiles::initialize_profiles_page(&application.clone(), builder).map_err(|e| {
        tracing::error!("Error updating the main window: {e:?}");
        e
    });

    // let _ = ui::macros::initialize_macros_page(&application.clone(), builder).map_err(|e| {
    //     tracing::error!("Error updating the canvas page: {e:?}");
    //     e
    // });

    // let _ = ui::keymaps::initialize_keymaps_page(&application.clone(), builder).map_err(|e| {
    //     tracing::error!("Error updating the main window: {e:?}");
    //     e
    // });

    let _ = ui::settings::initialize_settings_page(builder).map_err(|e| {
        tracing::error!("Error updating the main window: {e:?}");
        e
    });

    let _ = initialize_slot_bar(builder).map_err(|e| {
        tracing::error!("Error updating the main window: {e:?}");
        e
    });

    let _ = dbus_client::spawn_dbus_event_loop_system(builder, &update_ui_state);
    let _ = dbus_client::spawn_dbus_event_loop_session(builder, &update_ui_state);
}

// fn update_sub_pages_and_spawn_dbus_threads(builder: &gtk::Builder) {
//     // let _ = update_main_window(builder).map_err(|e| {
//     //     tracing::error!("Error updating the main window: {e:?}");
//     //     e
//     // });

//     let _ = ui::canvas::update_canvas_page(builder).map_err(|e| {
//         tracing::error!("Error updating the canvas page: {e:?}");
//         e
//     });

//     // let _ = ui::keyboards::update_keyboard_page(builder).map_err(|e| {
//     //     tracing::error!("Error updating the keyboard devices page: {e:?}");
//     //     e
//     // });

//     // let _ = ui::mice::update_mouse_page(builder).map_err(|e| {
//     //     tracing::error!("Error updating the mouse devices page: {e:?}");
//     //     e
//     // });

//     // let _ = ui::misc::update_misc_page(builder).map_err(|e| {
//     //     tracing::error!("Error updating the misc devices page: {e:?}");
//     //     e
//     // });

//     let _ = ui::color_schemes::update_color_schemes_page(builder).map_err(|e| {
//         tracing::error!("Error updating the color schemes page: {e:?}");
//         e
//     });

//     let _ = ui::automation_rules::update_automation_rules_page(builder).map_err(|e| {
//         tracing::error!("Error updating the color schemes page: {e:?}");
//         e
//     });

//     // let _ = ui::profiles::update_profiles_page(&application.clone(), builder).map_err(|e| {
//     //     tracing::error!("Error updating the profiles page: {e:?}");
//     //     e
//     // });

//     // let _ = ui::macros::update_macros_page(&application.clone(), builder).map_err(|e| {
//     //     tracing::error!("Error updating the macros page: {e:?}");
//     //     e
//     // });

//     // let _ = ui::keymaps::update_keymaps_page(&application.clone(), builder).map_err(|e| {
//     //     tracing::error!("Error updating the keymaps page: {e:?}");
//     //     e
//     // });

//     // let _ = ui::settings::update_settings_page(builder).map_err(|e| {
//     //     tracing::error!("Error updating the settings page: {e:?}");
//     //     e
//     // });

//     let _ = initialize_slot_bar(builder).map_err(|e| {
//         tracing::error!("Error updating the main window: {e:?}");
//         e
//     });

//     let _ = dbus_client::spawn_dbus_event_loop_system(builder, &update_ui_state);
//     let _ = dbus_client::spawn_dbus_event_loop_session(builder, &|_b, m| {
//         tracing::error!("{m:?}");
//         Ok(())
//     });
// }

/// Build and show the UI of the main application window
pub fn initialize_main_window<A: IsA<gtk::Application>>(application: &A) -> Result<()> {
    let application = application.as_ref();

    // we need to instantiate the GtkSourceView here at least once to
    // register it wih the GTK/GLib type system, so that gtk::Builder
    // is able to load it later on

    #[cfg(feature = "sourceview")]
    {
        let _temporary_sourceview = sourceview4::View::new();
    }

    // build UI
    let builder = gtk::Builder::from_resource("/org/eruption/eruption-gui-gtk3/ui/main.glade");

    let info_bar_box: gtk::Box = builder.object("info_bar_box").unwrap();

    let _notification_box_global: gtk::Box = builder.object("notification_box_global").unwrap();

    let main_window: gtk::ApplicationWindow = builder.object("main_window").unwrap();

    let main_stack: gtk::Stack = builder.object("main_stack").unwrap();

    let restart_eruption_daemon_button: gtk::Button =
        builder.object("restart_eruption_button_global").unwrap();

    let header_bar: gtk::HeaderBar = builder.object("header_bar").unwrap();
    let brightness_scale: gtk::Scale = builder.object("brightness_scale").unwrap();

    let about_item: gtk::MenuItem = builder.object("about_item").unwrap();
    let quit_item: gtk::MenuItem = builder.object("quit_item").unwrap();

    let about_button: gtk::Button = builder.object("about_button").unwrap();
    let info_button: gtk::Button = builder.object("info_button").unwrap();
    let lock_button: gtk::LockButton = builder.object("lock_button").unwrap();

    let ambientfx_switch: gtk::Switch = builder.object("ambientfx_switch").unwrap();
    let soundfx_switch: gtk::Switch = builder.object("soundfx_switch").unwrap();

    // enable custom CSS support
    let screen = gtk::prelude::GtkWindowExt::screen(&main_window).unwrap();
    let style = gtk::CssProvider::new();

    #[cfg(not(target_os = "windows"))]
    gtk::CssProvider::load_from_resource(
        &style,
        "/org/eruption/eruption-gui-gtk3/styles/default.css",
    );

    #[cfg(target_os = "windows")]
    gtk::CssProvider::load_from_resource(
        &style,
        "/org/eruption/eruption-gui-gtk3/styles/windows.css",
    );

    gtk::StyleContext::add_provider_for_screen(&screen, &style, gtk::STYLE_PROVIDER_PRIORITY_USER);

    // configure main window
    main_window.set_application(Some(application));
    main_window.set_position(gtk::WindowPosition::Center);
    main_window.set_title("Eruption GUI");
    main_window.set_icon_name(Some(
        "/org/eruption/eruption-gui-gtk3/img/eruption-logo.png",
    ));

    main_window.connect_delete_event(
        clone!(@weak application => @default-return glib::Propagation::Proceed, move |_, _| {
            application.quit();

            glib::Propagation::Proceed
        }),
    );

    register_actions(application, &builder)?;

    header_bar.set_subtitle(Some(&format!(
        "Version: {}",
        std::env!("CARGO_PKG_VERSION")
    )));

    main_stack.connect_visible_child_notify(|stack| {
        let name = stack.visible_child_name().unwrap();
        let name = name.as_str();

        tracing::debug!("Switched to stack page {}", name);

        match name {
            "page0" => crate::ACTIVE_PAGE.store(0, Ordering::SeqCst),
            "page1" => crate::ACTIVE_PAGE.store(1, Ordering::SeqCst),
            "page2" => crate::ACTIVE_PAGE.store(2, Ordering::SeqCst),
            "page3" => crate::ACTIVE_PAGE.store(3, Ordering::SeqCst),
            "page4" => crate::ACTIVE_PAGE.store(4, Ordering::SeqCst),
            "page5" => crate::ACTIVE_PAGE.store(5, Ordering::SeqCst),
            "page6" => crate::ACTIVE_PAGE.store(6, Ordering::SeqCst),
            "page7" => crate::ACTIVE_PAGE.store(7, Ordering::SeqCst),
            "page8" => crate::ACTIVE_PAGE.store(8, Ordering::SeqCst),
            "page9" => crate::ACTIVE_PAGE.store(9, Ordering::SeqCst),
            "page10" => crate::ACTIVE_PAGE.store(10, Ordering::SeqCst),

            // "No Connection" | "NoConnection" => crate::ACTIVE_PAGE.store(11, Ordering::SeqCst),
            _ => {
                tracing::error!("Could not get the name of the active stack page");
            }
        }
    });

    let message_label = gtk::Label::builder()
        .hexpand(true)
        .justify(Justification::Center)
        .build();
    let info_bar = gtk::InfoBar::builder()
        .parent(&info_bar_box)
        .show_close_button(true)
        .has_focus(false)
        .build();

    // info_bar.style_context().add_class("infobar");

    info_bar.set_message_type(gtk::MessageType::Other);
    info_bar.content_area().add(&message_label);

    info_bar.set_visible(false);

    // wire-up the gtk::InfoBar support
    notifications::set_notification_area(&info_bar);
    // notifications::info("Welcome to the Eruption GUI");

    // TODO: implement this
    // lock_button.set_permission();

    about_button.connect_clicked(clone!(@weak main_window => move |_|   {
        ui::about_dialog::show_about_dialog(&main_window);

    }));

    info_button.connect_clicked(clone!(@weak application => move |_|   {
        // let section1 = gtk::ShortcutsSection::builder().title("Eruption GUI").build();
        // let section2 = gtk::ShortcutsSection::builder().title("Automation Rules").build();

        let shortcuts_window =  gtk::ShortcutsWindow::builder().application(&application).build();
        shortcuts_window.show();
    }));

    lock_button.connect_clicked(|_btn| {
        let _result = dbus_client::ping_privileged();
    });

    // main menu items
    about_item.connect_activate(clone!(@weak main_window => move |_| {
        ui::about_dialog::show_about_dialog(&main_window);
    }));

    quit_item.connect_activate(clone!(@weak application => move |_| {
        application.quit();
    }));

    // brightness

    // no need to ignore events here, since handler is not connected
    brightness_scale.set_value(util::get_brightness().unwrap_or(0) as f64);

    brightness_scale.connect_value_changed(move |s| {
        if !events::shall_ignore_pending_ui_event() {
            util::set_brightness(s.value() as i64).unwrap();
        }
    });

    restart_eruption_daemon_button.connect_clicked(
        clone!(@weak application, @weak builder => move |_| {
            util::restart_eruption_daemon().unwrap_or_else(|e| tracing::error!("{}", e));
            update_main_window(&builder).unwrap_or_else(|e| tracing::error!("{}", e));
        }),
    );

    // special options
    ambientfx_switch.connect_state_set(
        clone!(@weak main_window => @default-return glib::Propagation::Proceed, move |_sw, enabled| {
            util::set_ambient_effect(enabled).unwrap_or_else(|e| { tracing::error!("{}", e); });

            glib::Propagation::Proceed
        }),
    );

    soundfx_switch.connect_state_set(
        clone!(@weak main_window => @default-return glib::Propagation::Proceed, move |_sw, enabled| {
            util::set_sound_fx(enabled).unwrap_or_else(|e| { tracing::error!("{}", e); });

            glib::Propagation::Proceed
        }),
    );

    initialize_sub_pages_and_spawn_dbus_threads(application, &builder);

    main_window.show_all();
    update_main_window(&builder)?;

    // timers::register_timer(
    //     timers::GLOBAL_CONFIG_TIMER_ID,
    //     TimerMode::Periodic,
    //     1000,
    //     clone!(@weak application, @weak builder, @weak ambientfx_switch, @weak soundfx_switch => @default-return Ok(()), move || {

    //         // ambientfx_switch.set_active(util::get_ambient_fx().unwrap_or(false));
    //         // soundfx_switch.set_active(util::get_sound_fx().unwrap_or(false));

    //         Ok(())
    //     }),
    // )?;

    // should we update the GUI, e.g. were new devices attached?
    timers::register_timer(
        timers::HOTPLUG_TIMER_ID,
        TimerMode::Periodic,
        500,
        clone!(@weak application => @default-return Ok(()), move || {
            // if crate::dbus_client::ping().is_err() {
            //     set_application_state(ConnectionState::Disconnected, &builder)?;

            //     notification_box_global.show();

            //     tracing::error!("Lost connection to the Eruption daemon");
            //     events::LOST_CONNECTION.store(true, Ordering::SeqCst);

            //     timers::clear_timers()?;

            // } else {

            //     notification_box_global.hide();

            //     if events::LOST_CONNECTION.load(Ordering::SeqCst) {
            //         tracing::info!("Re-establishing connection to the Eruption daemon...");

            //         // we re-established the connection to the Eruption daemon,
            //         events::LOST_CONNECTION.store(false, Ordering::SeqCst);

            //         // initialize_sub_pages_and_spawn_dbus_threads(&application, &builder);
            //     }
            // }

            if events::LOST_CONNECTION.load(Ordering::SeqCst) {
                set_application_state(ConnectionState::Disconnected, &builder)?;
                crate::LOST_CONNECTION.store(false, Ordering::SeqCst);

                notifications::warn(
                    "Please restart the Eruption daemon to re-establish the connection",
                );

                update_main_window(&builder).unwrap_or_else(|e| tracing::error!("Error updating the main window: {e}"));

            } else if events::GAINED_CONNECTION.load(Ordering::SeqCst) {
                set_application_state(ConnectionState::Connected, &builder)?;
                events::GAINED_CONNECTION.store(false, Ordering::SeqCst);

                let mut state = STATE.write();

                state.active_slot = util::get_active_slot().ok();
                state.active_profile = util::get_active_profile().ok();

                let _ = dbus_client::spawn_dbus_event_loop_system(&builder, &update_ui_state);
                let _ = dbus_client::spawn_dbus_event_loop_session(&builder, &update_ui_state);

                update_main_window(&builder).unwrap_or_else(|e| tracing::error!("Error updating the main window: {e}"));
            }

            if *crate::CONNECTION_STATE.read() == ConnectionState::Initializing ||
               *crate::CONNECTION_STATE.read() == ConnectionState::Disconnected {
                set_application_state(ConnectionState::Initializing, &builder)?;

                // connect to Eruption daemon
                match Connection::new(ConnectionType::Local) {
                    Ok(connection) => {

                        if let Err(e) = connection.connect() {
                            tracing::error!("Could not connect to Eruption daemon: {e}");

                            *crate::CONNECTION_STATE.write() = ConnectionState::Disconnected;
                            events::LOST_CONNECTION.store(true, Ordering::SeqCst);
                        } else {
                            let _ = connection
                                .get_server_status()
                                .map_err(|e| tracing::error!("{e}"));

                            *crate::CONNECTION.lock() = Some(connection);

                            notifications::info(
                                "Successfully re-established connection to the Eruption daemon",
                            );

                            *crate::CONNECTION_STATE.write() = ConnectionState::Connected;
                            events::LOST_CONNECTION.store(true, Ordering::SeqCst);
                        }
                    }

                    Err(e) => {
                        tracing::error!("Could not connect to Eruption daemon: {}", e);

                        *crate::CONNECTION_STATE.write() = ConnectionState::Disconnected;
                        events::LOST_CONNECTION.store(true, Ordering::SeqCst);
                    }
                }

                let mut state = STATE.write();

                state.active_slot = util::get_active_slot().ok();
                state.active_profile = util::get_active_profile().ok();

                let _ = dbus_client::spawn_dbus_event_loop_system(&builder, &update_ui_state);
                let _ = dbus_client::spawn_dbus_event_loop_session(&builder, &update_ui_state);

                update_main_window(&builder).unwrap_or_else(|e| tracing::error!("Error updating the main window: {e}"));
            }

            if events::UPDATE_MAIN_WINDOW.load(Ordering::SeqCst) {
                events::UPDATE_MAIN_WINDOW.store(false, Ordering::SeqCst);

                update_main_window(&builder).unwrap_or_else(|e| tracing::error!("Error updating the main window: {e}"));
            }

            Ok(())
        }),
    )?;

    // update the global LED color map vector
    timers::register_timer(
        timers::COLOR_MAP_TIMER_ID,
        TimerMode::Periodic,
        1000 / (crate::constants::TARGET_FPS_LIMIT * 2),
        move || {
            // HACK: hijack this timer routine to set the application-wide cursor shape
            if let Some(window) = main_window.window() {
                let display = window.display();

                if let Some(cursor_type) = *CURSOR_TYPE.read() {
                    window.set_cursor(gdk::Cursor::for_display(&display, cursor_type).as_ref());
                } else {
                    window.set_cursor(None);
                }
            }

            let page = crate::ACTIVE_PAGE.load(Ordering::SeqCst);
            if page == Pages::Canvas as usize
                || page == Pages::Keyboards as usize
                || page == Pages::Mice as usize
                || page == Pages::Misc as usize
            {
                crate::update_canvas().map_err(|e| {
                    crate::LOST_CONNECTION.store(true, Ordering::SeqCst);

                    ratelimited::error!("Could not update the color map: {e}");
                    e
                })?;
            }

            Ok(())
        },
    )?;

    SUBPAGES_INITIALIZED.store(true, Ordering::SeqCst);

    Ok(())
}

pub fn update_main_window(builder: &gtk::Builder) -> Result<()> {
    let outer_stack: gtk::Stack = builder.object("outer_stack").unwrap();
    // let main_stack: gtk::Stack = builder.object("main_stack").unwrap();

    let stack_switcher: gtk::StackSwitcher = builder.object("stack_switcher").unwrap();
    let brightness_scale: gtk::Scale = builder.object("brightness_scale").unwrap();

    // let canvas_stack: gtk::Stack = builder.object("canvas_stack").unwrap();
    let keyboard_devices_stack: gtk::Stack = builder.object("keyboard_devices_stack").unwrap();
    let mouse_devices_stack: gtk::Stack = builder.object("mouse_devices_stack").unwrap();
    let misc_devices_stack: gtk::Stack = builder.object("misc_devices_stack").unwrap();

    // // clean up all previously instantiated sub-pages
    // while canvas_stack.children().len() > 2 {
    //     let child = &canvas_stack.children()[2];
    //     canvas_stack.remove(child);

    //     unsafe {
    //         child.destroy();
    //     }
    // }

    // hide unified canvas page
    // let child = &canvas_stack.children()[0];
    // child.hide();

    while !keyboard_devices_stack.children().is_empty() {
        let child = &keyboard_devices_stack.children()[0];
        keyboard_devices_stack.remove(child);

        unsafe {
            child.destroy();
        }
    }

    while !mouse_devices_stack.children().is_empty() {
        let child = &mouse_devices_stack.children()[0];
        mouse_devices_stack.remove(child);

        unsafe {
            child.destroy();
        }
    }

    while !misc_devices_stack.children().is_empty() {
        let child = &misc_devices_stack.children()[0];
        misc_devices_stack.remove(child);

        unsafe {
            child.destroy();
        }
    }

    if *crate::CONNECTION_STATE.read() == ConnectionState::Initializing {
        outer_stack.set_visible_child_name("connecting");

        stack_switcher.set_visible(false);
        brightness_scale.set_visible(false);
    } else if *crate::CONNECTION_STATE.read() == ConnectionState::Disconnected {
        // outer_stack.set_visible_child_name("no_connection");
        outer_stack.set_visible_child_name("connecting");

        stack_switcher.set_visible(false);
        brightness_scale.set_visible(false);

        let no_device_template =
            gtk::Builder::from_resource("/org/eruption/eruption-gui-gtk3/ui/no-device-template.ui");

        let page: gtk::Grid = no_device_template.object("no_device_template").unwrap();

        keyboard_devices_stack.add_titled(&page, "None", "None");

        let no_device_template =
            gtk::Builder::from_resource("/org/eruption/eruption-gui-gtk3/ui/no-device-template.ui");

        let page: gtk::Grid = no_device_template.object("no_device_template").unwrap();

        mouse_devices_stack.add_titled(&page, "None", "None");

        let no_device_template =
            gtk::Builder::from_resource("/org/eruption/eruption-gui-gtk3/ui/no-device-template.ui");

        let page: gtk::Grid = no_device_template.object("no_device_template").unwrap();

        misc_devices_stack.add_titled(&page, "None", "None");
    } else if *crate::CONNECTION_STATE.read() == ConnectionState::Connected {
        outer_stack.set_visible_child_name("eruption_gui");

        stack_switcher.set_visible(true);
        brightness_scale.set_visible(true);

        // instantiate the devices sub-pages
        let devices = dbus_client::get_managed_devices()?;

        ui::canvas::fetch_device_info(builder)?;
        ui::canvas::fetch_allocated_zones(builder)?;

        let mut device_index = 0;

        let mut any_keyboard_device = false;
        let mut any_mouse_device = false;
        let mut any_misc_device = false;

        // // clean up all previously instantiated sub-pages on the canvas stack
        // while canvas_stack.children().len() > 2 {
        //     let child = &canvas_stack.children()[2];
        //     canvas_stack.remove(child);

        //     unsafe {
        //         child.destroy();
        //     }
        // }

        // show pages
        // let child = &canvas_stack.children()[0];
        // child.show_all();

        // let child = &canvas_stack.children()[1];
        // child.show_all();

        let _ = ui::canvas::update_canvas_page(builder).map_err(|e| {
            tracing::error!("Error updating the canvas page: {e:?}");
            e
        });

        // instantiate stack pages for all keyboard devices
        for (_device, (vid, pid)) in devices.0.iter().enumerate() {
            let template = gtk::Builder::from_resource(
                "/org/eruption/eruption-gui-gtk3/ui/keyboard-device-template.ui",
            );

            let page =
                ui::keyboards::initialize_keyboard_page(builder, &template, device_index as u64)?;

            let device_name = format!(
                "{} {}",
                device::get_device_make(*vid, *pid).unwrap_or("<unknown>"),
                device::get_device_model(*vid, *pid).unwrap_or("<unknown>")
            );

            keyboard_devices_stack.add_titled(&page, &device_name, &device_name);

            device_index += 1;

            any_keyboard_device = true;
        }

        if !any_keyboard_device {
            let no_device_template = gtk::Builder::from_resource(
                "/org/eruption/eruption-gui-gtk3/ui/no-device-template.ui",
            );

            let page: gtk::Grid = no_device_template.object("no_device_template").unwrap();

            keyboard_devices_stack.add_titled(&page, "None", "None");
        }

        // instantiate stack pages for all mouse devices
        for (_device, (vid, pid)) in devices.1.iter().enumerate() {
            let template = gtk::Builder::from_resource(
                "/org/eruption/eruption-gui-gtk3/ui/mouse-device-template.ui",
            );

            let page = initialize_mouse_page(builder, &template, device_index as u64)?;

            let device_name = format!(
                "{} {}",
                device::get_device_make(*vid, *pid).unwrap_or("<unknown>"),
                device::get_device_model(*vid, *pid).unwrap_or("<unknown>")
            );

            mouse_devices_stack.add_titled(&page, &device_name, &device_name);

            device_index += 1;

            any_mouse_device = true;
        }

        if !any_mouse_device {
            let no_device_template = gtk::Builder::from_resource(
                "/org/eruption/eruption-gui-gtk3/ui/no-device-template.ui",
            );

            let page: gtk::Grid = no_device_template.object("no_device_template").unwrap();

            mouse_devices_stack.add_titled(&page, "None", "None");
        }

        // instantiate stack pages for all miscellaneous devices
        for (_device, (vid, pid)) in devices.2.iter().enumerate() {
            let template = gtk::Builder::from_resource(
                "/org/eruption/eruption-gui-gtk3/ui/misc-device-template.ui",
            );

            let page = ui::misc::initialize_misc_page(builder, &template, device_index as u64)?;

            let device_name = format!(
                "{} {}",
                device::get_device_make(*vid, *pid).unwrap_or("<unknown>"),
                device::get_device_model(*vid, *pid).unwrap_or("<unknown>")
            );

            misc_devices_stack.add_titled(&page, &device_name, &device_name);

            device_index += 1;

            any_misc_device = true;
        }

        if !any_misc_device {
            let no_device_template = gtk::Builder::from_resource(
                "/org/eruption/eruption-gui-gtk3/ui/no-device-template.ui",
            );

            let page: gtk::Grid = no_device_template.object("no_device_template").unwrap();

            misc_devices_stack.add_titled(&page, "None", "None");
        }
    }

    Ok(())
}
