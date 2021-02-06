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
use crate::dbus_client;
use crate::events;
use crate::ui;
use crate::update_ui_state;
use crate::util;
use crate::STATE;
use crate::{switch_to_slot, switch_to_slot_and_profile};
use gio::prelude::*;
use glib::clone;
use gtk::prelude::*;
use std::path::PathBuf;

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
        let file = model
            .get_value(&iter, 2)
            .to_value()
            .get::<String>()
            .unwrap()
            .unwrap();
        let path = PathBuf::from(&file);

        if slot_profile_path.file_name() == path.file_name() {
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
    let window: gtk::Window = builder.get_object("main_window").unwrap();

    let slot1_frame: gtk::Frame = builder.get_object("slot1_frame").unwrap();
    let slot2_frame: gtk::Frame = builder.get_object("slot2_frame").unwrap();
    let slot3_frame: gtk::Frame = builder.get_object("slot3_frame").unwrap();
    let slot4_frame: gtk::Frame = builder.get_object("slot4_frame").unwrap();

    let slot1_radio_button: gtk::RadioButton = builder.get_object("slot1_radio_button").unwrap();
    let slot2_radio_button: gtk::RadioButton = builder.get_object("slot2_radio_button").unwrap();
    let slot3_radio_button: gtk::RadioButton = builder.get_object("slot3_radio_button").unwrap();
    let slot4_radio_button: gtk::RadioButton = builder.get_object("slot4_radio_button").unwrap();

    let slot1_entry: gtk::Entry = builder.get_object("slot1_entry").unwrap();
    let slot2_entry: gtk::Entry = builder.get_object("slot2_entry").unwrap();
    let slot3_entry: gtk::Entry = builder.get_object("slot3_entry").unwrap();
    let slot4_entry: gtk::Entry = builder.get_object("slot4_entry").unwrap();

    let edit_slot1_button: gtk::Button = builder.get_object("edit_slot1_button").unwrap();
    let edit_slot2_button: gtk::Button = builder.get_object("edit_slot2_button").unwrap();
    let edit_slot3_button: gtk::Button = builder.get_object("edit_slot3_button").unwrap();
    let edit_slot4_button: gtk::Button = builder.get_object("edit_slot4_button").unwrap();

    let slot1_combo: gtk::ComboBox = builder.get_object("slot1_combo").unwrap();
    let slot2_combo: gtk::ComboBox = builder.get_object("slot2_combo").unwrap();
    let slot3_combo: gtk::ComboBox = builder.get_object("slot3_combo").unwrap();
    let slot4_combo: gtk::ComboBox = builder.get_object("slot4_combo").unwrap();

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

    edit_slot1_button.connect_clicked(clone!(@strong window, @strong slot1_entry => move |_btn| {
            window.set_focus(Some(&slot1_entry));
    }));

    edit_slot2_button.connect_clicked(clone!(@strong window, @strong slot2_entry => move |_btn| {
            window.set_focus(Some(&slot2_entry));
    }));

    edit_slot3_button.connect_clicked(clone!(@strong window, @strong slot3_entry => move |_btn| {
        window.set_focus(Some(&slot3_entry));
    }));

    edit_slot4_button.connect_clicked(clone!(@strong window, @strong slot4_entry => move |_btn| {
        window.set_focus(Some(&slot4_entry));
    }));

    slot1_entry.connect_focus_out_event(|edit, _event| {
        let slot_name = edit.get_text().to_string();
        util::set_slot_name(0, &slot_name).unwrap_or_else(|e| log::error!("{}", e));

        gtk::Inhibit(false)
    });

    slot2_entry.connect_focus_out_event(|edit, _event| {
        let slot_name = edit.get_text().to_string();
        util::set_slot_name(1, &slot_name).unwrap_or_else(|e| log::error!("{}", e));

        gtk::Inhibit(false)
    });

    slot3_entry.connect_focus_out_event(|edit, _event| {
        let slot_name = edit.get_text().to_string();
        util::set_slot_name(2, &slot_name).unwrap_or_else(|e| log::error!("{}", e));

        gtk::Inhibit(false)
    });

    slot4_entry.connect_focus_out_event(|edit, _event| {
        let slot_name = edit.get_text().to_string();
        util::set_slot_name(3, &slot_name).unwrap_or_else(|e| log::error!("{}", e));

        gtk::Inhibit(false)
    });

    // profiles list
    let profiles_treestore = gtk::TreeStore::new(&[
        glib::Type::U64,
        String::static_type(),
        String::static_type(),
    ]);

    // TODO: use configuration values from eruption.conf
    let path = PathBuf::from(constants::DEFAULT_PROFILE_DIR);
    for profile in util::enumerate_profiles(&path)? {
        let name = profile.name;
        let filename = profile
            .profile_file
            .to_string_lossy()
            .to_owned()
            .to_string();

        profiles_treestore.insert_with_values(None, None, &[0, 1, 2], &[&0, &name, &filename]);
    }

    let cell_renderer_id = gtk::CellRendererText::new();
    let cell_renderer_name = gtk::CellRendererText::new();
    let cell_renderer_filename = gtk::CellRendererText::new();

    slot1_combo.pack_start(&cell_renderer_name, true);

    slot1_combo.add_attribute(&cell_renderer_id, &"text", 0);
    slot1_combo.add_attribute(&cell_renderer_name, &"text", 1);
    slot1_combo.add_attribute(&cell_renderer_filename, &"text", 2);

    slot1_combo.set_model(Some(&profiles_treestore));
    slot1_combo.show_all();
    slot1_combo.set_active(find_profile_index(0, &profiles_treestore).ok());

    slot1_combo.connect_changed(clone!(@weak profiles_treestore => move |cb| {
        let id = cb.get_active().unwrap();
        let entry = cb
            .get_model()
            .unwrap()
            .iter_nth_child(None, id as i32)
            .unwrap();

        let file = profiles_treestore
            .get_value(&entry, 2)
            .to_value().get::<String>()
            .unwrap()
            .unwrap();


        switch_to_slot_and_profile(0, file).unwrap();
    }));

    slot2_combo.pack_start(&cell_renderer_name, true);

    slot2_combo.add_attribute(&cell_renderer_id, &"text", 0);
    slot2_combo.add_attribute(&cell_renderer_name, &"text", 1);
    slot2_combo.add_attribute(&cell_renderer_filename, &"text", 2);

    slot2_combo.set_model(Some(&profiles_treestore));
    slot2_combo.show_all();
    slot2_combo.set_active(find_profile_index(1, &profiles_treestore).ok());

    slot2_combo.connect_changed(clone!(@weak profiles_treestore => move |cb| {
        let id = cb.get_active().unwrap();
        let entry = cb
            .get_model()
            .unwrap()
            .iter_nth_child(None, id as i32)
            .unwrap();

        let file = profiles_treestore
            .get_value(&entry, 2)
            .to_value().get::<String>()
            .unwrap()
            .unwrap();


        switch_to_slot_and_profile(1, file).unwrap();
    }));

    slot3_combo.pack_start(&cell_renderer_name, true);

    slot3_combo.add_attribute(&cell_renderer_id, &"text", 0);
    slot3_combo.add_attribute(&cell_renderer_name, &"text", 1);
    slot3_combo.add_attribute(&cell_renderer_filename, &"text", 2);

    slot3_combo.set_model(Some(&profiles_treestore));
    slot3_combo.show_all();
    slot3_combo.set_active(find_profile_index(2, &profiles_treestore).ok());

    slot3_combo.connect_changed(clone!(@weak profiles_treestore => move |cb| {
        let id = cb.get_active().unwrap();
        let entry = cb
            .get_model()
            .unwrap()
            .iter_nth_child(None, id as i32)
            .unwrap();

        let file = profiles_treestore
            .get_value(&entry, 2)
            .to_value().get::<String>()
            .unwrap()
            .unwrap();


        switch_to_slot_and_profile(2, file).unwrap();
    }));

    slot4_combo.pack_start(&cell_renderer_name, true);

    slot4_combo.add_attribute(&cell_renderer_id, &"text", 0);
    slot4_combo.add_attribute(&cell_renderer_name, &"text", 1);
    slot4_combo.add_attribute(&cell_renderer_filename, &"text", 2);

    slot4_combo.set_model(Some(&profiles_treestore));
    slot4_combo.show_all();
    slot4_combo.set_active(find_profile_index(3, &profiles_treestore).ok());

    slot4_combo.connect_changed(clone!(@weak profiles_treestore => move |cb| {
        let id = cb.get_active().unwrap();
        let entry = cb
            .get_model()
            .unwrap()
            .iter_nth_child(None, id as i32)
            .unwrap();

        let file = profiles_treestore
            .get_value(&entry, 2)
            .to_value().get::<String>()
            .unwrap()
            .unwrap();


        switch_to_slot_and_profile(3, file).unwrap();
    }));

    events::ignore_next_ui_events(1);
    let active_slot = STATE.read().active_slot.unwrap();

    match active_slot {
        0 => {
            slot1_radio_button.set_active(true);

            let context = slot1_frame.get_style_context();
            context.add_class("active");

            let context = slot2_frame.get_style_context();
            context.remove_class("active");

            let context = slot3_frame.get_style_context();
            context.remove_class("active");

            let context = slot4_frame.get_style_context();
            context.remove_class("active");
        }

        1 => {
            slot2_radio_button.set_active(true);

            let context = slot1_frame.get_style_context();
            context.remove_class("active");

            let context = slot2_frame.get_style_context();
            context.add_class("active");

            let context = slot3_frame.get_style_context();
            context.remove_class("active");

            let context = slot4_frame.get_style_context();
            context.remove_class("active");
        }

        2 => {
            slot3_radio_button.set_active(true);

            let context = slot1_frame.get_style_context();
            context.remove_class("active");

            let context = slot2_frame.get_style_context();
            context.remove_class("active");

            let context = slot3_frame.get_style_context();
            context.add_class("active");

            let context = slot4_frame.get_style_context();
            context.remove_class("active");
        }

        3 => {
            slot4_radio_button.set_active(true);

            let context = slot1_frame.get_style_context();
            context.remove_class("active");

            let context = slot2_frame.get_style_context();
            context.remove_class("active");

            let context = slot3_frame.get_style_context();
            context.remove_class("active");

            let context = slot4_frame.get_style_context();
            context.add_class("active");
        }

        _ => panic!("Invalid slot index"),
    };

    events::reenable_ui_events();

    Ok(())
}

