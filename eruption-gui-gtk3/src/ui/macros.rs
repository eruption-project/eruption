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




use glib::IsA;
use gtk::glib;
use gtk::{
    prelude::*, Builder, TreeView,
};







#[cfg(not(feature = "sourceview"))]
use gtk::ApplicationWindow;
#[cfg(not(feature = "sourceview"))]
use gtk::{TextBuffer, TextView};

use std::path::{Path, PathBuf};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

type Result<T> = std::result::Result<T, eyre::Error>;

cfg_if::cfg_if! {
    if #[cfg(feature = "sourceview")] {
        thread_local! {
            /// Holds the source code buffers and the respective paths in the file system
            static TEXT_BUFFERS: Rc<RefCell<HashMap<PathBuf, (usize, sourceview4::Buffer)>>> = Rc::new(RefCell::new(HashMap::new()));
        }
    } else {
        thread_local! {
            /// Holds the source code buffers and the respective paths in the file system
            static TEXT_BUFFERS: Rc<RefCell<HashMap<PathBuf, (usize, TextBuffer)>>> = Rc::new(RefCell::new(HashMap::new()));
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum MacrosError {
    #[error("Unknown error: {description}")]
    UnknownError { description: String },

    #[error("Parameter has an invalid data type")]
    TypeMismatch {},

    #[error("Method call failed: {description}")]
    MethodCallError { description: String },
}

macro_rules! declare_config_widget_numeric {
    (i64) => {
        paste! {
            fn [<build_config_widget_ i64>] <F: Fn(i64) + 'static>(
                name: &str,
                description: &str,
                default: i64,
                min: Option<i64>,
                max: Option<i64>,
                value:i64,
                callback: F,
            ) -> Result<gtk::Box> {
                let container = Box::builder()
                    .border_width(16)
                    .halign(Align::Fill)
                    .valign(Align::Fill)
                    .orientation(Orientation::Vertical)
                    .homogeneous(false)
                    .build();

                let row1 = Box::builder()
                    .halign(Align::Fill)
                    .valign(Align::Fill)
                    .spacing(8)
                    .orientation(Orientation::Horizontal)
                    .homogeneous(false)
                    .build();

                container.pack_start(&row1, true, true, 8);

                let row2 = Box::builder()
                    .halign(Align::Fill)
                    .valign(Align::Fill)
                    .spacing(8)
                    .orientation(Orientation::Horizontal)
                    .homogeneous(false)
                    .build();

                container.pack_start(&row2, true, true, 8);

                let label = Label::builder()
                    .expand(false)
                    .halign(Align::Start)
                    .justify(Justification::Left)
                    .use_markup(true)
                    .label(&format!("<b>{}</b>", name))
                    .build();

                row1.pack_start(&label, false, false, 8);

                let label = Label::builder()
                    .expand(false)
                    .halign(Align::Start)
                    .justify(Justification::Left)
                    .label(description)
                    .build();

                row1.pack_start(&label, false, false, 8);

                // "reset to default value" button
                let image = Image::from_icon_name(Some("reload"), IconSize::Button);
                let reset_button = Button::builder()
                    .halign(Align::Start)
                    .image(&image)
                    .tooltip_text("Reset this parameter to its default value")
                    .build();

                row2.pack_start(&reset_button, false, false, 8);

                // scale widget
                // set constraints
                let mut adjustment = Adjustment::builder();

                adjustment = adjustment.value(value as f64);
                adjustment = adjustment.step_increment(1.0);

                if let Some(min) = min {
                    adjustment = adjustment.lower(min as f64);
                }

                if let Some(max) = max {
                    adjustment = adjustment.upper(max as f64);
                }

                let adjustment = adjustment.build();

                let scale = Scale::builder()
                    .halign(Align::Fill)
                    .hexpand(true)
                    .adjustment(&adjustment)
                    .digits(0)
                    .value_pos(PositionType::Left)
                    .build();

                row2.pack_start(&scale, false, true, 8);

                scale.connect_value_changed(move |c| {
                    let value = c.value() as i64;
                    callback(value);
                });

                reset_button.connect_clicked(clone!(@weak adjustment => move |_b| {
                    adjustment.set_value(default as f64);
                }));

                Ok(container)
            }
        }
    };

    ($t:ty) => {
        paste! {
            fn [<build_config_widget_ $t>] <F: Fn($t) + 'static>(
                name: &str,
                description: &str,
                default: $t,
                min: Option<$t>,
                max: Option<$t>,
                value: $t,
                callback: F,
            ) -> Result<gtk::Box> {
                let container = Box::builder()
                    .border_width(16)
                    .halign(Align::Fill)
                    .valign(Align::Fill)
                    .orientation(Orientation::Vertical)
                    .homogeneous(false)
                    .build();

                let row1 = Box::builder()
                    .halign(Align::Fill)
                    .valign(Align::Fill)
                    .spacing(8)
                    .orientation(Orientation::Horizontal)
                    .homogeneous(false)
                    .build();

                container.pack_start(&row1, true, true, 8);

                let row2 = Box::builder()
                    .halign(Align::Fill)
                    .valign(Align::Fill)
                    .spacing(8)
                    .orientation(Orientation::Horizontal)
                    .homogeneous(false)
                    .build();

                container.pack_start(&row2, true, true, 8);

                let label = Label::builder()
                    .expand(false)
                    .halign(Align::Start)
                    .justify(Justification::Left)
                    .use_markup(true)
                    .label(&format!("<b>{}</b>", name))
                    .build();

                row1.pack_start(&label, false, false, 8);

                let label = Label::builder()
                    .expand(false)
                    .halign(Align::Start)
                    .justify(Justification::Left)
                    .label(description)
                    .build();

                row1.pack_start(&label, false, false, 8);

                // "reset to default value" button
                let image = Image::from_icon_name(Some("reload"), IconSize::Button);
                let reset_button = Button::builder()
                    .halign(Align::Start)
                    .image(&image)
                    .tooltip_text("Reset this parameter to its default value")
                    .build();

                row2.pack_start(&reset_button, false, false, 8);

                // scale widget
                // set constraints
                let mut adjustment = Adjustment::builder();

                adjustment = adjustment.value(value as f64);
                adjustment = adjustment.step_increment(0.01);

                if let Some(min) = min {
                    adjustment = adjustment.lower(min as f64);
                }

                if let Some(max) = max {
                    adjustment = adjustment.upper(max as f64);
                }

                let adjustment = adjustment.build();

                let scale = Scale::builder()
                    .halign(Align::Fill)
                    .hexpand(true)
                    .adjustment(&adjustment)
                    .digits(2)
                    .value_pos(PositionType::Left)
                    .build();

                row2.pack_start(&scale, false, true, 8);

                scale.connect_value_changed(move |c| {
                    let value = c.value() as $t;
                    callback(value);
                });

                reset_button.connect_clicked(clone!(@weak adjustment => move |_b| {
                    adjustment.set_value(default as f64);
                }));

                Ok(container)
            }
        }
    };
}

