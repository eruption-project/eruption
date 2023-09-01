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

use async_trait::async_trait;
use flume::Sender;
use lazy_static::lazy_static;
use parking_lot::{Mutex, RwLock};
use std::{
    collections::HashMap,
    env,
    fmt::Debug,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};
use wayland_client::{
    event_created_child,
    protocol::{wl_compositor, wl_registry},
    Connection, Dispatch, EventQueue, Proxy, QueueHandle,
};
use wayland_protocols_wlr::foreign_toplevel::v1::client::{
    zwlr_foreign_toplevel_handle_v1::{ZwlrForeignToplevelHandleV1, EVT_TITLE_OPCODE},
    zwlr_foreign_toplevel_manager_v1::ZwlrForeignToplevelManagerV1,
};

use crate::{constants, CONFIG, QUIT};

use super::{Sensor, SensorConfiguration, SENSORS_CONFIGURATION};

type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum WaylandSensorError {
    // #[error("Unknown error: {description}")]
    // UnknownError { description: String },
    #[error("Sensor error: {description}")]
    SensorError { description: String },

    #[error("Operation not supported")]
    NotSupported,
}

#[derive(Debug, Clone)]
pub struct WaylandSensorData {
    pub window_title: String,
    pub window_instance: String,
    pub window_class: String,
}

impl super::SensorData for WaylandSensorData {
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}

impl super::WindowSensorData for WaylandSensorData {
    fn window_name(&self) -> Option<&str> {
        Some(&self.window_title)
    }

    fn window_instance(&self) -> Option<&str> {
        Some(&self.window_instance)
    }

    fn window_class(&self) -> Option<&str> {
        Some(&self.window_class)
    }
}

/// Specifies whether we successfully connected to a Wayland compositor
pub static WAYLAND_CONNECTION_SUCCESSFULL: AtomicBool = AtomicBool::new(false);

lazy_static! {
    /// Events tx to the main thread
    static ref WAYLAND_TX: Arc<Mutex<Option<Sender<WaylandSensorData>>>> = Arc::new(Mutex::new(None));

    /// Holds the attributes of all tracked toplevels
    static ref WAYLAND_TOPLEVEL_WINDOWS: Arc<RwLock<HashMap<String, WaylandToplevelAttributes>>> = Arc::new(RwLock::new(HashMap::new()));

    /// Is the background ("root") window active?
    static ref WAYLAND_ROOT_WINDOW_ACTIVE: AtomicBool = AtomicBool::new(false);
}

#[derive(Debug, Clone, Default)]
pub struct WaylandToplevelAttributes {
    pub title: Option<String>,
    pub app_id: Option<String>,
    pub state: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct WaylandSensor {
    event_queue: Option<Arc<RwLock<EventQueue<AppData>>>>,
    is_failed: bool,
}

impl WaylandSensor {
    pub fn new() -> Self {
        match Connection::connect_to_env() {
            Ok(conn) => {
                let display = conn.display();

                let mut event_queue = conn.new_event_queue();
                let qh = event_queue.handle();

                let _registry = display.get_registry(&qh, ());

                let mut data = AppData::default();
                let _ = event_queue.roundtrip(&mut data);

                WAYLAND_CONNECTION_SUCCESSFULL.store(true, Ordering::SeqCst);

                Self {
                    event_queue: Some(Arc::new(RwLock::new(event_queue))),
                    is_failed: false,
                }
            }

            Err(e) => {
                tracing::debug!(
                    "Could not connect to the Wayland compositor from the current environment: {}",
                    e
                );

                tracing::debug!(
                    "Trying to establish a connection to Wayland via the configured parameters..."
                );

                let display = (*CONFIG.lock())
                    .as_ref()
                    .unwrap()
                    .get_string("Wayland.display")
                    .unwrap_or_else(|_| "wayland-0".to_string());

                let xdg_runtime_dir =
                    env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/run/user/1000".to_string());

                // let xdg_desktop_environment = (*CONFIG.lock())
                //     .as_ref()
                //     .unwrap()
                //     .get_string("Wayland.desktop_environment")
                //     .unwrap_or_default();

                env::set_var("WAYLAND_DISPLAY", display);
                env::set_var("XDG_RUNTIME_DIR", xdg_runtime_dir);

                // env::set_var("XDG_DESKTOP_ENVIRONMENT", xdg_desktop_environment);

                match Connection::connect_to_env() {
                    Ok(conn) => {
                        let display = conn.display();

                        let mut event_queue = conn.new_event_queue();
                        let qh = event_queue.handle();

                        let _registry = display.get_registry(&qh, ());

                        let mut data = AppData::default();
                        let _ = event_queue.roundtrip(&mut data);

                        WAYLAND_CONNECTION_SUCCESSFULL.store(true, Ordering::SeqCst);

                        Self {
                            event_queue: Some(Arc::new(RwLock::new(event_queue))),
                            is_failed: false,
                        }
                    }

                    Err(e) => {
                        tracing::error!(
                            "Could not connect to a Wayland compositor, giving up: {}",
                            e
                        );

                        Self {
                            event_queue: None,
                            is_failed: true,
                        }
                    }
                }
            }
        }
    }

