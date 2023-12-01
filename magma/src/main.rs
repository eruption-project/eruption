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

use clap::Parser;
use config::Config;
use constants::CANVAS_SIZE;
use crossterm::event::{
    KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags,
};
use crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use eruption_sdk::connection::{Connection, ConnectionType};
use events::LOST_CONNECTION;
// use clap::Parser;
// use config::Config;

use flume::bounded;
use lazy_static::lazy_static;
use ratatui::backend::CrosstermBackend;
use ratatui::Terminal;
use std::collections::HashMap;
use std::io::stdout;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;
use std::{env, process};
use tracing::instrument;
use tracing_mutex::stdsync::{Mutex, RwLock};

use util::ratelimited;

mod color_scheme;
mod constants;
mod custom_widgets;
mod dbus_client;
mod device;
mod profiles;
mod scripting;
mod subcommands;
mod themes;
mod threads;
mod timers;
mod translations;
mod tui;
mod util;
mod zone;

use translations::tr;

pub use crate::tui::hwdevices::{self, DeviceStatus};
use crate::tui::pages;
pub use crate::util::RGBA;

pub type Result<T> = std::result::Result<T, eyre::Error>;

lazy_static! {
    /// Global configuration
    pub static ref CONFIG: Arc<RwLock<Option<config::Config>>> = Arc::new(RwLock::new(None));

    /// Global verbosity amount
    pub static ref VERBOSE: AtomicU8 = AtomicU8::new(0);

    /// Global repeat flag
    pub static ref REPEAT: AtomicBool = AtomicBool::new(false);

    /// Global "quit" status flag
    pub static ref QUIT: AtomicBool = AtomicBool::new(false);
}

lazy_static! {
    /// Global application state data
    pub static ref STATE: Arc<RwLock<State>> = Arc::new(RwLock::new(State::new()));

    /// Are we connected to the Eruption daemon?
    pub static ref CONNECTION_STATE: Arc<RwLock<ConnectionState>> = Arc::new(RwLock::new(ConnectionState::Initializing));

    /// Current connection to the Eruption daemon (Eruption SDK)
    pub static ref CONNECTION: Arc<Mutex<Option<Connection>>> = Arc::new(Mutex::new(None));

    /// Current LED color map
    pub static ref CANVAS: Arc<RwLock<Vec<RGBA>>> = Arc::new(RwLock::new(vec![RGBA { r: 0, g: 0, b: 0, a: 0 }; constants::CANVAS_SIZE]));

    /// Device status
    pub static ref DEVICE_STATUS: Arc<RwLock<HashMap<u64, DeviceStatus>>> = Arc::new(RwLock::new(HashMap::new()));

    /// The index of the currently active page in the main navigation stack
    pub static ref ACTIVE_PAGE: AtomicUsize = AtomicUsize::new(0);

    /// Tab pages
    pub static ref PAGES: Arc<RwLock<Vec<Box<dyn pages::Page + Sync + Send>>>> = Arc::new(RwLock::new(Vec::new()));
}

#[derive(Debug, thiserror::Error)]
pub enum MainError {
    #[error("Connection error: {description}")]
    ConnectionError { description: String },