macro_rules! declare_config_widget_input {
    ($t:ty) => {
        paste! {
            fn [<build_config_widget_input_ $t:lower>] <F: Fn($t) + 'static>(
                name: &str,
                description: &str,
                default: String,
                value: String,
                callback: F,
            ) -> Result<gtk::Box> {
                let container = Box::builder()
                    .border_width(16)
                    .halign(Align::Fill)
                    .valign(Align::Fill)
                    .orientation(Orientation::Vertical)
                    .homogeneous(false)
                    .build();

                let row1 = Box::builder()
                    .halign(Align::Fill)
                    .valign(Align::Fill)
                    .spacing(8)
                    .orientation(Orientation::Horizontal)
                    .homogeneous(false)
                    .build();

                container.pack_start(&row1, true, true, 8);

                let row2 = Box::builder()
                    .halign(Align::Fill)
                    .valign(Align::Fill)
                    .spacing(8)
                    .orientation(Orientation::Horizontal)
                    .homogeneous(false)
                    .build();

                container.pack_start(&row2, true, true, 8);

                let label = Label::builder()
                    .expand(false)
                    .halign(Align::Start)
                    .justify(Justification::Left)
                    .use_markup(true)
                    .label(&format!("<b>{}</b>", name))
                    .build();

                row1.pack_start(&label, false, false, 8);

                let label = Label::builder()
                    .expand(false)
                    .halign(Align::Start)
                    .justify(Justification::Left)
                    .label(description)
                    .build();

                row1.pack_start(&label, false, false, 8);

                // "reset to default value" button
                let image = Image::from_icon_name(Some("reload"), IconSize::Button);
                let reset_button = Button::builder()
                    .halign(Align::Start)
                    .image(&image)
                    .tooltip_text("Reset this parameter to its default value")
                    .build();

                row2.pack_start(&reset_button, false, false, 8);

                // entry widget
                let entry = Entry::builder().text(&value).build();

                row2.pack_start(&entry, false, true, 8);

                entry.connect_changed(move |e| {
                    let value = e.text();
                    callback(value.to_string());
                });

                reset_button.connect_clicked(clone!(@weak entry, @strong default => move |_b| {
                    entry.set_text(&default);
                }));

                Ok(container)
            }
        }
    };
}

