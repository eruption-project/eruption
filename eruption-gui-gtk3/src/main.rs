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

use config::Config;
use gio::{prelude::*, ApplicationFlags};
use glib::clone;
use glib::{OptionArg, OptionFlags};
// use glib::{OptionArg, OptionFlags};
use gtk::Application;
use gtk::{prelude::*, MessageDialog};
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};
use lazy_static::lazy_static;
use parking_lot::{Mutex, RwLock};
use rust_embed::RustEmbed;
use std::cell::RefCell;
use std::convert::TryFrom;
use std::env::args;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use std::{env, process};

use util::RGBA;

use crate::error_log::ErrorType;

mod constants;
mod dbus_client;
mod device;
mod error_log;
mod preferences;
mod profiles;
mod scripting;
mod ui;
mod util;

#[derive(RustEmbed)]
#[folder = "i18n"] // path to the compiled localization resources
struct Localizations;

lazy_static! {
    /// Global configuration
    pub static ref STATIC_LOADER: Arc<Mutex<Option<FluentLanguageLoader>>> = Arc::new(Mutex::new(None));
}

#[allow(unused)]
macro_rules! tr {
    ($message_id:literal) => {{
        let loader = $crate::STATIC_LOADER.lock();
        let loader = loader.as_ref().unwrap();

        i18n_embed_fl::fl!(loader, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        let loader = $crate::STATIC_LOADER.lock();
        let loader = loader.as_ref().unwrap();

        i18n_embed_fl::fl!(loader, $message_id, $($args), *)
    }};
}

type Result<T> = std::result::Result<T, eyre::Error>;

type Callback = dyn Fn() -> Result<()> + 'static;

thread_local! {
    /// Global timers (interval millis, last fired, callback Fn())
    pub static TIMERS: RefCell<Vec<(u64, Instant, Box<Callback>)>> = RefCell::new(Vec::new());
}

/// Register a timer callback
pub fn register_timer<T>(timeout: u64, callback: T) -> Result<()>
where
    T: Fn() -> Result<()> + 'static,
{
    TIMERS.with(|f| {
        let mut timers = f.borrow_mut();

        timers.push((timeout, Instant::now(), Box::new(callback)));
    });

    Ok(())
}

/// Handle timer callbacks
pub fn handle_timers() -> Result<()> {
    TIMERS.with(|f| -> Result<()> {
        let mut timers = f.borrow_mut();

        for (ref timeout_millis, ref mut last_fired, callback) in timers.iter_mut() {
            if Instant::now() - *last_fired > Duration::from_millis(*timeout_millis) {
                callback()?;

                *last_fired = Instant::now();
            }
        }

        Ok(())
    })?;

    Ok(())
}

#[derive(Debug, thiserror::Error)]
pub enum MainError {
    #[error("Unknown error: {description}")]
    UnknownError { description: String },
}

/// Global application state
#[derive(Default)]
pub struct State {
    active_slot: Option<usize>,
    active_profile: Option<String>,
    current_brightness: Option<i64>,
}

impl State {
    fn new() -> Self {
        Self {
            active_slot: None,
            active_profile: None,
            current_brightness: None,
        }
    }
}

lazy_static! {
    /// Global application state
    pub static ref STATE: Arc<RwLock<State>> = Arc::new(RwLock::new(State::new()));

    /// Current LED color map
    pub static ref COLOR_MAP: Arc<Mutex<Vec<RGBA>>> = Arc::new(Mutex::new(vec![RGBA { r: 0, g: 0, b: 0, a: 0 }; constants::CANVAS_SIZE]));

    /// Global configuration
    pub static ref CONFIG: Arc<RwLock<Option<config::Config>>> = Arc::new(RwLock::new(None));
}

/// Event handling utilities
pub mod events {
    use lazy_static::lazy_static;
    use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

    lazy_static! {
        /// stores how many consecutive events shall be ignored
        static ref IGNORE_NEXT_UI_EVENTS: AtomicUsize = AtomicUsize::new(0);

        /// stores how many consecutive events shall be ignored
        static ref IGNORE_NEXT_DBUS_EVENTS: AtomicUsize = AtomicUsize::new(0);

        /// signals whether we should re-initialize the GUI asap (e.g.: used when hot-plugging new devices)
        pub static ref UPDATE_MAIN_WINDOW: AtomicBool = AtomicBool::new(false);

        /// signals whether we have lost the connection to the Eruption daemon
        pub static ref LOST_CONNECTION: AtomicBool = AtomicBool::new(false);
    }

    /// ignore next n events (do not act on them)
    pub(crate) fn ignore_next_ui_events(count: usize) {
        IGNORE_NEXT_UI_EVENTS.fetch_add(count, Ordering::SeqCst);
    }

    /// test whether the current event shall be ignored
    pub(crate) fn shall_ignore_pending_ui_event() -> bool {
        IGNORE_NEXT_UI_EVENTS.load(Ordering::SeqCst) > 0
    }

    /// re-enable events
    pub(crate) fn reenable_ui_events() {
        IGNORE_NEXT_UI_EVENTS.fetch_sub(1, Ordering::SeqCst);
    }

    /// ignore next n events (do not act on them)
    pub(crate) fn ignore_next_dbus_events(count: usize) {
        IGNORE_NEXT_DBUS_EVENTS.fetch_add(count, Ordering::SeqCst);
    }

    /// test whether the current event shall be ignored
    pub(crate) fn shall_ignore_pending_dbus_event() -> bool {
        IGNORE_NEXT_DBUS_EVENTS.load(Ordering::SeqCst) > 0
    }

    /// re-enable events
    pub(crate) fn reenable_dbus_events() {
        IGNORE_NEXT_DBUS_EVENTS.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Print license information
#[allow(dead_code)]
fn print_header() {
    println!(
        r#"Eruption is free software: you can redistribute it and/or modify
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
"#
    );
}

/// Update the global color map vector
pub fn update_color_map() -> Result<()> {
    let mut led_colors = dbus_client::get_led_colors()?;

    let mut color_map = crate::COLOR_MAP.lock();

    color_map.clear();
    color_map.append(&mut led_colors);

    Ok(())
}

/// Switch to slot `index`
pub fn switch_to_slot(index: usize) -> Result<()> {
    if !events::shall_ignore_pending_ui_event() {
        // tracing::info!("Switching to slot: {}", index);
        util::switch_slot(index)?;

        STATE.write().active_slot = Some(index);
    }

    Ok(())
}

/// Switch to profile `file_name`
pub fn switch_to_profile<P: AsRef<Path>>(file_name: P) -> Result<()> {
    if !events::shall_ignore_pending_ui_event() {
        let file_name = file_name.as_ref();

        // tracing::info!(
        //     "Switching to profile: {}",
        //     file_name.to_string_lossy()
        // );

        util::switch_profile(&file_name.to_string_lossy())?;
    }

    Ok(())
}

/// Switch to slot `slot_index` and then change the current profile to `file_name`
pub fn switch_to_slot_and_profile<P: AsRef<Path>>(slot_index: usize, file_name: P) -> Result<()> {
    if !events::shall_ignore_pending_ui_event() {
        let file_name = file_name.as_ref();

        // tracing::info!(
        //     "Switching to slot: {}, using profile: {}",
        //     slot_index,
        //     file_name.to_string_lossy()
        // );

        util::switch_slot(slot_index)?;
        STATE.write().active_slot = Some(slot_index);

        util::switch_profile(&file_name.to_string_lossy())?;
    }

    Ok(())
}

/// Update the state of the GUI to reflect the current system state
/// This function is called from the D-Bus event loop
pub fn update_ui_state(builder: &gtk::Builder, event: &dbus_client::Message) -> Result<()> {
    if !events::shall_ignore_pending_dbus_event() {
        match *event {
            dbus_client::Message::SlotChanged(slot_index) => {
                STATE.write().active_slot = Some(slot_index);

                let slot1_radio_button: gtk::RadioButton =
                    builder.object("slot1_radio_button").unwrap();
                let slot2_radio_button: gtk::RadioButton =
                    builder.object("slot2_radio_button").unwrap();
                let slot3_radio_button: gtk::RadioButton =
                    builder.object("slot3_radio_button").unwrap();
                let slot4_radio_button: gtk::RadioButton =
                    builder.object("slot4_radio_button").unwrap();

                let slot1_frame: gtk::Frame = builder.object("slot1_frame").unwrap();
                let slot2_frame: gtk::Frame = builder.object("slot2_frame").unwrap();
                let slot3_frame: gtk::Frame = builder.object("slot3_frame").unwrap();
                let slot4_frame: gtk::Frame = builder.object("slot4_frame").unwrap();

                events::ignore_next_ui_events(1);
                events::ignore_next_dbus_events(1);

                match slot_index {
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
                events::reenable_dbus_events();

                ui::profiles::update_profile_state(builder)?;
            }

            dbus_client::Message::SlotNamesChanged(ref names) => {
                let slot1_entry: gtk::Entry = builder.object("slot1_entry").unwrap();
                let slot2_entry: gtk::Entry = builder.object("slot2_entry").unwrap();
                let slot3_entry: gtk::Entry = builder.object("slot3_entry").unwrap();
                let slot4_entry: gtk::Entry = builder.object("slot4_entry").unwrap();

                slot1_entry.set_text(names.get(0).unwrap_or(&"Profile Slot 1".to_string()));
                slot2_entry.set_text(names.get(1).unwrap_or(&"Profile Slot 2".to_string()));
                slot3_entry.set_text(names.get(2).unwrap_or(&"Profile Slot 3".to_string()));
                slot4_entry.set_text(names.get(3).unwrap_or(&"Profile Slot 4".to_string()));
            }

            dbus_client::Message::ProfileChanged(ref profile) => {
                events::ignore_next_ui_events(1);

                match STATE.read().active_slot.unwrap() {
                    0 => {
                        // slot 1
                        let combo_box: gtk::ComboBox = builder.object("slot1_combo").unwrap();

                        combo_box.model().unwrap().foreach(|model, _path, iter| {
                            let file = model.value(iter, 2).get::<String>().unwrap();
                            let file = PathBuf::from(file).to_string_lossy().to_string();

                            if *profile == file {
                                // found a match
                                combo_box.set_active_iter(Some(iter));

                                true
                            } else {
                                false
                            }
                        });
                    }

                    1 => {
                        // slot 2
                        let combo_box: gtk::ComboBox = builder.object("slot2_combo").unwrap();

                        combo_box.model().unwrap().foreach(|model, _path, iter| {
                            let file = model.value(iter, 2).get::<String>().unwrap();
                            let file = PathBuf::from(file).to_string_lossy().to_string();

                            if *profile == file {
                                // found a match
                                combo_box.set_active_iter(Some(iter));

                                true
                            } else {
                                false
                            }
                        });
                    }

                    2 => {
                        // slot 3
                        let combo_box: gtk::ComboBox = builder.object("slot3_combo").unwrap();

                        combo_box.model().unwrap().foreach(|model, _path, iter| {
                            let file = model.value(iter, 2).get::<String>().unwrap();
                            let file = PathBuf::from(file).to_string_lossy().to_string();

                            if *profile == file {
                                // found a match
                                combo_box.set_active_iter(Some(iter));

                                true
                            } else {
                                false
                            }
                        });
                    }

                    3 => {
                        // slot 4
                        let combo_box: gtk::ComboBox = builder.object("slot4_combo").unwrap();

                        combo_box.model().unwrap().foreach(|model, _path, iter| {
                            let file = model.value(iter, 2).get::<String>().unwrap();
                            let file = PathBuf::from(file).to_string_lossy().to_string();

                            if *profile == file {
                                // found a match
                                combo_box.set_active_iter(Some(iter));

                                true
                            } else {
                                false
                            }
                        });
                    }

                    _ => tracing::error!("Internal error detected"),
                }

                events::reenable_ui_events();

                STATE.write().active_profile = Some(profile.clone());
                ui::profiles::update_profile_state(builder)?;
            }

            dbus_client::Message::BrightnessChanged(brightness) => {
                STATE.write().current_brightness = Some(brightness);

                let brightness_scale: gtk::Scale = builder.object("brightness_scale").unwrap();

                events::ignore_next_dbus_events(1);
                events::ignore_next_ui_events(1);

                brightness_scale.set_value(brightness as f64);

                events::reenable_ui_events();
                events::reenable_dbus_events();
            }

            dbus_client::Message::SoundFxChanged(enabled) => {
                let switch_button: gtk::Switch = builder.object("soundfx_switch").unwrap();

                events::ignore_next_dbus_events(1);
                events::ignore_next_ui_events(1);

                switch_button.set_state(enabled);

                events::reenable_ui_events();
                events::reenable_dbus_events();
            }

            dbus_client::Message::RulesChanged => {
                tracing::info!("Process monitor ruleset has changed");
                ui::process_monitor::update_rules_view(builder)?;
            }

            dbus_client::Message::DeviceHotplug(_device_info) => {
                tracing::info!("A device has been hotplugged/removed");
                ui::main::update_main_window(builder).unwrap();
            }
        }
    }

    Ok(())
}

/// Main program entrypoint
pub fn main() -> std::result::Result<(), eyre::Error> {
    // initialize logging
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::util::SubscriberInitExt;

    // let filter = tracing_subscriber::EnvFilter::from_default_env();
    // let journald_layer = tracing_journald::layer()?.with_filter(filter);

    let filter = tracing_subscriber::EnvFilter::from_default_env();
    let format_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_filter(filter);

    cfg_if::cfg_if! {
        if #[cfg(feature = "debug-async")] {
            let console_layer = console_subscriber::ConsoleLayer::builder()
                .with_default_env()
                .spawn();

            tracing_subscriber::registry()
                .with(console_layer)
                // .with(journald_layer)
                .with(format_layer)
                .init();
        } else {
            tracing_subscriber::registry()
                // .with(journald_layer)
                .with(format_layer)
                .init();
        }
    };

    // i18n/l10n support
    let language_loader: FluentLanguageLoader = fluent_language_loader!();

    let requested_languages = DesktopLanguageRequester::requested_languages();
    i18n_embed::select(&language_loader, &Localizations, &requested_languages)?;

    STATIC_LOADER.lock().replace(language_loader);

    cfg_if::cfg_if! {
        if #[cfg(debug_assertions)] {
            color_eyre::config::HookBuilder::default()
            .panic_section("Please consider reporting a bug at https://github.com/X3n0m0rph59/eruption")
            .install()?;
        } else {
            color_eyre::config::HookBuilder::default()
            .panic_section("Please consider reporting a bug at https://github.com/X3n0m0rph59/eruption")
            .display_env_section(false)
            .install()?;
        }
    }

    // print a license header, except if we are generating shell completions
    if !env::args().any(|a| a.eq_ignore_ascii_case("completions")) && env::args().count() < 2 {
        print_header();
    }

    let application = Application::new(
        Some("org.eruption.eruption-gui-gtk3"),
        ApplicationFlags::FLAGS_NONE,
    );

    application.add_main_option(
        "configuration",
        glib::Char::try_from('c').unwrap(),
        OptionFlags::NONE,
        OptionArg::String,
        "The configuration file to use",
        Some(constants::DEFAULT_CONFIG_FILE),
    );

    application.connect_handle_local_options(|_app, opts| {
        // process configuration file
        let config_file = opts
            .lookup_value("configuration", None)
            .map(|v| v.str().unwrap().to_owned())
            .unwrap_or_else(|| constants::DEFAULT_CONFIG_FILE.to_string());

        let config_file = if config_file.trim().is_empty() {
            constants::DEFAULT_CONFIG_FILE.to_string()
        } else {
            config_file
        };

        let config = Config::builder()
            .add_source(config::File::new(&config_file, config::FileFormat::Toml))
            .build()
            .map_err(|e| {
                tracing::error!("Could not parse configuration file: {}", e);
                error_log::fatal_error(&format!("Could not parse configuration file: {e}"), 4);
            });

        *CONFIG.write() = config.ok();

        // request default processing of command line arguments
        -1
    });

    application.connect_activate(move |app| {
        // load the compiled resource bundle
        let resources_bytes = include_bytes!("../resources/resources.gresource");
        let resource_data = glib::Bytes::from(&resources_bytes[..]);
        let res = gio::Resource::from_data(&resource_data).unwrap();

        gio::resources_register(&res);

        // process all errors that occurred in the meantime
        for error in error_log::ERRORS.read().iter() {
            let message = match error.error_type {
                ErrorType::Fatal => "A fatal error occurred",
            };

            let secondary = &error.message;

            let message_dialog = MessageDialog::builder()
                .destroy_with_parent(true)
                .message_type(gtk::MessageType::Error)
                .text(message)
                .secondary_text(secondary)
                .title("Error")
                .buttons(gtk::ButtonsType::Ok)
                .build();

            message_dialog.run();
            message_dialog.hide();

            if error.error_type == ErrorType::Fatal {
                process::exit(error.code);
            }
        }

        {
            // initialize global state
            let mut state = STATE.write();

            state.active_slot = util::get_active_slot().ok();
            state.active_profile = util::get_active_profile().ok();
        }

        if let Err(e) = ui::main::initialize_main_window(app) {
            tracing::error!("Could not start the Eruption GUI: {}", e);

            let message =
                "Could not start the Eruption GUI, is the Eruption daemon running?".to_string();
            let secondary = format!("Reason:\n{e}");

            let message_dialog = MessageDialog::builder()
                .destroy_with_parent(true)
                .message_type(gtk::MessageType::Error)
                .text(message)
                .secondary_text(secondary)
                .title("Error")
                .buttons(gtk::ButtonsType::Ok)
                .build();

            message_dialog.run();
            message_dialog.hide();

            app.quit();
        }
    });

    // global timer support
    glib::idle_add_local(
        clone!(@weak application => @default-return Continue(true), move || {
            if let Err(e) = handle_timers() {
                tracing::error!("An error occurred in a timer callback: {}", e);
            }

            Continue(true)
        }),
    );

    application.run_with_args(&args().collect::<Vec<_>>());

    Ok(())
}