fn update_slot_indicator_state(builder: &gtk::Builder, active_slot: usize) {
    let slot1_frame: gtk::Frame = builder.get_object("slot1_frame").unwrap();
    let slot2_frame: gtk::Frame = builder.get_object("slot2_frame").unwrap();
    let slot3_frame: gtk::Frame = builder.get_object("slot3_frame").unwrap();
    let slot4_frame: gtk::Frame = builder.get_object("slot4_frame").unwrap();

    // let slot1_radio_button: gtk::RadioButton = builder.get_object("slot1_radio_button").unwrap();
    // let slot2_radio_button: gtk::RadioButton = builder.get_object("slot2_radio_button").unwrap();
    // let slot3_radio_button: gtk::RadioButton = builder.get_object("slot3_radio_button").unwrap();
    // let slot4_radio_button: gtk::RadioButton = builder.get_object("slot4_radio_button").unwrap();

    // events::ignore_next_ui_events(1);
    // let active_slot = STATE.read().active_slot.unwrap();

    match active_slot {
        0 => {
            // slot1_radio_button.set_active(true);

            let context = slot1_frame.get_style_context();
            context.add_class("active");

            let context = slot2_frame.get_style_context();
            context.remove_class("active");

            let context = slot3_frame.get_style_context();
            context.remove_class("active");

            let context = slot4_frame.get_style_context();
            context.remove_class("active");
        }

        1 => {
            // slot2_radio_button.set_active(true);

            let context = slot1_frame.get_style_context();
            context.remove_class("active");

            let context = slot2_frame.get_style_context();
            context.add_class("active");

            let context = slot3_frame.get_style_context();
            context.remove_class("active");

            let context = slot4_frame.get_style_context();
            context.remove_class("active");
        }

        2 => {
            // slot3_radio_button.set_active(true);

            let context = slot1_frame.get_style_context();
            context.remove_class("active");

            let context = slot2_frame.get_style_context();
            context.remove_class("active");

            let context = slot3_frame.get_style_context();
            context.add_class("active");

            let context = slot4_frame.get_style_context();
            context.remove_class("active");
        }

        3 => {
            // slot4_radio_button.set_active(true);

            let context = slot1_frame.get_style_context();
            context.remove_class("active");

            let context = slot2_frame.get_style_context();
            context.remove_class("active");

            let context = slot3_frame.get_style_context();
            context.remove_class("active");

            let context = slot4_frame.get_style_context();
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

    let switch_to_slot1 = gio::SimpleAction::new("switch-to-slot-1", None);
    switch_to_slot1.connect_activate(clone!(@strong builder => move |_, _| {
        if !events::shall_ignore_pending_ui_event() {
            switch_to_slot(0).unwrap();
            update_slot_indicator_state(&builder, 0);
        }
    }));

    application.add_action(&switch_to_slot1);
    // application.set_accels_for_action("app.switch-to-slot-1", &["<Primary>1"]);

    let switch_to_slot2 = gio::SimpleAction::new("switch-to-slot-2", None);
    switch_to_slot2.connect_activate(clone!(@strong builder => move |_, _| {
        if !events::shall_ignore_pending_ui_event() {
            switch_to_slot(1).unwrap();
            update_slot_indicator_state(&builder, 1);
        }
    }));

    application.add_action(&switch_to_slot2);
    // application.set_accels_for_action("app.switch-to-slot-2", &["<Primary>2"]);

    let switch_to_slot3 = gio::SimpleAction::new("switch-to-slot-3", None);
    switch_to_slot3.connect_activate(clone!(@strong builder => move |_, _| {
        if !events::shall_ignore_pending_ui_event() {
            switch_to_slot(2).unwrap();
            update_slot_indicator_state(&builder, 2);
        }
    }));

    application.add_action(&switch_to_slot3);
    // application.set_accels_for_action("app.switch-to-slot-3", &["<Primary>3"]);

    let switch_to_slot4 = gio::SimpleAction::new("switch-to-slot-4", None);
    switch_to_slot4.connect_activate(clone!(@strong builder => move |_, _| {
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
    {
        let _temporary_sourceview = sourceview::View::new();
    }

    // build UI
    let builder = gtk::Builder::from_resource("/org/eruption/eruption-gui/ui/main.glade");

    let main_window: gtk::ApplicationWindow = builder.get_object("main_window").unwrap();
    let header_bar: gtk::HeaderBar = builder.get_object("header_bar").unwrap();
    let brightness_scale: gtk::Scale = builder.get_object("brightness_scale").unwrap();
    let about_item: gtk::MenuItem = builder.get_object("about_item").unwrap();
    let quit_item: gtk::MenuItem = builder.get_object("quit_item").unwrap();
    let lock_button: gtk::LockButton = builder.get_object("lock_button").unwrap();

    // enable custom CSS support
    let screen = main_window.get_screen().unwrap();
    let style = gtk::CssProvider::new();
    gtk::CssProviderExt::load_from_resource(&style, "/org/eruption/eruption-gui/styles/app.css");
    gtk::StyleContext::add_provider_for_screen(&screen, &style, gtk::STYLE_PROVIDER_PRIORITY_USER);

    // configure main window
    main_window.set_application(Some(application));
    main_window.set_position(gtk::WindowPosition::Center);
    main_window.set_title("Eruption GUI");
    main_window.set_icon_name(Some("/org/eruption/eruption-gui/img/eruption-gui.png"));

    main_window.connect_delete_event(clone!(@strong application => move |_, _| {
        application.quit();
        Inhibit(false)
    }));

    register_actions(application, &builder)?;

    header_bar.set_subtitle(Some(&format!(
        "Version: {}",
        std::env!("CARGO_PKG_VERSION")
    )));

    // TODO: implement this
    lock_button.set_permission::<gio::Permission>(None);

    lock_button.connect_clicked(|_btn| {
        dbus_client::test().unwrap();
    });

    // main menu items
    about_item.connect_activate(clone!(@weak main_window => move |_| {
        ui::about::show_about_dialog(&main_window);
    }));

    quit_item.connect_activate(clone!(@strong application => move |_| {
        application.quit();
    }));

    // brightness

    // no need to ignore events here, since handler is not connected
    brightness_scale.set_value(util::get_brightness()? as f64);

    brightness_scale.connect_value_changed(move |s| {
        if !events::shall_ignore_pending_ui_event() {
            util::set_brightness(s.get_value() as i64).unwrap();
        }
    });

    ui::keyboard::initialize_keyboard_page(&builder)?;
    ui::mouse::initialize_mouse_page(&builder)?;
    ui::profiles::initialize_profiles_page(&builder)?;
    ui::process_monitor::initialize_process_monitor_page(&builder)?;
    ui::settings::initialize_settings_page(&builder)?;

    initialize_slot_bar(&builder)?;

    dbus_client::spawn_dbus_event_loop_system(&builder, &update_ui_state)?;
    dbus_client::spawn_dbus_event_loop_session(&builder, &update_ui_state)?;

    main_window.show_all();

    // TODO: add support for privileged write operations
    lock_button.hide();

    Ok(())
}