macro_rules! declare_config_widget_color {
    ($t:ty) => {
        paste! {
            fn [<build_config_widget_color_ $t>] <F: Clone + Fn($t) + 'static>(
                name: &str,
                description: &str,
                default: $t,
                _min: Option<$t>,
                _max: Option<$t>,
                value: $t,
                callback: F,
            ) -> Result<gtk::Box> {
                let container = Box::builder()
                    .border_width(16)
                    .halign(Align::Fill)
                    .valign(Align::Fill)
                    .orientation(Orientation::Vertical)
                    .homogeneous(false)
                    .build();

                let row1 = Box::builder()
                    .halign(Align::Fill)
                    .valign(Align::Fill)
                    .spacing(8)
                    .orientation(Orientation::Horizontal)
                    .homogeneous(false)
                    .build();

                container.pack_start(&row1, true, true, 8);

                let row2 = Box::builder()
                    .halign(Align::Fill)
                    .valign(Align::Fill)
                    .spacing(8)
                    .orientation(Orientation::Horizontal)
                    .homogeneous(false)
                    .build();

                container.pack_start(&row2, true, true, 8);

                let label = Label::builder()
                    .expand(false)
                    .halign(Align::Start)
                    .justify(Justification::Left)
                    .use_markup(true)
                    .label(&format!("<b>{}</b>", name))
                    .build();

                row1.pack_start(&label, false, false, 8);

                let label = Label::builder()
                    .expand(false)
                    .halign(Align::Start)
                    .justify(Justification::Left)
                    .label(description)
                    .build();

                row1.pack_start(&label, false, false, 8);

                // "reset to default value" button
                let image = Image::from_icon_name(Some("reload"), IconSize::Button);
                let reset_button = Button::builder()
                    .halign(Align::Start)
                    .image(&image)
                    .tooltip_text("Reset this parameter to its default value")
                    .build();

                row2.pack_start(&reset_button, false, false, 8);

                // color chooser widget
                let rgba = util::color_to_gdk_rgba(value);
                let chooser = ColorButton::builder()
                    .rgba(&rgba)
                    .use_alpha(true)
                    .show_editor(true)
                    .build();

                row2.pack_start(&chooser, false, true, 8);

                chooser.connect_color_set(clone!(@strong callback => move |c| {
                    let color = c.rgba();
                    let value = util::gdk_rgba_to_color(&color);

                    callback(value);
                }));

                reset_button.connect_clicked(clone!(@strong callback, @strong chooser => move |_b| {
                    chooser.set_rgba(&util::color_to_gdk_rgba(default));
                    callback(default);
                }));

                Ok(container)
            }
        }
    };
}