    #[error("Unknown error: {description}")]
    UnknownError { description: String },
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

/// Global application state
#[derive(Default)]
pub struct State {
    slot_names: Option<Vec<String>>,
    active_slot: Option<usize>,
    active_profile: Option<String>,
    slot_profiles: Option<Vec<String>>,
    profiles: Option<Vec<profiles::Profile>>,
    current_brightness: Option<i64>,
    canvas_hue: Option<f64>,
    canvas_saturation: Option<f64>,
    canvas_lightness: Option<f64>,
    sound_fx: bool,
    ambient_effect: bool,
}

impl State {
    fn new() -> Self {
        Self {
            active_slot: None,
            active_profile: None,
            slot_names: None,
            slot_profiles: None,
            profiles: None,
            current_brightness: None,
            canvas_hue: None,
            canvas_saturation: None,
            canvas_lightness: None,
            sound_fx: false,
            ambient_effect: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConnectionState {
    Initializing,
    Connected,
    Disconnected,
}

/// Update the global canvas
pub fn update_canvas() -> Result<()> {
    if let Some(connection) = crate::CONNECTION.lock().unwrap().as_ref() {
        if let Ok(canvas) = connection.get_canvas() {
            let mut colors = Vec::with_capacity(constants::CANVAS_SIZE);
            for i in 0..CANVAS_SIZE {
                let color = RGBA {
                    r: canvas[i].r(),
                    g: canvas[i].g(),
                    b: canvas[i].b(),
                    a: canvas[i].a(),
                };

                colors.push(color);
            }

            let mut canvas = crate::CANVAS.write().unwrap();
            *canvas = colors;

            Ok(())
        } else {
            crate::LOST_CONNECTION.store(true, Ordering::SeqCst);

            Err(MainError::ConnectionError {
                description: "Could not connect to Eruption".to_string(),
            }
            .into())
        }
    } else {
        // Err(MainError::ConnectionError {
        //     description: "Not connect to the Eruption daemon".to_string(),
        // }
        // .into())

        tracing::warn!("Not connect to the Eruption daemon");

        crate::LOST_CONNECTION.store(true, Ordering::SeqCst);

        Ok(())
    }
}

/// Switch to slot `index`
pub fn switch_to_slot(index: usize) -> Result<()> {
    // tracing::info!("Switching to slot: {}", index);
    util::switch_slot(index)?;

    STATE.write().unwrap().active_slot = Some(index);

    Ok(())
}

/// Switch to profile `file_name`
pub fn switch_to_profile<P: AsRef<Path>>(file_name: P) -> Result<()> {
    let file_name = file_name.as_ref();

    // tracing::info!(
    //     "Switching to profile: {}",
    //     file_name.to_string_lossy()
    // );

    util::switch_profile(&file_name.to_string_lossy())?;

    Ok(())
}

/// Switch to slot `slot_index` and then change the current profile to `file_name`
pub fn switch_to_slot_and_profile<P: AsRef<Path>>(slot_index: usize, file_name: P) -> Result<()> {
    let file_name = file_name.as_ref();

    // tracing::info!(
    //     "Switching to slot: {}, using profile: {}",
    //     slot_index,
    //     file_name.to_string_lossy()
    // );

    util::switch_slot(slot_index)?;
    STATE.write().unwrap().active_slot = Some(slot_index);

    util::switch_profile(&file_name.to_string_lossy())?;

    Ok(())
}

/// Main program entrypoint
#[instrument]
pub fn main() -> std::result::Result<(), eyre::Error> {
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::Layer;

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
                // .with(journald_layer)
                .with(console_layer)
                .with(format_layer)
                .init();
        } else {
            tracing_subscriber::registry()
                // .with(journald_layer)
                // .with(console_layer)
                .with(format_layer)
                .init();
        }
    };

    // i18n/l10n support
    translations::load()?;

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async move { async_main().await })
}

pub async fn async_main() -> std::result::Result<(), eyre::Error> {
    cfg_if::cfg_if! {
        if #[cfg(debug_assertions)] {
            color_eyre::config::HookBuilder::default()
            .panic_section("Please consider reporting a bug at https://github.com/eruption-project/eruption")
            .install()?;
        } else {
            color_eyre::config::HookBuilder::default()
            .panic_section("Please consider reporting a bug at https://github.com/eruption-project/eruption")
            .display_env_section(false)
            .install()?;
        }
    }

    // print a license header, except if we are generating shell completions
    if !env::args().any(|a| a.eq_ignore_ascii_case("completions")) && env::args().count() < 2 {
        print_header();
    }

    register_sigint_handler();

    let opts = Options::parse();
    apply_opts(&opts);

    if let Some(command) = opts.command {
        subcommands::handle_command(command).await?;
    } else {
        install_panic_hook();
        init_terminal()?;

        let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

        *crate::CONNECTION_STATE.write().unwrap() = ConnectionState::Initializing;
        initialize_global_state_and_connection();

        let dbus_system_bus_rx = dbus_client::spawn_dbus_event_loop_system()?;
        let dbus_session_bus_rx = dbus_client::spawn_dbus_event_loop_session()?;

        terminal.draw(tui::clear_screen)?;

        let mut should_quit = false;
        while !should_quit {
            update_canvas()
                .unwrap_or_else(|e| ratelimited::error!("Could not update the canvas: {e}"));

            if let Ok(system_msg) = dbus_system_bus_rx.recv_timeout(Duration::from_millis(0)) {
                update_state(&system_msg).unwrap_or_else(|e| tracing::error!("{e}"));
            }

            if let Ok(session_msg) = dbus_session_bus_rx.recv_timeout(Duration::from_millis(0)) {
                update_state(&session_msg).unwrap_or_else(|e| tracing::error!("{e}"));
            }

            terminal.draw(tui::render_ui)?;

            should_quit = tui::handle_events()?;
            timers::handle_timers()?;
        }

        terminal.draw(tui::clear_screen)?;
        terminal.flush()?;

        restore_terminal()?;
    }

