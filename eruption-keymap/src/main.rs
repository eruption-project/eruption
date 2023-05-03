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
use color_eyre::owo_colors::OwoColorize;
use colored::*;
use comfy_table::{
    modifiers::UTF8_ROUND_CORNERS, presets::UTF8_FULL, Cell, ContentArrangement, Table,
};
use evdev_rs::enums::{EventCode, EV_KEY};
use flume::unbounded;
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};
use lazy_static::lazy_static;
use log::*;
use parking_lot::Mutex;
use rust_embed::RustEmbed;
use std::{
    collections::BTreeMap,
    env,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use walkdir::WalkDir;

use crate::{
    backends::Backend,
    backends::{lua::LuaBackend, native::NativeBackend},
    lua_introspection::LuaSyntaxIntrospection,
    mapping::{KeyMappingTable, Rule},
};

// mod assistants;
mod backends;
mod constants;
mod dbus_client;
mod device;
mod hwdevices;
mod lua_introspection;
mod mapping;
mod messages;
mod parsers;
mod util;

#[allow(unused)]
type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(RustEmbed)]
#[folder = "i18n"] // path to the compiled localization resources
struct Localizations;

lazy_static! {
    /// Global configuration
    pub static ref STATIC_LOADER: Arc<Mutex<Option<FluentLanguageLoader>>> = Arc::new(Mutex::new(None));
}

#[allow(unused)]
#[macro_export]
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

lazy_static! {
    /// Global "quit" status flag
    pub static ref QUIT: AtomicBool = AtomicBool::new(false);
}

#[derive(Debug, thiserror::Error)]
pub enum MainError {
    #[error("Unknown error: {description}")]
    UnknownError { description: String },
}

lazy_static! {
    static ref ABOUT: String = tr!("about");
    static ref VERBOSE_ABOUT: String = tr!("verbose-about");
    static ref COMPLETIONS_ABOUT: String = tr!("completions-about");
    static ref ASSISTANT_ABOUT: String = tr!("assistant-about");
    static ref LIST_ABOUT: String = tr!("list-about");
    static ref MAPPING_ABOUT: String = tr!("mapping-about");
    static ref DESCRIPTION_ABOUT: String = tr!("description-about");
    static ref SHOW_ABOUT: String = tr!("show-about");
    static ref MACROS_ABOUT: String = tr!("macros-about");
    static ref EVENTS_ABOUT: String = tr!("events-about");
    static ref COMPILE_ABOUT: String = tr!("compile-about");
    static ref MAPPING_ADD_ABOUT: String = tr!("mapping-add-about");
    static ref MAPPING_REMOVE_ABOUT: String = tr!("mapping-remove-about");
    static ref MAPPING_ENABLE_ABOUT: String = tr!("mapping-enable-about");
    static ref MAPPING_DISABLE_ABOUT: String = tr!("mapping-disable-about");
}

/// Supported command line arguments
#[derive(Debug, clap::Parser)]
#[clap(
    version = env!("CARGO_PKG_VERSION"),
    author = "X3n0m0rph59 <x3n0m0rph59@gmail.com>",
    about = ABOUT.as_str(),
)]
pub struct Options {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(help(VERBOSE_ABOUT.as_str()), short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    #[clap(subcommand)]
    command: Subcommands,
}

// Sub-commands
#[derive(Debug, clap::Parser)]
pub enum Subcommands {
    /// Run an assistant that guides you through creating one or more key mappings
    //#[clap(about(ASSISTANT_ABOUT.as_str()))]
    //Assistant { keymap: PathBuf },

    /// List all available keymaps
    #[clap(about(LIST_ABOUT.as_str()))]
    List,

    /// Add or remove a single mapping entry
    #[clap(about(MAPPING_ABOUT.as_str()))]
    Mapping {
        #[clap(subcommand)]
        command: MappingSubcommands,
    },

    /// Show or set the description of the specified keymap
    #[clap(about(DESCRIPTION_ABOUT.as_str()))]
    Description {
        #[clap(required = false, short, long, default_value = "default.keymap")]
        keymap: PathBuf,
        description: Option<String>,
    },