    pub fn spawn_wayland_events_thread(
        &mut self,
        wayland_tx: Sender<WaylandSensorData>,
    ) -> Result<()> {
        *WAYLAND_TX.lock() = Some(wayland_tx);

        let event_queue = self.event_queue.clone();

        thread::Builder::new()
            .name("wayland-events".to_owned())
            .spawn(move || -> Result<()> {
                'WAYLAND_EVENTS_LOOP: loop {
                    // check if we shall terminate the thread
                    if QUIT.load(Ordering::SeqCst) {
                        break Ok(());
                    }

                    match &event_queue {
                        Some(event_queue) => {
                            let mut data = AppData::default();
                            event_queue.write().roundtrip(&mut data)?;
                        }

                        None => {
                            break 'WAYLAND_EVENTS_LOOP Err(WaylandSensorError::SensorError {
                                description: "Lost connection to the Wayland compositor".to_owned(),
                            }
                            .into())
                        }
                    }

                    thread::sleep(Duration::from_millis(constants::MAIN_LOOP_SLEEP_MILLIS));
                }
            })?;

        Ok(())
    }
}

#[async_trait]
impl Sensor for WaylandSensor {
    fn initialize(&mut self) -> Result<()> {
        Ok(())
    }

    fn is_enabled(&self) -> bool {
        SENSORS_CONFIGURATION
            .read()
            .contains(&SensorConfiguration::EnableWayland)
    }

    fn get_id(&self) -> String {
        "wayland".to_string()
    }

    fn get_name(&self) -> String {
        "Wayland".to_string()
    }

    fn get_description(&self) -> String {
        "Watches the state of windows on supported Wayland compositors".to_string()
    }

    fn get_usage_example(&self) -> String {
        r#"
Wayland:
rules add [window-class|window-class-instance] <regex> [<profile-name.profile>|<slot number>]

rules add window-class '.*YouTube.*Mozilla Firefox' /var/lib/eruption/profiles/profile1.profile
rules add window-instance gnome-calculator 2
"#
        .to_string()
    }

    fn is_failed(&self) -> bool {
        self.is_failed
    }

    fn set_failed(&mut self, failed: bool) {
        self.is_failed = failed;
    }

    fn is_pollable(&self) -> bool {
        false
    }

