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

use gdk::prelude::ActionMapExt;
use glib::{clone, Cast, StaticType, ToValue};
use glib::{Continue, IsA};
use gtk::glib;
use gtk::prelude::BuilderExtManual;
use gtk::prelude::ButtonExt;
use gtk::prelude::TreeStoreExtManual;
use gtk::prelude::WidgetExt;
use gtk::traits::{CellRendererToggleExt, DialogExt, TreeViewExt};
use gtk::traits::{GtkApplicationExt, TreeModelExt, TreeSelectionExt, TreeStoreExt};
use gtk::MessageDialog;
use gtk::TreeViewColumn;
use std::time::Duration;

use crate::dbus_client;
use crate::timers::{self, TimerMode};
use crate::{ui::rule, util};

use super::Pages;

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

    // let main_window: gtk::ApplicationWindow = builder.object("main_window").unwrap();

    let notification_box: gtk::Box = builder.object("notification_box").unwrap();

    let rules_box: gtk::Box = builder.object("rules_box").unwrap();
    let rules_treeview: gtk::TreeView = builder.object("rules_treeview").unwrap();

    if let Ok(rules) = dbus_client::enumerate_process_monitor_rules() {
        notification_box.hide();
        rules_box.show();

        // save selection state
        let sel = rules_treeview.selection().selected_rows();

        let rules_treestore = rules_treeview
            .model()
            .unwrap()
            .downcast::<gtk::TreeStore>()
            .unwrap();

        // clear the tree store
        rules_treestore.clear();

        for (index, rule) in rules.iter().enumerate() {
            let enabled = rule.3.contains("enabled");

            let sensor = &rule.0;
            let selector = &rule.1;
            let action = &rule.2;
            let metadata = &rule.3;

            rules_treestore.insert_with_values(
                None,
                None,
                &[
                    (0, &enabled),
                    (1, &(index as u64)),
                    (2, &sensor),
                    (3, &selector),
                    (4, &action),
                    (5, &metadata),
                ],
            );
        }

        rules_box.show();

        // restore selection state
        if !sel.0.is_empty() {
            rules_treeview.selection().select_path(&sel.0[0]);
        }

        Ok(())
    } else {
        notification_box.show_now();
        rules_box.hide();

        Err(ProcessMonitorError::ConnectionFailed {}.into())
    }
}