    /// Show some information about a keymap
    #[clap(about(SHOW_ABOUT.as_str()))]
    Show {
        #[clap(required = false, short, long, default_value = "default.keymap")]
        keymap: PathBuf,
    },

    /// Show a list of available macros in a Lua file
    #[clap(about(MACROS_ABOUT.as_str()))]
    Macros {
        #[clap(required = false, short, long, default_value = "user-macros.lua")]
        lua_path: PathBuf,
    },

    /// Show a list of available Linux EVDEV events
    #[clap(about(EVENTS_ABOUT.as_str()))]
    Events,

    /// Compile a keymap to Lua code and make it available to Eruption
    #[clap(about(COMPILE_ABOUT.as_str()))]
    Compile {
        #[clap(required = false, short, long, default_value = "default.keymap")]
        keymap: PathBuf,
    },

    /// Generate shell completions
    #[clap(hide = true, about(COMPLETIONS_ABOUT.as_str()))]
    Completions {
        // #[clap(subcommand)]
        shell: Shell,
    },
}

/// Subcommands of the "mapping" command
#[derive(Debug, clap::Parser)]
pub enum MappingSubcommands {
    /// Add a mapping rule for `source` that executes `action`
    #[clap(about(MAPPING_ADD_ABOUT.as_str()))]
    Add {
        /// Specify the device to add the rule for
        #[clap(required = false, short, long, default_value = "0")]
        device: String,

        /// Specify the enabled status of the newly added rule
        #[clap(required = false, short, long, default_value = "true")]
        enabled: bool,

        /// Specify a description for a rule
        #[clap(required = false, long, default_value = "")]
        description: String,

        /// Specify a list of layers
        #[clap(required = false, short, long)]
        layers: Vec<usize>,

        /// The filename of the keymap
        #[clap(required = false, short, long, default_value = "default.keymap")]
        keymap: PathBuf,

        source: String,
        action: String,
    },

    /// Remove the mapping rule for `source`
    #[clap(about(MAPPING_REMOVE_ABOUT.as_str()))]
    Remove {
        /// Specify the device to remove the mapping from
        #[clap(required = false, short, long, default_value = "0")]
        device: String,

        /// Specify a list of layers
        //#[clap(required = false, short, long)]
        //layers: Vec<usize>,

        #[clap(required = false, short, long, default_value = "default.keymap")]
        keymap: PathBuf,

        index: usize,
    },

    /// Enable a key mapping rule
    #[clap(about(MAPPING_ENABLE_ABOUT.as_str()))]
    Enable {
        /// Specify the device
        #[clap(required = false, short, long, default_value = "0")]
        device: String,

        /// Specify a list of layers
        //#[clap(required = false, short, long)]
        //layers: Vec<usize>,

        #[clap(required = false, short, long, default_value = "default.keymap")]
        keymap: PathBuf,

        index: usize,
    },

