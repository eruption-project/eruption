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

use clap::{IntoApp, Parser};
use clap_complete::Shell;
use colored::Colorize;
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};
use jwalk::WalkDir;
use lazy_static::lazy_static;
use parking_lot::{Mutex, RwLock};
use rust_embed::RustEmbed;
use std::{cmp::Ordering, env, thread};
use std::{path::PathBuf, sync::Arc};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};
use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::Duration;

mod backends;
mod constants;
mod hwdevices;
mod utils;

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

// type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum MainError {
    #[error("Unknown error: {description}")]
    UnknownError { description: String },
}

/// Supported command line arguments
#[derive(Debug, Clone, clap::Parser)]
#[clap(
    version = env!("CARGO_PKG_VERSION"),
    author = "X3n0m0rph59 <x3n0m0rph59@gmail.com>",
    about = "A Network FX protocol client for Eruption",
)]
pub struct Options {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, parse(from_occurrences))]
    verbose: u8,

    /// The keyboard model, e.g. "ROCCAT Vulcan Pro TKL" or "1e7d:311a"
    model: Option<String>,

    hostname: Option<String>,
    port: Option<u16>,

    #[clap(subcommand)]
    command: Subcommands,
}

// Sub-commands
#[derive(Debug, Clone, clap::Parser)]
pub enum Subcommands {
    /// Ping the server
    Ping,

    /// Send Network FX raw protocol commands to the server
    Command { data: String },

    /// Load an image file and display it on the connected devices
    Image { filename: PathBuf },

    /// Load image files from a directory and display each one on the connected devices
    Animation {
        directory_name: PathBuf,
        frame_delay: Option<u64>,
    },

    /// Make the LEDs of connected devices reflect what is shown on the screen
    Ambient { frame_delay: Option<u64> },

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

