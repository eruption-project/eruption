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

use clap::{IntoApp, Parser};
use clap_generate::Shell;
use crossbeam::channel::{unbounded, Receiver};
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};
use lazy_static::lazy_static;
use log::{debug, error, info, trace};
use parking_lot::Mutex;
use prost::Message;
use rust_embed::RustEmbed;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::time::Duration;
use std::{env, thread};
use syslog::Facility;
use tokio::io::{self, AsyncWriteExt, Interest};
use tokio::net::UnixStream;

use crate::audio::AudioBackend;

use protocol::command::CommandType;
use protocol::response::ResponseType;
use protocol::Command;

mod audio;
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

pub mod protocol {
    include!(concat!(env!("OUT_DIR"), "/audio_proxy.rs"));
}

type Result<T> = std::result::Result<T, eyre::Error>;

lazy_static! {
    // /// Global command line options
    // pub static ref OPTIONS: Arc<Mutex<Option<Options>>> = Arc::new(Mutex::new(None));

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

/// Supported command line arguments
#[derive(Debug, clap::Parser)]
#[clap(
    version = env!("CARGO_PKG_VERSION"),
    author = "X3n0m0rph59 <x3n0m0rph59@gmail.com>",
    about = "Audio proxy daemon for the Eruption Linux user-mode driver",
)]
pub struct Options {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Sets the configuration file to use
    #[clap(short, long)]
    config: Option<String>,

    // subcommands
    #[clap(subcommand)]
    command: Subcommands,
}

// Sub-commands
#[derive(Debug, clap::Parser)]
pub enum Subcommands {
    /// Run in background
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
        r#"
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
"#
    );
}

