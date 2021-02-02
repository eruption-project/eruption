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

use crate::{constants, manifest, profiles::Profile};
use crate::{manifest::Manifest, util};
use gdk::RGBA;
use glib::clone;
use glib::prelude::*;
use gtk::{prelude::*, Orientation};
use gtk::{ShadowType, StackExt};
use sourceview::prelude::*;
use sourceview::BufferBuilder;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, eyre::Error>;

fn build_int_config(
    name: &str,
    description: &str,
    _default: i64,
    min: i64,
    max: i64,
    value: i64,
) -> Result<gtk::Grid> {
    let container = gtk::GridBuilder::new()
        .border_width(10)
        .column_spacing(10)
        .row_spacing(10)
        .build();

    // let child = gtk::LabelBuilder::new().label("integer").build();
    // container.pack_start(&child, true, true, 8);

    let child = gtk::LabelBuilder::new()
        .use_markup(true)
        .label(&format!("<b>{}</b>", name))
        .build();
    container.attach(&child, 0, 0, 1, 1);

    let child = gtk::LabelBuilder::new().label(&description).build();
    container.attach(&child, 1, 0, 1, 1);

    let adjustment = gtk::AdjustmentBuilder::new()
        .lower(min as f64)
        .upper(max as f64)
        .build();

    let child = gtk::ScaleBuilder::new()
        .adjustment(&adjustment)
        .fill_level(value as f64)
        .build();

    container.attach(&child, 0, 1, 2, 1);

    Ok(container)
}

fn build_float_config(
    name: &str,
    description: &str,
    _default: f64,
    min: f64,
    max: f64,
    value: f64,
) -> Result<gtk::Grid> {
    let container = gtk::GridBuilder::new()
        .border_width(10)
        .column_spacing(10)
        .row_spacing(10)
        .build();

    // let child = gtk::LabelBuilder::new().label("float").build();
    // container.pack_start(&child, true, true, 8);

    let child = gtk::LabelBuilder::new()
        .use_markup(true)
        .label(&format!("<b>{}</b>", name))
        .build();
    container.attach(&child, 0, 0, 1, 1);

    let child = gtk::LabelBuilder::new().label(&description).build();
    container.attach(&child, 1, 0, 1, 1);

    let adjustment = gtk::AdjustmentBuilder::new()
        .lower(min as f64)
        .upper(max as f64)
        .build();

    let child = gtk::ScaleBuilder::new()
        .adjustment(&adjustment)
        .fill_level(value as f64)
        .build();

    container.attach(&child, 0, 1, 2, 1);

    Ok(container)
}

fn build_bool_config(
    name: &str,
    description: &str,
    _default: bool,
    value: bool,
) -> Result<gtk::Grid> {
    let container = gtk::GridBuilder::new()
        .border_width(10)
        .column_spacing(10)
        .row_spacing(10)
        .build();

    // let child = gtk::LabelBuilder::new().label("boolean").build();
    // container.pack_start(&child, true, true, 8);

    let child = gtk::LabelBuilder::new()
        .use_markup(true)
        .label(&format!("<b>{}</b>", name))
        .build();
    container.attach(&child, 0, 0, 1, 1);

    let child = gtk::LabelBuilder::new().label(&description).build();
    container.attach(&child, 1, 0, 1, 1);

    let child = gtk::SwitchBuilder::new().expand(false).state(value).build();
    container.attach(&child, 0, 1, 1, 1);

    Ok(container)
}

fn build_string_config(
    name: &str,
    description: &str,
    _default: &str,
    value: &str,
) -> Result<gtk::Grid> {
    let container = gtk::GridBuilder::new()
        .border_width(10)
        .column_spacing(10)
        .row_spacing(10)
        .build();

    // let child = gtk::LabelBuilder::new().label("string").build();
    // container.pack_start(&child, true, true, 8);

    let child = gtk::LabelBuilder::new()
        .use_markup(true)
        .label(&format!("<b>{}</b>", name))
        .build();
    container.attach(&child, 0, 0, 1, 1);

    let child = gtk::LabelBuilder::new().label(&description).build();
    container.attach(&child, 1, 0, 1, 1);

    let child = gtk::EntryBuilder::new().text(&value).build();
    container.attach(&child, 0, 1, 2, 1);

    Ok(container)
}

fn build_color_config(
    name: &str,
    description: &str,
    _default: u32,
    value: u32,
) -> Result<gtk::Grid> {
    let container = gtk::GridBuilder::new()
        .border_width(10)
        .column_spacing(10)
        .row_spacing(10)
        .build();

    // let child = gtk::LabelBuilder::new().label("color").build();
    // container.pack_start(&child, true, true, 8);

    let child = gtk::LabelBuilder::new()
        .use_markup(true)
        .label(&format!("<b>{}</b>", name))
        .build();
    container.attach(&child, 0, 0, 1, 1);

    let child = gtk::LabelBuilder::new().label(&description).build();
    container.attach(&child, 1, 0, 1, 1);

    let colors = util::color_to_rgba(value);
    let rgba = RGBA {
        red: colors.0 as f64 / 255.0,
        green: colors.1 as f64 / 255.0,
        blue: colors.2 as f64 / 255.0,
        alpha: colors.3 as f64 / 255.0,
    };

    let child = gtk::ColorChooserWidgetBuilder::new().rgba(&rgba).build();
    container.attach(&child, 0, 1, 2, 1);

    Ok(container)
}

