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
use color_eyre::owo_colors::OwoColorize;
use colored::*;
use flume::unbounded;
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};
use lazy_static::lazy_static;
use log::*;
use parking_lot::Mutex;
use prettytable::{cell, row, Cell, Row, Table};
use rust_embed::RustEmbed;
use std::{
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
    mapping::KeyMappingTable,
};

// mod assistants;
mod backends;
mod constants;
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
    static ref COMPILE_ABOUT: String = tr!("compile-about");
    static ref MAPPING_ADD_ABOUT: String = tr!("mapping-add-about");
    static ref MAPPING_REMOVE_ABOUT: String = tr!("mapping-remove-about");
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
    #[clap(help(VERBOSE_ABOUT.as_str()), short, long, parse(from_occurrences))]
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
        keymap: PathBuf,
        description: Option<String>,
    },

    /// Show some information about a keymap
    #[clap(about(SHOW_ABOUT.as_str()))]
    Show { keymap: PathBuf },

    /// Compile a keymap to Lua code and make it available to Eruption
    #[clap(about(COMPILE_ABOUT.as_str()))]
    Compile { keymap: PathBuf },

    /// Generate shell completions
    Completions {
        // #[clap(subcommand)]
        shell: Shell,
    },
}

/// Subcommands of the "mapping" command
#[derive(Debug, clap::Parser)]
pub enum MappingSubcommands {
    /// Add a mapping for `source` that executes `action`
    #[clap(about(MAPPING_ADD_ABOUT.as_str()))]
    Add {
        /// Specify a list of layers
        #[clap(required = false, short, long)]
        layers: Vec<usize>,

        keymap: PathBuf,
        source: String,
        action: String,
    },

    /// Remove the mapping for `source`
    #[clap(about(MAPPING_REMOVE_ABOUT.as_str()))]
    Remove {
        /// Specify a list of layers
        //#[clap(required = false, short, long)]
        //layers: Vec<usize>,
        keymap: PathBuf,
        source: String,
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

            for entry in WalkDir::new(&path).follow_links(true).sort_by_file_name() {
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
                layers,
                keymap,
                source,
                action,
            } => {
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

                let mut source = parsers::source::parse(&source)?;
                if !layers.is_empty() {
                    source.get_layers_mut().clear();
                    source.get_layers_mut().extend(&layers);
                }

                let action = parsers::action::parse(&action)?;

                table.insert(source, action);

                NativeBackend::new().write_to_file(&path, &table)?
            }

            MappingSubcommands::Remove {
                //layers,
                keymap,
                source,
            } => {
                let path = if keymap.components().count() > 1 {
                    keymap
                } else {
                    PathBuf::from(constants::DEFAULT_KEYMAP_DIR).join(keymap)
                };

                let mut table = NativeBackend::from_file(&path)?;

                let source = parsers::source::parse(&source)?;
                table.remove(&source);

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
            tab.add_row(row!('#', "Source", "Action"));

            for (index, (source, action)) in table.mappings().iter().enumerate() {
                tab.add_row(Row::new(vec![
                    Cell::new(&format!("{}", index + 1)),
                    Cell::new(&format!("{}", source)),
                    Cell::new(&format!("{}", action)),
                ]));
            }

            tab.printstd();
        }

        Subcommands::Compile { keymap } => {
            let mut path = if keymap.components().count() > 1 {
                keymap
            } else {
                PathBuf::from(constants::DEFAULT_KEYMAP_DIR).join(keymap)
            };

            println!("Compiling keymap {}", &path.display().bold());

            let table = NativeBackend::from_file(&path)?;
            path.set_extension("lua");

            LuaBackend::new().write_to_file(&path, &table)?;

            println!("Success");
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
