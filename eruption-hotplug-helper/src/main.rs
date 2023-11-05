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
use is_terminal::IsTerminal;
use lazy_static::lazy_static;
use rust_embed::RustEmbed;
use std::path::PathBuf;
use tracing_mutex::stdsync::Mutex;
// use colored::*;
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
        let loader = $crate::STATIC_LOADER.lock().unwrap();
        let loader = loader.as_ref().unwrap();

        i18n_embed_fl::fl!(loader, $message_id)
    }};

    ($message_id:literal, $($args:expr),*) => {{
        let loader = $crate::STATIC_LOADER.lock().unwrap();
        let loader = loader.as_ref().unwrap();

        i18n_embed_fl::fl!(loader, $message_id, $($args), *)
    }};
}

// type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum MainError {
    #[error("Unknown error: {description}")]
    UnknownError { description: String },
}

lazy_static! {
    static ref ABOUT: String = tr!("about");
    static ref VERBOSE_ABOUT: String = tr!("verbose-about");
    static ref HOTPLUG_ABOUT: String = tr!("hotplug-about");
    static ref RESUME_ABOUT: String = tr!("resume-about");
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
    #[clap(about(HOTPLUG_ABOUT.as_str()))]
    Hotplug { devpath: Option<PathBuf> },

    #[clap(about(RESUME_ABOUT.as_str()))]
    Resume,

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

// pub fn restart_eruption_daemon() -> Result<()> {
//     // sleep until udev has settled
//     tracing::info!("Waiting for the devices to settle...");
//
//     let status = Command::new("/bin/udevadm")
//         .arg("settle")
//         .stdout(Stdio::null())
//         .status();
//
//     if let Err(e) = status {
//         // udev-settle has failed, sleep a while and let the devices settle
//
//         tracing::error!("udevadm settle has failed: {}", e);
//
//         thread::sleep(Duration::from_millis(3500));
//     } else {
//         // sleep a while just to be safe
//         thread::sleep(Duration::from_millis(1500));
//
//         tracing::info!("Done, all devices have settled");
//     }
//
//     tracing::info!("Now restarting the eruption.service...");
//
//     let status = Command::new("/bin/systemctl")
//         .arg("restart")
//         .arg("eruption.service")
//         .stdout(Stdio::null())
//         .status()?;
//
//     if status.success() {
//         // wait for the eruption.service to be fully operational...
//         tracing::info!("Waiting for Eruption to be fully operational...");
//
//         let mut retry_counter = 0;
//
//         'WAIT_START_LOOP: loop {
//             let result = Command::new("/bin/systemctl")
//                 .arg("is-active")
//                 .arg("eruption.service")
//                 .stdout(Stdio::null())
//                 .status();
//
//             match result {
//                 Ok(status) => {
//                     if status.success() {
//                         tracing::info!("Eruption has been started successfully, exiting now");
//
//                         break 'WAIT_START_LOOP;
//                     } else {
//                         thread::sleep(Duration::from_millis(2000));
//
//                         if retry_counter >= 5 {
//                             tracing::error!("Timeout while starting eruption.service");
//
//                             break 'WAIT_START_LOOP;
//                         } else {
//                             retry_counter += 1;
//                         }
//                     }
//                 }
//
//                 Err(e) => {
//                     tracing::error!("Error while waiting for Eruption to start: {}", e);
//
//                     break 'WAIT_START_LOOP;
//                 }
//             }
//         }
//     } else {
//         tracing::error!("Could not start Eruption, an error occurred");
//     }
//
//     Ok(())
// }

/// Main program entrypoint
pub fn main() -> std::result::Result<(), eyre::Error> {
    // initialize logging
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::util::SubscriberInitExt;

    if std::io::stdout().is_terminal() {
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

    STATIC_LOADER.lock().unwrap().replace(language_loader);

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

    if std::io::stdout().is_terminal() {
        // print a license header, except if we are generating shell completions
        if !env::args().any(|a| a.eq_ignore_ascii_case("completions"))
            && !env::args().any(|a| a.eq_ignore_ascii_case("hotplug"))
            && env::args().count() < 2
        {
            print_header();
        }
    }

    let opts = Options::parse();
    match opts.command {
        Subcommands::Hotplug { devpath } => {
            tracing::info!("A hotplug event has been triggered, notifying the Eruption daemon...");

            // sleep until udev has settled
            /* tracing::info!("Waiting for the devices to settle...");

            let status = Command::new("/bin/udevadm")
                .arg("settle")
                .stdout(Stdio::null())
                .status();

            if let Err(e) = status {
                // udev-settle has failed, sleep a while and let the devices settle
                tracing::error!("udevadm settle has failed: {}", e);

                thread::sleep(Duration::from_millis(3500));
            } else {
                tracing::info!("Done, all devices have settled");
            } */

            tracing::info!("Connecting to the Eruption daemon...");

            let connection = Connection::new(ConnectionType::Local)?;
            match connection.connect() {
                Ok(()) => {
                    tracing::debug!("Successfully connected to the Eruption daemon");
                    // let _status = connection.get_server_status()?;

                    tracing::info!("Notifying the Eruption daemon about the hotplug event...");

                    let hotplug_info = util::get_hotplug_info(devpath.as_ref())?;
                    connection.notify_device_hotplug(&hotplug_info)?;

                    connection.disconnect()?;
                    tracing::info!("Disconnected from the Eruption daemon");
                }

                Err(e) => {
                    tracing::error!("Failed to connect to the Eruption daemon: {}", e);
                }
            }
        }

        Subcommands::Resume => {
            tracing::info!("A resume event has been triggered, notifying the Eruption daemon...");

            // sleep until udev has settled
            tracing::info!("Waiting for the devices to settle...");

            let status = Command::new("/bin/udevadm")
                .arg("settle")
                .stdout(Stdio::null())
                .status();

            if let Err(e) = status {
                // udev-settle has failed, sleep a while and let the devices settle
                tracing::error!("udevadm settle has failed: {}", e);

                thread::sleep(Duration::from_millis(3500));
            } else {
                tracing::info!("Done, all devices have settled");
            }

            tracing::info!("Connecting to the Eruption daemon...");

            let connection = Connection::new(ConnectionType::Local)?;
            match connection.connect() {
                Ok(()) => {
                    tracing::debug!("Successfully connected to the Eruption daemon");
                    // let _status = connection.get_server_status()?;

                    tracing::info!("Notifying the Eruption daemon about the resume event...");

                    connection.notify_resume_from_suspend()?;

                    connection.disconnect()?;
                    tracing::info!("Disconnected from the Eruption daemon");
                }

                Err(e) => {
                    tracing::error!("Failed to connect to the Eruption daemon: {}", e);
                }
            }
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