    /// Disable a key mapping rule
    #[clap(about(MAPPING_DISABLE_ABOUT.as_str()))]
    Disable {
        /// Specify the device
        #[clap(required = false, short, long, default_value = "0")]
        device: String,

        /// Specify a list of layers
        //#[clap(required = false, short, long)]
        //layers: Vec<usize>,

        #[clap(required = false, short, long, default_value = "default.keymap")]
        keymap: PathBuf,

        index: usize,
    },
}

/// Subcommands of the "completions" command
#[derive(Debug, clap::Parser)]
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
    println!("{}", tr!("license-header"));
    println!();
}

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

    // start the thread deadlock detector
    #[cfg(debug_assertions)]
    thread_util::deadlock_detector()
        .unwrap_or_else(|e| error!("Could not spawn deadlock detector thread: {}", e));

    // register ctrl-c handler
    let (ctrl_c_tx, _ctrl_c_rx) = unbounded();
    ctrlc::set_handler(move || {
        QUIT.store(true, Ordering::SeqCst);

        ctrl_c_tx
            .send(true)
            .unwrap_or_else(|e| error!("Could not send on a channel: {}", e));
    })
    .unwrap_or_else(|e| error!("Could not set CTRL-C handler: {}", e));

    let opts = Options::parse();
    match opts.command {
        /* Subcommands::Assistant { keymap: _ } => {
            let mut assistants = assistants::register_assistants();

            println!(
                "{} - {}",
                "Eruption keymap utility".bold(),
                "Please choose an assistant to proceed".bold()
            );

            println!();

            for (index, assistant) in assistants.iter_mut().enumerate() {
                println!(
                    "{index}.\t{}\n\tDescription:\n\t{}\n",
                    assistant.title().bold(),
                    assistant.description().bold(),
                    index = index + 1,
                );
            }

            println!();
        } */
        Subcommands::List {} => {
            let path = Path::new(constants::DEFAULT_KEYMAP_DIR);

            for entry in WalkDir::new(path).follow_links(true).sort_by_file_name() {
                let entry = entry?;

                // skip directories and the README file
                if !entry.path().is_file() || entry.path().ends_with("README") {
                    continue;
                }

                if entry
                    .path()
                    .extension()
                    .iter()
                    .any(|e| e.eq_ignore_ascii_case("keymap"))
                {
                    let table = NativeBackend::from_file(entry.path())?;
                    println!("{}: {}", entry.path().display(), table.description().bold());
                }
            }
        }

        Subcommands::Mapping { command } => match command {
            MappingSubcommands::Add {
                device,
                description,
                enabled,
                layers,
                keymap,
                source,
                action,
            } => {
                let device = device.parse::<usize>()?;

                let path = if keymap.components().count() > 1 {
                    keymap
                } else {
                    PathBuf::from(constants::DEFAULT_KEYMAP_DIR).join(keymap)
                };

                let mut table = if path.exists() {
                    NativeBackend::from_file(&path)?
                } else {
                    KeyMappingTable::new()
                };

                let mut source = parsers::source::parse(&source, device)?;
                if !layers.is_empty() {
                    source.get_layers_mut().clear();
                    source.get_layers_mut().extend(&layers);
                }

                let action = parsers::action::parse(&action)?;

                let rule = Rule::new(action, &description, enabled);

                table.insert(source, rule);

                NativeBackend::new().write_to_file(&path, &table)?
            }

            MappingSubcommands::Remove {
                device: _,
                //layers,
                keymap,
                index,
            } => {
                let path = if keymap.components().count() > 1 {
                    keymap
                } else {
                    PathBuf::from(constants::DEFAULT_KEYMAP_DIR).join(keymap)
                };

                let mut table = NativeBackend::from_file(&path)?;

                // copy the data structure, except the item that is to be removed
                let mut mappings = BTreeMap::new();
                for (idx, (source, rule)) in table.mappings().iter().enumerate() {
                    if idx == index - 1 {
                        continue;
                    }

                    mappings.insert(source.clone(), rule.clone());
                }

                table.mappings = mappings;

                NativeBackend::new().write_to_file(&path, &table)?
            }

            MappingSubcommands::Enable {
                device: _,
                //layers,
                keymap,
                index,
            } => {
                let path = if keymap.components().count() > 1 {
                    keymap
                } else {
                    PathBuf::from(constants::DEFAULT_KEYMAP_DIR).join(keymap)
                };

                let mut table = NativeBackend::from_file(&path)?;

                table
                    .mappings_mut()
                    .iter_mut()
                    .nth(index - 1)
                    .expect("Index out of bounds")
                    .1
                    .enabled = true;

                NativeBackend::new().write_to_file(&path, &table)?
            }

            MappingSubcommands::Disable {
                device: _,
                //layers,
                keymap,
                index,
            } => {
                let path = if keymap.components().count() > 1 {
                    keymap
                } else {
                    PathBuf::from(constants::DEFAULT_KEYMAP_DIR).join(keymap)
                };

                let mut table = NativeBackend::from_file(&path)?;

                table
                    .mappings_mut()
                    .iter_mut()
                    .nth(index - 1)
                    .expect("Index out of bounds")
                    .1
                    .enabled = false;

                NativeBackend::new().write_to_file(&path, &table)?
            }
        },

        Subcommands::Description {
            keymap,
            description,
        } => {
            let path = if keymap.components().count() > 1 {
                keymap
            } else {
                PathBuf::from(constants::DEFAULT_KEYMAP_DIR).join(keymap)
            };

            if let Some(description) = description {
                let mut table = NativeBackend::from_file(&path)?;

                table.set_description(&description);

                NativeBackend::new().write_to_file(&path, &table)?;
            } else {
                let table = NativeBackend::from_file(&path)?;

                println!("{}", table.description().bold());
            }
        }

        Subcommands::Show { keymap } => {
            let path = if keymap.components().count() > 1 {
                keymap
            } else {
                PathBuf::from(constants::DEFAULT_KEYMAP_DIR).join(keymap)
            };

            let table = NativeBackend::from_file(&path)?;

            println!("File: {}", &path.display().bold());
            println!("Description: {}", table.description().bold());

            let mut tab = Table::new();
            tab.load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_content_arrangement(ContentArrangement::Dynamic)
                // .set_width(40)
                .set_header(vec![
                    "#",
                    &tr!("source"),
                    &tr!("action"),
                    &tr!("description"),
                    &tr!("flags"),
                ]);

            for (index, (source, action)) in table.mappings().iter().enumerate() {
                let description = if action.description.trim().is_empty() {
                    Cell::new(format!("{}", tr!("n-a").italic()))
                } else {
                    Cell::new(action.description.to_string())
                };

                let enabled = if action.enabled {
                    Cell::new(tr!("enabled").to_string())
                } else {
                    Cell::new(tr!("disabled").to_string())
                };

                tab.add_row(vec![
                    Cell::new(format!("{}", index + 1)),
                    Cell::new(format!("{}", source)),
                    Cell::new(format!("{}", action)),
                    description,
                    enabled,
                ]);
            }

            println!("{}", tab);
        }

        Subcommands::Compile { keymap } => {
            let mut path = if keymap.components().count() > 1 {
                keymap
            } else {
                PathBuf::from(constants::DEFAULT_KEYMAP_DIR).join(keymap)
            };

            println!("{} {}", tr!("compiling"), &path.display().bold());

            let table = NativeBackend::from_file(&path)?;
            path.set_extension("lua");

            LuaBackend::new().write_to_file(&path, &table)?;

            println!("{}", tr!("success"));
        }

        Subcommands::Macros { lua_path } => {
            let path = if lua_path.components().count() > 1 {
                lua_path
            } else {
                PathBuf::from(constants::DEFAULT_SCRIPT_DIR)
                    .join("lib/macros/")
                    .join(lua_path)
            };

            println!("{} {}\n", tr!("functions-in-file"), &path.display().bold());

            let lua_file = LuaSyntaxIntrospection::new_from_file(&path)?;

            for function in lua_file.functions() {
                println!("{}", function.name());
            }
        }

        Subcommands::Events => {
            let event = EventCode::EV_KEY(EV_KEY::KEY_RESERVED);

            let mut tab = Table::new();
            tab.load_preset(UTF8_FULL)
                .apply_modifier(UTF8_ROUND_CORNERS)
                .set_content_arrangement(ContentArrangement::Dynamic)
                // .set_width(40)
                .set_header(vec![&tr!("symbol"), &tr!("code")]);

            for (index, code) in event.iter().enumerate() {
                tab.add_row(vec![
                    Cell::new(format!("{}", index + 1)),
                    Cell::new(format!("{}", code)),
                    Cell::new(format!("{}", util::evdev_key_event_to_int(code))),
                ]);
            }

            println!("{}", tab);
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

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("worker")
        .enable_all()
        // .worker_threads(4)
        .build()?;

    runtime.block_on(async move { async_main().await })
}
