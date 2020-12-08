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

use crate::dbus_client;

type Result<T> = std::result::Result<T, eyre::Error>;

/// Initialize page "Process Monitor"
pub fn initialize_process_monitor_page(builder: &gtk::Builder) -> Result<()> {
    let rules_treeview: gtk::TreeView = builder.get_object("rules_treeview").unwrap();

    // rules list
    let rules_treestore = gtk::TreeStore::new(&[
        glib::Type::U64,
        String::static_type(),
        String::static_type(),
        String::static_type(),
        String::static_type(),
    ]);

    for (index, rule) in dbus_client::enumerate_process_monitor_rules()?
        .iter()
        .enumerate()
    {
        let sensor = &rule.0;
        let selector = &rule.1;
        let action = &rule.2;
        let metadata = &rule.3;

        rules_treestore.insert_with_values(
            None,
            None,
            &[0, 1, 2, 3, 4],
            &[&(index as u64), &sensor, &selector, &action, &metadata],
        );
    }

    let index_column = gtk::TreeViewColumnBuilder::new()
        .title(&"#")
        .sizing(gtk::TreeViewColumnSizing::Autosize)
        .build();
    let sensor_column = gtk::TreeViewColumnBuilder::new()
        .title(&"Sensor")
        .sizing(gtk::TreeViewColumnSizing::Autosize)
        .build();
    let selector_column = gtk::TreeViewColumnBuilder::new()
        .title(&"Selector")
        .sizing(gtk::TreeViewColumnSizing::Autosize)
        .build();
    let action_column = gtk::TreeViewColumnBuilder::new().title(&"Action").build();
    let metadata_column = gtk::TreeViewColumnBuilder::new().title(&"Metadata").build();

    let cell_renderer_text = gtk::CellRendererText::new();

    index_column.pack_start(&cell_renderer_text, false);
    sensor_column.pack_start(&cell_renderer_text, false);
    selector_column.pack_start(&cell_renderer_text, true);
    action_column.pack_start(&cell_renderer_text, true);
    metadata_column.pack_start(&cell_renderer_text, false);

    rules_treeview.insert_column(&index_column, 0);
    rules_treeview.insert_column(&sensor_column, 1);
    rules_treeview.insert_column(&selector_column, 2);
    rules_treeview.insert_column(&action_column, 3);
    rules_treeview.insert_column(&metadata_column, 4);

    index_column.add_attribute(&cell_renderer_text, &"text", 0);
    sensor_column.add_attribute(&cell_renderer_text, &"text", 1);
    selector_column.add_attribute(&cell_renderer_text, &"text", 2);
    action_column.add_attribute(&cell_renderer_text, &"text", 3);
    metadata_column.add_attribute(&cell_renderer_text, &"text", 4);

    rules_treeview.set_model(Some(&rules_treestore));

    rules_treeview.connect_row_activated(clone!(@strong builder => move |_tv, _path, _column| {
        // let index = tv.get_model().unwrap().get_value(&tv.get_model().unwrap().get_iter(&path).unwrap(), 0).get::<u64>().unwrap().unwrap();
    }));

    rules_treeview.show_all();

    Ok(())
}
