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

use clap::Parser;
use config::Config;
use flume::bounded;
use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::env;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::{process, sync::Arc};
use tracing::instrument;

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
    pub static ref CONFIG: Arc<RwLock<Option<config::Config>>> = Arc::new(RwLock::new(None));

    /// Global verbosity amount
    pub static ref VERBOSE: AtomicU8 = AtomicU8::new(0);

    /// Global repeat flag
    pub static ref REPEAT: AtomicBool = AtomicBool::new(false);

    /// Global "quit" status flag
    pub static ref QUIT: AtomicBool = AtomicBool::new(false);
}

/// Supported command line arguments
#[derive(Debug, clap::Parser)]
#[clap(
    version = env!("CARGO_PKG_VERSION"),
    author = "X3n0m0rph59 <x3n0m0rph59@gmail.com>",
    about = tr!("about"),
)]
pub struct Options {
    /// Subcommand
    #[clap(subcommand)]
    command: subcommands::Subcommands,

    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(display_order = 0, help(tr!("verbose-about")), short, long, action = clap::ArgAction::Count)]
    verbose: u8,

    /// Repeat output until ctrl+c is pressed
    #[clap(display_order = 1, short, long)]
    repeat: bool,

    /// Sets the configuration file to use
    #[clap(display_order = 2, short = 'c', long)]
    config: Option<String>,
}

/// Main program entrypoint
#[instrument]
pub fn main() -> std::result::Result<(), eyre::Error> {
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::Layer;

    // let filter = tracing_subscriber::EnvFilter::from_default_env();
    // let journald_layer = tracing_journald::layer()?.with_filter(filter);

    let filter = tracing_subscriber::EnvFilter::from_default_env();
    let format_layer = tracing_subscriber::fmt::layer()
        .compact()
        .with_filter(filter);

    cfg_if::cfg_if! {
        if #[cfg(feature = "debug-async")] {
            let console_layer = console_subscriber::ConsoleLayer::builder()
                .with_default_env()
                .spawn();

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

    // i18n/l10n support
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
    //     println!(
    //         r"
    //  ********                          **   **
    //  /**/////                 ******   /**  //
    //  /**       ****** **   **/**///** ****** **  ******  *******
    //  /******* //**//*/**  /**/**  /**///**/ /** **////**//**///**
    //  /**////   /** / /**  /**/******   /**  /**/**   /** /**  /**
    //  /**       /**   /**  /**/**///    /**  /**/**   /** /**  /**
    //  /********/***   //******/**       //** /**//******  ***  /**
    //  //////// ///     ////// //         //  //  //////  ///   //
    // "
    //     );
}

fn register_sigint_handler() {
    // register ctrl-c handler
    let (ctrl_c_tx, _ctrl_c_rx) = bounded(32);
    ctrlc::set_handler(move || {
        QUIT.store(true, Ordering::SeqCst);

        ctrl_c_tx.send(true).unwrap_or_else(|e| {
            tracing::error!(
                "{}",
                tr!("could-not-send-on-channel", message = e.to_string())
            );
        });
    })
    .unwrap_or_else(|e| {
        tracing::error!(
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
        .add_source(config::File::new(config_file, config::FileFormat::Toml))
        .build()
        .unwrap_or_else(|e| {
            tracing::error!("{}", tr!("could-not-parse-config", message = e.to_string()));
            process::exit(4);
        });

    *CONFIG.write() = Some(config);
}