macro_rules! declare_config_widget_switch {
    ($t:ty) => {
        paste! {
            fn [<build_config_widget_switch_ $t>] <F: Fn($t) + 'static>(
                name: &str,
                description: &str,
                default: $t,
                value: $t,
                callback: F,
            ) -> Result<gtk::Box> {
                let container = Box::builder()
                    .border_width(16)
                    .halign(Align::Fill)
                    .valign(Align::Fill)
                    .orientation(Orientation::Vertical)
                    .homogeneous(false)
                    .build();

                let row1 = Box::builder()
                    .halign(Align::Fill)
                    .valign(Align::Fill)
                    .spacing(8)
                    .orientation(Orientation::Horizontal)
                    .homogeneous(false)
                    .build();

                container.pack_start(&row1, true, true, 8);

                let row2 = Box::builder()
                    .halign(Align::Fill)
                    .valign(Align::Fill)
                    .spacing(8)
                    .orientation(Orientation::Horizontal)
                    .homogeneous(false)
                    .build();

                container.pack_start(&row2, true, true, 8);

                let label = Label::builder()
                    .expand(false)
                    .halign(Align::Start)
                    .justify(Justification::Left)
                    .use_markup(true)
                    .label(&format!("<b>{}</b>", name))
                    .build();

                row1.pack_start(&label, false, false, 8);

                let label = Label::builder()
                    .expand(false)
                    .halign(Align::Start)
                    .justify(Justification::Left)
                    .label(description)
                    .build();

                row1.pack_start(&label, false, false, 8);

                // "reset to default value" button
                let image = Image::from_icon_name(Some("reload"), IconSize::Button);
                let reset_button = Button::builder()
                    .halign(Align::Start)
                    .image(&image)
                    .tooltip_text("Reset this parameter to its default value")
                    .build();

                row2.pack_start(&reset_button, false, false, 8);

                // switch widget
                let switch = Switch::builder()
                    .expand(false)
                    .valign(Align::Center)
                    .state(value)
                    .build();

                row2.pack_start(&switch, false, false, 8);

                switch.connect_changed_active(move |s| {
                    let value = s.state();
                    callback(value);
                });

                reset_button.connect_clicked(clone!(@weak switch => move |_| {
                    switch.set_state(default);
                }));

                Ok(container)
            }
        }
    };
}

/// Populate the configuration tab with settings/GUI controls
fn populate_visual_config_editor<P: AsRef<Path>>(_builder: &Builder, _profile: P) -> Result<()> {
    Ok(())
}

/// Initialize page "Macros"
pub fn initialize_macros_page<A: IsA<gtk::Application>>(
    _application: &A,
    builder: &Builder,
) -> Result<()> {
    let _profiles_treeview: TreeView = builder.object("profiles_treeview").unwrap();
    // let sourceview: sourceview4::View = builder.object("source_view").unwrap();

    // profiles list
    // let profiles_treestore = TreeStore::new(&[
    //     glib::Type::U64,
    //     String::static_type(),
    //     String::static_type(),
    //     String::static_type(),
    // ]);

    // for (index, profile) in util::enumerate_profiles()
    //     .unwrap_or_else(|_| vec![])
    //     .iter()
    //     .enumerate()
    // {
    //     let name = &profile.name;
    //     let filename = profile
    //         .profile_file
    //         .file_name()
    //         .unwrap_or_else(|| OsStr::new("<error>"))
    //         .to_string_lossy()
    //         .to_owned()
    //         .to_string();

    //     let path = profile
    //         .profile_file
    //         // .file_name()
    //         // .unwrap_or_else(|| OsStr::new("<error>"))
    //         .to_string_lossy()
    //         .to_owned()
    //         .to_string();

    //     profiles_treestore.insert_with_values(
    //         None,
    //         None,
    //         &[(0, &(index as u64)), (1, &name), (2, &filename), (3, &path)],
    //     );
    // }

    // let id_column = TreeViewColumn::builder()
    //     .title("ID")
    //     .sizing(TreeViewColumnSizing::Autosize)
    //     .visible(false)
    //     .build();
    // let name_column = TreeViewColumn::builder()
    //     .title("Name")
    //     .sizing(TreeViewColumnSizing::Autosize)
    //     .build();
    // let filename_column = TreeViewColumn::builder()
    //     .title("Filename")
    //     .sizing(TreeViewColumnSizing::Autosize)
    //     .build();
    // let path_column = TreeViewColumn::builder()
    //     .visible(false)
    //     .title("Path")
    //     .build();

    // let cell_renderer_id = CellRendererText::new();
    // let cell_renderer_name = CellRendererText::new();
    // let cell_renderer_filename = CellRendererText::new();

    // gtk::prelude::CellLayoutExt::pack_start(&id_column, &cell_renderer_id, false);
    // gtk::prelude::CellLayoutExt::pack_start(&name_column, &cell_renderer_name, true);
    // gtk::prelude::CellLayoutExt::pack_start(&filename_column, &cell_renderer_filename, true);

    // profiles_treeview.insert_column(&id_column, 0);
    // profiles_treeview.insert_column(&name_column, 1);
    // profiles_treeview.insert_column(&filename_column, 2);
    // profiles_treeview.insert_column(&path_column, 3);

    // gtk::prelude::TreeViewColumnExt::add_attribute(&id_column, &cell_renderer_id, "text", 0);
    // gtk::prelude::TreeViewColumnExt::add_attribute(&name_column, &cell_renderer_name, "text", 1);
    // gtk::prelude::TreeViewColumnExt::add_attribute(
    //     &filename_column,
    //     &cell_renderer_filename,
    //     "text",
    //     2,
    // );
    // gtk::prelude::TreeViewColumnExt::add_attribute(
    //     &path_column,
    //     &cell_renderer_filename,
    //     "text",
    //     3,
    // );

    // profiles_treeview.set_model(Some(&profiles_treestore));

    // profiles_treeview.connect_row_activated(clone!(@weak builder => move |tv, path, _column| {
    //     let profile = tv.model().unwrap().value(&tv.model().unwrap().iter(path).unwrap(), 3).get::<String>().unwrap();

    //     let _result = populate_visual_config_editor(&builder, &profile).map_err(|e| { tracing::error!("{}", e) });

    //     remove_elements_from_stack_widget(&builder);
    //     let _result = populate_stack_widget(&builder, &profile).map_err(|e| { tracing::error!("{}", e) });
    // }));

    // profiles_treeview.show_all();

    // update_profile_state(builder)?;
    // register_actions(application, builder)?;

    Ok(())
}