pub async fn run_main_loop(_ctrl_c_rx: &Receiver<bool>) -> Result<()> {
    debug!("Opening audio device(s)");
    let mut audio_backend = audio::PulseAudioBackend::new()?;

    debug!("Entering the main loop now...");

    'MAIN_LOOP: loop {
        if QUIT.load(Ordering::SeqCst) {
            break 'MAIN_LOOP Ok(());
        }

        debug!("Connecting to the Eruption audio socket...");

        match UnixStream::connect(constants::AUDIO_SOCKET_NAME).await {
            Ok(mut socket) => {
                'IO_LOOP: loop {
                    if QUIT.load(Ordering::SeqCst) {
                        break 'MAIN_LOOP Ok(());
                    }

                    // record samples to the global buffer
                    if let Err(e) = audio_backend.record_samples() {
                        error!("An error occurred while recording audio: {}", e);

                        // sleep a while then re-open audio devices
                        thread::sleep(Duration::from_millis(constants::SLEEP_TIME_MILLIS));

                        debug!("Re-opening audio device(s)");
                        audio_backend = audio::PulseAudioBackend::new()?;

                        // break 'IO_LOOP;
                    }

                    let ready = socket
                        .ready(Interest::READABLE | Interest::WRITABLE)
                        .await?;

                    if ready.is_readable() {
                        let mut buf = bytes::BytesMut::new();
                        buf.resize(constants::BUFFER_CAPACITY + 64, 0x00);

                        match socket.try_read_buf(&mut buf) {
                            Ok(0) => {
                                if QUIT.load(Ordering::SeqCst) {
                                    break 'MAIN_LOOP Ok(());
                                }

                                trace!("Short read from audio socket");

                                break 'IO_LOOP;
                            }

                            Ok(n) => {
                                if QUIT.load(Ordering::SeqCst) {
                                    break 'MAIN_LOOP Ok(());
                                }

                                trace!("Read {} bytes from audio socket", n);
                            }

                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                if QUIT.load(Ordering::SeqCst) {
                                    break 'MAIN_LOOP Ok(());
                                }

                                // not an error, so continue
                                continue 'IO_LOOP;
                            }

                            Err(e) => {
                                error!("An error occurred during socket read: {}", e);

                                break 'IO_LOOP;
                            }
                        }

                        let message = Command::decode_length_delimited(&mut buf)?;
                        let mut buf = match message.command_type() {
                            CommandType::AudioMutedState => {
                                let mut response = protocol::Response::default();

                                response.set_response_type(ResponseType::AudioMutedState);
                                response.payload = Some(protocol::response::Payload::Muted(
                                    audio_backend.is_audio_muted()?,
                                ));

                                message.encode_length_delimited_to_vec()
                            }

                            CommandType::AudioVolume => {
                                let mut response = protocol::Response::default();

                                response.set_response_type(ResponseType::AudioVolume);
                                response.payload = Some(protocol::response::Payload::Volume(
                                    audio_backend.get_audio_volume()?,
                                ));

                                message.encode_length_delimited_to_vec()
                            }
                        };

                        buf.resize(constants::BUFFER_CAPACITY + 64, 0x00);

                        if ready.is_writable() {
                            match socket.try_write(&buf) {
                                Ok(n) => {
                                    if QUIT.load(Ordering::SeqCst) {
                                        break 'MAIN_LOOP Ok(());
                                    }

                                    trace!("Wrote {} bytes to audio socket", n);

                                    // if n < constants::BUFFER_CAPACITY {
                                    //     error!("Short write");
                                    // }
                                }

                                Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                    if QUIT.load(Ordering::SeqCst) {
                                        break 'MAIN_LOOP Ok(());
                                    }

                                    // not an error, so continue
                                    continue 'IO_LOOP;
                                }

                                Err(e) => {
                                    error!("An error occurred during socket write: {}", e);

                                    break 'IO_LOOP;
                                }
                            }
                        }

                        let _ = socket.flush();

                        // continue by reading another command, if available
                        // continue;
                    } else if ready.is_writable() {
                        let samples = audio::AUDIO_BUFFER.read().clone();

                        let mut response = protocol::Response::default();

                        response.set_response_type(ResponseType::AudioData);
                        response.payload = Some(protocol::response::Payload::Data(samples));

                        let mut buf = response.encode_length_delimited_to_vec();
                        buf.resize(constants::BUFFER_CAPACITY + 64, 0x00);

                        match socket.try_write(&buf) {
                            Ok(n) => {
                                if QUIT.load(Ordering::SeqCst) {
                                    break 'MAIN_LOOP Ok(());
                                }

                                trace!("Wrote {} bytes to audio socket", n);

                                // if n < constants::BUFFER_CAPACITY {
                                //     error!("Short write");
                                // }
                            }

                            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                                if QUIT.load(Ordering::SeqCst) {
                                    break 'MAIN_LOOP Ok(());
                                }

                                // not an error, so continue
                                continue 'IO_LOOP;
                            }

                            Err(e) => {
                                error!("An error occurred during socket write: {}", e);

                                break 'IO_LOOP;
                            }
                        }

                        let _ = socket.flush();
                    }
                }
            }

            Err(e)
                if e.kind() == io::ErrorKind::NotFound
                    || e.kind() == io::ErrorKind::ConnectionRefused =>
            {
                debug!("Audio socket is currently not available, sleeping now...");

                if QUIT.load(Ordering::SeqCst) {
                    break 'MAIN_LOOP Ok(());
                }

                thread::sleep(Duration::from_millis(constants::SLEEP_TIME_MILLIS));
            }

            Err(e) => {
                error!(
                    "An unknown error occurred while connecting to audio socket: {}",
                    e
                );

                if QUIT.load(Ordering::SeqCst) {
                    break 'MAIN_LOOP Ok(());
                }

                thread::sleep(Duration::from_millis(constants::SLEEP_TIME_MILLIS));
            }
        }
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

    // if unsafe { libc::isatty(0) != 0 } {
    //     print_header();
    // }

    let opts = Options::parse();
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

            info!("Startup completed");

            // enter the main loop
            run_main_loop(&ctrl_c_rx)
                .await
                .unwrap_or_else(|e| error!("{}", e));

            debug!("Left the main loop");

            info!("Exiting now");
        }

        Subcommands::Completions { shell } => {
            const BIN_NAME: &str = env!("CARGO_PKG_NAME");

            let mut app = Options::into_app();
            let mut fd = std::io::stdout();

            clap_generate::generate(shell, &mut app, BIN_NAME.to_string(), &mut fd);
        }
    };

    Ok(())
}

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
