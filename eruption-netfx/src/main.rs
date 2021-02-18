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

use clap::Clap;
use colored::Colorize;
use std::path::PathBuf;
use std::{env, thread};
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt};
use tokio::io::{AsyncReadExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::Duration;
use walkdir::WalkDir;

mod constants;
mod hwdevices;
mod utils;
mod xwrap;

// type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum MainError {
    #[error("Unknown error: {description}")]
    UnknownError { description: String },
}

/// Supported command line arguments
#[derive(Debug, Clap)]
#[clap(
    version = env!("CARGO_PKG_VERSION"),
    author = "X3n0m0rph59 <x3n0m0rph59@gmail.com>",
    about = "A Network FX protocol client for the Eruption Linux user-mode driver",
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
#[derive(Debug, Clap)]
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
        #[clap(subcommand)]
        command: CompletionsSubcommands,
    },
}

/// Subcommands of the "completions" command
#[derive(Debug, Clap)]
pub enum CompletionsSubcommands {
    Bash,

    Elvish,

    Fish,

    PowerShell,

    Zsh,
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

#[tokio::main]
pub async fn main() -> std::result::Result<(), eyre::Error> {
    color_eyre::install()?;

    // if unsafe { libc::isatty(0) != 0 } {
    //     print_header();
    // }

    // initialize logging
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG_OVERRIDE", "info");
        pretty_env_logger::init_custom_env("RUST_LOG_OVERRIDE");
    } else {
        pretty_env_logger::init();
    }

    let opts = Options::parse();
    match opts.command {
        Subcommands::Ping => {
            let address = format!(
                "{}:{}",
                opts.hostname
                    .unwrap_or_else(|| constants::DEFAULT_HOST.to_owned()),
                opts.port.unwrap_or(constants::DEFAULT_PORT)
            );
            if opts.verbose > 1 {
                println!("Connecting to: {}", address);
            }

            let socket = TcpStream::connect(address).await?;
            let mut buf_reader = BufReader::new(socket);

            // print and send the specified command
            if opts.verbose > 0 {
                println!("Sending STATUS inquiry...");
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
                println!("Connecting to: {}", address);
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
                println!("Connecting to: {}", address);
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
                        println!("Sending data...");
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
                    println!("Sending data...");
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
            let device = hwdevices::get_keyboard_device(&opts.model)?;

            let address = format!(
                "{}:{}",
                opts.hostname
                    .unwrap_or_else(|| constants::DEFAULT_HOST.to_owned()),
                opts.port.unwrap_or(constants::DEFAULT_PORT)
            );
            if opts.verbose > 1 {
                println!("Connecting to: {}", address);
            }
            let socket = TcpStream::connect(address).await?;
            let mut buf_reader = BufReader::new(socket);

            // holds pre-processed command-sequences for each image
            let mut processed_images = vec![];

            if opts.verbose > 0 {
                println!("Pre-processing image files...");
            }

            // convert each image file to a command sequence beforehand
            for filename in WalkDir::new(&directory_name)
                .follow_links(true)
                .into_iter()
                .filter_map(|e| e.ok())
            {
                if !filename.path().is_file() {
                    continue;
                }

                if opts.verbose > 0 {
                    println!("{}", &filename.path().to_string_lossy());
                }

                let commands = utils::process_image_file(&filename.path(), &device)?;

                processed_images.push(commands);
            }

            if opts.verbose > 0 {
                println!("Entering loop...");
            }

            loop {
                for commands in processed_images.iter() {
                    // print and send the specified command
                    if opts.verbose > 1 {
                        println!("Sending data...");
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
            let device = hwdevices::get_keyboard_device(&opts.model)?;

            let address = format!(
                "{}:{}",
                opts.hostname
                    .unwrap_or_else(|| constants::DEFAULT_HOST.to_owned()),
                opts.port.unwrap_or(constants::DEFAULT_PORT)
            );
            if opts.verbose > 1 {
                println!("Connecting to: {}", address);
            }
            let socket = TcpStream::connect(address).await?;
            let mut buf_reader = BufReader::new(socket);

            let display = xwrap::Display::open(None).unwrap();
            let window = display.get_default_root();
            let window_rect = display.get_window_rect(window);
            let sel = xwrap::Rect {
                x: 0,
                y: 0,
                w: window_rect.w,
                h: window_rect.h,
            };

            loop {
                let image = display
                    .get_image(window, sel, xwrap::ALL_PLANES, x11::xlib::ZPixmap)
                    .unwrap();
                let commands = utils::process_screenshot(&image, &device)?;

                // print and send the specified commands
                if opts.verbose > 0 {
                    println!("Sending data...");
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

        Subcommands::Completions { command } => {
            use clap::IntoApp;
            use clap_generate::{generate, generators::*};

            const BIN_NAME: &str = env!("CARGO_PKG_NAME");

            let mut app = Options::into_app();
            let mut fd = std::io::stdout();

            match command {
                CompletionsSubcommands::Bash => {
                    generate::<Bash, _>(&mut app, BIN_NAME, &mut fd);
                }

                CompletionsSubcommands::Elvish => {
                    generate::<Elvish, _>(&mut app, BIN_NAME, &mut fd);
                }

                CompletionsSubcommands::Fish => {
                    generate::<Fish, _>(&mut app, BIN_NAME, &mut fd);
                }

                CompletionsSubcommands::PowerShell => {
                    generate::<PowerShell, _>(&mut app, BIN_NAME, &mut fd);
                }

                CompletionsSubcommands::Zsh => {
                    generate::<Zsh, _>(&mut app, BIN_NAME, &mut fd);
                }
            }
        }
    };

    Ok(())
}
