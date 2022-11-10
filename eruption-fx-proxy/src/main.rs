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

use clap::CommandFactory;
use clap::Parser;
use clap_complete::Shell;

use flume::unbounded;
use flume::Receiver;
use flume::Sender;
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};

use lazy_static::lazy_static;
use log::{debug, error, info};
use parking_lot::{Mutex, RwLock};
use rust_embed::RustEmbed;
use std::sync::Arc;
use std::{
    env,
    sync::atomic::{AtomicBool, Ordering},
    thread,
};
use syslog::Facility;

use tokio::time::Duration;

use eruption_sdk::canvas::Canvas;
use eruption_sdk::color::Color;
use eruption_sdk::connection::{Connection, ConnectionType};

mod backends;
mod constants;
mod dbus_client;
mod dbus_interface;
mod hwdevices;
mod util;

#[derive(RustEmbed)]
#[folder = "i18n"] // path to the compiled localization resources
struct Localizations;

lazy_static! {
    /// Global configuration
    pub static ref STATIC_LOADER: Arc<Mutex<Option<FluentLanguageLoader>>> = Arc::new(Mutex::new(None));

    pub static ref OPTIONS: Arc<RwLock<Option<Options>>> = Arc::new(RwLock::new(None));
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

lazy_static! {
    /// Enable Ambient effect flag
    pub static ref ENABLE_AMBIENT_EFFECT: AtomicBool = AtomicBool::new(false);

    /// Global "quit" status flag
    pub static ref QUIT: AtomicBool = AtomicBool::new(false);
}

#[derive(Debug, thiserror::Error)]
pub enum MainError {
    #[error("Could not parse syslog log-level")]
    SyslogLevelError {},

    #[error("Unknown error: {description}")]
    UnknownError { description: String },
}

lazy_static! {
    static ref ABOUT: String = tr!("about");
    static ref VERBOSE_ABOUT: String = tr!("verbose-about");
    static ref CONFIG_ABOUT: String = tr!("config-about");
    static ref DAEMON_ABOUT: String = tr!("daemon-about");
    static ref COMPLETIONS_ABOUT: String = tr!("completions-about");
}

/// Supported command line arguments
#[derive(Debug, Clone, clap::Parser)]
#[clap(
    version = env!("CARGO_PKG_VERSION"),
    author = "X3n0m0rph59 <x3n0m0rph59@gmail.com>",
    about = ABOUT.as_str(),
)]
pub struct Options {
    #[clap(
        help(VERBOSE_ABOUT.as_str()),
        short,
        long,
        action = clap::ArgAction::Count
    )]
    verbose: u8,

    #[clap(help(CONFIG_ABOUT.as_str()), short, long)]
    config: Option<String>,

    #[clap(subcommand)]
    command: Subcommands,
}

// Sub-commands
#[derive(Debug, Clone, clap::Parser)]
pub enum Subcommands {
    #[clap(about(DAEMON_ABOUT.as_str()))]
    Daemon,

    /// Generate shell completions
    Completions {
        // #[clap(subcommand)]
        shell: Shell,
    },
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

#[derive(Debug, Clone)]
pub enum DbusApiEvent {}

/// Spawns the D-Bus API thread and executes it's main loop
fn spawn_dbus_api_thread(dbus_tx: Sender<dbus_interface::Message>) -> Result<Sender<DbusApiEvent>> {
    let (dbus_api_tx, dbus_api_rx) = unbounded();

    thread::Builder::new()
        .name("dbus-interface".into())
        .spawn(move || -> Result<()> {
            let dbus = dbus_interface::initialize(dbus_tx)?;

            loop {
                // process events, destined for the dbus api
                match dbus_api_rx.recv_timeout(Duration::from_millis(0)) {
                    Ok(result) => match result {},

                    // ignore timeout errors
                    Err(_e) => (),
                }

                dbus.get_next_event_timeout(constants::DBUS_TIMEOUT_MILLIS as u32)
                    .unwrap_or_else(|e| error!("Could not get the next D-Bus event: {}", e));
            }
        })?;

    Ok(dbus_api_tx)
}

pub async fn run_main_loop(_ctrl_c_rx: &Receiver<bool>) -> Result<()> {
    debug!("Entering the main loop now...");

    'MAIN_LOOP: loop {
        log::trace!("Main loop iteration");

        if QUIT.load(Ordering::SeqCst) {
            break 'MAIN_LOOP Ok(());
        }

        // instantiate the best fitting backend for the current system configuration
        let mut backend = backends::get_best_fitting_backend()?;

        log::debug!("Connecting to Eruption...");

        let connection = Connection::new(ConnectionType::Local)?;
        connection.connect()?;

        log::debug!("Successfully connected to the Eruption daemon");

        let _status = connection.get_server_status()?;

        // get device; used for topology information
        let device = util::get_primary_keyboard_device()?;

        let mut canvas_cleared = false;

        // create a new canvas
        let mut canvas = Canvas::new();
        canvas.fill(Color::new(0, 0, 0, 0));

        'EVENT_LOOP: loop {
            // log::trace!("Event loop iteration");

            let mut any_updates = false;

            if QUIT.load(Ordering::SeqCst) {
                break 'EVENT_LOOP;
            }

            if ENABLE_AMBIENT_EFFECT.load(Ordering::SeqCst) {
                // request a screenshot from the backend and convert the image to the device's topology
                let image_buffer = backend.poll()?;
                let result = util::process_image_buffer(image_buffer, &device)?;

                // TODO: Implement blend code
                // utils::blend(&mut canvas, &result);
                canvas = result;

                any_updates = true;
            }

            if any_updates {
                log::debug!("Submitting canvas...");

                connection.submit_canvas(&canvas)?;
                canvas_cleared = false;

                thread::sleep(Duration::from_millis(constants::DEFAULT_FRAME_DELAY_MILLIS));
            } else {
                if !canvas_cleared {
                    // cleanup, clear the canvas
                    log::debug!("Clearing canvas...");

                    canvas.fill(Color::new(0, 0, 0, 0));
                    connection.submit_canvas(&canvas)?;

                    canvas_cleared = true;
                } else {
                    log::debug!("Nothing updated");

                    thread::sleep(Duration::from_millis(constants::MAIN_LOOP_SLEEP_MILLIS));
                }
            }
        }