    Ok(())
}

/// Install a panic hook that restores the terminal before panicking.
fn install_panic_hook() {
    better_panic::install();

    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = restore_terminal();
        prev_hook(info);
    }));
}

fn init_terminal() -> Result<()> {
    enable_raw_mode()?;

    stdout().execute(EnterAlternateScreen)?;
    // stdout().execute(EnableMouseCapture)?;
    stdout().execute(PushKeyboardEnhancementFlags(
        KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES,
    ))?;
    stdout().execute(PushKeyboardEnhancementFlags(
        KeyboardEnhancementFlags::REPORT_ALTERNATE_KEYS,
    ))?;

    Ok(())
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;

    stdout().execute(PopKeyboardEnhancementFlags)?;
    stdout().execute(LeaveAlternateScreen)?;
    // stdout().execute(DisableMouseCapture)?;

    Ok(())
}

/// Print license information
#[allow(dead_code)]
fn print_header() {
    println!("{}", tr!("license-header"));
    println!();

    //     println!(
    //         r"
    //  ********                          **   **
    //  /**/////                 ******   /**  //
    //  /**       ****** **   **/**///** ****** **  ******  *******
    //  /******* //**//*/**  /**/**  /**///**/ /** **////**//**///**
    //  /**////   /** / /**  /**/******   /**  /**/**   /** /**  /**
    //  /**       /**   /**  /**/**///    /**  /**/**   /** /**  /**
    //  /********/***   //******/**       //** /**//******  ***  /**
    //  //////// ///     ////// //         //  //  //////  ///   //
    // "
    //     );
}

fn register_sigint_handler() {
    // register ctrl-c handler
    let (ctrl_c_tx, _ctrl_c_rx) = bounded(8);
    ctrlc::set_handler(move || {
        QUIT.store(true, Ordering::SeqCst);

        ctrl_c_tx.send(true).unwrap_or_else(|e| {
            tracing::error!(
                "{}",
                tr!("could-not-send-on-channel", message = e.to_string())
            );
        });
    })
    .unwrap_or_else(|e| {
        tracing::error!(
            "{}",
            tr!("could-not-set-ctrl-c-handler", message = e.to_string())
        )
    });
}

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
            tracing::error!("{}", tr!("could-not-parse-config", message = e.to_string()));
            process::exit(4);
        });

    *CONFIG.write().unwrap() = Some(config);
}

/// initialize global state
pub fn initialize_global_state_and_connection() {
    let mut pages = PAGES.write().unwrap();

    pages.push(Box::new(pages::CanvasPage::new()));
    pages.push(Box::new(pages::KeyboardsPage::new()));
    pages.push(Box::new(pages::MicePage::new()));
    pages.push(Box::new(pages::MiscPage::new()));
    pages.push(Box::new(pages::ColorSchemesPage::new()));
    pages.push(Box::new(pages::RulesPage::new()));
    pages.push(Box::new(pages::ProfilesPage::new()));
    pages.push(Box::new(pages::MacrosPage::new()));
    pages.push(Box::new(pages::KeymapsPage::new()));
    pages.push(Box::new(pages::SettingsPage::new()));
    pages.push(Box::new(pages::LogsPage::new()));
    pages.push(Box::new(pages::HelpPage::new()));
    pages.push(Box::new(pages::AboutPage::new()));

    let mut state = STATE.write().unwrap();

    state.slot_names = util::get_slot_names().ok();
    state.active_slot = util::get_active_slot().ok();
    state.active_profile = util::get_active_profile().ok();
    state.profiles = util::enumerate_profiles().ok();

    // connect to Eruption daemon
    match Connection::new(ConnectionType::Local) {
        Ok(connection) => {
            if let Err(e) = connection.connect() {
                tracing::error!("Could not connect to Eruption daemon: {e}");

                *crate::CONNECTION_STATE.write().unwrap() = ConnectionState::Disconnected;
            } else {
                let _ = connection
                    .get_server_status()
                    .map_err(|e| tracing::error!("{e}"));

                *crate::CONNECTION.lock().unwrap() = Some(connection);

                *crate::CONNECTION_STATE.write().unwrap() = ConnectionState::Connected;
            }
        }

        Err(e) => {
            tracing::error!("Could not connect to Eruption daemon: {}", e);

            *crate::CONNECTION_STATE.write().unwrap() = ConnectionState::Disconnected;
        }
    }
}

