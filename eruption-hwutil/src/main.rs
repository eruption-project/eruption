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

use clap::{lazy_static::lazy_static, Clap};
// use colored::*;
use parking_lot::Mutex;
use std::{process, sync::Arc};

mod constants;
mod device;
mod util;

// type Result<T> = std::result::Result<T, eyre::Error>;

lazy_static! {
    /// Global configuration
    pub static ref CONFIG: Arc<Mutex<Option<config::Config>>> = Arc::new(Mutex::new(None));
}

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
    about = "A CLI control utility for hardware supported by the Eruption Linux user-mode driver",
)]
pub struct Options {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, parse(from_occurrences))]
    verbose: u8,

    /// Sets the configuration file to use
    #[clap(short = 'c', long)]
    config: Option<String>,

    #[clap(subcommand)]
    command: Subcommands,
}

// Sub-commands
#[derive(Debug, Clap)]
pub enum Subcommands {
    /// Firmware update related subcommands
    FirmwareUpdate {
        #[clap(subcommand)]
        command: FirmwareUpdateSubcommands,
    },

    /// Generate shell completions
    Completions {
        #[clap(subcommand)]
        command: CompletionsSubcommands,
    },
}

/// Subcommands of the "firmware-update" command
#[derive(Debug, Clap)]
pub enum FirmwareUpdateSubcommands {
    /// Get some information about the currently installed firmware
    Info { device: u64 },

