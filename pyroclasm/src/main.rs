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

use clap::Parser;
use config::Config;
use eframe::{NativeOptions, Theme};
use egui::{Context, Vec2};
use flume::unbounded;
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};
use lazy_static::lazy_static;
use parking_lot::{Mutex, RwLock};
use rust_embed::RustEmbed;
use std::{
    cell::RefCell,
    process,
    sync::atomic::{AtomicBool, AtomicU8, Ordering},
};
use tracing::error;

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
mod highlighting;
mod profiles;
mod resources;
mod scripting;
mod subcommands;
mod threads;
mod translations;
mod ui;
mod util;

use translations::tr;

#[derive(RustEmbed)]
#[folder = "i18n"] // path to the compiled localization resources
struct Localizations;

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
    // egui Context
    egui_ctx: Option<Context>,

    _saved_profile: Option<String>,

    // Eruption daemon state
    slot_names: Option<Vec<String>>,
    active_slot: Option<usize>,
    active_profile: Option<String>,
    current_brightness: Option<i64>,
    sound_fx: Option<bool>,
}

impl State {
    fn new() -> Self {
        Self {
            egui_ctx: None,

            _saved_profile: None,

            active_slot: None,
            active_profile: None,
            current_brightness: None,
            slot_names: None,
            sound_fx: None,
        }
    }
}

lazy_static! {
    /// Global application state
    pub static ref STATE: Arc<RwLock<State>> = Arc::new(RwLock::new(State::new()));

    /// Current LED color map
    pub static ref COLOR_MAP: Arc<Mutex<Vec<RGBA>>> = Arc::new(Mutex::new(vec![RGBA { r: 0, g: 0, b: 0, a: 0 }; constants::CANVAS_SIZE]));

    /// Eruption managed devices
    pub static ref MANAGED_DEVICES: Arc<Mutex<(Vec<(u16, u16)>, Vec<(u16, u16)>, Vec<(u16, u16)>)>> = Arc::new(Mutex::new((Vec::new(), Vec::new(), Vec::new())));
}

lazy_static! {
    /// Global configuration
    pub static ref CONFIG: Arc<Mutex<Option<config::Config>>> = Arc::new(Mutex::new(None));

    /// Global verbosity amount
    pub static ref VERBOSE: AtomicU8 = AtomicU8::new(0);

    /// Global "quit" status flag
    pub static ref QUIT: AtomicBool = AtomicBool::new(false);
}

/// Supported command line arguments
#[derive(Debug, clap::Parser)]
#[clap(
    version = env!("CARGO_PKG_VERSION"),
    author = "X3n0m0rph59 <x3n0m0rph59@gmail.com>",
    about = tr!("about"),
)]
pub struct Options {
    /// Subcommand
    #[clap(subcommand)]
    command: Option<subcommands::Subcommands>,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(display_order = 0, help(tr!("verbose-about")), short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Sets the configuration file to use
    #[clap(display_order = 2, short = 'c', long)]
    config: Option<String>,
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
    pub(crate) fn _ignore_next_ui_events(count: usize) {
        IGNORE_NEXT_UI_EVENTS.fetch_add(count, Ordering::SeqCst);
    }

    /// test whether the current event shall be ignored
    pub(crate) fn shall_ignore_pending_ui_event() -> bool {
        IGNORE_NEXT_UI_EVENTS.load(Ordering::SeqCst) > 0
    }

    /// re-enable events
    pub(crate) fn _reenable_ui_events() {
        IGNORE_NEXT_UI_EVENTS.fetch_sub(1, Ordering::SeqCst);
    }

    /// ignore next n events (do not act on them)
    pub(crate) fn _ignore_next_dbus_events(count: usize) {
        IGNORE_NEXT_DBUS_EVENTS.fetch_add(count, Ordering::SeqCst);
    }

    /// test whether the current event shall be ignored
    pub(crate) fn _shall_ignore_pending_dbus_event() -> bool {
        IGNORE_NEXT_DBUS_EVENTS.load(Ordering::SeqCst) > 0
    }

    /// re-enable events
    pub(crate) fn _reenable_dbus_events() {
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

/// Switch to slot `index`
pub fn switch_to_slot(index: usize) -> Result<()> {
    if !events::shall_ignore_pending_ui_event() {
        // info!("Switching to slot: {}", index);
        util::switch_slot(index)?;

        STATE.write().active_slot = Some(index);
    }

    Ok(())
}

/// Switch to profile `file_name`
pub fn switch_to_profile<P: AsRef<Path>>(file_name: P) -> Result<()> {
    if !events::shall_ignore_pending_ui_event() {
        let file_name = file_name.as_ref();

        // info!(
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

        // info!(
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
    translations::load()?;

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async move { async_main().await })
}

pub async fn async_main() -> std::result::Result<(), eyre::Error> {
    let language_loader: FluentLanguageLoader = fluent_language_loader!();

    let requested_languages = DesktopLanguageRequester::requested_languages();
    i18n_embed::select(&language_loader, &Localizations, &requested_languages)?;

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
    if !env::args().any(|a| a.eq_ignore_ascii_case("completions")) && env::args().count() > 0 {
        print_header();
    }

    // initialize logging
    tracing_subscriber::fmt()
        .with_line_number(true)
        .with_thread_names(true)
        // .pretty()
        .init();

    // egui_logger::init().unwrap();

    // register_sigint_handler();

    let opts = Options::parse();
    apply_opts(&opts);
    if let Some(command) = opts.command {
        subcommands::handle_command(command).await?;
    }

    if !QUIT.load(Ordering::SeqCst) {
        // spawn our event loop
        let (events_tx, _events_rx) = unbounded();
        threads::spawn_events_thread(events_tx)?;

        // build and map main window
        let native_options = NativeOptions {
            default_theme: Theme::Dark,
            initial_window_size: Some(Vec2::new(1600.0_f32, 900.0_f32)),
            decorated: false,
            resizable: true,
            transparent: true,
            ..NativeOptions::default()
        };

        eframe::run_native(
            "pyroclasm",
            native_options,
            Box::new(|cc| {
                let mut global_state = STATE.write();
                global_state.egui_ctx = Some(cc.egui_ctx.clone());

                Box::new(app::Pyroclasm::new(cc))
            }),
        )
        .unwrap_or_else(|e| {
            tracing::error!("{}", e);
        });
    }

    Ok(())
}

// fn register_sigint_handler() {
//     // register ctrl-c handler
//     let (ctrl_c_tx, _ctrl_c_rx) = unbounded();
//     ctrlc::set_handler(move || {
//         QUIT.store(true, Ordering::SeqCst);

//         ctrl_c_tx.send(true).unwrap_or_else(|e| {
//             error!(
//                 "{}",
//                 tr!("could-not-send-on-channel", message = e.to_string())
//             );
//         });
//     })
//     .unwrap_or_else(|e| {
//         error!(
//             "{}",
//             tr!("could-not-set-ctrl-c-handler", message = e.to_string())
//         )
//     });
// }

fn apply_opts(opts: &Options) {
    VERBOSE.store(opts.verbose, Ordering::SeqCst);

    // process configuration file
    let config_file = opts
        .config
        .as_deref()
        .unwrap_or(constants::DEFAULT_CONFIG_FILE);

    let config = Config::builder()
        .add_source(config::File::new(config_file, config::FileFormat::Toml))
        .build()
        .unwrap_or_else(|e| {
            error!("{}", tr!("could-not-parse-config", message = e.to_string()));
            process::exit(4);
        });

    *CONFIG.lock() = Some(config);
}