/// Updates the ruleset of the eruption-process-monitor daemon to match the current state of the GUI
pub fn transmit_rules_to_process_monitor(builder: &gtk::Builder) -> Result<()> {
    let rules_treeview: gtk::TreeView = builder.object("rules_treeview").unwrap();

    let mut rules: Vec<(String, String, String, String)> = Vec::new();

    // generate a Vec<_> from the ruleset
    rules_treeview
        .model()
        .unwrap()
        .foreach(|model, _path, iter| {
            let metadata = model.value(iter, 5).get::<String>().unwrap();

            if !metadata.contains("internal") {
                let enabled = model.value(iter, 0).get::<bool>().unwrap();

                let sensor = model.value(iter, 2).get::<String>().unwrap();
                let selector = model.value(iter, 3).get::<String>().unwrap();
                let action = model.value(iter, 4).get::<String>().unwrap();

                let metadata = format!(
                    "{},user-defined",
                    if enabled { "enabled" } else { "disabled" },
                );

                rules.push((sensor, selector, action, metadata));
            } else {
                let sensor = model.value(iter, 2).get::<String>().unwrap();
                let selector = model.value(iter, 3).get::<String>().unwrap();
                let action = model.value(iter, 4).get::<String>().unwrap();
                let metadata = model.value(iter, 5).get::<String>().unwrap();

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
pub fn initialize_automation_rules_page<A: IsA<gtk::Application>>(
    application: &A,
    builder: &gtk::Builder,
) -> Result<()> {
    let application = application.as_ref();

    let main_window: gtk::ApplicationWindow = builder.object("main_window").unwrap();

    let notification_box: gtk::Box = builder.object("notification_box").unwrap();

    let rules_box: gtk::Box = builder.object("rules_box").unwrap();

    let restart_process_monitor_button: gtk::Button = builder
        .object("restart_process_monitor_button_global")
        .unwrap();

    let rules_treeview: gtk::TreeView = builder.object("rules_treeview").unwrap();

    let rules_treestore = gtk::TreeStore::new(&[
        glib::Type::BOOL,
        glib::Type::U64,
        String::static_type(),
        String::static_type(),
        String::static_type(),
        String::static_type(),
    ]);

    rules_treeview.set_model(Some(&rules_treestore));

    // register actions
    let add_rule = gio::SimpleAction::new("add-rule", None);
    add_rule.connect_activate(clone!(@weak main_window, @weak builder, @weak rules_treeview, @weak notification_box, @weak rules_box => move |_, _| {
        let (response, rule) = rule::show_new_rule_dialog(&main_window);
        if response == gtk::ResponseType::Ok {
            let rule = rule.unwrap();

            let tree_model = rules_treeview.model().unwrap();

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
                &[
                    (0, &rule.enabled),
                    (1, &(index as u64)),
                    (2, &rule.sensor),
                    (3, &rule.selector),
                    (4, &rule.action),
                    (5, &metadata),
                ],
            );

            if let Err(e) = transmit_rules_to_process_monitor(&builder) {
                tracing::error!("{}", e);

                let message = "Could not transmit ruleset".to_string();
                let secondary =
                    format!("Could not transmit the ruleset to the eruption-process-monitor daemon {e}");

                let message_dialog = MessageDialog::builder()
                    .parent(&main_window)
                    .destroy_with_parent(true)
                    .decorated(true)
                    .message_type(gtk::MessageType::Error)
                    .text(message)
                    .secondary_text(secondary)
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

    // application.add_action(&add_rule);
    application.set_accels_for_action("app.add-rule", &["<primary><shift>n"]);

    let remove_rule = gio::SimpleAction::new("remove-rule", None);
    remove_rule.connect_activate(clone!(@weak builder, @weak notification_box, @weak rules_box,
                                        @weak main_window, @weak rules_treeview => move |_, _| {

        let selection = &rules_treeview.selection().selected_rows().0;
        let rules_treestore: gtk::TreeStore = rules_treeview.model().unwrap().downcast::<gtk::TreeStore>().unwrap();

        if !selection.is_empty() {
            for p in selection.iter() {
                rules_treestore.remove(&rules_treestore.iter(p).unwrap());
            }

            if let Err(e) = transmit_rules_to_process_monitor(&builder) {
                tracing::error!("{}", e);

                let message = "Could not transmit ruleset".to_string();
                let secondary =
                    format!("Could not transmit the ruleset to the eruption-process-monitor daemon {e}");

                let message_dialog = MessageDialog::builder()
                    .parent(&main_window)
                    .destroy_with_parent(true)
                    .decorated(true)
                    .message_type(gtk::MessageType::Error)
                    .text(message)
                    .secondary_text(secondary)
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
        clone!(@weak builder, @weak rules_box, @weak notification_box => move |_, _| {
            // update the rules view or show an error notification
            if let Err(e) = update_rules_view(&builder) {
                tracing::error!("Could not update the rules view: {}", e);

                notification_box.show_now();
                rules_box.hide();
            } else {
                notification_box.hide();
                rules_box.show();
            }
        }),
    );

    update_view.set_enabled(true);

    application.add_action(&update_view);
    application.set_accels_for_action("app.update-rules-view", &[]);

    let edit_rule = gio::SimpleAction::new("edit-rule", None);
    edit_rule.connect_activate(clone!(@weak builder, @weak notification_box, @weak rules_box,
                                        @weak main_window, @weak rules_treeview => move |_, _| {

        let selection = &rules_treeview.selection().selected_rows();
        let p = &selection.0;

        if !p.is_empty() {
            let p = &p[0];

            let rules_treestore: gtk::TreeStore = rules_treeview.model().unwrap().downcast::<gtk::TreeStore>().unwrap();

            let rule_enabled = rules_treestore.value(&rules_treestore.iter(p).unwrap(), 0).get::<bool>().unwrap();
            let index = rules_treestore.value(&rules_treestore.iter(p).unwrap(), 1).get::<u64>().unwrap();
            let sensor = rules_treestore.value(&rules_treestore.iter(p).unwrap(), 2).get::<String>().unwrap();
            let selector = rules_treestore.value(&rules_treestore.iter(p).unwrap(), 3).get::<String>().unwrap();
            let action = rules_treestore.value(&rules_treestore.iter(p).unwrap(), 4).get::<String>().unwrap();
            let metadata = rules_treestore.value(&rules_treestore.iter(p).unwrap(), 5).get::<String>().unwrap();

            let rule = rule::Rule::new(Some(index as usize), rule_enabled, sensor, selector, action, metadata);

            let (response, rule) = rule::show_edit_rule_dialog(&main_window, &rule);
            if response == gtk::ResponseType::Ok {
                let rule = rule.unwrap();

                let tree_model: gtk::TreeModel = rules_treeview.model().unwrap();
                let rules_treestore: gtk::TreeStore = tree_model.downcast::<gtk::TreeStore>().unwrap();

                let index = rule.index.unwrap() as u64;

                rules_treestore.insert_with_values(
                    None,
                    None,
                    &[
                        (0, &rule.enabled),
                        (1, &index),
                        (2, &rule.sensor),
                        (3, &rule.selector),
                        (4, &rule.action),
                        (5, &rule.metadata),
                    ],
                );

                // remove original item
                rules_treestore.remove(&rules_treestore.iter(p).unwrap());

                if let Err(e) = transmit_rules_to_process_monitor(&builder) {
                    tracing::error!("{}", e);

                    let message = "Could not transmit ruleset".to_string();
                    let secondary =
                        format!("Could not transmit the ruleset to the eruption-process-monitor daemon {e}");

                    let message_dialog = MessageDialog::builder()
                        .parent(&main_window)
                        .destroy_with_parent(true)
                        .decorated(true)
                        .message_type(gtk::MessageType::Error)
                        .text(message)
                        .secondary_text(secondary)
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
    rules_treeview.selection().connect_changed(
        clone!(@weak edit_rule, @weak remove_rule  => move |sel| {
            if !sel.selected_rows().0.is_empty() {
                edit_rule.set_enabled(true);
                remove_rule.set_enabled(true);
            } else {
                edit_rule.set_enabled(false);
                remove_rule.set_enabled(false);
            }
        }),
    );

    // let action_group = gio::SimpleActionGroup::new();

    // action_group.add_action(&add_rule);
    // action_group.add_action(&remove_rule);
    // action_group.add_action(&edit_rule);
    // action_group.add_action(&update_view);

    // application.set_action_group(Some(&action_group));

    rules_treeview.selection().connect_mode_notify(
        clone!(@weak edit_rule, @weak remove_rule  => move |_sel| {

        }),
    );

    // hide all notifications
    notification_box.hide();

    restart_process_monitor_button.connect_clicked(
        clone!(@weak builder, @weak application => move |_| {
            util::restart_process_monitor_daemon().unwrap_or_else(|e| tracing::error!("{}", e));

            glib::timeout_add_local(
                Duration::from_millis(1000),
                clone!(@weak builder => @default-return Continue(true), move || {
                    if let Err(e) = update_automation_rules_page(&builder) {
                        tracing::error!("{}", e);
                        Continue(true)
                    } else {
                        Continue(false)
                    }
                }),
            );
        }),
    );

    let enabled_column = TreeViewColumn::builder()
        .title("Enabled")
        .sizing(gtk::TreeViewColumnSizing::Autosize)
        .resizable(false)
        .build();

    let index_column = TreeViewColumn::builder()
        .title("#")
        .sizing(gtk::TreeViewColumnSizing::Autosize)
        .resizable(false)
        .build();

    let sensor_column = TreeViewColumn::builder()
        .title("Sensor")
        .sizing(gtk::TreeViewColumnSizing::Autosize)
        .build();

    let selector_column = TreeViewColumn::builder()
        .title("Selector")
        .sizing(gtk::TreeViewColumnSizing::Autosize)
        .resizable(true)
        .build();

    let action_column = TreeViewColumn::builder().title("Action").build();

    let metadata_column = TreeViewColumn::builder().title("Metadata").build();

    let cell_renderer_toggle = gtk::CellRendererToggle::new();
    let cell_renderer_text = gtk::CellRendererText::new();

    cell_renderer_toggle.connect_toggled(clone!(@weak builder, @weak rules_treeview => move |_cr, p| {
            let rules_treestore: gtk::TreeStore = rules_treeview.model().unwrap().downcast::<gtk::TreeStore>().unwrap();

            let value = rules_treestore.value(&rules_treestore.iter(&p).unwrap(), 0).get::<bool>().unwrap();
            rules_treestore.set_value(&rules_treestore.iter(&p).unwrap(), 0, &(!value).to_value());

            transmit_rules_to_process_monitor(&builder).unwrap_or_else(|e| tracing::error!("{}", e));
        }));

    gtk::prelude::CellLayoutExt::pack_start(&enabled_column, &cell_renderer_toggle, false);
    gtk::prelude::CellLayoutExt::pack_start(&index_column, &cell_renderer_text, false);
    gtk::prelude::CellLayoutExt::pack_start(&sensor_column, &cell_renderer_text, false);
    gtk::prelude::CellLayoutExt::pack_start(&selector_column, &cell_renderer_text, true);
    gtk::prelude::CellLayoutExt::pack_start(&action_column, &cell_renderer_text, true);
    gtk::prelude::CellLayoutExt::pack_start(&metadata_column, &cell_renderer_text, false);

    rules_treeview
        .columns()
        .iter()
        .for_each(clone!(@weak rules_treeview => move |c| {
            rules_treeview.remove_column(c);
        }));

    rules_treeview.insert_column(&enabled_column, 0);
    rules_treeview.insert_column(&index_column, 1);
    rules_treeview.insert_column(&sensor_column, 2);
    rules_treeview.insert_column(&selector_column, 3);
    rules_treeview.insert_column(&action_column, 4);
    rules_treeview.insert_column(&metadata_column, 5);

    gtk::prelude::TreeViewColumnExt::add_attribute(
        &enabled_column,
        &cell_renderer_toggle,
        "active",
        0,
    );
    gtk::prelude::TreeViewColumnExt::add_attribute(&index_column, &cell_renderer_text, "text", 1);
    gtk::prelude::TreeViewColumnExt::add_attribute(&sensor_column, &cell_renderer_text, "text", 2);
    gtk::prelude::TreeViewColumnExt::add_attribute(
        &selector_column,
        &cell_renderer_text,
        "text",
        3,
    );
    gtk::prelude::TreeViewColumnExt::add_attribute(&action_column, &cell_renderer_text, "text", 4);
    gtk::prelude::TreeViewColumnExt::add_attribute(
        &metadata_column,
        &cell_renderer_text,
        "text",
        5,
    );

    // update the rules view or show an error notification
    update_rules_view(builder).unwrap_or_else(
        clone!(@weak notification_box,@weak rules_box => move |_e| {
            notification_box.show_now();
            rules_box.hide();
        }),
    );

    timers::register_timer(
        timers::PROCESS_MONITOR_TIMER_ID,
        TimerMode::ActiveStackPage(Pages::AutomationRules as u8),
        1000,
        clone!(@weak builder => @default-return Ok(()), move || {
            let _result = update_rules_view(&builder).map_err(|e| tracing::error!("Could not poll eruption-process-monitor ruleset: {e}"));

            Ok(())
        }),
    )?;

    Ok(())
}

/// Initialize page "Process Monitor"
pub fn update_automation_rules_page(builder: &gtk::Builder) -> Result<()> {
    // let main_window: gtk::ApplicationWindow = builder.object("main_window").unwrap();

    let notification_box: gtk::Box = builder.object("notification_box").unwrap();

    let rules_box: gtk::Box = builder.object("rules_box").unwrap();

    // let restart_process_monitor_button: gtk::Button = builder
    //     .object("restart_process_monitor_button_global")
    //     .unwrap();

    // let rules_treeview: gtk::TreeView = builder.object("rules_treeview").unwrap();

    // update the rules view or show an error notification
    update_rules_view(builder).unwrap_or_else(
        clone!(@weak notification_box, @strong rules_box => move |_e| {
            notification_box.show_now();
            rules_box.hide();
        }),
    );

    Ok(())
}