/// Update the state of the UI to reflect the current system state
/// This function is called from the D-Bus event loop
pub fn update_state(event: &dbus_client::Message) -> Result<()> {
    if crate::LOST_CONNECTION.load(Ordering::SeqCst) {
        crate::LOST_CONNECTION.store(false, Ordering::SeqCst);

        // set_application_state(ConnectionState::Disconnected, builder)?;
    }

    match *event {
        dbus_client::Message::SlotChanged(slot_index) => {
            STATE.write().unwrap().active_slot = Some(slot_index);
        }

        dbus_client::Message::SlotNamesChanged(ref names) => {
            STATE.write().unwrap().slot_names = Some(names.clone());
        }

        dbus_client::Message::ProfileChanged(ref profile) => {
            STATE.write().unwrap().active_profile = Some(profile.clone());
        }

        dbus_client::Message::BrightnessChanged(brightness) => {
            STATE.write().unwrap().current_brightness = Some(brightness);
        }

        dbus_client::Message::HueChanged(hue) => {
            STATE.write().unwrap().canvas_hue = Some(hue);
        }

        dbus_client::Message::SaturationChanged(saturation) => {
            STATE.write().unwrap().canvas_saturation = Some(saturation);
        }

        dbus_client::Message::LightnessChanged(lightness) => {
            STATE.write().unwrap().canvas_lightness = Some(lightness);
        }

        dbus_client::Message::SoundFxChanged(enabled) => {
            STATE.write().unwrap().sound_fx = enabled;
        }

        dbus_client::Message::AmbientEffectChanged(enabled) => {
            STATE.write().unwrap().ambient_effect = enabled;
        }

        dbus_client::Message::RulesChanged => {
            tracing::info!("Process monitor ruleset has changed");
        }

        dbus_client::Message::DeviceHotplug(_device_info) => {
            tracing::info!("A device has been hotplugged/removed");
        }

        dbus_client::Message::DeviceStatusChanged(ref status_json) => {
            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
            struct DeviceStatus {
                index: u64,
                usb_vid: u16,
                usb_pid: u16,
                status: hwdevices::DeviceStatus,
            }

            let status_vec = serde_json::from_str::<Vec<DeviceStatus>>(status_json)?;

            let mut status_map = HashMap::new();
            status_vec.iter().for_each(|s| {
                status_map.insert(s.index, s.status.clone());
            });

            // tracing::debug!("A device status update has been received: {status_map:#?}");

            *crate::DEVICE_STATUS.write().unwrap() = status_map;
        }
    }

    Ok(())
}

/// Event handling utilities
pub mod events {
    use lazy_static::lazy_static;
    use std::sync::atomic::{AtomicBool, AtomicUsize};

    lazy_static! {
        /// stores how many consecutive events shall be ignored
        static ref IGNORE_NEXT_UI_EVENTS: AtomicUsize = AtomicUsize::new(0);

        /// stores how many consecutive events shall be ignored
        static ref IGNORE_NEXT_DBUS_EVENTS: AtomicUsize = AtomicUsize::new(0);

        /// signals whether we should reinitialize the GUI asap (e.g.: used when hot-plugging new devices)
        pub static ref UPDATE_MAIN_WINDOW: AtomicBool = AtomicBool::new(false);

        /// signals whether we have just lost the connection to the Eruption daemon
        pub static ref LOST_CONNECTION: AtomicBool = AtomicBool::new(false);

        /// signals whether we have just re-connected to the Eruption daemon
        pub static ref GAINED_CONNECTION: AtomicBool = AtomicBool::new(false);
    }

    /* /// ignore next n events (do not act on them)
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
    } */
}
