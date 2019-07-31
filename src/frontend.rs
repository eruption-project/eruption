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

#[cfg(feature = "frontend")]
use rocket::config::{Config, Environment, LoggingLevel};
#[cfg(feature = "frontend")]
use rocket::{request::Form, request::FormItems, request::FromForm, response::Redirect, *};
#[cfg(feature = "frontend")]
use rocket_contrib::templates::tera::Context;
#[cfg(feature = "frontend")]
use rocket_contrib::*;
#[cfg(feature = "frontend")]
use rocket_contrib::{
    serve::StaticFiles,
    templates::tera::{self, Value},
    templates::{Engines, Template},
};

use failure::Fail;
use lazy_static::*;
use log::*;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};

use crate::constants;
use crate::scripting::manifest;
use crate::profiles;

/// Web-Frontend messages and signals that are processed by the main thread
#[derive(Debug, Clone)]
pub enum Message {
    LoadScript(PathBuf),
}

pub type Result<T> = std::result::Result<T, WebFrontendError>;

#[derive(Debug, Fail)]
pub enum WebFrontendError {
    #[fail(display = "Web frontend inaccessible")]
    FrontendInaccessible {},

    #[fail(display = "Could not enumerate script files")]
    ScriptEnumerationError {},

    #[fail(display = "Could not load script files")]
    ScriptLoadError {},
    // #[fail(display = "Unknown error: {}", description)]
    // UnknownError { description: String },
}

/// A web-based configuration ui
#[cfg(feature = "frontend")]
#[derive(Debug, Clone)]
pub struct WebFrontend {
    profile_path: PathBuf,
    script_path: PathBuf,
}

lazy_static! {
    static ref WEB_FRONTEND: Arc<Mutex<Option<WebFrontend>>> = Arc::new(Mutex::new(None));
    static ref MESSAGE_TX: Arc<Mutex<Option<Sender<Message>>>> = Arc::new(Mutex::new(None));
}

#[cfg(feature = "frontend")]
impl WebFrontend {
    pub fn new(frontend_tx: Sender<Message>, profile_path: PathBuf, script_path: PathBuf) -> Self {
        let frontend = WebFrontend { profile_path, script_path };

        *WEB_FRONTEND.lock().unwrap() = Some(frontend.clone());
        *MESSAGE_TX.lock().unwrap() = Some(frontend_tx);

        #[cfg(not(debug_assertions))]
        let config = Config::build(Environment::Production)
            .address(constants::WEB_FRONTEND_LISTEN_ADDR)
            .port(constants::WEB_FRONTEND_PORT)
            .log_level(LoggingLevel::Off)
            .workers(2)
            .finalize()
            .unwrap();

        #[cfg(debug_assertions)]
        let config = Config::build(Environment::Development)
            .address(constants::WEB_FRONTEND_LISTEN_ADDR)
            .port(constants::WEB_FRONTEND_PORT)
            .log_level(LoggingLevel::Normal)
            .workers(2)
            .finalize()
            .unwrap();

        rocket::custom(config)
            .mount(
                "/",
                routes![
                    index,
                    profiles,
                    settings,
                    settings_of_id,
                    settings_apply,
                    documentation,
                    about
                ],
            )
            .mount("/", StaticFiles::from("static"))
            .attach(Template::custom(|engines: &mut Engines| {
                engines.tera.register_filter("to_html_color", to_html_color);
            }))
            .launch();

        frontend
    }
}

/// Initialize the Eruption Web-Frontend support
#[cfg(feature = "frontend")]
pub fn initialize(frontend_tx: Sender<Message>, profile_path: PathBuf, script_path: PathBuf) -> Result<WebFrontend> {
    Ok(WebFrontend::new(frontend_tx, profile_path, script_path))
}

/// An empty dummy struct
#[cfg(not(feature = "frontend"))]
pub struct WebFrontend {}

/// Get an empty dummy implementation of the Web-Frontend
#[cfg(not(feature = "frontend"))]
pub fn initialize_dummy() -> WebFrontend {
    WebFrontend {}
}

