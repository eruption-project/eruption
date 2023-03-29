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

use crate::{
    dbus_client,
    profiles::Profile,
    scripting::manifest::Manifest,
    scripting::parameters::{self, ManifestValue, TypedValue},
    util,
};

use glib::clone;
use glib::IsA;
use gtk::glib;
use gtk::{
    prelude::*, Adjustment, Align, Box, Builder, Button, ButtonsType, CellRendererText,
    ColorButton, Entry, Expander, Frame, IconSize, Image, Justification, Label, MessageDialog,
    MessageType, Orientation, PositionType, Scale, ScrolledWindow, ShadowType, Stack,
    StackSwitcher, Switch, TextBuffer, TreeStore, TreeView, TreeViewColumn, TreeViewColumnSizing,
};
use paste::paste;

#[cfg(feature = "sourceview")]
use gtk::TextView;

#[cfg(feature = "sourceview")]
use sourceview4::prelude::*;
use sourceview4::Buffer;

#[cfg(not(feature = "sourceview"))]
use gtk::ApplicationWindow;
#[cfg(not(feature = "sourceview"))]
use gtk::{TextBuffer, TextView};

use std::path::{Path, PathBuf};
use std::{cell::RefCell, collections::HashMap, ffi::OsStr, rc::Rc};

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
pub enum ProfilesError {
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

declare_config_widget_numeric!(i64);
declare_config_widget_numeric!(f64);

declare_config_widget_input!(String);
declare_config_widget_color!(u32);
declare_config_widget_switch!(bool);

fn create_config_editor(
    profile: &Profile,
    script: &Manifest,
    manifest_parameter: &parameters::ManifestParameter,
    profile_parameter: &Option<&parameters::ProfileParameter>,
) -> Result<Frame> {
    fn parameter_changed<T>(profile: &Profile, script: &Manifest, name: &str, value: T)
    where
        T: std::fmt::Display,
    {
        tracing::debug!(
            "Setting parameter {}: {}: {} to '{}'",
            &profile.profile_file.display(),
            &script.script_file.display(),
            &name,
            &value
        );

        crate::dbus_client::set_parameter(
            &profile.profile_file.to_string_lossy(),
            &script.script_file.to_string_lossy(),
            name,
            &format!("{}", &value),
        )
        .unwrap();
    }

    let outer = Frame::builder()
        .border_width(16)
        // .label(&format!("{}", param.get_name()))
        // .label_xalign(0.0085)
        .build();

    let name = &manifest_parameter.name;
    let description = &manifest_parameter.description;
    let profile_value_or_default = match profile_parameter {
        Some(profile_parameter) => profile_parameter.value.to_owned(),
        None => manifest_parameter.get_default(),
    };

    let widget = match (profile_value_or_default, &manifest_parameter.manifest) {
        (TypedValue::Int(value), ManifestValue::Int { min, max, default }) => {
            build_config_widget_i64(
                name,
                description,
                *default,
                *min,
                *max,
                value,
                clone!(@strong profile, @strong script, @strong name => move |value| {
                    parameter_changed(&profile, &script, &name, value);
                }),
            )
        }
        (TypedValue::Float(value), ManifestValue::Float { min, max, default }) => {
            build_config_widget_f64(
                name,
                description,
                *default,
                *min,
                *max,
                value,
                clone!(@strong profile, @strong script, @strong name => move |value| {
                    parameter_changed(&profile, &script, &name, value);
                }),
            )
        }
        (TypedValue::Bool(value), ManifestValue::Bool { default }) => {
            build_config_widget_switch_bool(
                name,
                description,
                *default,
                value,
                clone!(@strong profile, @strong script, @strong name => move |value| {
                    parameter_changed(&profile, &script, &name, value);
                }),
            )
        }
        (TypedValue::String(value), ManifestValue::String { default }) => {
            build_config_widget_input_string(
                name,
                description,
                default.to_owned(),
                value,
                clone!(@strong profile, @strong script, @strong name => move |value| {
                    parameter_changed(&profile, &script, &name, value);
                }),
            )
        }
        (TypedValue::Color(value), ManifestValue::Color { min, max, default }) => {
            build_config_widget_color_u32(
                name,
                description,
                *default,
                *min,
                *max,
                value,
                clone!(@strong profile, @strong script, @strong name => move |value| {
                    parameter_changed(&profile, &script, &name, value);
                }),
            )
        }
        _ => return Err(ProfilesError::TypeMismatch {}.into()),
    };

    outer.add(&widget?);
    Ok(outer)
}

/// Populate the configuration tab with settings/GUI controls
fn populate_visual_config_editor<P: AsRef<Path>>(builder: &Builder, profile: P) -> Result<()> {
    let config_window: ScrolledWindow = builder.object("config_window").unwrap();

    // first, clear all child widgets
    config_window.foreach(|widget| {
        config_window.remove(widget);
    });

    // then add config items
    let container = Box::builder()
        .border_width(8)
        .orientation(Orientation::Vertical)
        .spacing(8)
        .homogeneous(false)
        .build();

    let profile = Profile::load_fully(profile.as_ref())?;

    let label = Label::builder()
        .label(&profile.name)
        .justify(Justification::Fill)
        .halign(Align::Start)
        .build();

    let context = label.style_context();
    context.add_class("heading");

    container.pack_start(&label, false, false, 8);

    for manifest in profile.manifests.values() {
        let expander = Expander::builder()
            .border_width(8)
            .label(&format!(
                "{} ({})",
                &manifest.name,
                &manifest.script_file.display()
            ))
            .build();

        let expander_frame = Frame::builder()
            .border_width(8)
            .shadow_type(ShadowType::None)
            .build();

        let expander_container = Box::builder()
            .orientation(Orientation::Vertical)
            .homogeneous(false)
            .build();

        expander_frame.add(&expander_container);
        expander.add(&expander_frame);

        container.pack_start(&expander, false, false, 8);

        let profile_script_parameters = profile.config.get_parameters(&manifest.name);
        for param in manifest.config.iter() {
            let value = profile_script_parameters.and_then(|p| p.get_parameter(&param.name));

            let child = create_config_editor(&profile, manifest, param, &value)?;
            expander_container.pack_start(&child, false, true, 0);
        }
    }

    config_window.add(&container);
    config_window.show_all();

    Ok(())
}

/// Remove unused elements from the profiles stack, except the "Configuration" page
fn remove_elements_from_stack_widget(builder: &Builder) {
    let stack_widget: Stack = builder.object("profile_stack").unwrap();

    stack_widget.foreach(|widget| {
        stack_widget.remove(widget);
    });

    TEXT_BUFFERS.with(|b| b.borrow_mut().clear());
}

cfg_if::cfg_if! {
    if #[cfg(feature = "sourceview")] {
        fn save_buffer_contents_to_file<P: AsRef<Path>>(
            path: &P,
            buffer: &sourceview4::Buffer,
            builder: &Builder,
        ) -> Result<()> {
            let main_window: gtk::ApplicationWindow = builder.object("main_window").unwrap();

            let buffer = buffer.dynamic_cast_ref::<TextBuffer>().unwrap();
            let (start, end) = buffer.bounds();
            let data = buffer.text(&start, &end, true).map(|v| v.to_string());

            match data {
                Some(data) => {
                    // tracing::debug!("{}", &data);

                    if let Err(e) = dbus_client::write_file(&path.as_ref(), &data) {
                        tracing::error!("{}", e);

                        let message = "Could not write file".to_string();
                        let secondary =
                            format!("Error writing to file {}: {}", &path.as_ref().display(), e);

                        let message_dialog = MessageDialog::builder()
                            .parent(&main_window)
                            .destroy_with_parent(true)
                            .decorated(true)
                            .message_type(MessageType::Error)
                            .text(&message)
                            .secondary_text(&secondary)
                            .title("Error")
                            .buttons(ButtonsType::Ok)
                            .build();

                        message_dialog.run();
                        message_dialog.hide();

                        Err(ProfilesError::MethodCallError {
                            description: "Could not write file".to_string(),
                        }
                        .into())
                    } else {
                        tracing::info!("Wrote file: {}", &path.as_ref().display());

                        Ok(())
                    }
                }

                _ => {
                    tracing::error!("Could not get buffer contents");

                    Err(ProfilesError::UnknownError {
                        description: "Could not get buffer contents".to_string(),
                    }
                    .into())
                }
            }
        }
    } else {
        fn save_buffer_contents_to_file<P: AsRef<Path>>(
            path: &P,
            buffer: &TextBuffer,
            builder: &Builder,
        ) -> Result<()> {
            let main_window: ApplicationWindow = builder.object("main_window").unwrap();
                // tracing::debug!("{}", &data);

            let (start, end) = buffer.bounds();
            let data = buffer.text(&start, &end, true).map(|v| v.to_string());

            match data {
                Some(data) => {
                    // tracing::debug!("{}", &data);

                    if let Err(e) = dbus_client::write_file(&path.as_ref(), &data) {
                        tracing::error!("{}", e);

                        let message = "Could not write file".to_string();
                        let secondary =
                            format!("Error writing to file {}: {}", &path.as_ref().display(), e);

                        let message_dialog = MessageDialog::builder()
                            .parent(&main_window)
                            .destroy_with_parent(true)
                            .decorated(true)
                            .message_type(MessageType::Error)
                            .text(&message)
                            .secondary_text(&secondary)
                            .title("Error")
                            .buttons(ButtonsType::Ok)
                            .build();

                        message_dialog.run();
                        message_dialog.hide();

                        Err(ProfilesError::MethodCallError {
                            description: "Could not write file".to_string(),
                        }
                        .into())
                    } else {
                        tracing::info!("Wrote file: {}", &path.as_ref().display());

                        Ok(())
                    }
                }

                _ => {
                    tracing::error!("Could not get buffer contents");

                    Err(ProfilesError::UnknownError {
                        description: "Could not get buffer contents".to_string(),
                    }
                    .into())
                }
            }
        }
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "sourceview")] {
        /// Instantiate one page per .profile or .lua file, each page holds a GtkSourceView widget
        /// showing the respective files contents
        fn populate_stack_widget<P: AsRef<Path>>(builder: &Builder, profile: P) -> Result<()> {
            let stack_widget: Stack = builder.object("profile_stack").unwrap();
            let stack_switcher: StackSwitcher = builder.object("profile_stack_switcher").unwrap();

            let context = stack_switcher.style_context();
            context.add_class("small-font");

            let language_manager = sourceview4::LanguageManager::default().unwrap();

            let toml = language_manager.language("toml").unwrap();
            let lua = language_manager.language("lua").unwrap();

            // load and show .profile file
            let source_code = std::fs::read_to_string(PathBuf::from(&profile.as_ref())).unwrap();

            let mut buffer_index = 0;
            let buffer = Buffer::builder()
                .language(&toml)
                .highlight_syntax(true)
                .text(&source_code)
                .build();

            // add buffer to global text buffers map for later reference
            TEXT_BUFFERS.with(|b| {
                let mut text_buffers = b.borrow_mut();
                text_buffers.insert(
                    PathBuf::from(&profile.as_ref()),
                    (buffer_index, buffer.clone()),
                );
            });

            buffer_index += 1;

            let sourceview = sourceview4::View::with_buffer(&buffer);
            sourceview.set_show_line_marks(true);
            sourceview.set_show_line_numbers(true);

            let sourceview = sourceview.dynamic_cast::<TextView>().unwrap();

            sourceview.set_editable(true);

            let filename = profile
                .as_ref()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();

            let scrolled_window = ScrolledWindow::builder()
                .shadow_type(ShadowType::None)
                .build();
            scrolled_window.add(&sourceview);

            scrolled_window.show_all();

            stack_widget.add_titled(
                &scrolled_window,
                &profile.as_ref().to_string_lossy(),
                &filename,
            );

            scrolled_window.show_all();

            // add associated .lua files

            for p in util::enumerate_profiles()? {
                if p.profile_file == profile.as_ref() {
                    for f in &p.active_scripts {
                        let abs_path = util::match_script_path(f)?;

                        let source_code = std::fs::read_to_string(&abs_path)?;

                        let buffer = Buffer::builder()
                            .language(&lua)
                            .highlight_syntax(true)
                            .text(&source_code)
                            .build();

                        // add buffer to global text buffers map for later reference
                        TEXT_BUFFERS.with(|b| {
                            let mut text_buffers = b.borrow_mut();
                            text_buffers.insert(abs_path.clone(), (buffer_index, buffer.clone()));
                        });

                        buffer_index += 1;

                        // script file editor
                        let sourceview = sourceview4::View::with_buffer(&buffer);
                        sourceview.set_show_line_marks(true);
                        sourceview.set_show_line_numbers(true);

                        let sourceview = sourceview.dynamic_cast::<TextView>().unwrap();

                        sourceview.set_editable(true);

                        let path = f.file_name().unwrap().to_string_lossy().to_string();

                        let scrolled_window = ScrolledWindow::builder().build();
                        scrolled_window.add(&sourceview);

                        stack_widget.add_titled(
                            &scrolled_window,
                            &path,
                            &f.file_name().unwrap().to_string_lossy(),
                        );

                        scrolled_window.show_all();

                        let manifest_file =
                            format!("{}.manifest", abs_path.into_os_string().into_string().unwrap());
                        let f = PathBuf::from(manifest_file);

                        let manifest_data = std::fs::read_to_string(&f)?;

                        let buffer = Buffer::builder()
                            .language(&toml)
                            .highlight_syntax(true)
                            .text(&manifest_data)
                            .build();

                        // add buffer to global text buffers map for later reference
                        TEXT_BUFFERS.with(|b| {
                            let mut text_buffers = b.borrow_mut();
                            text_buffers.insert(f.clone(), (buffer_index, buffer.clone()));
                        });

                        buffer_index += 1;

                        // manifest file editor
                        let sourceview = sourceview4::View::with_buffer(&buffer);
                        sourceview.set_show_line_marks(true);
                        sourceview.set_show_line_numbers(true);

                        let sourceview = sourceview.dynamic_cast::<TextView>().unwrap();

                        sourceview.set_editable(true);

                        let path = f.file_name().unwrap().to_string_lossy().to_string();

                        let scrolled_window = ScrolledWindow::builder().build();
                        scrolled_window.add(&sourceview);

                        stack_widget.add_titled(
                            &scrolled_window,
                            &path,
                            &f.file_name().unwrap().to_string_lossy(),
                        );

                        scrolled_window.show_all();
                    }

                    break;
                }
            }

            Ok(())
        }
    } else {
        /// Instantiate one page per .profile or .lua file, each page holds a GtkSourceView widget
        /// showing the respective files contents
        fn populate_stack_widget<P: AsRef<Path>>(builder: &Builder, profile: P) -> Result<()> {
            let stack_widget: Stack = builder.object("profile_stack").unwrap();
            let stack_switcher: StackSwitcher = builder.object("profile_stack_switcher").unwrap();

            let context = stack_switcher.style_context();
            context.add_class("small-font");

            // load and show .profile file
            let source_code = std::fs::read_to_string(&PathBuf::from(&profile.as_ref())).unwrap();

            let buffer = TextBuffer::builder().text(&source_code).build();

            let text_view = TextView::builder()
                .buffer(&buffer)
                .build();

            let mut buffer_index = 0;
            // add buffer to global text buffers map for later reference
            TEXT_BUFFERS.with(|b| {
                let mut text_buffers = b.borrow_mut();
                text_buffers.insert(
                    PathBuf::from(&profile.as_ref()),
                    (buffer_index, buffer.clone()),
                );
            });

            buffer_index += 1;

            text_view.set_editable(true);

            let filename = profile
                .as_ref()
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string();

            let scrolled_window = ScrolledWindow::builder()
                .shadow_type(ShadowType::None)
                .build();
            scrolled_window.add(&text_view);

            scrolled_window.show_all();

            stack_widget.add_titled(
                &scrolled_window,
                &profile.as_ref().to_string_lossy(),
                &filename,
            );

            scrolled_window.show_all();

            // add associated .lua files

            for p in util::enumerate_profiles()? {
                if p.profile_file == profile.as_ref() {
                    for f in p.active_scripts {
                        let abs_path = util::match_script_path(&f)?;

                        let source_code = std::fs::read_to_string(&abs_path)?;

                        let buffer = TextBuffer::builder()
                            .text(&source_code)
                            .build();

                        // add buffer to global text buffers map for later reference
                        TEXT_BUFFERS.with(|b| {
                            let mut text_buffers = b.borrow_mut();
                            text_buffers.insert(abs_path.clone(), (buffer_index, buffer.clone()));
                        });

                        buffer_index += 1;

                        // script file editor
                        let text_view = TextView::builder()
                            .buffer(&buffer)
                            .build();

                        text_view.set_editable(true);

                        let path = f.file_name().unwrap().to_string_lossy().to_string();

                        let scrolled_window = ScrolledWindow::builder().build();
                        scrolled_window.add(&text_view);

                        stack_widget.add_titled(
                            &scrolled_window,
                            &path,
                            &f.file_name().unwrap().to_string_lossy(),
                        );

                        scrolled_window.show_all();

                        let manifest_file =
                            format!("{}.manifest", abs_path.into_os_string().into_string().unwrap());
                        let f = PathBuf::from(manifest_file);

                        let manifest_data = std::fs::read_to_string(&f)?;

                        // add buffer to global text buffers map for later reference
                        TEXT_BUFFERS.with(|b| {
                            let mut text_buffers = b.borrow_mut();
                            text_buffers.insert(f.clone(), (buffer_index, buffer.clone()));
                        });

                        buffer_index += 1;

                        // manifest file editor
                        let buffer = TextBuffer::builder()
                            .text(&manifest_data)
                            .build();

                        let text_view = TextView::builder()
                            .buffer(&buffer)
                            .build();

                        text_view.set_editable(true);

                        let path = f.file_name().unwrap().to_string_lossy().to_string();

                        let scrolled_window = ScrolledWindow::builder().build();
                        scrolled_window.add(&text_view);

                        stack_widget.add_titled(
                            &scrolled_window,
                            &path,
                            &f.file_name().unwrap().to_string_lossy(),
                        );

                        scrolled_window.show_all();
                    }

                    break;
                }
            }

            Ok(())
        }
    }
}