fn create_config_editor(param: &manifest::ConfigParam) -> Result<gtk::Frame> {
    let outer = gtk::FrameBuilder::new().border_width(10).build();

    match param {
        manifest::ConfigParam::Int {
            name,
            description,
            default,
        } => {
            let widget = build_int_config(&name, description, *default, 0, 200, 123)?;
            outer.add(&widget);
        }

        manifest::ConfigParam::Float {
            name,
            description,
            default,
        } => {
            let widget = build_float_config(&name, description, *default, 0.0, 200.0, 123.59)?;
            outer.add(&widget);
        }

        manifest::ConfigParam::Bool {
            name,
            description,
            default,
        } => {
            let widget = build_bool_config(&name, description, *default, true)?;
            outer.add(&widget);
        }

        manifest::ConfigParam::String {
            name,
            description,
            default,
        } => {
            let widget = build_string_config(&name, description, default, "123")?;
            outer.add(&widget);
        }

        manifest::ConfigParam::Color {
            name,
            description,
            default,
        } => {
            let widget = build_color_config(&name, description, *default, 123)?;
            outer.add(&widget);
        }
    }

    Ok(outer)
}

/// Populate the configuration tab with settings/GUI controls
fn populate_visual_config_editor<P: AsRef<Path>>(builder: &gtk::Builder, profile: P) -> Result<()> {
    let config_window: gtk::ScrolledWindow = builder.get_object("config_window").unwrap();

    // first, clear all child widgets
    config_window.foreach(|widget| {
        config_window.remove(widget);
    });

    // then add config items
    let container = gtk::BoxBuilder::new()
        .orientation(Orientation::Vertical)
        .build();

    let profile = Profile::new(profile.as_ref())?;

    let script_path = PathBuf::from(constants::DEFAULT_SCRIPT_DIR);

    for f in profile.active_scripts.iter() {
        let manifest = Manifest::from(&script_path.join(&f))?;

        if let Some(params) = manifest.config {
            for param in params {
                let child = create_config_editor(&param)?;
                container.pack_start(&child, false, true, 10);
            }
        }
    }

    // TODO: Add support for this
    // config_window.add(&container);
    // config_window.show_all();

    config_window.hide();

    Ok(())
}

/// Remove unused elements from the profiles stack, except the "Configuration" page
fn remove_elements_from_stack_widget(builder: &gtk::Builder) {
    let stack_widget: gtk::Stack = builder.get_object("profile_stack").unwrap();

    stack_widget.foreach(|widget| {
        stack_widget.remove(widget);
    });
}