    fn poll(&mut self) -> Result<Box<dyn super::SensorData>> {
        Err(WaylandSensorError::NotSupported.into())
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

#[derive(Debug, Clone, Default)]
pub struct AppData {
    pub name: String,
}

impl Dispatch<wl_registry::WlRegistry, ()> for AppData {
    fn event(
        _: &mut Self,
        registry: &wl_registry::WlRegistry,
        event: wl_registry::Event,
        _: &(),
        _: &Connection,
        qh: &QueueHandle<Self>,
    ) {
        if let wl_registry::Event::Global {
            name,
            interface,
            version,
        } = event
        {
            tracing::trace!("Interface: {interface}");

            match &interface[..] {
                "zwlr_foreign_toplevel_manager_v1" => {
                    tracing::debug!("Registering: zwlr_foreign_toplevel_manager_v1");

                    let _manager =
                        registry.bind::<ZwlrForeignToplevelManagerV1, _, _>(name, version, qh, ());
                }

                "zwlr_foreign_toplevel_handle_v1" => {
                    tracing::debug!("Registering: zwlr_foreign_toplevel_handle_v1");

                    let _manager =
                        registry.bind::<ZwlrForeignToplevelHandleV1, _, _>(name, version, qh, ());
                }

                _ => { /* do nothing */ }
            }
        }
    }
}

impl Dispatch<wl_compositor::WlCompositor, ()> for AppData {
    fn event(
        _: &mut Self,
        _: &wl_compositor::WlCompositor,
        _: wl_compositor::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

impl Dispatch<ZwlrForeignToplevelManagerV1, ()> for AppData {
    fn event(
        _state: &mut Self,
        _proxy: &ZwlrForeignToplevelManagerV1,
        _event: <ZwlrForeignToplevelManagerV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
    }

    event_created_child!(AppData, ZwlrForeignToplevelHandleV1, [
        EVT_TITLE_OPCODE => (ZwlrForeignToplevelHandleV1, ())
    ]);
}

impl Dispatch<ZwlrForeignToplevelHandleV1, ()> for AppData {
    fn event(
        _state: &mut Self,
        proxy: &ZwlrForeignToplevelHandleV1,
        event: <ZwlrForeignToplevelHandleV1 as wayland_client::Proxy>::Event,
        _data: &(),
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
    ) {
        use wayland_protocols_wlr::foreign_toplevel::v1::client::zwlr_foreign_toplevel_handle_v1::Event;

        let object = proxy.id().to_string();

        tracing::trace!("[T]: {object} - {0}", _state.name);

        match event {
            Event::Title { title } => {
                tracing::trace!("{object}: Title: {title}");

                let _previous = WAYLAND_TOPLEVEL_WINDOWS
                    .write()
                    .entry(object)
                    .or_default()
                    .title
                    .replace(title);
            }

            Event::AppId { app_id } => {
                tracing::trace!("{object}: App_id: {app_id}");

                let _previous = WAYLAND_TOPLEVEL_WINDOWS
                    .write()
                    .entry(object)
                    .or_default()
                    .app_id
                    .replace(app_id);
            }

            Event::State { state } => {
                tracing::trace!("{object}: State: {state:#?}");

                if state.len() <= 4 {
                    WAYLAND_ROOT_WINDOW_ACTIVE.store(true, Ordering::SeqCst);
                } else {
                    WAYLAND_ROOT_WINDOW_ACTIVE.store(false, Ordering::SeqCst);
                }

                let _previous = WAYLAND_TOPLEVEL_WINDOWS
                    .write()
                    .entry(object)
                    .or_default()
                    .state
                    .replace(state);
            }

            Event::Done => {
                let windows = WAYLAND_TOPLEVEL_WINDOWS.read();
                let attributes = windows.get(&object);

                if let Some(attributes) = attributes {
                    if attributes.app_id.is_none()
                        && attributes.title.is_none()
                        && attributes.state.is_none()
                    {
                        tracing::warn!(
                            "Received a 'done' event for {object}, that has no associated state"
                        );
                    } else {
                        // 2 == active state
                        if attributes.state.as_ref().unwrap().iter().any(|&e| e == 2) {
                            tracing::debug!("Emitting event: {object}: {attributes:?}");

                            WAYLAND_TX
                                .lock()
                                .as_ref()
                                .unwrap()
                                .send(WaylandSensorData {
                                    window_title: attributes.clone().title.unwrap_or_default(),
                                    window_instance: attributes.clone().app_id.unwrap_or_default(),
                                    window_class: attributes.clone().app_id.unwrap_or_default(),
                                })
                                .unwrap_or_else(|e| {
                                    tracing::error!("Could not send on a channel: {}", e)
                                });
                        } else if WAYLAND_ROOT_WINDOW_ACTIVE.load(Ordering::SeqCst) {
                            // heuristics for detecting the "root" window
                            tracing::debug!("Root window heuristics matched");

                            tracing::debug!("Emitting event: {object}: {attributes:?}");

                            WAYLAND_TX
                                .lock()
                                .as_ref()
                                .unwrap()
                                .send(WaylandSensorData {
                                    window_title: "".to_string(),
                                    window_instance: "".to_string(),
                                    window_class: "".to_string(),
                                })
                                .unwrap_or_else(|e| {
                                    tracing::error!("Could not send on a channel: {}", e)
                                });
                        }
                    }
                } else {
                    tracing::error!("Received a 'done' event for previously untracked {object}");
                }
            }

            _ => { /* do nothing */ }
        }

        // let manager = (proxy as &dyn Proxy<Event = _, Request = _>)
        //     .bind::<ZwlrForeignToplevelManagerV1, _, _>(event.opcode(), 1, qh, ());
    }
}