/// Initialize page "Profiles"
pub fn initialize_profiles_page<A: IsA<gtk::Application>>(
    application: &A,
    builder: &Builder,
) -> Result<()> {
    let profiles_treeview: TreeView = builder.object("profiles_treeview").unwrap();
    // let sourceview: sourceview4::View = builder.object("source_view").unwrap();

    // profiles list
    let profiles_treestore = TreeStore::new(&[
        glib::Type::U64,
        String::static_type(),
        String::static_type(),
        String::static_type(),
    ]);

    for (index, profile) in util::enumerate_profiles()
        .unwrap_or_else(|_| vec![])
        .iter()
        .enumerate()
    {
        let name = &profile.name;
        let filename = profile
            .profile_file
            .file_name()
            .unwrap_or_else(|| OsStr::new("<error>"))
            .to_string_lossy()
            .to_owned()
            .to_string();

        let path = profile
            .profile_file
            // .file_name()
            // .unwrap_or_else(|| OsStr::new("<error>"))
            .to_string_lossy()
            .to_owned()
            .to_string();

        profiles_treestore.insert_with_values(
            None,
            None,
            &[(0, &(index as u64)), (1, &name), (2, &filename), (3, &path)],
        );
    }

    let id_column = TreeViewColumn::builder()
        .title("ID")
        .sizing(TreeViewColumnSizing::Autosize)
        .visible(false)
        .build();
    let name_column = TreeViewColumn::builder()
        .title("Name")
        .sizing(TreeViewColumnSizing::Autosize)
        .build();
    let filename_column = TreeViewColumn::builder()
        .title("Filename")
        .sizing(TreeViewColumnSizing::Autosize)
        .build();
    let path_column = TreeViewColumn::builder()
        .visible(false)
        .title("Path")
        .build();

    let cell_renderer_id = CellRendererText::new();
    let cell_renderer_name = CellRendererText::new();
    let cell_renderer_filename = CellRendererText::new();

    gtk::prelude::CellLayoutExt::pack_start(&id_column, &cell_renderer_id, false);
    gtk::prelude::CellLayoutExt::pack_start(&name_column, &cell_renderer_name, true);
    gtk::prelude::CellLayoutExt::pack_start(&filename_column, &cell_renderer_filename, true);

    profiles_treeview.insert_column(&id_column, 0);
    profiles_treeview.insert_column(&name_column, 1);
    profiles_treeview.insert_column(&filename_column, 2);
    profiles_treeview.insert_column(&path_column, 3);

    gtk::prelude::TreeViewColumnExt::add_attribute(&id_column, &cell_renderer_id, "text", 0);
    gtk::prelude::TreeViewColumnExt::add_attribute(&name_column, &cell_renderer_name, "text", 1);
    gtk::prelude::TreeViewColumnExt::add_attribute(
        &filename_column,
        &cell_renderer_filename,
        "text",
        2,
    );
    gtk::prelude::TreeViewColumnExt::add_attribute(
        &path_column,
        &cell_renderer_filename,
        "text",
        3,
    );

    profiles_treeview.set_model(Some(&profiles_treestore));

    profiles_treeview.connect_row_activated(clone!(@weak builder => move |tv, path, _column| {
        let profile = tv.model().unwrap().value(&tv.model().unwrap().iter(path).unwrap(), 3).get::<String>().unwrap();

        let _result = populate_visual_config_editor(&builder, &profile).map_err(|e| { tracing::error!("{}", e) });

        remove_elements_from_stack_widget(&builder);
        let _result = populate_stack_widget(&builder, &profile).map_err(|e| { tracing::error!("{}", e) });
    }));

    profiles_treeview.show_all();

    update_profile_state(builder)?;
    register_actions(application, builder)?;

    Ok(())
}

