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

use clap::Parser;
use config::Config;
use flume::unbounded;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use std::env;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::{process, sync::Arc};

mod color_scheme;
mod constants;
mod dbus_client;
mod device;
mod profiles;
mod scripting;
mod subcommands;
mod translations;
mod util;

use translations::tr;

lazy_static! {
    /// Global configuration
    pub static ref CONFIG: Arc<Mutex<Option<config::Config>>> = Arc::new(Mutex::new(None));

    /// Global verbosity amount
    pub static ref VERBOSE: AtomicU8 = AtomicU8::new(0);

    /// Global repeat flag
    pub static ref REPEAT: AtomicBool = AtomicBool::new(false);

    /// Global "quit" status flag
    pub static ref QUIT: AtomicBool = AtomicBool::new(false);
}

#[derive(Debug, thiserror::Error)]
pub enum MainError {
    #[error("Unknown error: {description}")]
    UnknownError { description: String },
}

/// Supported command line arguments
#[derive(Debug, clap::Parser)]
#[clap(
    version = env!("CARGO_PKG_VERSION"),
    author = "X3n0m0rph59 <x3n0m0rph59@gmail.com>",
    about = tr!("about"),
)]
pub struct Options {
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(help(tr!("verbose-about")), short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Repeat output until ctrl+c is pressed
    #[clap(short, long)]
    repeat: bool,

    /// Sets the configuration file to use
    #[clap(short = 'c', long)]
    config: Option<String>,

    #[clap(subcommand)]
    command: subcommands::Subcommands,
}

/// Main program entrypoint
pub fn main() -> std::result::Result<(), eyre::Error> {
    translations::load()?;

    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async move { async_main().await })
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

    register_sigint_handler();

    let opts = Options::parse();
    apply_opts(&opts);
    subcommands::handle_command(opts.command).await
}

fn print_header() {
    println!("{}", tr!("license-header"));
    println!(
        r"
 ********                          **   **                  
 /**/////                 ******   /**  //                   
 /**       ****** **   **/**///** ****** **  ******  ******* 
 /******* //**//*/**  /**/**  /**///**/ /** **////**//**///**
 /**////   /** / /**  /**/******   /**  /**/**   /** /**  /**
 /**       /**   /**  /**/**///    /**  /**/**   /** /**  /**
 /********/***   //******/**       //** /**//******  ***  /**
 //////// ///     ////// //         //  //  //////  ///   //
"
    );
}

fn register_sigint_handler() {
    // register ctrl-c handler
    let (ctrl_c_tx, _ctrl_c_rx) = unbounded();
    ctrlc::set_handler(move || {
        QUIT.store(true, Ordering::SeqCst);

        ctrl_c_tx.send(true).unwrap_or_else(|e| {
            log::error!(
                "{}",
                tr!("could-not-send-on-channel", message = e.to_string())
            );
        });
    })
    .unwrap_or_else(|e| {
        log::error!(
            "{}",
            tr!("could-not-set-ctrl-c-handler", message = e.to_string())
        )
    });
}

fn apply_opts(opts: &Options) {
    VERBOSE.store(opts.verbose, Ordering::SeqCst);
    REPEAT.store(opts.repeat, Ordering::SeqCst);

    // process configuration file
    let config_file = opts
        .config
        .as_deref()
        .unwrap_or(constants::DEFAULT_CONFIG_FILE);

    let config = Config::builder()
        .add_source(config::File::new(&config_file, config::FileFormat::Toml))
        .build()
        .unwrap_or_else(|e| {
            log::error!("{}", tr!("could-not-parse-config", message = e.to_string()));
            process::exit(4);
        });

    *CONFIG.lock() = Some(config);
}