    /// Flash firmware
    Flash { device: u64 },
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

#[tokio::main(flavor = "multi_thread")]
pub async fn main() -> std::result::Result<(), eyre::Error> {
    color_eyre::install()?;

    // if unsafe { libc::isatty(0) != 0 } {
    //     print_header();
    // }

    let opts = Options::parse();

    // process configuration file
    let config_file = opts
        .config
        .unwrap_or(constants::DEFAULT_CONFIG_FILE.to_string());

    let mut config = config::Config::default();
    config
        .merge(config::File::new(&config_file, config::FileFormat::Toml))
        .unwrap_or_else(|e| {
            log::error!("Could not parse configuration file: {}", e);
            process::exit(4);
        });

    *CONFIG.lock() = Some(config);

    match opts.command {
        // device specific sub-commands
        //         Subcommands::Devices { command } => match command {
        //             DevicesSubcommands::List => {
        //                 let mut base_index = 0;

        //                 let (keyboards, mice, misc) = get_devices().await?;

        //                 if opts.verbose > 0 {
        //                     println!(
        //                         "{}",
        //                         r#"
        //  Use the `eruption-hwutil devices list` sub-command to find out the index of the device that
        //  you want to operate on. All the other device-related commands require a device index.

        //  Examples:

        //  Set the brightness of the first connected keyboard to 80 percent:

        //     $ eruption-hwutil devices brightness 0 80

        //  Query the DPI configuration of the first connected mouse (second device):

        //     $ eruption-hwutil devices dpi 1

        // "#
        //                     );
        //                 }

        //                 println!("{}", "Dumping Eruption managed devices list\n".bold());

        //                 println!("Keyboard devices:");

        //                 if keyboards.is_empty() {
        //                     println!("{}", "<No supported devices detected>\n".italic());
        //                 } else {
        //                     for (_index, dev) in keyboards.iter().enumerate() {
        //                         println!(
        //                             "Index: {}: ID: {}:{} {} {}",
        //                             format!("{:02}", base_index).bold(),
        //                             format!("{:04x}", dev.0),
        //                             format!("{:04x}", dev.1),
        //                             device::get_device_make(dev.0, dev.1)
        //                                 .unwrap_or("<unknown make>")
        //                                 .bold(),
        //                             device::get_device_model(dev.0, dev.1)
        //                                 .unwrap_or("<unknown model>")
        //                                 .bold()
        //                         );

        //                         base_index += 1;
        //                     }
        //                 }

        //                 println!("\nMouse devices:");

        //                 if mice.is_empty() {
        //                     println!("{}", "<No supported devices detected>\n".italic());
        //                 } else {
        //                     for (_index, dev) in mice.iter().enumerate() {
        //                         println!(
        //                             "Index: {}: ID: {}:{} {} {}",
        //                             format!("{:02}", base_index).bold(),
        //                             format!("{:04x}", dev.0),
        //                             format!("{:04x}", dev.1),
        //                             device::get_device_make(dev.0, dev.1)
        //                                 .unwrap_or("<unknown make>")
        //                                 .bold(),
        //                             device::get_device_model(dev.0, dev.1)
        //                                 .unwrap_or("<unknown model>")
        //                                 .bold()
        //                         );

        //                         base_index += 1;
        //                     }
        //                 }

        //                 println!("\nMiscellaneous devices:");

        //                 if misc.is_empty() {
        //                     println!("{}", "<No supported devices detected>\n".italic());
        //                 } else {
        //                     for (_index, dev) in misc.iter().enumerate() {
        //                         println!(
        //                             "Index: {}: ID: {}:{} {} {}",
        //                             format!("{:02}", base_index).bold(),
        //                             format!("{:04x}", dev.0),
        //                             format!("{:04x}", dev.1),
        //                             device::get_device_make(dev.0, dev.1)
        //                                 .unwrap_or("<unknown make>")
        //                                 .bold(),
        //                             device::get_device_model(dev.0, dev.1)
        //                                 .unwrap_or("<unknown model>")
        //                                 .bold()
        //                         );

        //                         base_index += 1;
        //                     }
        //                 }
        //             }

        //             DevicesSubcommands::Info { device } => {
        //                 let device = device.parse::<u64>()?;

        //                 print_device_header(device).await?;

        //                 let result = get_device_config(device, "info").await?;
        //                 println!("{}", format!("{}", result.bold()));
        //             }

        //             DevicesSubcommands::Profile { device, profile } => {
        //                 let device = device.parse::<u64>()?;

        //                 print_device_header(device).await?;

        //                 if let Some(profile) = profile {
        //                     let value = &format!("{}", profile);

        //                     set_device_config(device, "profile", value).await?
        //                 } else {
        //                     let result = get_device_config(device, "profile").await?;

        //                     println!("{}", format!("Current profile: {}", result.bold()));
        //                 }
        //             }

        //             DevicesSubcommands::Dpi { device, dpi } => {
        //                 let device = device.parse::<u64>()?;

        //                 print_device_header(device).await?;

        //                 if let Some(dpi) = dpi {
        //                     let value = &format!("{}", dpi);

        //                     set_device_config(device, "dpi", value).await?
        //                 } else {
        //                     let result = get_device_config(device, "dpi").await?;

        //                     println!("{}", format!("DPI config: {}", result.bold()));
        //                 }
        //             }

        //             DevicesSubcommands::Rate { device, rate } => {
        //                 let device = device.parse::<u64>()?;

        //                 print_device_header(device).await?;

        //                 if let Some(rate) = rate {
        //                     let value = &format!("{}", rate);

        //                     set_device_config(device, "rate", value).await?
        //                 } else {
        //                     let result = get_device_config(device, "rate").await?;

        //                     println!("{}", format!("Poll rate: {}", result.bold()));
        //                 }
        //             }

        //             DevicesSubcommands::Distance { device, param } => {
        //                 let device = device.parse::<u64>()?;

        //                 print_device_header(device).await?;

        //                 if let Some(param) = param {
        //                     let value = &format!("{}", param);

        //                     set_device_config(device, "dcu", value).await?
        //                 } else {
        //                     let result = get_device_config(device, "dcu").await?;

        //                     println!("{}", format!("DCU config: {}", result.bold()));
        //                 }
        //             }

        //             DevicesSubcommands::AngleSnapping { device, enable } => {
        //                 let device = device.parse::<u64>()?;

        //                 print_device_header(device).await?;

        //                 if let Some(enable) = enable {
        //                     let value = &format!("{}", enable);

        //                     set_device_config(device, "angle-snapping", value).await?
        //                 } else {
        //                     let result = get_device_config(device, "angle-snapping").await?;

        //                     println!("{}", format!("Angle-snapping: {}", result.bold()));
        //                 }
        //             }

        //             DevicesSubcommands::Debounce { device, enable } => {
        //                 let device = device.parse::<u64>()?;

        //                 print_device_header(device).await?;

        //                 if let Some(enable) = enable {
        //                     let value = &format!("{}", enable);

        //                     set_device_config(device, "debounce", value).await?
        //                 } else {
        //                     let result = get_device_config(device, "debounce").await?;

        //                     println!("{}", format!("Debounce: {}", result.bold()));
        //                 }
        //             }

        //             DevicesSubcommands::Brightness { device, brightness } => {
        //                 let device = device.parse::<u64>()?;

        //                 print_device_header(device).await?;

        //                 if let Some(brightness) = brightness {
        //                     let value = &format!("{}", brightness);

        //                     set_device_config(device, "brightness", value).await?
        //                 } else {
        //                     let result = get_device_config(device, "brightness").await?;

        //                     println!("{}", format!("Device brightness: {}%", result.bold()));
        //                 }
        //             }
        //         },
        Subcommands::FirmwareUpdate { command: _ } => {}

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
