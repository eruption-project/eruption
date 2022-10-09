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
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};
use lazy_static::lazy_static;
use log::*;
use parking_lot::Mutex;
use rust_embed::RustEmbed;
use std::{
    env,
    path::PathBuf,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

// mod assistants;
mod constants;
mod dbus_client;
mod device;
mod hwdevices;
mod lua_introspection;
mod messages;
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
    static ref RECORD_ABOUT: String = tr!("record-about");
    static ref DESCRIPTION_ABOUT: String = tr!("description-about");
    // static ref ASSISTANT_ABOUT: String = tr!("assistant-about");
    // static ref LIST_ABOUT: String = tr!("list-about");
    // static ref MAPPING_ABOUT: String = tr!("mapping-about");
    // static ref SHOW_ABOUT: String = tr!("show-about");
    // static ref MACROS_ABOUT: String = tr!("macros-about");
    // static ref EVENTS_ABOUT: String = tr!("events-about");
    // static ref COMPILE_ABOUT: String = tr!("compile-about");
    static ref MACRO_ADD_ABOUT: String = tr!("macro-add-about");
    static ref MACRO_REMOVE_ABOUT: String = tr!("macro-remove-about");
    // static ref MAPPING_ENABLE_ABOUT: String = tr!("mapping-enable-about");
    // static ref MAPPING_DISABLE_ABOUT: String = tr!("mapping-disable-about");
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

    /// Add a mapping rule for `source` that executes `action`
    #[clap(about(MACRO_ADD_ABOUT.as_str()))]
    Add {
        /// Specify the enabled status of the newly added rule
        #[clap(required = false, short, long, default_value = "true")]
        enabled: bool,

        /// Specify a description for a rule
        #[clap(required = false, long, default_value = "")]
        description: String,

        macro_code: String,
    },

    /// Remove the mapping rule for `source`
    #[clap(about(MACRO_REMOVE_ABOUT.as_str()))]
    Remove { index: usize },

    /// Record a key sequence and save it as a Lua function
    #[clap(about(RECORD_ABOUT.as_str()))]
    Record {
        // #[clap(required = false, short, long, default_value = "user-macros.lua")]
        // lua_file: PathBuf,
        macro_name: String,
        description: Option<String>,
    },

    /// Show or set the description of the specified keymap
    #[clap(about(DESCRIPTION_ABOUT.as_str()))]
    Description {
        #[clap(required = false, short, long, default_value = "user-macros.lua")]
        lua_file: PathBuf,
        description: Option<String>,
    },

    /// Generate shell completions
    #[clap(about(COMPLETIONS_ABOUT.as_str()))]
    Completions {
        // #[clap(subcommand)]
        shell: Shell,
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
    // #[cfg(debug_assertions)]
    // thread_util::deadlock_detector()
    //     .unwrap_or_else(|e| error!("Could not spawn deadlock detector thread: {}", e));

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
        Subcommands::Record {
            macro_name: _,
            description: _,
        } => {
            todo!()
        }

        Subcommands::Add {
            enabled: _,
            description: _,
            macro_code: _,
        } => todo!(),

        Subcommands::Remove { index: _ } => todo!(),

        Subcommands::Description {
            lua_file,
            description,
        } => {
            let _path = if lua_file.components().count() > 1 {
                lua_file
            } else {
                PathBuf::from(constants::DEFAULT_MACRO_DIR).join(lua_file)
            };

            if let Some(_description) = description {
            } else {
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

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .thread_name("worker")
        .enable_all()
        // .worker_threads(4)
        .build()?;

    runtime.block_on(async move { async_main().await })
}
