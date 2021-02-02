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

use glib::{clone, StaticType};
use gtk::prelude::*;

use crate::{dbus_client, util};

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
    // let main_window: gtk::ApplicationWindow = builder.get_object("main_window").unwrap();
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

        cell_renderer_toggle.connect_toggled(clone!(@strong builder, @strong rules_treestore => move |_cr, p| {
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

        rules_treeview.set_model(Some(&rules_treestore));

        rules_treeview.connect_row_activated(
            clone!(@strong builder => move |_tv, _path, _column| {
                // let index = tv.get_model().unwrap().get_value(&tv.get_model().unwrap().get_iter(&path).unwrap(), 0).get::<u64>().unwrap().unwrap();
            }),
        );

        // rules_treeview.connect_button_press_event(clone!(@strong main_window => move |_, _e| {
        //         ui::rule::show_rule_dialog(&main_window);

        //         Inhibit(false)
        // }));

        rules_treeview.show_all();

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
pub fn initialize_process_monitor_page(builder: &gtk::Builder) -> Result<()> {
    let notification_box: gtk::Box = builder.get_object("notification_box").unwrap();
    let restart_process_monitor_button: gtk::Button = builder
        .get_object("restart_process_monitor_button")
        .unwrap();

    // hide all notifications
    notification_box.hide();

    restart_process_monitor_button.connect_clicked(clone!(@strong builder => move |_| {
        util::restart_process_monitor_daemon().unwrap_or_else(|e| log::error!("{}", e));

        glib::timeout_add_local(
            1000,
            clone!(@strong builder => move || {
                if let Err(e) = initialize_process_monitor_page(&builder) {
                    log::error!("{}", e);
                    Continue(true)
                } else {
                    Continue(false)
                }
            }),
        );
    }));

    // update the rules view or show an error notification
    update_rules_view(&builder).unwrap_or_else(|_e| {
        notification_box.show_now();
    });

    Ok(())
}
