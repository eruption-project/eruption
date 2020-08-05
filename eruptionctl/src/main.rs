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
use colored::*;
use dbus::nonblock;
use dbus::nonblock::stdintf::org_freedesktop_dbus::Properties;
use dbus_tokio::connection;
use failure::Fail;
use std::sync::Arc;
use std::time::Duration;

type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug, Fail)]
pub enum MainError {
    #[fail(display = "Unknown error: {}", description)]
    UnknownError { description: String },
}

/// Supported command line arguments
#[derive(Debug, Clap)]
#[clap(
    version = env!("CARGO_PKG_VERSION"),
    author = "X3n0m0rph59 <x3n0m0rph59@gmail.com>",
    about = "A CLI control utility for the Eruption Linux user-mode driver",
)]
pub struct Options {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long, parse(from_occurrences))]
    verbose: u8,

    #[clap(subcommand)]
    command: Subcommands,
}

// Subcommands
#[derive(Debug, Clap)]
pub enum Subcommands {
    /// Configuration related subcommands
    Config {
        #[clap(subcommand)]
        command: ConfigSubcommands,
    },

    /// Switch to a different profile or slot
    Switch {
        #[clap(subcommand)]
        command: SwitchSubcommands,
    },

    /// Profile related subcommands
    Profiles {
        #[clap(subcommand)]
        command: ProfilesSubcommands,
    },

    /// Script related subcommands
    Scripts {
        #[clap(subcommand)]
        command: ScriptsSubcommands,
    },
}

/// Subcommands of the "config" command
#[derive(Debug, Clap)]
pub enum ConfigSubcommands {
    /// Get or set the brightness of the LEDs
    Brightness { brightness: Option<i64> },

    /// Get or set the state of SoundFX
    Soundfx { enable: Option<bool> },
}

/// Subcommands of the "switch" command
#[derive(Debug, Clap)]
pub enum SwitchSubcommands {
    /// Switch profiles
    Profile { profile_name: String },

    /// Switch slots
    Slot { index: usize },
}

/// Subcommands of the "profiles" command
#[derive(Debug, Clap)]
pub enum ProfilesSubcommands {
    /// Show info about a profile
    Info { profile_name: String },

    /// Edit a profile
    Edit { profile_name: String },

    /// List available profiles
    List,
}

/// Subcommands of the "scripts" command
#[derive(Debug, Clap)]
pub enum ScriptsSubcommands {
    /// Show info about a script
    Info { script_name: String },

    /// Edit a script
    Edit { script_name: String },

    /// List available scripts
    List,
}

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

pub async fn dbus_system_bus(
    path: &str,
) -> Result<dbus::nonblock::Proxy<'_, Arc<dbus::nonblock::SyncConnection>>> {
    let (resource, conn) = connection::new_system_sync()?;

    tokio::spawn(async {
        let err = resource.await;
        panic!("Lost connection to D-Bus: {}", err);
    });

    let proxy = nonblock::Proxy::new("org.eruption", path, Duration::from_secs(4), conn);

    Ok(proxy)
}

pub async fn switch_profile(name: &str) -> Result<()> {
    let file_name = name.to_owned();

    let (_result,): (bool,) = dbus_system_bus("/org/eruption/profile")
        .await?
        .method_call("org.eruption.Profile", "SwitchProfile", (file_name,))
        .await?;

    Ok(())
}

pub async fn switch_slot(index: usize) -> Result<()> {
    let (_result,): (bool,) = dbus_system_bus("/org/eruption/slot")
        .await?
        .method_call("org.eruption.Slot", "SwitchSlot", (index as u64,))
        .await?;

    Ok(())
}

pub async fn get_profiles() -> Result<Vec<(String, String)>> {
    let (result,): (Vec<(String, String)>,) = dbus_system_bus("/org/eruption/profile")
        .await?
        .method_call("org.eruption.Profile", "EnumProfiles", ())
        .await?;

    Ok(result)
}

pub async fn get_scripts() -> Result<Vec<(String, String)>> {
    Ok(vec![("Not implemented".into(), "None".into())])
}

pub async fn get_brightness() -> Result<i64> {
    let result = dbus_system_bus("/org/eruption/config")
        .await?
        .get("org.eruption.Config", "Brightness")
        .await?;

    Ok(result)
}

pub async fn set_brightness(brightness: i64) -> Result<()> {
    let arg = Box::new(brightness as i64);

    let _result = dbus_system_bus("/org/eruption/config")
        .await?
        .set("org.eruption.Config", "Brightness", arg)
        .await?;

    Ok(())
}

pub async fn get_sound_fx() -> Result<bool> {
    let result = dbus_system_bus("/org/eruption/config")
        .await?
        .get("org.eruption.Config", "EnableSfx")
        .await?;

    Ok(result)
}

pub async fn set_sound_fx(enabled: bool) -> Result<()> {
    let arg = Box::new(enabled);

    let _result = dbus_system_bus("/org/eruption/config")
        .await?
        .set("org.eruption.Config", "EnableSfx", arg)
        .await?;

    Ok(())
}

#[tokio::main]
pub async fn main() -> std::result::Result<(), failure::Error> {
    // if unsafe { libc::isatty(0) != 0 } {
    //     print_header();
    // }

    let opts = Options::parse();
    match opts.command {
        Subcommands::Config { command } => match command {
            ConfigSubcommands::Brightness { brightness } => {
                if let Some(brightness) = brightness {
                    set_brightness(brightness).await?
                } else {
                    let result = get_brightness().await?;
                    println!(
                        "{}",
                        format!("Brightness: {}", format!("{}%", result).bold())
                    );
                }
            }

            ConfigSubcommands::Soundfx { enable } => {
                if let Some(enable) = enable {
                    set_sound_fx(enable).await?
                } else {
                    let result = get_sound_fx().await?;
                    println!(
                        "{}",
                        format!("SoundFX enabled: {}", format!("{}", result).bold())
                    );
                }
            }
        },

        Subcommands::Profiles { command } => match command {
            ProfilesSubcommands::Edit { profile_name: _ } => {}

            ProfilesSubcommands::List => {
                for p in get_profiles().await? {
                    println!("{}: {}", p.0.bold(), p.1);
                }
            }

            ProfilesSubcommands::Info { profile_name: _ } => {}
        },

        Subcommands::Scripts { command } => match command {
            ScriptsSubcommands::Edit { script_name: _ } => {}

            ScriptsSubcommands::List => {
                for p in get_scripts().await? {
                    println!("{}: {}", p.0.bold(), p.1);
                }
            }

            ScriptsSubcommands::Info { script_name: _ } => {}
        },

        Subcommands::Switch { command } => match command {
            SwitchSubcommands::Profile { profile_name } => {
                println!("Switching to profile: {}", profile_name.bold());
                switch_profile(&profile_name).await?
            }

            SwitchSubcommands::Slot { index } => {
                println!("Switching to slot: {}", format!("{}", index).bold());
                let index = index - 1;
                switch_slot(index).await?
            }
        },
    };

    Ok(())
}
