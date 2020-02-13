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

use crate::profiles::GetAttr;
use failure::Fail;
use lazy_static::*;
use log::*;
use std::collections::HashMap;
use std::iter::FromIterator;
use std::path::{Path, PathBuf};
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use crate::constants;
use crate::plugins::audio::ENABLE_SFX;
use crate::profiles;
use crate::scripting::manifest;
use crate::scripting::manifest::GetAttr as GetAttrManifest;
use crate::scripting::manifest::ParseConfig;
use crate::{ACTIVE_PROFILE, ACTIVE_SCRIPTS};

/// Web-Frontend messages and signals that are processed by the main thread
#[derive(Debug, Clone)]
pub enum Message {
    LoadScript(PathBuf),
    SwitchProfile(PathBuf),
}

pub type Result<T> = std::result::Result<T, failure::Error>;

#[derive(Debug, Fail)]
pub enum WebFrontendError {
    #[fail(display = "Web frontend inaccessible")]
    FrontendInaccessible {},

    //#[fail(display = "Could not enumerate script files")]
    //ScriptEnumerationError {},
    #[fail(display = "Could not enumerate profile files")]
    ProfileEnumerationError {},

    #[fail(display = "Could not load script files")]
    ScriptLoadError {},

    #[fail(display = "Could not switch to a different profile")]
    ProfileSwitchError {},

    #[fail(display = "Could not parse a parameter value")]
    ParseParamError {},
    // #[fail(display = "Unknown error: {}", description)]
    // UnknownError { description: String },
}

/// A web-based configuration ui
#[cfg(feature = "frontend")]
#[derive(Debug, Clone)]
pub struct WebFrontend {
    profile_path: PathBuf,
    script_paths: Vec<PathBuf>,
}

lazy_static! {
    static ref WEB_FRONTEND: Arc<Mutex<Option<WebFrontend>>> = Arc::new(Mutex::new(None));
    static ref MESSAGE_TX: Arc<Mutex<Option<Sender<Message>>>> = Arc::new(Mutex::new(None));
}

#[cfg(feature = "frontend")]
impl WebFrontend {
    pub fn new(
        frontend_tx: Sender<Message>,
        profile_path: PathBuf,
        script_paths: Vec<PathBuf>,
    ) -> Self {
        let frontend = WebFrontend {
            profile_path,
            script_paths,
        };

        *WEB_FRONTEND.lock().unwrap() = Some(frontend.clone());
        *MESSAGE_TX.lock().unwrap() = Some(frontend_tx);

        #[cfg(not(debug_assertions))]
        let config = Config::build(Environment::Production)
            .address(constants::WEB_FRONTEND_LISTEN_ADDR)
            .port(constants::WEB_FRONTEND_PORT)
            .log_level(LoggingLevel::Off)
            .root(Path::new("/usr/share/eruption/"))
            .workers(2)
            .finalize()
            .unwrap();

        #[cfg(debug_assertions)]
        let config = Config::build(Environment::Development)
            .address(constants::WEB_FRONTEND_LISTEN_ADDR)
            .port(constants::WEB_FRONTEND_PORT)
            .log_level(LoggingLevel::Normal)
            .root(Path::new(env!("PWD")))
            .workers(2)
            .finalize()
            .unwrap();

        #[cfg(not(debug_assertions))]
        let static_files = StaticFiles::from("/usr/share/eruption/static/");

        #[cfg(debug_assertions)]
        let static_files = StaticFiles::from("static");

        rocket::custom(config)
            .mount(
                "/",
                routes![
                    index,
                    profiles_selection,
                    activate_profile,
                    settings,
                    settings_of_id,
                    settings_apply,
                    preview_script,
                    soundfx,
                    soundfx_apply,
                    documentation,
                    about
                ],
            )
            .mount("/", static_files)
            .attach(Template::custom(|engines: &mut Engines| {
                engines.tera.register_filter("to_html_color", to_html_color);
            }))
            // .attach(security::CsrfProtection)
            .launch();

        frontend
    }
}

