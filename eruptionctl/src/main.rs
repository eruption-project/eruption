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
use dbus::nonblock;
use dbus_tokio::connection;
use flume::unbounded;
use i18n_embed::{
    fluent::{fluent_language_loader, FluentLanguageLoader},
    DesktopLanguageRequester,
};
use lazy_static::lazy_static;
use parking_lot::Mutex;
use rust_embed::RustEmbed;
use std::env;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::time::Duration;
use std::{process, sync::Arc};

mod subcommands;

mod color_scheme;
mod constants;
mod dbus_client;
mod device;
mod profiles;
mod scripting;
mod util;

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
pub(crate) use tr;

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

lazy_static! {
    static ref ABOUT: String = tr!("about");
    static ref VERBOSE_ABOUT: String = tr!("verbose-about");
    static ref COMPLETIONS_ABOUT: String = tr!("completions-about");
    static ref COLOR_SCHEME_ABOUT: String = tr!("color-scheme-about");
    static ref CONFIG_ABOUT: String = tr!("config-about");
    static ref DEVICES_ABOUT: String = tr!("devices-about");
    static ref STATUS_ABOUT: String = tr!("status-about");
    static ref SWITCH_ABOUT: String = tr!("switch-about");
    static ref PROFILES_ABOUT: String = tr!("profiles-about");
    static ref NAMES_ABOUT: String = tr!("names-about");
    static ref SCRIPTS_ABOUT: String = tr!("scripts-about");
    static ref PARAM_ABOUT: String = tr!("param-about");
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

    /// Repeat output until ctrl+c is pressed
    #[clap(short, long)]
    repeat: bool,

    /// Sets the configuration file to use
    #[clap(short = 'c', long)]
    config: Option<String>,

    #[clap(subcommand)]
    command: subcommands::Subcommands,
}

/// Print license information
#[allow(dead_code)]
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

/// Returns a connection to the D-Bus system bus using the specified `path`
pub async fn dbus_system_bus(
    path: &str,
) -> Result<dbus::nonblock::Proxy<'_, Arc<dbus::nonblock::SyncConnection>>> {
    let (resource, conn) = connection::new_system_sync()?;

    tokio::spawn(async {
        let err = resource.await;
        panic!("Lost connection to D-Bus: {}", err);
    });

    let proxy = nonblock::Proxy::new(
        "org.eruption",
        path,
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
        conn,
    );

    Ok(proxy)
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

    let opts = Options::parse();

    // process configuration file
    let config_file = opts
        .config
        .unwrap_or_else(|| constants::DEFAULT_CONFIG_FILE.to_string());

    let config = Config::builder()
        .add_source(config::File::new(&config_file, config::FileFormat::Toml))
        .build()
        .unwrap_or_else(|e| {
            log::error!("{}", tr!("could-not-parse-config", message = e.to_string()));
            process::exit(4);
        });

    *CONFIG.lock() = Some(config);

    VERBOSE.store(opts.verbose, Ordering::SeqCst);
    REPEAT.store(opts.repeat, Ordering::SeqCst);

    subcommands::handle_command(opts.command).await
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
