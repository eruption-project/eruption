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

use gio::prelude::*;
use glib::clone;
use glib::IsA;
use gtk::glib;
use gtk::prelude::*;
use std::path::PathBuf;
use std::sync::atomic::Ordering;

use crate::dbus_client;
use crate::device;
use crate::events;
use crate::ui;
use crate::update_ui_state;
use crate::util;
use crate::CssProviderExt;
use crate::STATE;
use crate::{switch_to_slot, switch_to_slot_and_profile};

type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum MainError {
    #[error("Unknown profile: {description}")]
    UnknownProfileError { description: String },
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

    slot1_entry.set_text(names.get(0).unwrap_or(&"Profile Slot 1".to_string()));
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

    slot1_entry.connect_focus_out_event(|edit, _event| {
        let slot_name = edit.text().to_string();
        util::set_slot_name(0, &slot_name).unwrap_or_else(|e| log::error!("{}", e));

        gtk::Inhibit(false)
    });

    slot2_entry.connect_focus_out_event(|edit, _event| {
        let slot_name = edit.text().to_string();
        util::set_slot_name(1, &slot_name).unwrap_or_else(|e| log::error!("{}", e));

        gtk::Inhibit(false)
    });

    slot3_entry.connect_focus_out_event(|edit, _event| {
        let slot_name = edit.text().to_string();
        util::set_slot_name(2, &slot_name).unwrap_or_else(|e| log::error!("{}", e));

        gtk::Inhibit(false)
    });

    slot4_entry.connect_focus_out_event(|edit, _event| {
        let slot_name = edit.text().to_string();
        util::set_slot_name(3, &slot_name).unwrap_or_else(|e| log::error!("{}", e));

        gtk::Inhibit(false)
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

        profiles_treestore.insert_with_values(None, None, &[(0, &0), (1, &name), (2, &filename)]);
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

/// Register global actions and keyboard accelerators
fn register_actions<A: IsA<gtk::Application>>(
    application: &A,
    builder: &gtk::Builder,
) -> Result<()> {
    let application = application.as_ref();

    // let stack_switcher: gtk::StackSwitcher = builder.object("stack_switcher").unwrap();
    let main_stack: gtk::Stack = builder.object("main_stack").unwrap();

    // switching between stack pages
    let switch_to_page1 = gio::SimpleAction::new("switch-to-page-1", None);
    switch_to_page1.connect_activate(clone!(@weak main_stack => move |_, _| {
        main_stack.set_visible_child_name("page0");
    }));

    application.add_action(&switch_to_page1);
    application.set_accels_for_action("app.switch-to-page-1", &["<alt>1"]);

    let switch_to_page2 = gio::SimpleAction::new("switch-to-page-2", None);
    switch_to_page2.connect_activate(clone!(@weak main_stack => move |_, _| {
        main_stack.set_visible_child_name("page1");
    }));

    application.add_action(&switch_to_page2);
    application.set_accels_for_action("app.switch-to-page-2", &["<alt>2"]);

    let switch_to_page3 = gio::SimpleAction::new("switch-to-page-3", None);
    switch_to_page3.connect_activate(clone!(@weak main_stack => move |_, _| {
        main_stack.set_visible_child_name("page2");
    }));

    application.add_action(&switch_to_page3);
    application.set_accels_for_action("app.switch-to-page-3", &["<alt>3"]);

    let switch_to_page4 = gio::SimpleAction::new("switch-to-page-4", None);
    switch_to_page4.connect_activate(clone!(@weak main_stack => move |_, _| {
        main_stack.set_visible_child_name("page3");
    }));

    application.add_action(&switch_to_page4);
    application.set_accels_for_action("app.switch-to-page-4", &["<alt>4"]);

    let switch_to_page5 = gio::SimpleAction::new("switch-to-page-5", None);
    switch_to_page5.connect_activate(clone!(@weak main_stack => move |_, _| {
        main_stack.set_visible_child_name("page4");
    }));

    application.add_action(&switch_to_page5);
    application.set_accels_for_action("app.switch-to-page-5", &["<alt>5"]);

    let switch_to_page6 = gio::SimpleAction::new("switch-to-page-6", None);
    switch_to_page6.connect_activate(clone!(@weak main_stack => move |_, _| {
        main_stack.set_visible_child_name("page5");
    }));

    application.add_action(&switch_to_page6);
    application.set_accels_for_action("app.switch-to-page-6", &["<alt>6"]);

    // switching between slots
    let switch_to_slot1 = gio::SimpleAction::new("switch-to-slot-1", None);
    switch_to_slot1.connect_activate(clone!(@weak builder => move |_, _| {
        if !events::shall_ignore_pending_ui_event() {
            switch_to_slot(0).unwrap();
            update_slot_indicator_state(&builder, 0);
        }
    }));

    application.add_action(&switch_to_slot1);
    // application.set_accels_for_action("app.switch-to-slot-1", &["<Primary>1"]);

    let switch_to_slot2 = gio::SimpleAction::new("switch-to-slot-2", None);
    switch_to_slot2.connect_activate(clone!(@weak builder => move |_, _| {
        if !events::shall_ignore_pending_ui_event() {
            switch_to_slot(1).unwrap();
            update_slot_indicator_state(&builder, 1);
        }
    }));

    application.add_action(&switch_to_slot2);
    // application.set_accels_for_action("app.switch-to-slot-2", &["<Primary>2"]);

    let switch_to_slot3 = gio::SimpleAction::new("switch-to-slot-3", None);
    switch_to_slot3.connect_activate(clone!(@weak builder => move |_, _| {
        if !events::shall_ignore_pending_ui_event() {
            switch_to_slot(2).unwrap();
            update_slot_indicator_state(&builder, 2);
        }
    }));

    application.add_action(&switch_to_slot3);
    // application.set_accels_for_action("app.switch-to-slot-3", &["<Primary>3"]);

    let switch_to_slot4 = gio::SimpleAction::new("switch-to-slot-4", None);
    switch_to_slot4.connect_activate(clone!(@weak builder => move |_, _| {
        if !events::shall_ignore_pending_ui_event() {
            switch_to_slot(3).unwrap();
            update_slot_indicator_state(&builder, 3);
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

    Ok(())
}

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

    let notification_box_global: gtk::Box = builder.object("notification_box_global").unwrap();

    let main_window: gtk::ApplicationWindow = builder.object("main_window").unwrap();

    let restart_eruption_daemon_button: gtk::Button =
        builder.object("restart_eruption_button_global").unwrap();

    let header_bar: gtk::HeaderBar = builder.object("header_bar").unwrap();
    let brightness_scale: gtk::Scale = builder.object("brightness_scale").unwrap();
    let about_item: gtk::MenuItem = builder.object("about_item").unwrap();
    let quit_item: gtk::MenuItem = builder.object("quit_item").unwrap();
    let lock_button: gtk::LockButton = builder.object("lock_button").unwrap();

    let ambientfx_switch: gtk::Switch = builder.object("ambientfx_switch").unwrap();
    let soundfx_switch: gtk::Switch = builder.object("soundfx_switch").unwrap();

    // enable custom CSS support
    let screen = main_window.screen().unwrap();
    let style = gtk::CssProvider::new();
    gtk::CssProvider::load_from_resource(&style, "/org/eruption/eruption-gui-gtk3/styles/app.css");
    gtk::StyleContext::add_provider_for_screen(&screen, &style, gtk::STYLE_PROVIDER_PRIORITY_USER);

    // configure main window
    main_window.set_application(Some(application));
    main_window.set_position(gtk::WindowPosition::Center);
    main_window.set_title("Eruption GUI");
    main_window.set_icon_name(Some("/org/eruption/eruption-gui-gtk3/img/eruption-gui.png"));

    main_window.connect_delete_event(
        clone!(@weak application => @default-return Inhibit(false), move |_, _| {
            application.quit();
            Inhibit(false)
        }),
    );

    register_actions(application, &builder)?;

    header_bar.set_subtitle(Some(&format!(
        "Version: {}",
        std::env!("CARGO_PKG_VERSION")
    )));

    // TODO: implement this
    // lock_button.set_permission();

    lock_button.connect_clicked(|_btn| {
        let _result = dbus_client::ping_privileged();
    });

    // main menu items
    about_item.connect_activate(clone!(@weak main_window => move |_| {
        ui::about::show_about_dialog(&main_window);
    }));

    quit_item.connect_activate(clone!(@weak application => move |_| {
        application.quit();
    }));

    // brightness

    // no need to ignore events here, since handler is not connected
    brightness_scale.set_value(util::get_brightness()? as f64);

    brightness_scale.connect_value_changed(move |s| {
        if !events::shall_ignore_pending_ui_event() {
            util::set_brightness(s.value() as i64).unwrap();
        }
    });

    restart_eruption_daemon_button.connect_clicked(clone!(@weak builder => move |_| {
        util::restart_eruption_daemon().unwrap_or_else(|e| log::error!("{}", e));
    }));

    // special options
    ambientfx_switch.connect_state_set(
        clone!(@weak main_window => @default-return gtk::Inhibit(false), move |_sw, enabled| {
            util::set_ambient_effect(enabled).unwrap();

            gtk::Inhibit(false)
        }),
    );

    soundfx_switch.set_state(util::get_sound_fx().unwrap_or(false));

    soundfx_switch.connect_state_set(
        clone!(@weak main_window => @default-return gtk::Inhibit(false), move |_sw, enabled| {
            util::set_sound_fx(enabled).unwrap();

            gtk::Inhibit(false)
        }),
    );

    update_main_window(&builder)?;

    ui::profiles::initialize_profiles_page(application, &builder)?;
    ui::process_monitor::initialize_process_monitor_page(application, &builder)?;
    ui::settings::initialize_settings_page(&builder)?;

    initialize_slot_bar(&builder)?;

    dbus_client::spawn_dbus_event_loop_system(&builder, &update_ui_state)?;
    dbus_client::spawn_dbus_event_loop_session(&builder, &|_b, m| {
        println!("{:?}", m);
        Ok(())
    })?;

    main_window.show_all();

    // should we update the GUI, e.g. were new devices attached?
    crate::register_timer(
        250,
        clone!(@weak application => @default-return Ok(()), move || {
            if crate::dbus_client::ping().is_err() {
                notification_box_global.show();

                events::LOST_CONNECTION.store(true, Ordering::SeqCst);

                // remove all devices sub-pages for now, until we regain the connection
                update_main_window(&builder).unwrap();
            } else {
                notification_box_global.hide();

                if events::LOST_CONNECTION.load(Ordering::SeqCst) {
                    // we re-established the connection to the Eruption daemon,
                    events::LOST_CONNECTION.store(false, Ordering::SeqCst);

                    // update the GUI to show e.g. newly attached devices
                    events::UPDATE_MAIN_WINDOW.store(true, Ordering::SeqCst);
                }
            }

            if events::UPDATE_MAIN_WINDOW.load(Ordering::SeqCst) {
                events::UPDATE_MAIN_WINDOW.store(false, Ordering::SeqCst);

                update_main_window(&builder).unwrap();
            }

            Ok(())
        }),
    )?;

    // update the global LED color map vector
    crate::register_timer(
        1000 / (crate::constants::TARGET_FPS * 2),
        clone!(@weak application => @default-return Ok(()), move || {
            let _result = crate::update_color_map();

            Ok(())
        }),
    )?;

    Ok(())
}

pub fn update_main_window(builder: &gtk::Builder) -> Result<()> {
    let keyboard_devices_stack: gtk::Stack = builder.object("keyboard_devices_stack").unwrap();
    let mouse_devices_stack: gtk::Stack = builder.object("mouse_devices_stack").unwrap();
    let misc_devices_stack: gtk::Stack = builder.object("misc_devices_stack").unwrap();

    // clean up all previously instantiated sub-pages
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

    if events::LOST_CONNECTION.load(Ordering::SeqCst) {
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

        Ok(())
    } else {
        // instantiate the devices sub-pages
        let devices = dbus_client::get_managed_devices()?;

        let mut device_index = 0;

        let mut any_keyboard_device = false;
        let mut any_mouse_device = false;
        let mut any_misc_device = false;

        // instantiate stack pages for all keyboard devices
        for (_device, (vid, pid)) in devices.0.iter().enumerate() {
            let template = gtk::Builder::from_resource(
                "/org/eruption/eruption-gui-gtk3/ui/keyboard-device-template.ui",
            );

            let page =
                ui::keyboard::initialize_keyboard_page(builder, &template, device_index as u64)?;

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
            let no_device_template =
                gtk::Builder::from_resource("/org/eruption/eruption-gui-gtk3/ui/no-device-template.ui");

            let page: gtk::Grid = no_device_template.object("no_device_template").unwrap();

            keyboard_devices_stack.add_titled(&page, "None", "None");
        }

        // instantiate stack pages for all mouse devices
        for (_device, (vid, pid)) in devices.1.iter().enumerate() {
            let template = gtk::Builder::from_resource(
                "/org/eruption/eruption-gui-gtk3/ui/mouse-device-template.ui",
            );

            let page = ui::mouse::initialize_mouse_page(builder, &template, device_index as u64)?;

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
            let no_device_template =
                gtk::Builder::from_resource("/org/eruption/eruption-gui-gtk3/ui/no-device-template.ui");

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
            let no_device_template =
                gtk::Builder::from_resource("/org/eruption/eruption-gui-gtk3/ui/no-device-template.ui");

            let page: gtk::Grid = no_device_template.object("no_device_template").unwrap();

            misc_devices_stack.add_titled(&page, "None", "None");
        }

        Ok(())
    }
}