/// Instantiate one page per .profile or .lua file, each page holds a GtkSourceView widget
/// showing the respective files contents
fn populate_stack_widget<P: AsRef<Path>>(builder: &gtk::Builder, profile: P) -> Result<()> {
    let stack_widget: gtk::Stack = builder.get_object("profile_stack").unwrap();
    let stack_switcher: gtk::StackSwitcher = builder.get_object("profile_stack_switcher").unwrap();

    let context = stack_switcher.get_style_context();
    context.add_class("small-font");

    let language_manager = sourceview::LanguageManager::get_default().unwrap();

    let toml = language_manager.get_language("toml").unwrap();
    let lua = language_manager.get_language("lua").unwrap();

    // load and show .profile file
    let source_code = std::fs::read_to_string(&PathBuf::from(&profile.as_ref())).unwrap();

    let buffer = BufferBuilder::new()
        .language(&toml)
        .highlight_syntax(true)
        .text(&source_code)
        .build();

    let sourceview = sourceview::View::new_with_buffer(&buffer);
    sourceview.set_show_line_marks(true);
    sourceview.set_show_line_numbers(true);

    // TODO: Allow modification
    sourceview.set_editable(false);

    let filename = profile
        .as_ref()
        .file_name()
        .unwrap()
        .to_string_lossy()
        .to_string();

    let scrolled_window = gtk::ScrolledWindowBuilder::new()
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

    // TODO: use configuration values from eruption.conf
    let path = PathBuf::from(constants::DEFAULT_PROFILE_DIR);
    for p in util::enumerate_profiles(&path)? {
        if p.profile_file == profile.as_ref() {
            for f in p.active_scripts {
                // TODO: use configuration values from eruption.conf
                let script_path = PathBuf::from(constants::DEFAULT_SCRIPT_DIR);

                let source_code = std::fs::read_to_string(&script_path.join(&f))?;

                let buffer = BufferBuilder::new()
                    .language(&lua)
                    .highlight_syntax(true)
                    .text(&source_code)
                    .build();

                // script file editor
                let sourceview = sourceview::View::new_with_buffer(&buffer);
                sourceview.set_show_line_marks(true);
                sourceview.set_show_line_numbers(true);

                // TODO: Allow modification
                sourceview.set_editable(false);

                let path = f.file_name().unwrap().to_string_lossy().to_string();

                let scrolled_window = gtk::ScrolledWindowBuilder::new().build();
                scrolled_window.add(&sourceview);

                stack_widget.add_titled(
                    &scrolled_window,
                    &path,
                    &f.file_name().unwrap().to_string_lossy(),
                );

                scrolled_window.show_all();

                let manifest_file =
                    format!("{}.manifest", f.into_os_string().into_string().unwrap());
                let f = PathBuf::from(manifest_file);

                let script_path = PathBuf::from(constants::DEFAULT_SCRIPT_DIR);

                let manifest_data = std::fs::read_to_string(&script_path.join(&f))?;

                let buffer = BufferBuilder::new()
                    .language(&toml)
                    .highlight_syntax(true)
                    .text(&manifest_data)
                    .build();

                // manifest file editor
                let sourceview = sourceview::View::new_with_buffer(&buffer);
                sourceview.set_show_line_marks(true);
                sourceview.set_show_line_numbers(true);

                // TODO: Allow modification
                sourceview.set_editable(false);

                let path = f.file_name().unwrap().to_string_lossy().to_string();

                let scrolled_window = gtk::ScrolledWindowBuilder::new().build();
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

/// Initialize page "Profiles"
pub fn initialize_profiles_page(builder: &gtk::Builder) -> Result<()> {
    let profiles_treeview: gtk::TreeView = builder.get_object("profiles_treeview").unwrap();
    // let sourceview: sourceview::View = builder.get_object("source_view").unwrap();

    // profiles list
    let profiles_treestore = gtk::TreeStore::new(&[
        glib::Type::U64,
        String::static_type(),
        String::static_type(),
        String::static_type(),
    ]);

    // TODO: use configuration values from eruption.conf
    let path = PathBuf::from(constants::DEFAULT_PROFILE_DIR);
    for (index, ref profile) in util::enumerate_profiles(&path)?.iter().enumerate() {
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
            &[0, 1, 2, 3],
            &[&(index as u64), &name, &filename, &path],
        );
    }

    let id_column = gtk::TreeViewColumnBuilder::new()
        .title(&"ID")
        .sizing(gtk::TreeViewColumnSizing::Autosize)
        .visible(false)
        .build();
    let name_column = gtk::TreeViewColumnBuilder::new()
        .title(&"Name")
        .sizing(gtk::TreeViewColumnSizing::Autosize)
        .build();
    let filename_column = gtk::TreeViewColumnBuilder::new()
        .title(&"Filename")
        .sizing(gtk::TreeViewColumnSizing::Autosize)
        .build();
    let path_column = gtk::TreeViewColumnBuilder::new()
        .visible(false)
        .title(&"Path")
        .build();

    let cell_renderer_id = gtk::CellRendererText::new();
    let cell_renderer_name = gtk::CellRendererText::new();
    let cell_renderer_filename = gtk::CellRendererText::new();

    id_column.pack_start(&cell_renderer_id, false);
    name_column.pack_start(&cell_renderer_name, true);
    filename_column.pack_start(&cell_renderer_filename, true);

    profiles_treeview.insert_column(&id_column, 0);
    profiles_treeview.insert_column(&name_column, 1);
    profiles_treeview.insert_column(&filename_column, 2);
    profiles_treeview.insert_column(&path_column, 3);

    id_column.add_attribute(&cell_renderer_id, &"text", 0);
    name_column.add_attribute(&cell_renderer_name, &"text", 1);
    filename_column.add_attribute(&cell_renderer_filename, &"text", 2);
    path_column.add_attribute(&cell_renderer_filename, &"text", 3);

    profiles_treeview.set_model(Some(&profiles_treestore));

    profiles_treeview.connect_row_activated(clone!(@strong builder => move |tv, path, _column| {
        let profile = tv.get_model().unwrap().get_value(&tv.get_model().unwrap().get_iter(&path).unwrap(), 3).get::<String>().unwrap().unwrap();

        populate_visual_config_editor(&builder, &profile).unwrap();

        remove_elements_from_stack_widget(&builder);
        populate_stack_widget(&builder, &profile).unwrap();
    }));

    profiles_treeview.show_all();

    update_profile_state(&builder)?;

    Ok(())
}

pub fn update_profile_state(builder: &gtk::Builder) -> Result<()> {
    let profiles_treeview: gtk::TreeView = builder.get_object("profiles_treeview").unwrap();

    let model = profiles_treeview.get_model().unwrap();

    let state = crate::STATE.read();
    let active_profile = state
        .active_profile
        .clone()
        .unwrap_or_else(|| "".to_string());

    model.foreach(|model, path, iter| {
        let item = model.get_value(iter, 2).get::<String>().unwrap().unwrap();
        if item == active_profile {
            // found a match
            profiles_treeview.get_selection().select_iter(&iter);
            profiles_treeview.row_activated(&path, &profiles_treeview.get_column(1).unwrap());

            true
        } else {
            false
        }
    });

    Ok(())
}