 Copyright (c) 2019-2022, The Eruption Development Team
"#
    );
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

    // initialize logging
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG_OVERRIDE", "info");
        pretty_env_logger::init_custom_env("RUST_LOG_OVERRIDE");
    } else {
        pretty_env_logger::init();
    }

    let opts = Options::parse();
    *crate::OPTIONS.write() = Some(opts.clone());

    match opts.command {
        Subcommands::Ping => {
            let address = format!(
                "{}:{}",
                opts.hostname
                    .unwrap_or_else(|| constants::DEFAULT_HOST.to_owned()),
                opts.port.unwrap_or(constants::DEFAULT_PORT)
            );
            if opts.verbose > 1 {
                println!("{}", tr!("connecting-to", host = address.to_string()));
            }

            let socket = TcpStream::connect(address).await?;
            let mut buf_reader = BufReader::new(socket);

            // print and send the specified command
            if opts.verbose > 0 {
                println!("{}", tr!("sending-status-inquiry"));
            }
            buf_reader.write_all(&Vec::from("STATUS\n")).await?;

            // receive and print the response
            let mut buffer = String::new();
            buf_reader.read_line(&mut buffer).await?;

            println!("{}", &buffer.bold());
        }

        Subcommands::Command { data } => {
            let address = format!(
                "{}:{}",
                opts.hostname
                    .unwrap_or_else(|| constants::DEFAULT_HOST.to_owned()),
                opts.port.unwrap_or(constants::DEFAULT_PORT)
            );
            if opts.verbose > 1 {
                println!("{}", tr!("connecting-to", host = address.to_string()));
            }

            let socket = TcpStream::connect(address).await?;
            let mut buf_reader = BufReader::new(socket);

            if data == "-" {
                let stdin = io::stdin();
                let mut reader = BufReader::new(stdin);

                loop {
                    let mut line = String::new();
                    let len = reader.read_line(&mut line).await?;
                    if len == 0 {
                        break;
                    }

                    // print and send the current command
                    if opts.verbose > 0 {
                        println!("{}", line.italic());
                    }

                    let v = Vec::from(line);
                    buf_reader.write_all(&v).await?;

                    // receive and print the response
                    let mut buffer = String::new();
                    buf_reader.read_line(&mut buffer).await?;

                    println!("{}", buffer.bold());

                    if buffer.starts_with("BYE") || buffer.starts_with("ERROR:") {
                        break;
                    }
                }
            } else {
                // print and send the specified command
                if opts.verbose > 0 {
                    println!("{}", data.italic());
                }

                let v = Vec::from(format!("{}\n", data));
                buf_reader.write_all(&v).await?;

                // receive and print the response
                let mut buffer = String::new();
                buf_reader.read_line(&mut buffer).await?;

                println!("{}", &buffer.bold());
            }
        }

        Subcommands::Image { filename } => {
            let device = hwdevices::get_keyboard_device(&opts.model)?;

            let address = format!(
                "{}:{}",
                opts.hostname
                    .unwrap_or_else(|| constants::DEFAULT_HOST.to_owned()),
                opts.port.unwrap_or(constants::DEFAULT_PORT)
            );
            if opts.verbose > 1 {
                println!("{}", tr!("connecting-to", host = address.to_string()));
            }
            let socket = TcpStream::connect(address).await?;
            let mut buf_reader = BufReader::new(socket);

            if filename.to_string_lossy() == "-" {
                let stdin = io::stdin();
                let mut reader = BufReader::new(stdin);

                loop {
                    let mut buffer = Vec::new();
                    let _len = reader.read_to_end(&mut buffer).await?;

                    let commands = utils::process_image_buffer(&buffer, &device)?;

                    // print and send the specified command
                    if opts.verbose > 0 {
                        println!("{}", tr!("sending-data"));
                    }
                    if opts.verbose > 1 {
                        println!("{}", &commands);
                    }
                    buf_reader.write_all(&Vec::from(commands)).await?;

                    // receive and print the response
                    let mut buffer = String::new();
                    buf_reader.read_line(&mut buffer).await?;

                    println!("{}", buffer.bold());

                    if buffer.starts_with("BYE") || buffer.starts_with("ERROR:") {
                        break;
                    }
                }
            } else {
                let commands = utils::process_image_file(&filename, &device)?;

                // print and send the specified command
                if opts.verbose > 0 {
                    println!("{}", tr!("sending-data"));
                }
                if opts.verbose > 1 {
                    println!("{}", &commands);
                }
                buf_reader.write_all(&Vec::from(commands)).await?;

                // receive and print the response
                let mut buffer = String::new();
                buf_reader.read_line(&mut buffer).await?;

                println!("{}", &buffer.bold());
            }
        }

        Subcommands::Animation {
            directory_name,
            frame_delay,
        } => {
            let address = format!(
                "{}:{}",
                opts.hostname
                    .unwrap_or_else(|| constants::DEFAULT_HOST.to_owned()),
                opts.port.unwrap_or(constants::DEFAULT_PORT)
            );
            if opts.verbose > 1 {
                println!("{}", tr!("connecting-to", host = address.to_string()));
            }
            let socket = TcpStream::connect(address).await?;
            let mut buf_reader = BufReader::new(socket);

            // holds pre-processed command-sequences for each image
            let processed_images = Arc::new(Mutex::new(vec![]));

            if opts.verbose > 0 {
                println!("{}", tr!("processing-image-files"));
            }

            // convert each image file to a command sequence beforehand
            let walkdir = WalkDir::new(&directory_name)
                .follow_links(true)
                .process_read_dir(|_depth, _path, _read_dir_state, children| {
                    children.sort_by(|a, b| match (a, b) {
                        (Ok(a), Ok(b)) => a.file_name.cmp(&b.file_name),
                        (Ok(_), Err(_)) => Ordering::Less,
                        (Err(_), Ok(_)) => Ordering::Greater,
                        (Err(_), Err(_)) => Ordering::Equal,
                    });
                });

            for entry in walkdir {
                if let Ok(filename) = entry {
                    if !filename.path().is_file() {
                        continue;
                    }

                    if opts.verbose > 0 {
                        println!("{}", &filename.path().to_string_lossy());
                    }

                    let model = opts.model.clone();
                    let processed_images = processed_images.clone();

                    rayon::spawn(move || {
                        let device =
                            hwdevices::get_keyboard_device(&model).expect(&tr!("invalid-model"));

                        let _result = utils::process_image_file(&filename.path(), &device)
                            .map_err(|e| {
                                eprintln!("{}", tr!("image-error", message = e.to_string()))
                            })
                            .map(|commands| {
                                processed_images.lock().push(commands);
                            });
                    });
                }
            }

            if opts.verbose > 0 {
                println!("{}", tr!("entering-loop"));
            }

            loop {
                for commands in processed_images.lock().iter() {
                    // print and send the specified command
                    if opts.verbose > 1 {
                        println!("{}", tr!("sending-data"));
                    }
                    if opts.verbose > 2 {
                        println!("{}", &commands);
                    }
                    buf_reader.write_all(&Vec::from(commands.clone())).await?;

                    // receive and print the response
                    let mut buffer = String::new();
                    buf_reader.read_line(&mut buffer).await?;

                    if opts.verbose > 1 {
                        println!("{}", buffer.bold());
                    }

                    if buffer.starts_with("BYE") || buffer.starts_with("ERROR:") {
                        break;
                    }

                    thread::sleep(Duration::from_millis(
                        frame_delay.unwrap_or(constants::DEFAULT_ANIMATION_DELAY_MILLIS),
                    ));
                }
            }
        }

        Subcommands::Ambient { frame_delay } => {
            let address = format!(
                "{}:{}",
                opts.hostname
                    .unwrap_or_else(|| constants::DEFAULT_HOST.to_owned()),
                opts.port.unwrap_or(constants::DEFAULT_PORT)
            );
            if opts.verbose > 1 {
                println!("{}", tr!("connecting-to", host = address.to_string()));
            }
            let socket = TcpStream::connect(address).await?;
            let mut buf_reader = BufReader::new(socket);

            // register all available screenshot backends
            backends::register_backends()?;

            // instantiate the best fitting backend for the current system configuration
            let mut backend = backends::get_best_fitting_backend()?;

            loop {
                // request a screenshot encoded as Network FX protocol commands from the backend
                let commands = backend.poll()?;

                // print and send the commands
                if opts.verbose > 0 {
                    println!("{}", tr!("sending-data"));
                }
                if opts.verbose > 1 {
                    println!("{}", &commands);
                }
                buf_reader.write_all(&Vec::from(commands)).await?;

                // receive and print the response
                let mut buffer = String::new();
                buf_reader.read_line(&mut buffer).await?;

                if buffer.starts_with("BYE") || buffer.starts_with("ERROR:") {
                    break;
                }

                thread::sleep(Duration::from_millis(
                    frame_delay.unwrap_or(constants::DEFAULT_FRAME_DELAY_MILLIS),
                ));
            }
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
