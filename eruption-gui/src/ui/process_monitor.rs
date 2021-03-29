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

use gio::ActionMapExt;
use glib::{clone, StaticType};
use gtk::prelude::*;

use crate::{dbus_client, ui::rule, util};

type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum ProcessMonitorError {
    #[error("Connection to daemon failed")]
    ConnectionFailed,
    // #[error("Unknown error: {description}")]
    // UnknownError { description: String },
}

/// Initialize page "Process Monitor"
pub fn update_rules_view(builder: &gtk::Builder) -> Result<()> {
    // let application = application.as_ref();

    // let main_window: gtk::ApplicationWindow = builder.get_object("main_window").unwrap();

    let rules_box: gtk::Box = builder.get_object("rules_box").unwrap();
    let rules_treeview: gtk::TreeView = builder.get_object("rules_treeview").unwrap();

    if let Ok(rules) = dbus_client::enumerate_process_monitor_rules() {
        // rules list
        let rules_treestore = gtk::TreeStore::new(&[
            glib::Type::Bool,
            glib::Type::U64,
            String::static_type(),
            String::static_type(),
            String::static_type(),
            String::static_type(),
        ]);

        for (index, rule) in rules.iter().enumerate() {
            let enabled = rule.3.contains("enabled");

            let sensor = &rule.0;
            let selector = &rule.1;
            let action = &rule.2;
            let metadata = &rule.3;

            rules_treestore.insert_with_values(
                None,
                None,
                &[0, 1, 2, 3, 4, 5],
                &[
                    &enabled,
                    &(index as u64),
                    &sensor,
                    &selector,
                    &action,
                    &metadata,
                ],
            );
        }

        rules_treeview.set_model(Some(&rules_treestore));

        // rules_treeview.connect_row_activated(
        //     clone!(@strong builder => move |_tv, _path, _column| {
        //         // let index = tv.get_model().unwrap().get_value(&tv.get_model().unwrap().get_iter(&path).unwrap(), 0).get::<u64>().unwrap().unwrap();
        //     }),
        // );

        // rules_treeview.connect_button_press_event(clone!(@strong main_window => move |_, _e| {
        //         ui::rule::show_rule_dialog(&main_window);

        //         Inhibit(false)
        // }));

        rules_box.show();

        Ok(())
    } else {
        Err(ProcessMonitorError::ConnectionFailed {}.into())
    }
}

/// Updates the ruleset of the eruption-process-monitor daemon to match the current state of the GUI
pub fn transmit_rules_to_process_monitor(builder: &gtk::Builder) -> Result<()> {
    let rules_treeview: gtk::TreeView = builder.get_object("rules_treeview").unwrap();

    let mut rules: Vec<(String, String, String, String)> = Vec::new();

    // generate a Vec<_> from the ruleset
    rules_treeview
        .get_model()
        .unwrap()
        .foreach(|model, _path, iter| {
            let metadata = model.get_value(iter, 5).get::<String>().unwrap().unwrap();

            if !metadata.contains("internal") {
                let enabled = model.get_value(iter, 0).get::<bool>().unwrap().unwrap();

                let sensor = model.get_value(iter, 2).get::<String>().unwrap().unwrap();
                let selector = model.get_value(iter, 3).get::<String>().unwrap().unwrap();
                let action = model.get_value(iter, 4).get::<String>().unwrap().unwrap();

                let metadata = format!(
                    "{},user-defined",
                    if enabled { "enabled" } else { "disabled" },
                );

                rules.push((sensor, selector, action, metadata));
            } else {
                let sensor = model.get_value(iter, 2).get::<String>().unwrap().unwrap();
                let selector = model.get_value(iter, 3).get::<String>().unwrap().unwrap();
                let action = model.get_value(iter, 4).get::<String>().unwrap().unwrap();
                let metadata = model.get_value(iter, 5).get::<String>().unwrap().unwrap();

                rules.push((sensor, selector, action, metadata));
            }

            false
        });

    // send full ruleset to the eruption-process-monitor daemon
    dbus_client::transmit_process_monitor_rules(
        &rules
            .iter()
            .map(|e| (e.0.as_str(), e.1.as_str(), e.2.as_str(), e.3.as_str()))
            .collect::<Vec<(&str, &str, &str, &str)>>(),
    )?;

    Ok(())
}