/// Request the main thread to load a script with id `script_id`.
fn request_load_script_by_id(script_id: usize) -> Result<()> {
    let tx = MESSAGE_TX.lock().unwrap();
    let tx = tx.as_ref().unwrap();

    let frontend = WEB_FRONTEND.lock().unwrap();
    let frontend = frontend.as_ref();

    match frontend {
        Some(frontend) => {
            let base_path = frontend.script_path.parent().unwrap();
            match manifest::get_script_files(base_path) {
                Ok(ref script_file) => {
                    match tx.send(Message::LoadScript(script_file[script_id].to_path_buf())) {
                        Ok(()) => Ok(()),

                        Err(e) => {
                            error!("Could not send an event to the main thread: {}", e);
                            Err(WebFrontendError::ScriptLoadError {})
                        }
                    }
                }

                Err(_e) => Err(WebFrontendError::ScriptEnumerationError {}),
            }
        }

        None => {
            error!("Web frontend went away");
            Err(WebFrontendError::FrontendInaccessible {})
        }
    }
}

fn to_html_color(value: Value, _: HashMap<String, Value>) -> tera::Result<Value> {
    if let tera::Value::Number(v) = value {
        let num = v.as_i64().unwrap();
        let result = format!("#{:01$x}", num, 6);
        Ok(tera::Value::String(result))
    } else {
        Err(tera::Error::from_kind(tera::ErrorKind::Msg(
            "Invalid color".to_string(),
        )))
    }
}

#[get("/")]
fn index() -> templates::Template {
    let mut context = Context::new();

    let config = crate::CONFIG.read().unwrap();
    let frontend_theme = config
        .as_ref()
        .unwrap()
        .get_str("frontend.theme")
        .unwrap_or_else(|_| constants::DEFAULT_FRONTEND_THEME.to_string());

    context.insert("theme", &frontend_theme);

    context.insert("title", "Eruption: Home");
    context.insert("heading", "Home");

    templates::Template::render("index", &context)
}

#[get("/profiles")]
fn profiles() -> templates::Template {
    let mut context = Context::new();

    let globals = crate::GLOBALS.read().unwrap();
    let active_profile = globals.active_profile.as_ref().unwrap();

    let frontend = WEB_FRONTEND.lock().unwrap_or_else(|e| {
        error!("Could not lock a shared data structure: {}", e);
        panic!()
    });

    match *frontend {
        Some(ref frontend) => {
            let base_path = frontend.profile_path.as_ref();
            let profiles = profiles::get_profiles(&base_path).unwrap();

            let config = crate::CONFIG.read().unwrap();
            let frontend_theme = config
                .as_ref()
                .unwrap()
                .get_str("frontend.theme")
                .unwrap_or_else(|_| constants::DEFAULT_FRONTEND_THEME.to_string());

            context.insert("theme", &frontend_theme);

            context.insert("title", "Eruption: Profiles");
            context.insert("heading", "Profiles");

            context.insert("active_profile", &active_profile);
            context.insert("profiles", &profiles);

            templates::Template::render("profiles", &context)
        }

        None => {
            error!("Web frontend inaccessible");
            panic!()
        }
    }
}

#[get("/settings")]
fn settings() -> manifest::Result<templates::Template> {
    let mut context = Context::new();

    let config = crate::CONFIG.read().unwrap();
    let script_dir = config.as_ref().unwrap()
        .get_str("global.script_dir")
        .unwrap_or_else(|_| constants::DEFAULT_SCRIPT_DIR.to_string());
    let script_path = PathBuf::from(&script_dir);

    let scripts = manifest::get_scripts(&script_path)?;

    let config = crate::CONFIG.read().unwrap();
    let frontend_theme = config
        .as_ref()
        .unwrap()
        .get_str("frontend.theme")
        .unwrap_or_else(|_| constants::DEFAULT_FRONTEND_THEME.to_string());

    context.insert("theme", &frontend_theme);

    context.insert("title", "Eruption: Settings");
    context.insert("heading", "Select Effect");

    context.insert("scripts", &scripts);

    Ok(templates::Template::render("settings", &context))
}

