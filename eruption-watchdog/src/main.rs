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

use clap::CommandFactory;
use clap::Parser;
use clap_complete::Shell;
use eruption_sdk::connection::{Connection, ConnectionType};
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use rust_embed::RustEmbed;
// use colored::*;
use flume::unbounded;
use std::sync::atomic::{AtomicBool, Ordering};
use std::{
    env,
    process::{Command, Stdio},
    sync::Arc,
    thread,
    time::Duration,
};
use tracing::*;

mod constants;
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

lazy_static! {
    /// Global configuration
    // pub static ref CONFIG: Arc<Mutex<Option<config::Config>>> = Arc::new(Mutex::new(None));

    /// Global "quit" status flag
    pub static ref QUIT: AtomicBool = AtomicBool::new(false);
}

#[derive(Debug, thiserror::Error)]
pub enum MainError {
    #[error("Unknown error: {description}")]
    UnknownError { description: String },

    #[error("Could not parse syslog log-level")]
    SyslogLevelError {},
}

lazy_static! {
    static ref ABOUT: String = tr!("about");
    static ref VERBOSE_ABOUT: String = tr!("verbose-about");
    static ref DAEMON_ABOUT: String = tr!("daemon-about");
    static ref COMPLETIONS_ABOUT: String = tr!("completions-about");
}

/// Supported command line arguments
#[derive(Debug, clap::Parser)]
#[clap(
    version = env!("CARGO_PKG_VERSION"),
    author = "X3n0m0rph59 <x3n0m0rph59@gmail.com>",
    about = ABOUT.as_str()
)]
pub struct Options {
    #[clap(
        help(VERBOSE_ABOUT.as_str()),
        short,
        long,
        action = clap::ArgAction::Count
    )]
    verbose: u8,

    #[clap(subcommand)]
    command: Subcommands,
}

// Sub-commands
#[derive(Debug, clap::Parser)]
pub enum Subcommands {
    #[clap(about(DAEMON_ABOUT.as_str()))]
    Daemon,

    #[clap(hide = true, about(COMPLETIONS_ABOUT.as_str()))]
    Completions {
        // #[clap(subcommand)]
        shell: Shell,
    },
}

/// Print license information
#[allow(dead_code)]
fn print_header() {
    println!("{}", tr!("license-header"));
    println!();
}

pub fn stop_or_kill_eruption_daemon() -> Result<()> {
    tracing::info!("Now stopping the eruption.service...");

    /* let status = Command::new("/bin/systemctl")
        .arg("stop")
        .arg("eruption.service")
        .stdout(Stdio::null())
        .status();

    if status.is_ok() && status.unwrap().success() {
        // eruption.service has stopped
        tracing::info!("Eruption has been stopped successfully, exiting now");

        Ok(())
    } else { */
    let mut retry_counter = 0;

    'WAIT_KILL_LOOP: loop {
        let status = Command::new("/bin/systemctl")
            .arg("kill")
            .arg("-sSIGKILL")
            .arg("eruption.service")
            .stdout(Stdio::null())
            .status();

        match status {
            Ok(status) => {
                if status.success() {
                    tracing::info!("Eruption has been killed successfully, exiting now");

                    break 'WAIT_KILL_LOOP Ok(());
                } else {
                    thread::sleep(Duration::from_millis(2000));

                    if retry_counter >= 3 {
                        tracing::error!("Error while killing the Eruption daemon");

                        break 'WAIT_KILL_LOOP Ok(());
                    } else {
                        retry_counter += 1;
                    }
                }
            }

            Err(e) => {
                tracing::error!("Error while killing Eruption: {}", e);

                break 'WAIT_KILL_LOOP Ok(());
            }
        }
    }
    /*}*/
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

    // register ctrl-c handler
    let (ctrl_c_tx, _ctrl_c_rx) = unbounded();
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

    let opts = Options::parse();
    match opts.command {
        Subcommands::Daemon => {
            tracing::info!("Eruption watchdog daemon initializing...");

            'MAIN_LOOP: loop {
                if QUIT.load(Ordering::SeqCst) {
                    tracing::debug!("CTRL-C pressed, terminating now...");

                    break 'MAIN_LOOP;
                }

                tracing::info!("Polling the Eruption daemon now...");

                tracing::debug!("Connecting to the Eruption daemon...");

                let connection = Connection::new(ConnectionType::Local)?;
                match connection.connect() {
                    Ok(()) => {
                        tracing::debug!("Successfully connected to the Eruption daemon");

                        let status = connection.get_server_status();

                        match status {
                            Ok(status) => {
                                tracing::debug!("Response: {}", status.server);

                                connection.disconnect()?;
                                tracing::debug!("Disconnected from the Eruption daemon");
                            }

                            Err(e) => {
                                tracing::warn!("Eruption daemon seems to have crashed: {}", e);

                                tracing::info!("Attempting to kill the Eruption daemon now...");

                                stop_or_kill_eruption_daemon()?;
                            }
                        }
                    }

                    Err(e) => {
                        tracing::error!("Failed to connect to the Eruption daemon: {}", e);
                    }
                }

                thread::sleep(Duration::from_millis(2000));
            }

            tracing::info!("Eruption watchdog daemon terminating now");
        }

        Subcommands::Completions { shell } => {
            const BIN_NAME: &str = env!("CARGO_PKG_NAME");

            let mut command = Options::command();
            let mut fd = std::io::stdout();

            clap_complete::generate(shell, &mut command, BIN_NAME.to_string(), &mut fd);
        }
    }

    Ok(())
}

/// Main program entrypoint
pub fn main() -> std::result::Result<(), eyre::Error> {
    // initialize logging
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::util::SubscriberInitExt;

    if atty::is(atty::Stream::Stdout) {
        // let filter = tracing_subscriber::EnvFilter::from_default_env();
        // let journald_layer = tracing_journald::layer()?.with_filter(filter);

        #[cfg(not(target_os = "windows"))]
        let ansi = true;

        #[cfg(target_os = "windows")]
        let ansi = false;

        let filter = tracing_subscriber::EnvFilter::from_default_env();
        let format_layer = tracing_subscriber::fmt::layer()
            .compact()
            .with_ansi(ansi)
            .with_line_number(true)
            .with_filter(filter);

        cfg_if::cfg_if! {
            if #[cfg(feature = "debug-async")] {
                let console_layer = Some(console_subscriber::ConsoleLayer::builder()
                    .with_default_env()
                    .spawn());

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
    } else {
        let filter = tracing_subscriber::EnvFilter::from_default_env();
        let journald_layer = tracing_journald::layer()?.with_filter(filter);

        tracing_subscriber::registry().with(journald_layer).init();
    }

    // i18n/l10n support
    let language_loader: FluentLanguageLoader = fluent_language_loader!();

    let requested_languages = DesktopLanguageRequester::requested_languages();
    i18n_embed::select(&language_loader, &Localizations, &requested_languages)?;

    STATIC_LOADER.lock().replace(language_loader);

    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    runtime.block_on(async move { async_main().await })
}