        // on exit, clear the canvas
        log::debug!("Clearing canvas...");

        canvas.fill(Color::new(0, 0, 0, 0));
        connection.submit_canvas(&canvas)?;
    }
}

pub async fn async_main() -> std::result::Result<(), eyre::Error> {
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

    // start the thread deadlock detector
    // #[cfg(debug_assertions)]
    // thread_util::deadlock_detector()
    //     .unwrap_or_else(|e| error!("Could not spawn deadlock detector thread: {}", e));

    let opts = Options::parse();
    *crate::OPTIONS.write() = Some(opts.clone());

    let daemon = matches!(opts.command, Subcommands::Daemon);

    if unsafe { libc::isatty(0) != 0 } && daemon {
        // initialize logging on console
        if env::var("RUST_LOG").is_err() {
            env::set_var("RUST_LOG_OVERRIDE", "info");
            pretty_env_logger::init_custom_env("RUST_LOG_OVERRIDE");
        } else {
            pretty_env_logger::init();
        }
    } else {
        // initialize logging to syslog
        let mut errors_present = false;

        let level_filter = match env::var("RUST_LOG")
            .unwrap_or_else(|_| "info".to_string())
            .to_lowercase()
            .as_str()
        {
            "off" => log::LevelFilter::Off,
            "error" => log::LevelFilter::Error,
            "warn" => log::LevelFilter::Warn,
            "info" => log::LevelFilter::Info,
            "debug" => log::LevelFilter::Debug,
            "trace" => log::LevelFilter::Trace,

            _ => {
                errors_present = true;
                log::LevelFilter::Info
            }
        };

        syslog::init(
            Facility::LOG_USER,
            level_filter,
            Some(env!("CARGO_PKG_NAME")),
        )
        .map_err(|_e| MainError::SyslogLevelError {})?;

        if errors_present {
            log::error!("Could not parse syslog log-level");
        }
    }

    match opts.command {
        Subcommands::Daemon => {
            info!("Starting up...");

            // register ctrl-c handler
            let (ctrl_c_tx, ctrl_c_rx) = unbounded();
            ctrlc::set_handler(move || {
                QUIT.store(true, Ordering::SeqCst);

                ctrl_c_tx
                    .send(true)
                    .unwrap_or_else(|e| error!("Could not send on a channel: {}", e));
            })
            .unwrap_or_else(|e| error!("Could not set CTRL-C handler: {}", e));

            // initialize the D-Bus API
            let (dbus_tx, _dbus_rx) = unbounded();
            let _dbus_api_tx = spawn_dbus_api_thread(dbus_tx)?;

            // register all available screenshot backends
            backends::register_backends()?;

            log::info!("Startup completed");

            // enter the main loop
            run_main_loop(&ctrl_c_rx)
                .await
                .unwrap_or_else(|e| error!("{}", e));

            log::debug!("Left the main loop");

            log::info!("Exiting now");
        }

        Subcommands::Completions { shell } => {
            const BIN_NAME: &str = env!("CARGO_PKG_NAME");

            let mut command = Options::command();
            let mut fd = std::io::stdout();

            clap_complete::generate(shell, &mut command, BIN_NAME.to_string(), &mut fd);
        }
    };

    Ok(())
}

/*
#[cfg(debug_assertions)]
mod thread_util {
    use crate::Result;
    use log::*;
    use parking_lot::deadlock;
    use std::thread;
    use std::time::Duration;

    /// Creates a background thread which checks for deadlocks every 5 seconds
    pub(crate) fn deadlock_detector() -> Result<()> {
        thread::Builder::new()
            .name("deadlockd".to_owned())
            .spawn(move || loop {
                thread::sleep(Duration::from_secs(5));
                let deadlocks = deadlock::check_deadlock();
                if !deadlocks.is_empty() {
                    error!("{} deadlocks detected", deadlocks.len());

                    for (i, threads) in deadlocks.iter().enumerate() {
                        error!("Deadlock #{}", i);

                        for t in threads {
                            error!("Thread Id {:#?}", t.thread_id());
                            error!("{:#?}", t.backtrace());
                        }
                    }
                }
            })?;

        Ok(())
    }
}
*/

/// Main program entrypoint
pub fn main() -> std::result::Result<(), eyre::Error> {
    let language_loader: FluentLanguageLoader = fluent_language_loader!();

    let requested_languages = DesktopLanguageRequester::requested_languages();
    i18n_embed::select(&language_loader, &Localizations, &requested_languages)?;

    STATIC_LOADER.lock().replace(language_loader);

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async move { async_main().await })
}