/// Register global actions and keyboard accelerators
fn register_actions<A: IsA<gtk::Application>>(application: &A, builder: &Builder) -> Result<()> {
    let application = application.as_ref();

    let stack_widget: Stack = builder.object("profile_stack").unwrap();
    // let stack_switcher: StackSwitcher = builder.object("profile_stack_switcher").unwrap();

    let save_current_buffer = gio::SimpleAction::new("save-current-buffer", None);
    save_current_buffer.connect_activate(clone!(@weak builder => move |_, _| {
        if let Some(view) = stack_widget.visible_child()
        // .map(|w| w.dynamic_cast::<sourceview4::View>().unwrap())
        {
            let index = stack_widget.child_position(&view) as usize;

            TEXT_BUFFERS.with(|b| {
                if let Some((path, buffer)) = b
                    .borrow()
                    .iter()
                    .find(|v| v.1 .0 == index)
                    .map(|v| (v.0, &v.1 .1))
                {
                    let _result = save_buffer_contents_to_file(&path, buffer, &builder);
                }
            });
        }
    }));

    application.add_action(&save_current_buffer);
    application.set_accels_for_action("app.save-current-buffer", &["<Primary>S"]);

    let save_all_buffers = gio::SimpleAction::new("save-all-buffers", None);
    save_all_buffers.connect_activate(clone!(@weak builder => move |_, _| {
        TEXT_BUFFERS.with(|b| {
            'SAVE_LOOP: for (k, (_, v)) in b.borrow().iter() {
                let result = save_buffer_contents_to_file(&k, v, &builder);

                // stop saving files if an error occurred, or auth has failed
                if result.is_err() {
                    break 'SAVE_LOOP;
                }
            }
        });
    }));

    application.add_action(&save_all_buffers);
    application.set_accels_for_action("app.save-all-buffers", &["<Primary><Shift>S"]);

    Ok(())
}

pub fn update_profile_state(builder: &Builder) -> Result<()> {
    let profiles_treeview: TreeView = builder.object("profiles_treeview").unwrap();

    let model = profiles_treeview.model().unwrap();

    let state = crate::STATE.read();
    let active_profile = state.active_profile.clone().unwrap_or_default();

    model.foreach(|model, path, iter| {
        let item = model.value(iter, 3).get::<String>().unwrap();
        if item == active_profile {
            // found a match
            profiles_treeview.selection().select_iter(iter);
            profiles_treeview.row_activated(path, &profiles_treeview.column(1).unwrap());

            true
        } else {
            false
        }
    });

    Ok(())
}
