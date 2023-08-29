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

use std::sync::Arc;

use colorgrad::BlendMode;
use glib::{clone, Cast, IsA, StaticType};
use gtk::{
    glib,
    prelude::{BuilderExtManual, TreeStoreExtManual},
    traits::{TreeModelExt, TreeSelectionExt, TreeStoreExt, TreeViewExt, WidgetExt},
    TreeViewColumn,
};
use lazy_static::lazy_static;
use parking_lot::RwLock;

use crate::{
    dbus_client, notifications,
    timers::{self, TimerMode},
};

use super::Pages;

type Result<T> = std::result::Result<T, eyre::Error>;

lazy_static! {
    /// Selected color-scheme for preview
    pub static ref SELECTED_COLOR_SCHEME: Arc<RwLock<Option<colorgrad::Gradient>>> = Arc::new(RwLock::new(None));
}

#[derive(Debug, thiserror::Error)]
pub enum ColorSchemesError {
    #[error("Connection to daemon failed")]
    ConnectionFailed,
    // #[error("Unknown error: {description}")]
    // UnknownError { description: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ColorSchemeType {
    UserScheme,
    StockGradient,
}

impl From<&ColorSchemeType> for String {
    fn from(ty: &ColorSchemeType) -> Self {
        match ty {
            ColorSchemeType::UserScheme => "User defined".to_string(),
            ColorSchemeType::StockGradient => "Stock gradient".to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColorSchemeEntry {
    pub name: String,
    pub ty: ColorSchemeType,
}

// impl ColorSchemeEntry {
//     pub fn new(name: String, ty: ColorSchemeType) -> Self {
//         Self { name, ty }
//     }
// }

/// Initialize page "Color Schemes"
pub fn update_color_schemes_view(builder: &gtk::Builder) -> Result<()> {
    // let application = application.as_ref();

    // let main_window: gtk::ApplicationWindow = builder.object("main_window").unwrap();

    let notification_box: gtk::Box = builder.object("notification_box").unwrap();

    let color_schemes_box: gtk::Box = builder.object("color_schemes_box").unwrap();
    let color_schemes_treeview: gtk::TreeView = builder.object("color_schemes_treeview").unwrap();

    if let Ok(user_color_schemes) = dbus_client::get_color_schemes() {
        let mut color_schemes = Vec::new();

        color_schemes.extend(user_color_schemes.iter().map(|e| ColorSchemeEntry {
            name: e.to_owned(),
            ty: ColorSchemeType::UserScheme,
        }));

        color_schemes.push(ColorSchemeEntry {
            name: "system".to_string(),
            ty: ColorSchemeType::StockGradient,
        });
        color_schemes.push(ColorSchemeEntry {
            name: "rainbow-smooth".to_string(),
            ty: ColorSchemeType::StockGradient,
        });
        color_schemes.push(ColorSchemeEntry {
            name: "sinebow-smooth".to_string(),
            ty: ColorSchemeType::StockGradient,
        });
        color_schemes.push(ColorSchemeEntry {
            name: "spectral-smooth".to_string(),
            ty: ColorSchemeType::StockGradient,
        });
        color_schemes.push(ColorSchemeEntry {
            name: "rainbow-sharp".to_string(),
            ty: ColorSchemeType::StockGradient,
        });
        color_schemes.push(ColorSchemeEntry {
            name: "sinebow-sharp".to_string(),
            ty: ColorSchemeType::StockGradient,
        });
        color_schemes.push(ColorSchemeEntry {
            name: "spectral-sharp".to_string(),
            ty: ColorSchemeType::StockGradient,
        });

        notification_box.hide();
        color_schemes_box.show();

        // save selection state
        let sel = color_schemes_treeview.selection().selected_rows();

        let color_schemes_treestore = color_schemes_treeview
            .model()
            .unwrap()
            .downcast::<gtk::TreeStore>()
            .unwrap();

        // clear the tree store
        color_schemes_treestore.clear();

        for (index, color_scheme) in color_schemes.iter().enumerate() {
            let ty = &color_scheme.ty;
            let name = &color_scheme.name;

            color_schemes_treestore.insert_with_values(
                None,
                None,
                &[
                    (0, &(index as u64)),
                    (1, &Into::<String>::into(ty)),
                    (2, &name),
                ],
            );
        }

        color_schemes_box.show();

        // restore selection state
        if !sel.0.is_empty() {
            color_schemes_treeview.selection().select_path(&sel.0[0]);
        }

        Ok(())
    } else {
        notification_box.show_now();
        color_schemes_box.hide();

        Err(ColorSchemesError::ConnectionFailed {}.into())
    }
}

// pub fn transmit_color_schemes_to_eruption(_builder: &gtk::Builder) -> Result<()> {
//     Ok(())
// }

fn paint_gradient(cr: &cairo::Context, width: f64, height: f64) -> Result<()> {
    if let Some(gradient) = SELECTED_COLOR_SCHEME.read().as_ref() {
        let segment_width = width / 100.0;

        for x in 0..width.round() as u32 {
            let color = gradient.at(x as f64 / width);
            cr.set_source_rgba(color.r, color.g, color.b, color.a);

            cr.rectangle(x as f64, 0.0, segment_width, height);
            cr.fill()?;
        }
    } else {
        // tracing::warn!("no color scheme selected");
    }

    Ok(())
}

/// Initialize page "Color Schemes"
pub fn initialize_color_schemes_page<A: IsA<gtk::Application>>(
    _application: &A,
    builder: &gtk::Builder,
) -> Result<()> {
    // let application = application.as_ref();

    // let main_window: gtk::ApplicationWindow = builder.object("main_window").unwrap();

    let notification_box: gtk::Box = builder.object("notification_box").unwrap();

    let drawing_area_color_scheme: gtk::DrawingArea =
        builder.object("drawing_area_color_scheme").unwrap();

    let color_schemes_box: gtk::Box = builder.object("color_schemes_box").unwrap();

    let color_schemes_treeview: gtk::TreeView = builder.object("color_schemes_treeview").unwrap();

    drawing_area_color_scheme.connect_draw(
        move |da: &gtk::DrawingArea, context: &cairo::Context| {
            let _ = paint_gradient(
                context,
                da.allocated_width() as f64,
                da.allocated_height() as f64,
            );

            glib::Propagation::Stop
        },
    );

    let color_schemes_treestore = gtk::TreeStore::new(&[
        glib::Type::U64,
        String::static_type(),
        String::static_type(),
    ]);

    color_schemes_treeview.set_model(Some(&color_schemes_treestore));

    // // register actions
    // let add_rule = gio::SimpleAction::new("add-rule", None);
    // add_rule.connect_activate(clone!(@weak main_window, @weak builder, @weak rules_treeview, @weak notification_box, @weak rules_box => move |_, _| {
    //     let (response, rule) = rule::show_new_rule_dialog(&main_window);
    //     if response == gtk::ResponseType::Ok {
    //         let rule = rule.unwrap();

    //         let tree_model = rules_treeview.model().unwrap();

    //         let mut counter = 1;
    //         tree_model.foreach(|_m, _p, _i| { counter += 1; true });

    //         let index = counter + 1;
    //         let metadata = format!(
    //             "{},user-defined",
    //             if rule.enabled { "enabled" } else { "disabled" },
    //         );

    //         let rules_treestore = tree_model.downcast::<gtk::TreeStore>().unwrap();
    //         rules_treestore.insert_with_values(
    //             None,
    //             None,
    //             &[
    //                 (0, &rule.enabled),
    //                 (1, &(index as u64)),
    //                 (2, &rule.sensor),
    //                 (3, &rule.selector),
    //                 (4, &rule.action),
    //                 (5, &metadata),
    //             ],
    //         );

    //         if let Err(e) = transmit_rules_to_process_monitor(&builder) {
    //             tracing::error!("{}", e);

    //             let message = "Could not transmit ruleset".to_string();
    //             let secondary =
    //                 format!("Could not transmit the ruleset to the eruption-process-monitor daemon {e}");

    //             let message_dialog = MessageDialog::builder()
    //                 .parent(&main_window)
    //                 .destroy_with_parent(true)
    //                 .decorated(true)
    //                 .message_type(gtk::MessageType::Error)
    //                 .text(message)
    //                 .secondary_text(secondary)
    //                 .title("Error")
    //                 .buttons(gtk::ButtonsType::Ok)
    //                 .build();

    //             message_dialog.run();
    //             message_dialog.hide();
    //         }

    //         if let Err(_e) = update_rules_view(&builder) {
    //             notification_box.show_now();
    //             rules_box.hide();
    //         }
    //     }
    // }));

    // add_rule.set_enabled(true);

    // application.add_action(&add_rule);
    // application.set_accels_for_action("app.add-rule", &["<primary><shift>n"]);

    // let remove_rule = gio::SimpleAction::new("remove-rule", None);
    // remove_rule.connect_activate(clone!(@weak builder, @weak notification_box, @weak rules_box,
    //                                     @weak main_window, @weak rules_treeview => move |_, _| {

    //     let selection = &rules_treeview.selection().selected_rows().0;
    //     let rules_treestore: gtk::TreeStore = rules_treeview.model().unwrap().downcast::<gtk::TreeStore>().unwrap();

    //     if !selection.is_empty() {
    //         for p in selection.iter() {
    //             rules_treestore.remove(&rules_treestore.iter(p).unwrap());
    //         }

    //         if let Err(e) = transmit_rules_to_process_monitor(&builder) {
    //             tracing::error!("{}", e);

    //             let message = "Could not transmit ruleset".to_string();
    //             let secondary =
    //                 format!("Could not transmit the ruleset to the eruption-process-monitor daemon {e}");

    //             let message_dialog = MessageDialog::builder()
    //                 .parent(&main_window)
    //                 .destroy_with_parent(true)
    //                 .decorated(true)
    //                 .message_type(gtk::MessageType::Error)
    //                 .text(message)
    //                 .secondary_text(secondary)
    //                 .title("Error")
    //                 .buttons(gtk::ButtonsType::Ok)
    //                 .build();

    //             message_dialog.run();
    //             message_dialog.hide();
    //         }

    //         if let Err(_e) = update_rules_view(&builder) {
    //             notification_box.show_now();
    //             rules_box.hide();
    //         }
    //     }
    // }));

    // remove_rule.set_enabled(true);

    // application.add_action(&remove_rule);
    // application.set_accels_for_action("app.remove-rule", &["<delete>"]);

    // let update_view = gio::SimpleAction::new("update-rules-view", None);
    // update_view.connect_activate(
    //     clone!(@weak builder, @weak rules_box, @weak notification_box => move |_, _| {
    //         // update the rules view or show an error notification
    //         if let Err(e) = update_rules_view(&builder) {
    //             tracing::error!("Could not update the rules view: {}", e);

    //             notification_box.show_now();
    //             rules_box.hide();
    //         } else {
    //             notification_box.hide();
    //             rules_box.show();
    //         }
    //     }),
    // );

    // update_view.set_enabled(true);

    // application.add_action(&update_view);
    // application.set_accels_for_action("app.update-rules-view", &[]);

    // let edit_rule = gio::SimpleAction::new("edit-rule", None);
    // edit_rule.connect_activate(clone!(@weak builder, @weak notification_box, @weak rules_box,
    //                                     @weak main_window, @weak rules_treeview => move |_, _| {

    //     let selection = &rules_treeview.selection().selected_rows();
    //     let p = &selection.0;

    //     if !p.is_empty() {
    //         let p = &p[0];

    //         let rules_treestore: gtk::TreeStore = rules_treeview.model().unwrap().downcast::<gtk::TreeStore>().unwrap();

    //         let rule_enabled = rules_treestore.value(&rules_treestore.iter(p).unwrap(), 0).get::<bool>().unwrap();
    //         let index = rules_treestore.value(&rules_treestore.iter(p).unwrap(), 1).get::<u64>().unwrap();
    //         let sensor = rules_treestore.value(&rules_treestore.iter(p).unwrap(), 2).get::<String>().unwrap();
    //         let selector = rules_treestore.value(&rules_treestore.iter(p).unwrap(), 3).get::<String>().unwrap();
    //         let action = rules_treestore.value(&rules_treestore.iter(p).unwrap(), 4).get::<String>().unwrap();
    //         let metadata = rules_treestore.value(&rules_treestore.iter(p).unwrap(), 5).get::<String>().unwrap();

    //         let rule = rule::Rule::new(Some(index as usize), rule_enabled, sensor, selector, action, metadata);

    //         let (response, rule) = rule::show_edit_rule_dialog(&main_window, &rule);
    //         if response == gtk::ResponseType::Ok {
    //             let rule = rule.unwrap();

    //             let tree_model: gtk::TreeModel = rules_treeview.model().unwrap();
    //             let rules_treestore: gtk::TreeStore = tree_model.downcast::<gtk::TreeStore>().unwrap();

    //             let index = rule.index.unwrap() as u64;

    //             rules_treestore.insert_with_values(
    //                 None,
    //                 None,
    //                 &[
    //                     (0, &rule.enabled),
    //                     (1, &index),
    //                     (2, &rule.sensor),
    //                     (3, &rule.selector),
    //                     (4, &rule.action),
    //                     (5, &rule.metadata),
    //                 ],
    //             );

    //             // remove original item
    //             rules_treestore.remove(&rules_treestore.iter(p).unwrap());

    //             if let Err(e) = transmit_rules_to_process_monitor(&builder) {
    //                 tracing::error!("{}", e);

    //                 let message = "Could not transmit ruleset".to_string();
    //                 let secondary =
    //                     format!("Could not transmit the ruleset to the eruption-process-monitor daemon {e}");

    //                 let message_dialog = MessageDialog::builder()
    //                     .parent(&main_window)
    //                     .destroy_with_parent(true)
    //                     .decorated(true)
    //                     .message_type(gtk::MessageType::Error)
    //                     .text(message)
    //                     .secondary_text(secondary)
    //                     .title("Error")
    //                     .buttons(gtk::ButtonsType::Ok)
    //                     .build();

    //                 message_dialog.run();
    //                 message_dialog.hide();
    //             }

    //             if let Err(_e) = update_rules_view(&builder) {
    //                 notification_box.show_now();
    //                 rules_box.hide();
    //             }
    //         }
    //     }
    // }));

    // edit_rule.set_enabled(true);

    // application.add_action(&edit_rule);
    // application.set_accels_for_action("app.edit-rule", &[]);

    // // enable/disable tool-buttons
    // rules_treeview.selection().connect_changed(
    //     clone!(@weak edit_rule, @weak remove_rule  => move |sel| {
    //         if !sel.selected_rows().0.is_empty() {
    //             edit_rule.set_enabled(true);
    //             remove_rule.set_enabled(true);
    //         } else {
    //             edit_rule.set_enabled(false);
    //             remove_rule.set_enabled(false);
    //         }
    //     }),
    // );

    // rules_treeview.selection().connect_mode_notify(
    //     clone!(@weak edit_rule, @weak remove_rule  => move |_sel| {

    //     }),
    // );

    // // hide all notifications
    // notification_box.hide();

    // restart_process_monitor_button.connect_clicked(
    //     clone!(@weak builder, @weak application => move |_| {
    //         util::restart_process_monitor_daemon().unwrap_or_else(|e| tracing::error!("{}", e));

    //         glib::timeout_add_local(
    //             Duration::from_millis(1000),
    //             clone!(@weak builder => @default-return Continue(true), move || {
    //                 if let Err(e) = update_process_monitor_page(&builder) {
    //                     tracing::error!("{}", e);
    //                     Continue(true)
    //                 } else {
    //                     Continue(false)
    //                 }
    //             }),
    //         );
    //     }),
    // );

    // let enabled_column = TreeViewColumn::builder()
    //     .title("Enabled")
    //     .sizing(gtk::TreeViewColumnSizing::Autosize)
    //     .resizable(false)
    //     .build();

    let index_column = TreeViewColumn::builder()
        .title("#")
        .sizing(gtk::TreeViewColumnSizing::Autosize)
        .resizable(false)
        .build();

    let type_column = TreeViewColumn::builder()
        .title("Type")
        .sizing(gtk::TreeViewColumnSizing::Autosize)
        .build();

    let name_column = TreeViewColumn::builder()
        .title("Name")
        .sizing(gtk::TreeViewColumnSizing::Autosize)
        .resizable(true)
        .build();

    let cell_renderer_text = gtk::CellRendererText::new();

    // gtk::prelude::CellLayoutExt::pack_start(&enabled_column, &cell_renderer_toggle, false);
    gtk::prelude::CellLayoutExt::pack_start(&index_column, &cell_renderer_text, false);
    gtk::prelude::CellLayoutExt::pack_start(&type_column, &cell_renderer_text, false);
    gtk::prelude::CellLayoutExt::pack_start(&name_column, &cell_renderer_text, true);

    // color_schemes_treeview
    //     .columns()
    //     .iter()
    //     .for_each(clone!(@weak color_schemes_treeview => move |c| {
    //         color_schemes_treeview.remove_column(c);
    //     }));

    color_schemes_treeview.insert_column(&index_column, 0);
    color_schemes_treeview.insert_column(&type_column, 1);
    color_schemes_treeview.insert_column(&name_column, 2);

    gtk::prelude::TreeViewColumnExt::add_attribute(&index_column, &cell_renderer_text, "text", 0);
    gtk::prelude::TreeViewColumnExt::add_attribute(&type_column, &cell_renderer_text, "text", 1);
    gtk::prelude::TreeViewColumnExt::add_attribute(&name_column, &cell_renderer_text, "text", 2);

    // selection changed handler
    color_schemes_treeview
        .selection()
        .connect_changed(move |sel| {
            if let Some((model, selection)) = sel.selected() {
                if let Ok(name) = model.value(&selection, 2).get::<String>() {
                    match name.as_str() {
                        // stock-gradients
                        "rainbow-smooth" => {
                            let gradient = colorgrad::rainbow();
                            *SELECTED_COLOR_SCHEME.write() = Some(gradient);

                            drawing_area_color_scheme.queue_draw();
                        }

                        "sinebow-smooth" => {
                            let gradient = colorgrad::sinebow();
                            *SELECTED_COLOR_SCHEME.write() = Some(gradient);

                            drawing_area_color_scheme.queue_draw();
                        }

                        "spectral-smooth" => {
                            let gradient = colorgrad::spectral();
                            *SELECTED_COLOR_SCHEME.write() = Some(gradient);

                            drawing_area_color_scheme.queue_draw();
                        }

                        "rainbow-sharp" => {
                            let gradient = colorgrad::rainbow().sharp(5, 0.15);
                            *SELECTED_COLOR_SCHEME.write() = Some(gradient);

                            drawing_area_color_scheme.queue_draw();
                        }

                        "sinebow-sharp" => {
                            let gradient = colorgrad::sinebow().sharp(5, 0.15);
                            *SELECTED_COLOR_SCHEME.write() = Some(gradient);

                            drawing_area_color_scheme.queue_draw();
                        }

                        "spectral-sharp" => {
                            let gradient = colorgrad::spectral().sharp(5, 0.15);
                            *SELECTED_COLOR_SCHEME.write() = Some(gradient);

                            drawing_area_color_scheme.queue_draw();
                        }

                        // special handling for the "system" color-scheme; it can be overridden by the user
                        "system" => match dbus_client::get_color_scheme(&name) {
                            Ok(color_scheme) => {
                                match colorgrad::CustomGradient::new()
                                    .mode(BlendMode::LinearRgb)
                                    .colors(&color_scheme.colors)
                                    .build()
                                {
                                    Ok(custom_gradient) => {
                                        *SELECTED_COLOR_SCHEME.write() = Some(custom_gradient);
                                        drawing_area_color_scheme.queue_draw();
                                    }

                                    Err(e) => {
                                        tracing::error!(
                                            "Could not instantiate a color scheme: {}",
                                            e
                                        );
                                        notifications::error(&format!(
                                            "Could not instantiate a color scheme: {}",
                                            e
                                        ));
                                    }
                                }
                            }

                            Err(_) => {
                                let gradient = colorgrad::sinebow();
                                *SELECTED_COLOR_SCHEME.write() = Some(gradient);

                                drawing_area_color_scheme.queue_draw();
                            }
                        },

                        // user-defined color-schemes
                        _ => match dbus_client::get_color_scheme(&name) {
                            Ok(color_scheme) => {
                                match colorgrad::CustomGradient::new()
                                    .mode(BlendMode::LinearRgb)
                                    .colors(&color_scheme.colors)
                                    .build()
                                {
                                    Ok(custom_gradient) => {
                                        *SELECTED_COLOR_SCHEME.write() = Some(custom_gradient);
                                        drawing_area_color_scheme.queue_draw();
                                    }

                                    Err(e) => {
                                        tracing::error!(
                                            "Could not instantiate a color scheme: {}",
                                            e
                                        );
                                        notifications::error(&format!(
                                            "Could not instantiate a color scheme: {}",
                                            e
                                        ));
                                    }
                                }
                            }

                            Err(e) => {
                                tracing::error!("Could not load color scheme: {}", e);
                                notifications::error(&format!(
                                    "Could not load color scheme: {}",
                                    e
                                ));
                            }
                        },
                    }
                }
            }
        });

    // update the color schemes view or show an error notification
    update_color_schemes_view(builder).unwrap_or_else(
        clone!(@weak notification_box,@weak color_schemes_box => move |_e| {
            notification_box.show_now();
            color_schemes_box.hide();
        }),
    );

    if let Some(iter) = color_schemes_treestore.iter_first() {
        color_schemes_treeview.selection().select_iter(&iter);
    }

    timers::register_timer(
        timers::COLOR_SCHEMES_TIMER_ID,
        TimerMode::ActiveStackPage(Pages::ColorSchemes as u8),
        1000,
        clone!(@weak builder => @default-return Ok(()), move || {
            let _result = update_color_schemes_view(&builder).map_err(|e| tracing::error!("Could not poll color schemes: {e}"));

            Ok(())
        }),
    )?;

    Ok(())
}

// /// Initialize page "Color Schemes"
// pub fn update_color_schemes_page(_builder: &gtk::Builder) -> Result<()> {
//     // let main_window: gtk::ApplicationWindow = builder.object("main_window").unwrap();

//     // let notification_box: gtk::Box = builder.object("notification_box").unwrap();

//     Ok(())
// }
