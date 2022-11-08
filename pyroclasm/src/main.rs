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

    Copyright (c) 2019-2022, The Eruption Development Team
*/

use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};
use lazy_static::lazy_static;
use parking_lot::{Mutex, RwLock};
use rust_embed::RustEmbed;
use std::cell::RefCell;

use std::env;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use util::RGBA;

mod app;
mod constants;
mod dbus_client;
mod device;
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
    saved_profile: Option<String>,
    current_brightness: Option<i64>,
}

impl State {
    fn new() -> Self {
        Self {
            active_slot: None,
            active_profile: None,
            saved_profile: None,
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
    pub static ref CONFIG: Arc<Mutex<Option<config::Config>>> = Arc::new(Mutex::new(None));
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

Copyright (c) 2019-2022, The Eruption Development Team
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
        // log::info!("Switching to slot: {}", index);
        util::switch_slot(index)?;

        STATE.write().active_slot = Some(index);
    }

    Ok(())
}

/// Switch to profile `file_name`
pub fn switch_to_profile<P: AsRef<Path>>(file_name: P) -> Result<()> {
    if !events::shall_ignore_pending_ui_event() {
        let file_name = file_name.as_ref();

        // log::info!(
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

        // log::info!(
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

/// Main program entrypoint
pub fn main() -> std::result::Result<(), eyre::Error> {
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

    // initialize logging
    /* if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG_OVERRIDE", "info");
        pretty_env_logger::init_custom_env("RUST_LOG_OVERRIDE");
    } else {
        pretty_env_logger::init();
    } */

    tracing_subscriber::fmt::init();

    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Pyroclasm UI",
        native_options,
        Box::new(|cc| Box::new(app::Pyroclasm::new(cc))),
    );

    Ok(())
}