/// Initialize page "Process Monitor"
pub fn initialize_process_monitor_page<A: IsA<gtk::Application>>(
    application: &A,
    builder: &gtk::Builder,
) -> Result<()> {
    let application = application.as_ref();

    let main_window: gtk::ApplicationWindow = builder.get_object("main_window").unwrap();

    let notification_box: gtk::Box = builder.get_object("notification_box").unwrap();

    let rules_box: gtk::Box = builder.get_object("rules_box").unwrap();

    let restart_process_monitor_button: gtk::Button = builder
        .get_object("restart_process_monitor_button")
        .unwrap();

    let rules_treeview: gtk::TreeView = builder.get_object("rules_treeview").unwrap();

    // register actions
    let add_rule = gio::SimpleAction::new("add-rule", None);
    add_rule.connect_activate(clone!(@strong main_window, @strong builder, @weak rules_treeview, @weak notification_box, @weak rules_box => move |_, _| {
        let (response, rule) = rule::show_new_rule_dialog(&main_window);
        if response == gtk::ResponseType::Ok {
            let rule = rule.unwrap();

            let tree_model = rules_treeview.get_model().unwrap();

            let mut counter = 1;
            tree_model.foreach(|_m, _p, _i| { counter += 1; true });

            let index = counter + 1;
            let metadata = format!(
                "{},user-defined",
                if rule.enabled { "enabled" } else { "disabled" },
            );

            let rules_treestore = tree_model.downcast::<gtk::TreeStore>().unwrap();
            rules_treestore.insert_with_values(
                None,
                None,
                &[0, 1, 2, 3, 4, 5],
                &[
                    &rule.enabled,
                    &(index as u64),
                    &rule.sensor,
                    &rule.selector,
                    &rule.action,
                    &metadata,
                ],
            );

            if let Err(e) = transmit_rules_to_process_monitor(&builder) {
                log::error!("{}", e);

                let message = "Could not transmit ruleset".to_string();
                let secondary =
                    format!("Could not transmit the ruleset to the eruption-process-monitor daemon {}", e);

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
            }

            if let Err(_e) = update_rules_view(&builder) {
                notification_box.show_now();
                rules_box.hide();
            }
        }
    }));

    add_rule.set_enabled(true);

    application.add_action(&add_rule);
    application.set_accels_for_action("app.add-rule", &["<primary><shift>n"]);

    let remove_rule = gio::SimpleAction::new("remove-rule", None);
    remove_rule.connect_activate(clone!(@strong builder, @strong notification_box, @strong rules_box,
                                        @strong main_window, @strong rules_treeview => move |_, _| {

        let selection = &rules_treeview.get_selection().get_selected_rows().0;
        let rules_treestore: gtk::TreeStore = rules_treeview.get_model().unwrap().downcast::<gtk::TreeStore>().unwrap();

        if !selection.is_empty() {
            for p in selection.iter() {
                rules_treestore.remove(&rules_treestore.get_iter(&p).unwrap());
            }

            if let Err(e) = transmit_rules_to_process_monitor(&builder) {
                log::error!("{}", e);

                let message = "Could not transmit ruleset".to_string();
                let secondary =
                    format!("Could not transmit the ruleset to the eruption-process-monitor daemon {}", e);

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
            }

            if let Err(_e) = update_rules_view(&builder) {
                notification_box.show_now();
                rules_box.hide();
            }
        }
    }));

    remove_rule.set_enabled(true);

    application.add_action(&remove_rule);
    application.set_accels_for_action("app.remove-rule", &["<delete>"]);

    let update_view = gio::SimpleAction::new("update-rules-view", None);
    update_view.connect_activate(
        clone!(@strong builder, @weak notification_box => move |_, _| {
            // update the rules view or show an error notification
            update_rules_view(&builder).unwrap_or_else(move|_e| {
                notification_box.show_now();
            });
        }),
    );

    update_view.set_enabled(true);

    application.add_action(&update_view);
    application.set_accels_for_action("app.update-rules-view", &[]);

    let edit_rule = gio::SimpleAction::new("edit-rule", None);
    edit_rule.connect_activate(clone!(@strong builder, @strong notification_box, @strong rules_box,
                                        @strong main_window, @strong rules_treeview => move |_, _| {

        let selection = &rules_treeview.get_selection().get_selected_rows();
        let p = &selection.0;

        if !p.is_empty() {
            let p = &p[0];

            let rules_treestore: gtk::TreeStore = rules_treeview.get_model().unwrap().downcast::<gtk::TreeStore>().unwrap();

            let rule_enabled = rules_treestore.get_value(&rules_treestore.get_iter(&p).unwrap(), 0).get::<bool>().unwrap().unwrap();
            let index = rules_treestore.get_value(&rules_treestore.get_iter(&p).unwrap(), 1).get::<u64>().unwrap().unwrap();
            let sensor = rules_treestore.get_value(&rules_treestore.get_iter(&p).unwrap(), 2).get::<String>().unwrap().unwrap();
            let selector = rules_treestore.get_value(&rules_treestore.get_iter(&p).unwrap(), 3).get::<String>().unwrap().unwrap();
            let action = rules_treestore.get_value(&rules_treestore.get_iter(&p).unwrap(), 4).get::<String>().unwrap().unwrap();
            let metadata = rules_treestore.get_value(&rules_treestore.get_iter(&p).unwrap(), 5).get::<String>().unwrap().unwrap();

            let rule = rule::Rule::new(Some(index as usize), rule_enabled, sensor, selector, action, metadata);

            let (response, rule) = rule::show_edit_rule_dialog(&main_window, &rule);
            if response == gtk::ResponseType::Ok {
                let rule = rule.unwrap();

                let tree_model: gtk::TreeModel = rules_treeview.get_model().unwrap();
                let rules_treestore: gtk::TreeStore = tree_model.downcast::<gtk::TreeStore>().unwrap();

                let index = rule.index.unwrap() as u64;

                rules_treestore.insert_with_values(
                    None,
                    None,
                    &[0, 1, 2, 3, 4, 5],
                    &[
                        &rule.enabled,
                        &index,
                        &rule.sensor,
                        &rule.selector,
                        &rule.action,
                        &rule.metadata,
                    ],
                );

                // remove original item
                rules_treestore.remove(&rules_treestore.get_iter(&p).unwrap());

                if let Err(e) = transmit_rules_to_process_monitor(&builder) {
                    log::error!("{}", e);

                    let message = "Could not transmit ruleset".to_string();
                    let secondary =
                        format!("Could not transmit the ruleset to the eruption-process-monitor daemon {}", e);

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
                }

                if let Err(_e) = update_rules_view(&builder) {
                    notification_box.show_now();
                    rules_box.hide();
                }
            }
        }
    }));

    edit_rule.set_enabled(true);

    application.add_action(&edit_rule);
    application.set_accels_for_action("app.edit-rule", &[]);

    // enable/disable tool-buttons
    rules_treeview.get_selection().connect_changed(
        clone!(@weak edit_rule, @weak remove_rule  => move |sel| {
            if !sel.get_selected_rows().0.is_empty() {
                edit_rule.set_enabled(true);
                remove_rule.set_enabled(true);
            } else {
                edit_rule.set_enabled(false);
                remove_rule.set_enabled(false);
            }
        }),
    );

    // hide all notifications
    notification_box.hide();

    restart_process_monitor_button.connect_clicked(
        clone!(@strong builder, @strong application => move |_| {
            util::restart_process_monitor_daemon().unwrap_or_else(|e| log::error!("{}", e));

            glib::timeout_add_local(
                1000,
                clone!(@strong builder, @strong application => move || {
                    if let Err(e) = initialize_process_monitor_page(&application, &builder) {
                        log::error!("{}", e);
                        Continue(true)
                    } else {
                        Continue(false)
                    }
                }),
            );
        }),
    );

    let enabled_column = gtk::TreeViewColumnBuilder::new()
        .title(&"Enabled")
        .sizing(gtk::TreeViewColumnSizing::Autosize)
        .resizable(false)
        .build();

    let index_column = gtk::TreeViewColumnBuilder::new()
        .title(&"#")
        .sizing(gtk::TreeViewColumnSizing::Autosize)
        .resizable(false)
        .build();

    let sensor_column = gtk::TreeViewColumnBuilder::new()
        .title(&"Sensor")
        .sizing(gtk::TreeViewColumnSizing::Autosize)
        .build();

    let selector_column = gtk::TreeViewColumnBuilder::new()
        .title(&"Selector")
        .sizing(gtk::TreeViewColumnSizing::Autosize)
        .resizable(true)
        .build();

    let action_column = gtk::TreeViewColumnBuilder::new().title(&"Action").build();

    let metadata_column = gtk::TreeViewColumnBuilder::new().title(&"Metadata").build();

    let cell_renderer_toggle = gtk::CellRendererToggle::new();
    let cell_renderer_text = gtk::CellRendererText::new();

    cell_renderer_toggle.connect_toggled(clone!(@strong builder, @strong rules_treeview => move |_cr, p| {
            let rules_treestore: gtk::TreeStore = rules_treeview.get_model().unwrap().downcast::<gtk::TreeStore>().unwrap();

            let value = rules_treestore.get_value(&rules_treestore.get_iter(&p).unwrap(), 0).get::<bool>().unwrap().unwrap();
            rules_treestore.set_value(&rules_treestore.get_iter(&p).unwrap(), 0, &(!value).to_value());

            transmit_rules_to_process_monitor(&builder).unwrap_or_else(|e| log::error!("{}", e));
        }));

    enabled_column.pack_start(&cell_renderer_toggle, false);
    index_column.pack_start(&cell_renderer_text, false);
    sensor_column.pack_start(&cell_renderer_text, false);
    selector_column.pack_start(&cell_renderer_text, true);
    action_column.pack_start(&cell_renderer_text, true);
    metadata_column.pack_start(&cell_renderer_text, false);

    rules_treeview
        .get_columns()
        .iter()
        .for_each(clone!(@strong rules_treeview => move |c| {
            rules_treeview.remove_column(c);
        }));

    rules_treeview.insert_column(&enabled_column, 0);
    rules_treeview.insert_column(&index_column, 1);
    rules_treeview.insert_column(&sensor_column, 2);
    rules_treeview.insert_column(&selector_column, 3);
    rules_treeview.insert_column(&action_column, 4);
    rules_treeview.insert_column(&metadata_column, 5);

    enabled_column.add_attribute(&cell_renderer_toggle, &"active", 0);
    index_column.add_attribute(&cell_renderer_text, &"text", 1);
    sensor_column.add_attribute(&cell_renderer_text, &"text", 2);
    selector_column.add_attribute(&cell_renderer_text, &"text", 3);
    action_column.add_attribute(&cell_renderer_text, &"text", 4);
    metadata_column.add_attribute(&cell_renderer_text, &"text", 5);

    // update the rules view or show an error notification
    update_rules_view(&builder).unwrap_or_else(
        clone!(@strong notification_box,@strong rules_box => move |_e| {
            notification_box.show_now();
            rules_box.hide();
        }),
    );

    Ok(())
}