#[get("/settings/<script_id>")]
fn settings_of_id(script_id: Option<usize>) -> manifest::Result<templates::Template> {
    let mut context = Context::new();

    let config = crate::CONFIG.read().unwrap();
    let frontend_theme = config
        .as_ref()
        .unwrap()
        .get_str("frontend.theme")
        .unwrap_or_else(|_| constants::DEFAULT_FRONTEND_THEME.to_string());

    context.insert("theme", &frontend_theme);

    let script_dir = config.as_ref().unwrap()
        .get_str("global.script_dir")
        .unwrap_or_else(|_| constants::DEFAULT_SCRIPT_DIR.to_string());
    let script_path = PathBuf::from(&script_dir);

    if script_id.is_some() {
        let scripts = manifest::get_scripts(&script_path)?;

        // apply effect script
        request_load_script_by_id(script_id.unwrap()).unwrap_or_else(|e| {
            error!("Request to load a script has failed: {}", e);
        });

        context.insert("title", "Eruption: Settings");
        context.insert("heading", "Effect Settings");
        context.insert("script", &scripts[script_id.unwrap()]);

        context.insert("config_params", &scripts[script_id.unwrap()].config);

        context.insert("scripts", &scripts);

        Ok(templates::Template::render("detail", &context))
    } else {
        let scripts = manifest::get_scripts(&script_path)?;

        context.insert("title", "Eruption: Settings");
        context.insert("heading", "Select Effect");

        context.insert("scripts", &scripts);

        Ok(templates::Template::render("settings", &context))
    }
}

#[post("/settings/apply/<script_id>", data = "<params>")]
fn settings_apply(script_id: usize, params: Form<ValueMap<String, String>>) -> Redirect {
    info!("{:?}", params);

    Redirect::to(format!("/settings/{}", script_id))
}

#[get("/documentation")]
fn documentation() -> templates::Template {
    let mut context = Context::new();

    let config = crate::CONFIG.read().unwrap();
    let frontend_theme = config
        .as_ref()
        .unwrap()
        .get_str("frontend.theme")
        .unwrap_or_else(|_| constants::DEFAULT_FRONTEND_THEME.to_string());

    context.insert("theme", &frontend_theme);

    context.insert("title", "Eruption: Documentation");
    context.insert("heading", "Documentation");

    templates::Template::render("documentation", &context)
}

#[get("/about")]
fn about() -> templates::Template {
    let mut context = Context::new();

    let config = crate::CONFIG.read().unwrap();
    let frontend_theme = config
        .as_ref()
        .unwrap()
        .get_str("frontend.theme")
        .unwrap_or_else(|_| constants::DEFAULT_FRONTEND_THEME.to_string());

    context.insert("theme", &frontend_theme);

    context.insert("title", "Eruption: About");
    context.insert("heading", "About");

    templates::Template::render("about", &context)
}

/// HashMap like data structure, used to represent dynamic form values
#[derive(Debug, Clone)]
pub struct ValueMap<K, V>
where
    K: std::hash::Hash + Eq,
{
    store: HashMap<K, V>,
}

impl<K: std::hash::Hash + Eq, V> ValueMap<K, V> {
    pub fn new() -> Self {
        ValueMap {
            store: HashMap::new(),
        }
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.store.insert(k, v)
    }
}

impl<'f, K, V> FromForm<'f> for ValueMap<K, V>
where
    K: std::hash::Hash + Eq + std::convert::From<String>,
    V: std::convert::From<String>,
{
    type Error = WebFrontendError;

    fn from_form(it: &mut FormItems<'f>, _strict: bool) -> Result<Self> {
        let mut params = Self::new();

        for item in it {
            let (key, value) = item.key_value_decoded();
            params.insert(key.into(), value.into());
        }

        Ok(params)
    }
}