/// Initialize the Eruption Web-Frontend support
#[cfg(feature = "frontend")]
pub fn initialize(
    frontend_tx: Sender<Message>,
    profile_path: PathBuf,
    script_paths: Vec<PathBuf>,
) -> Result<WebFrontend> {
    Ok(WebFrontend::new(frontend_tx, profile_path, script_paths))
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
fn request_load_script_by_id(script_id: usize) -> Result<PathBuf> {
    let tx = MESSAGE_TX.lock().unwrap();
    let tx = tx.as_ref().unwrap();

    let frontend = WEB_FRONTEND.lock().unwrap();
    let frontend = frontend.as_ref();

    match frontend {
        Some(frontend) => {
            // TODO: find better way to handle base path
            let base_path = frontend.script_paths[0].parent().unwrap();
            let script_files: Vec<PathBuf> = manifest::get_scripts(&base_path)?
                .iter()
                .map(|e| e.script_file.clone())
                .collect();

            let script_path = script_files[script_id].to_path_buf();

            match tx.send(Message::LoadScript(script_path.clone())) {
                Ok(()) => Ok(script_path),

                Err(e) => {
                    error!("Could not send an event to the main thread: {}", e);
                    Err(WebFrontendError::ScriptLoadError {}.into())
                }
            }
        }

        None => {
            error!("Web frontend went away");
            Err(WebFrontendError::FrontendInaccessible {}.into())
        }
    }
}

/// Request the main thread to load the profile with the name `profile_name`.
fn request_switch_profile_by_id(profile_id: Uuid) -> Result<()> {
    let tx = MESSAGE_TX.lock().unwrap();
    let tx = tx.as_ref().unwrap();

    let frontend = WEB_FRONTEND.lock().unwrap();
    let frontend = frontend.as_ref();

    match frontend {
        Some(frontend) => {
            let base_path = &frontend.profile_path;
            if let Some(profile_path) = profiles::find_path_by_uuid(profile_id, base_path) {
                match tx.send(Message::SwitchProfile(profile_path)) {
                    Ok(()) => Ok(()),

                    Err(e) => {
                        error!("Could not send an event to the main thread: {}", e);
                        Err(WebFrontendError::ProfileSwitchError {}.into())
                    }
                }
            } else {
                error!("Could not enumerated profile files");
                Err(WebFrontendError::ProfileEnumerationError {}.into())
            }
        }

        None => {
            error!("Web frontend went away");
            Err(WebFrontendError::FrontendInaccessible {}.into())
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
fn index() -> manifest::Result<Redirect> {
    Ok(Redirect::to("/settings"))
}

#[get("/profiles")]
fn profiles_selection() -> templates::Template {
    let mut context = Context::new();

    let profile = ACTIVE_PROFILE.read().unwrap();
    let profile_name = &profile.as_ref().unwrap().name;
    //let active_scripts = ACTIVE_SCRIPTS.read().unwrap();

    let active_profile = &*ACTIVE_PROFILE.read().unwrap();
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
            context.insert("active_profile_name", &profile_name);
            //context.insert("active_scripts", &active_scripts);
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

#[post("/profiles/active/<profile_id>")]
fn activate_profile(profile_id: Option<String>) -> Redirect {
    request_switch_profile_by_id(Uuid::parse_str(&profile_id.unwrap()).unwrap()).unwrap();

    Redirect::to("/profiles")
}

#[get("/settings")]
fn settings() -> manifest::Result<templates::Template> {
    let mut context = Context::new();

    let profile = ACTIVE_PROFILE.read().unwrap();
    let profile_name = &profile.as_ref().unwrap().name;
    //let active_scripts = ACTIVE_SCRIPTS.read().unwrap();

    let config = crate::CONFIG.read().unwrap();
    let script_dir = config
        .as_ref()
        .unwrap()
        .get_str("global.script_dir")
        .unwrap_or_else(|_| constants::DEFAULT_SCRIPT_DIR.to_string());
    let script_path = PathBuf::from(&script_dir);
    let scripts = manifest::get_scripts(&script_path)?;
    let active_script_ids = ACTIVE_SCRIPTS
        .read()
        .unwrap()
        .iter()
        .map(|s| s.id)
        .collect::<Vec<usize>>();
    let config = crate::CONFIG.read().unwrap();
    let frontend_theme = config
        .as_ref()
        .unwrap()
        .get_str("frontend.theme")
        .unwrap_or_else(|_| constants::DEFAULT_FRONTEND_THEME.to_string());
    context.insert("theme", &frontend_theme);
    context.insert("title", "Eruption: Settings");
    context.insert("active_profile_name", &profile_name);
    //context.insert("active_scripts", &active_scripts);
    context.insert("heading", "Select Effect");

    context.insert("scripts", &scripts);
    context.insert("active_script_ids", &active_script_ids);

    Ok(templates::Template::render("settings", &context))
}

#[get("/settings/<script_id>")]
fn settings_of_id(script_id: Option<usize>) -> Result<templates::Template> {
    let mut context = Context::new();

    let config = crate::CONFIG.read().unwrap();
    let frontend_theme = config
        .as_ref()
        .unwrap()
        .get_str("frontend.theme")
        .unwrap_or_else(|_| constants::DEFAULT_FRONTEND_THEME.to_string());

    context.insert("theme", &frontend_theme);

    let script_dir = config
        .as_ref()
        .unwrap()
        .get_str("global.script_dir")
        .unwrap_or_else(|_| constants::DEFAULT_SCRIPT_DIR.to_string());
    let script_path = PathBuf::from(&script_dir);

    if let Some(script_id) = script_id {
        let scripts = manifest::get_scripts(&script_path)?;

        // apply effect script
        let script_path = request_load_script_by_id(script_id).map_err(|e| {
            error!("Request to load a script has failed: {}", e);
            e
        })?;

        {
            let profile = ACTIVE_PROFILE.read().unwrap();
            let profile_name = &profile.as_ref().unwrap().name;
            context.insert("active_profile_name", &profile_name);
            //context.insert("active_scripts", &active_scripts);
        }

        context.insert("title", "Eruption: Settings");
        context.insert("heading", "Effect Settings");
        context.insert("script", &scripts[script_id]);

        let active_profile = &mut *ACTIVE_PROFILE.write().unwrap();
        let profile = active_profile.as_mut().unwrap();
        let script_name = scripts[script_id].name.clone();

        if !profile.active_scripts.contains(&script_path) {
            profile
                .active_scripts
                .push(Path::new(script_path.file_name().unwrap()).to_path_buf());
            profile.save()?
        }

        // get config values from current profile
        let mut config_values: HashMap<String, String> = HashMap::from_iter(
            profile
                .config
                .as_ref()
                .unwrap_or(&HashMap::new())
                .get(&script_name)
                .unwrap_or(&vec![])
                .iter()
                .map(|e| (e.get_name().clone(), e.get_value())),
        );

        // insert missing config values, by using the respective
        // defaults from the manifest file
        scripts[script_id].config.iter().for_each(|se| {
            let result = config_values
                .iter()
                .find(|(ce_k, _ce_v)| *ce_k == se.get_name());

            if result.is_none() {
                config_values.insert(se.get_name().to_owned(), se.get_default());
            } else {
            }
        });

        context.insert("config_values", &config_values);
        context.insert("config_params", &scripts[script_id].config);

        context.insert("scripts", &scripts);

        let active_script_id = scripts.iter().position(|e| e.id == script_id);
        context.insert("active_script_id", &active_script_id.ok_or(-1).unwrap());

        Ok(templates::Template::render("detail", &context))
    } else {
        let scripts = manifest::get_scripts(&script_path)?;

        let profile = ACTIVE_PROFILE.read().unwrap();
        let profile_name = &profile.as_ref().unwrap().name;
        //let active_scripts = ACTIVE_SCRIPTS.read().unwrap();
        //let script_name = &script.as_ref().unwrap().name;

        context.insert("title", "Eruption: Settings");
        context.insert("active_profile_name", &profile_name);
        //context.insert("active_scripts", &active_scripts);
        context.insert("heading", "Select Effect");

        context.insert("scripts", &scripts);

        Ok(templates::Template::render("settings", &context))
    }
}

#[post("/settings/apply/<script_id>", data = "<params>")]
fn settings_apply(script_id: usize, params: Form<ValueMap<String, String>>) -> Result<Redirect> {
    let active_scripts = &*ACTIVE_SCRIPTS.read().unwrap();
    let active_profile = &mut *ACTIVE_PROFILE.write().unwrap();

    let mut profile = active_profile.as_mut().unwrap().clone();

    let mut default_map = HashMap::new();
    let mut default_config = vec![];

    let script = active_scripts.iter().find(|e| e.id == script_id).unwrap();
    let config = profile
        .config
        .as_mut()
        .unwrap_or(&mut default_map)
        .get_mut(&script.name)
        .unwrap_or(&mut default_config);

    for (k, v) in &params.store {
        // does our profile already store this parameter?
        let result = config.iter_mut().find(|param| param.get_name() == k);

        let param = script.config.parse_config_param(k, v).unwrap();
        if let Some(result) = result {
            // modify existing param
            *result = param;
        } else {
            // param is currently not listed in the profile
            // if the param is "the default value", then don't store it in the profile
            let manifest_param = script
                .config
                .iter()
                .find(|p| p.get_name() == param.get_name())
                .unwrap();

            if param.get_value() != manifest_param.get_default() {
                config.push(param);
            }
        }
    }

    let mut default_map = HashMap::new();

    active_profile
        .as_mut()
        .unwrap()
        .config
        .as_mut()
        .unwrap_or(&mut default_map)
        .insert(script.name.clone(), config.clone());

    active_profile.as_mut().unwrap().save()?;

    Ok(Redirect::to(format!("/settings/{}", script_id)))
}

#[get("/preview/<script_id>")]
fn preview_script(script_id: Option<usize>) -> Result<templates::Template> {
    let mut context = Context::new();

    let config = crate::CONFIG.read().unwrap();
    let frontend_theme = config
        .as_ref()
        .unwrap()
        .get_str("frontend.theme")
        .unwrap_or_else(|_| constants::DEFAULT_FRONTEND_THEME.to_string());

    context.insert("theme", &frontend_theme);

    let script_dir = config
        .as_ref()
        .unwrap()
        .get_str("global.script_dir")
        .unwrap_or_else(|_| constants::DEFAULT_SCRIPT_DIR.to_string());
    let script_path = PathBuf::from(&script_dir);

    let scripts = manifest::get_scripts(&script_path)?;

    let profile = ACTIVE_PROFILE.read().unwrap();
    let profile_name = &profile.as_ref().unwrap().name;
    //let active_scripts = ACTIVE_SCRIPTS.read().unwrap();
    //let script_name = &active_scripts.iter().find(|s| s.id == script_id).unwrap().name;

    context.insert("title", "Eruption: Settings");
    context.insert("active_profile_name", &profile_name);
    //context.insert("active_scripts", &active_scripts);
    context.insert("heading", "Source Code");

    let script_file = &scripts[script_id.unwrap()].script_file;
    let manifest_file = crate::util::get_manifest_for(&scripts[script_id.unwrap()].script_file);

    let code = std::fs::read_to_string(&script_file).unwrap();
    let manifest = std::fs::read_to_string(&manifest_file).unwrap();

    context.insert("filename", &script_file);
    context.insert("code", &code);
    context.insert("manifest", &manifest);

    Ok(templates::Template::render("preview", &context))
}

#[get("/soundfx")]
fn soundfx() -> manifest::Result<templates::Template> {
    let mut context = Context::new();

    let profile = ACTIVE_PROFILE.read().unwrap();
    let profile_name = &profile.as_ref().unwrap().name;
    //let active_scripts = ACTIVE_SCRIPTS.read().unwrap();

    let config = crate::CONFIG.read().unwrap();
    let frontend_theme = config
        .as_ref()
        .unwrap()
        .get_str("frontend.theme")
        .unwrap_or_else(|_| constants::DEFAULT_FRONTEND_THEME.to_string());
    context.insert("theme", &frontend_theme);
    context.insert("title", "Eruption: Settings");
    context.insert("active_profile_name", &profile_name);
    //context.insert("active_scripts", &active_scripts);
    context.insert("heading", "SoundFX");

    let mut config_params = vec![];

    config_params.push(util::ConfigParam::Bool {
        name: "enable_sfx".into(),
        description: "Enable sound effects globally".into(),
        default: false,
        value: ENABLE_SFX.load(Ordering::SeqCst),
    });

    context.insert("config_params", &config_params);

    Ok(templates::Template::render("soundfx", &context))
}

#[post("/soundfx", data = "<params>")]
fn soundfx_apply(params: Form<ValueMap<String, String>>) -> Result<Redirect> {
    let default = "false".to_owned();

    let enable_sfx = params
        .store
        .get("enable_sfx")
        .unwrap_or(&default)
        .parse::<bool>()
        .unwrap();

    ENABLE_SFX.store(enable_sfx, Ordering::SeqCst);

    Ok(Redirect::to("/soundfx"))
}

#[get("/documentation")]
fn documentation() -> templates::Template {
    let mut context = Context::new();

    let profile = ACTIVE_PROFILE.read().unwrap();
    let profile_name = &profile.as_ref().unwrap().name;
    //let active_scripts = ACTIVE_SCRIPTS.read().unwrap();

    let config = crate::CONFIG.read().unwrap();
    let frontend_theme = config
        .as_ref()
        .unwrap()
        .get_str("frontend.theme")
        .unwrap_or_else(|_| constants::DEFAULT_FRONTEND_THEME.to_string());
    context.insert("theme", &frontend_theme);
    context.insert("title", "Eruption: Documentation");
    context.insert("active_profile_name", &profile_name);
    //context.insert("active_scripts", &active_scripts);
    context.insert("heading", "Documentation");

    templates::Template::render("documentation", &context)
}

#[get("/about")]
fn about() -> templates::Template {
    let mut context = Context::new();

    let profile = ACTIVE_PROFILE.read().unwrap();
    let profile_name = &profile.as_ref().unwrap().name;
    //let active_scripts = ACTIVE_SCRIPTS.read().unwrap();

    let config = crate::CONFIG.read().unwrap();
    let frontend_theme = config
        .as_ref()
        .unwrap()
        .get_str("frontend.theme")
        .unwrap_or_else(|_| constants::DEFAULT_FRONTEND_THEME.to_string());

    context.insert("theme", &frontend_theme);

    context.insert("title", "Eruption: About");
    context.insert("active_profile_name", &profile_name);
    //context.insert("active_scripts", &active_scripts);
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
    type Error = failure::Error;

    fn from_form(it: &mut FormItems<'f>, _strict: bool) -> Result<Self> {
        let mut params = Self::new();

        for item in it {
            let (key, value) = item.key_value_decoded();
            params.insert(key.into(), value.into());
        }

        Ok(params)
    }
}

// #[cfg(feature = "frontend")]
// mod security {
//     use rocket::fairing::{Fairing, Info, Kind};
//     use rocket::http::uri::Origin;
//     use rocket::http::Status;
//     use rocket::http::{Cookie, Method, StatusClass};
//     use rocket::*;
//     use rocket::{Data, Request, Response, Rocket};
//     use uuid::Uuid;

//     const XSRF_TOKEN: &str = "XSRF-TOKEN";
//     const X_XSRF_TOKEN: &str = "X-XSRF-TOKEN";

//     #[get("/403")]
//     fn error_403() -> Result<(), Status> {
//         Err(Status::Forbidden)
//     }

//     pub struct CsrfProtection;

//     impl Fairing for CsrfProtection {
//         fn info(&self) -> Info {
//             Info {
//                 name: "CSRF Protection",
//                 kind: Kind::Request | Kind::Response | Kind::Attach,
//             }
//         }

//         fn on_request(&self, request: &mut Request, _: &Data) {
//             match request.method() {
//                 Method::Post | Method::Put | Method::Patch | Method::Delete => {
//                     let result = !request
//                         .cookies()
//                         .get(XSRF_TOKEN)
//                         .map(Cookie::value)
//                         .and_then(|cookie_token| {
//                             request
//                                 .headers()
//                                 .get_one(X_XSRF_TOKEN)
//                                 .map(|header_token| cookie_token == header_token)
//                         })
//                         .unwrap_or(false);

//                     if result {
//                         // let new_uri = format!("/403#{}", &request.uri());
//                         request.set_uri(Origin::parse("/403").unwrap());
//                         request.set_method(Method::Get);
//                     }
//                 }
//                 _ => (),
//             }
//         }

//         fn on_response(&self, _request: &Request, response: &mut Response) {
//             if response.status().class() == StatusClass::Success {
//                 response.set_header(
//                     &Cookie::build(XSRF_TOKEN, Uuid::new_v4().to_string())
//                         .path("/")
//                         .finish(),
//                 );
//             }
//         }

//         fn on_attach(&self, rocket: Rocket) -> Result<Rocket, Rocket> {
//             Ok(rocket.mount("/", routes![error_403]))
//         }
//     }
// }

mod util {
    use serde::{Deserialize, Serialize};
    use std::str::FromStr;

    use super::Result;
    use super::WebFrontendError;

    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
    #[serde(tag = "type", rename_all = "lowercase")]
    pub enum ConfigParam {
        //Int {
        //name: String,
        //description: String,
        //default: i64,
        //value: i64,
        //},
        //Float {
        //name: String,
        //description: String,
        //default: f64,
        //value: f64,
        //},
        Bool {
            name: String,
            description: String,
            default: bool,
            value: bool,
        },
        //String {
        //name: String,
        //description: String,
        //default: String,
        //value: String,
        //},
        //Color {
        //name: String,
        //description: String,
        //default: u32,
        //value: u32,
        //},
    }

    pub trait ParseConfig {
        fn parse_config_param(&self, param: &str, val: &str) -> Result<ConfigParam>;
    }

    impl ParseConfig for Vec<ConfigParam> {
        fn parse_config_param(&self, param: &str, val: &str) -> Result<ConfigParam> {
            for p in self.iter() {
                match &p {
                    //ConfigParam::Int { name, .. } => {
                    //if name == param {
                    //let value =
                    //i64::from_str(&val).map_err(|_e| WebFrontendError::ParseParamError {})?;

                    //return Ok(ConfigParam::Int {
                    //name: name.to_string(),
                    //value,
                    //});
                    //}
                    //}

                    //ConfigParam::Float { name, .. } => {
                    //if name == param {
                    //let value =
                    //f64::from_str(&val).map_err(|_e| WebFrontendError::ParseParamError {})?;

                    //return Ok(ConfigParam::Float {
                    //name: name.to_string(),
                    //value,
                    //});
                    //}
                    //}
                    ConfigParam::Bool {
                        name,
                        description,
                        default,
                        ..
                    } => {
                        if name == param {
                            let value = bool::from_str(&val)
                                .map_err(|_e| WebFrontendError::ParseParamError {})?;

                            return Ok(ConfigParam::Bool {
                                name: name.to_string(),
                                description: description.to_string(),
                                default: default.clone(),
                                value,
                            });
                        }
                    } //ConfigParam::String { name, .. } => {
                      //if name == param {
                      //let value = val.to_owned();

                      //return Ok(ConfigParam::String {
                      //name: name.to_string(),
                      //value,
                      //});
                      //}
                      //}

                      //ConfigParam::Color { name, .. } => {
                      //if name == param {
                      //let value = u32::from_str_radix(&val[1..], 16)
                      //.map_err(|_e| WebFrontendError::ParseParamError {})?;

                      //return Ok(ConfigParam::Color {
                      //name: name.to_string(),
                      //value,
                      //});
                      //}
                      //}
                }
            }

            Err(WebFrontendError::ParseParamError {}.into())
        }
    }

    pub trait GetAttr {
        fn get_name(&self) -> &String;
        fn get_default(&self) -> String;
    }

    impl GetAttr for ConfigParam {
        fn get_name(&self) -> &String {
            match self {
                //ConfigParam::Int { ref name, .. } => name,

                //ConfigParam::Float { ref name, .. } => name,
                ConfigParam::Bool { ref name, .. } => name,
                //ConfigParam::String { ref name, .. } => name,

                //ConfigParam::Color { ref name, .. } => name,
            }
        }

        fn get_default(&self) -> String {
            match self {
                //ConfigParam::Int { ref default, .. } => format!("{}", default),

                //ConfigParam::Float { ref default, .. } => format!("{}", default),
                ConfigParam::Bool { ref default, .. } => format!("{}", default),
                //ConfigParam::String { ref default, .. } => default.to_owned(),

                //ConfigParam::Color { ref default, .. } => format!("#{:06x}", default),
            }
        }
    }
}