/// Register global actions and keyboard accelerators
fn register_actions<A: IsA<gtk::Application>>(_application: &A, _builder: &Builder) -> Result<()> {
    // let application = application.as_ref();

    // let stack_widget: Stack = builder.object("profile_stack").unwrap();
    // // let stack_switcher: StackSwitcher = builder.object("profile_stack_switcher").unwrap();

    // let save_current_buffer = gio::SimpleAction::new("save-current-buffer", None);
    // save_current_buffer.connect_activate(clone!(@weak builder => move |_, _| {
    //     if let Some(view) = stack_widget.visible_child()
    //     // .map(|w| w.dynamic_cast::<sourceview4::View>().unwrap())
    //     {
    //         let index = stack_widget.child_position(&view) as usize;

    //         TEXT_BUFFERS.with(|b| {
    //             if let Some((path, buffer)) = b
    //                 .borrow()
    //                 .iter()
    //                 .find(|v| v.1 .0 == index)
    //                 .map(|v| (v.0, &v.1 .1))
    //             {
    //                 let _result = save_buffer_contents_to_file(&path, buffer, &builder);
    //             }
    //         });
    //     }
    // }));

    // application.add_action(&save_current_buffer);
    // application.set_accels_for_action("app.save-current-buffer", &["<Primary>S"]);

    // let save_all_buffers = gio::SimpleAction::new("save-all-buffers", None);
    // save_all_buffers.connect_activate(clone!(@weak builder => move |_, _| {
    //     TEXT_BUFFERS.with(|b| {
    //         'SAVE_LOOP: for (k, (_, v)) in b.borrow().iter() {
    //             let result = save_buffer_contents_to_file(&k, v, &builder);

    //             // stop saving files if an error occurred, or auth has failed
    //             if result.is_err() {
    //                 break 'SAVE_LOOP;
    //             }
    //         }
    //     });
    // }));

    // application.add_action(&save_all_buffers);
    // application.set_accels_for_action("app.save-all-buffers", &["<Primary><Shift>S"]);

    Ok(())
}

pub fn update_macros_state(_builder: &Builder) -> Result<()> {
    // let profiles_treeview: TreeView = builder.object("profiles_treeview").unwrap();

    // let model = profiles_treeview.model().unwrap();

    // let state = crate::STATE.read();
    // let active_profile = state.active_profile.clone().unwrap_or_default();

    // model.foreach(|model, path, iter| {
    //     let item = model.value(iter, 3).get::<String>().unwrap();
    //     if item == active_profile {
    //         // found a match
    //         profiles_treeview.selection().select_iter(iter);
    //         profiles_treeview.row_activated(path, &profiles_treeview.column(1).unwrap());

    //         true
    //     } else {
    //         false
    //     }
    // });

    Ok(())
}
