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

use dbus::strings::{Interface as FaceName, Path as PathName};
use dbus::{ffidisp::BusType, ffidisp::Connection, ffidisp::NameFlag};
use dbus_tree::{MTFn, MethodErr, MethodResult, Signal};
use flume::Sender;
use lazy_static::lazy_static;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tracing::*;

use self::convenience::TreeAdd;
use crate::hwdevices;

pub type Factory = dbus_tree::Factory<MTFn<()>, ()>;
pub type Interface = dbus_tree::Interface<MTFn<()>, ()>;
pub type Method = dbus_tree::Method<MTFn<()>, ()>;
pub type MethodInfo<'a> = dbus_tree::MethodInfo<'a, MTFn<()>, ()>;
pub type Property = dbus_tree::Property<MTFn<()>, ()>;
pub type PropertyInfo<'a> = dbus_tree::PropInfo<'a, MTFn<()>, ()>;
pub type PropertyResult = std::result::Result<(), MethodErr>;
pub type Tree = dbus_tree::Tree<MTFn<()>, ()>;

pub type Result<T> = std::result::Result<T, eyre::Error>;

mod bus;
mod canvas;
mod config;
mod convenience;
mod devices;
mod perms;
mod polkit;
mod profile;
mod slot;
mod status;

/// D-Bus messages and signals that are processed by the main thread
#[derive(Debug, Clone)]
pub enum Message {
    SwitchSlot(usize),
    SwitchProfile(PathBuf),
}

#[derive(Debug, thiserror::Error)]
pub enum DbusApiError {
    #[error("D-Bus not connected")]
    BusNotConnected {},

    #[error("Invalid device")]
    InvalidDevice {},

    #[error("Invalid device class")]
    InvalidDeviceClass {},

    #[error("Invalid parameter")]
    InvalidParameter {},

    #[error("Update of parameter value failed due to a synchronization error")]
    LockingFailed {},
    // #[error("Operation not supported")]
    // OpNotSupported {},
}

lazy_static! {
    static ref INTERFACE_ROOT: FaceName<'static> = "org.eruption".into();
    static ref CONFIG_PATH: PathName<'static> = "/org/eruption/config".into();
    static ref CONFIG_FACE: FaceName<'static> = "org.eruption.Config".into();
    static ref DEVICES_PATH: PathName<'static> = "/org/eruption/devices".into(); // plural
    static ref DEVICES_FACE: FaceName<'static> = "org.eruption.Device".into(); // singular
    static ref PROFILE_PATH: PathName<'static> = "/org/eruption/profile".into();
    static ref PROFILE_FACE: FaceName<'static> = "org.eruption.Profile".into();
    static ref SLOT_PATH: PathName<'static> = "/org/eruption/slot".into();
    static ref SLOT_FACE: FaceName<'static> = "org.eruption.Slot".into();
    static ref CANVAS_PATH: PathName<'static> = "/org/eruption/canvas".into();
    static ref CANVAS_FACE: FaceName<'static> = "org.eruption.Canvas".into();
    static ref STATUS_PATH: PathName<'static> = "/org/eruption/status".into();
    static ref STATUS_FACE: FaceName<'static> = "org.eruption.Status".into();
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DeviceStatus {
    pub index: u64,
    pub usb_vid: u16,
    pub usb_pid: u16,
    pub status: hwdevices::DeviceStatus,
}

/// D-Bus API support
pub struct DbusApi {
    connection: Option<Arc<Connection>>,

    active_slot_changed: Arc<Signal<()>>,
    active_profile_changed: Arc<Signal<()>>,
    profiles_changed: Arc<Signal<()>>,
    brightness_changed: Arc<Signal<()>>,

    hue_changed: Arc<Signal<()>>,
    saturation_changed: Arc<Signal<()>>,
    lightness_changed: Arc<Signal<()>>,

    device_status_changed: Arc<Signal<()>>,
    device_hotplug: Arc<Signal<()>>,
}

impl DbusApi {
    /// Initialize the D-Bus API
    pub fn new(dbus_tx: Sender<Message>) -> Result<Self> {
        let dbus_tx = Arc::new(dbus_tx);
        let conn = Arc::new(Connection::get_private(BusType::System)?);
        conn.register_name(&INTERFACE_ROOT, NameFlag::ReplaceExisting as u32)?;

        let f = Factory::new_fn::<()>();

        let canvas_interface = canvas::CanvasInterface::new(&f);
        let status_interface = status::StatusInterface::new();
        let devices_interface = devices::DevicesInterface::new(&f);
        let config_interface = config::ConfigInterface::new(&f);
        let slot_interface = slot::SlotInterface::new(&f, dbus_tx.clone(), conn.clone());
        let profile_interface = profile::ProfileInterface::new(&f, dbus_tx, conn.clone());

        let tree = f
            .tree(())
            .add_path_and_interface(&f, &CONFIG_PATH, &CONFIG_FACE, &config_interface)
            .add_path_and_interface(&f, &DEVICES_PATH, &DEVICES_FACE, &devices_interface)
            .add_path_and_interface(&f, &PROFILE_PATH, &PROFILE_FACE, &profile_interface)
            .add_path_and_interface(&f, &SLOT_PATH, &SLOT_FACE, &slot_interface)
            .add_path_and_interface(&f, &CANVAS_PATH, &CANVAS_FACE, &canvas_interface)
            .add_path_and_interface(&f, &STATUS_PATH, &STATUS_FACE, &status_interface);

        tree.set_registered(&conn, true)
            .unwrap_or_else(|e| error!("Could not register the tree: {}", e));
        conn.add_handler(tree);

        Ok(Self {
            connection: Some(conn),
            active_slot_changed: slot_interface.active_slot_changed_signal,
            active_profile_changed: profile_interface.active_profile_changed_signal,
            profiles_changed: profile_interface.profiles_changed_signal,
            brightness_changed: config_interface.brightness_changed_signal,
            hue_changed: canvas_interface.hue_changed_signal,
            saturation_changed: canvas_interface.saturation_changed_signal,
            lightness_changed: canvas_interface.lightness_changed_signal,
            device_status_changed: devices_interface.device_status_changed_signal,
            device_hotplug: devices_interface.device_hotplug_signal,
        })
    }

    pub fn notify_device_status_changed(&self) -> Result<()> {
        let device_status = &*crate::DEVICE_STATUS.as_ref().read();

        let device_status = device_status
            .iter()
            .map(|(k, v)| {
                let (usb_vid, usb_pid) = devices::get_device_specific_ids(*k).unwrap_or_default();

                DeviceStatus {
                    index: *k,
                    usb_vid,
                    usb_pid,
                    status: v.clone(),
                }
            })
            .collect::<Vec<DeviceStatus>>();

        let result = serde_json::to_string_pretty(&device_status)
            .map_err(|e| MethodErr::failed(&format!("{e}")))?;

        let _ = self
            .connection
            .as_ref()
            .unwrap()
            .send(
                self.device_status_changed
                    .emit(&DEVICES_PATH, &DEVICES_FACE, &[result]),
            )
            .map_err(|_| error!("D-Bus error during send call"));

        Ok(())
    }

    pub fn notify_device_hotplug(&self, device_info: (u16, u16), removed: bool) -> Result<()> {
        let _ = self
            .connection
            .as_ref()
            .unwrap()
            .send(self.device_hotplug.emit(
                &DEVICES_PATH,
                &DEVICES_FACE,
                &[(device_info.0, device_info.1, removed)],
            ))
            .map_err(|_| error!("D-Bus error during send call"));

        Ok(())
    }

    pub fn notify_brightness_changed(&self) -> Result<()> {
        let brightness = crate::BRIGHTNESS.load(Ordering::SeqCst);

        let _ = self
            .connection
            .as_ref()
            .unwrap()
            .send(
                self.brightness_changed
                    .emit(&CONFIG_PATH, &CONFIG_FACE, &[brightness as i64]),
            )
            .map_err(|_| error!("D-Bus error during send call"));

        Ok(())
    }

    pub fn notify_hue_changed(&self) -> Result<()> {
        let hue = crate::CANVAS_HSL.read().0;

        let _ = self
            .connection
            .as_ref()
            .unwrap()
            .send(self.hue_changed.emit(
                &"/org/eruption/canvas".into(),
                &"org.eruption.Canvas".into(),
                &[hue],
            ))
            .map_err(|_| error!("D-Bus error during send call"));

        Ok(())
    }

    pub fn notify_saturation_changed(&self) -> Result<()> {
        let saturation = crate::CANVAS_HSL.read().1;

        let _ = self
            .connection
            .as_ref()
            .unwrap()
            .send(self.saturation_changed.emit(
                &"/org/eruption/canvas".into(),
                &"org.eruption.Canvas".into(),
                &[saturation],
            ))
            .map_err(|_| error!("D-Bus error during send call"));

        Ok(())
    }

    pub fn notify_lightness_changed(&self) -> Result<()> {
        let lightness = crate::CANVAS_HSL.read().2;

        let _ = self
            .connection
            .as_ref()
            .unwrap()
            .send(self.lightness_changed.emit(
                &"/org/eruption/canvas".into(),
                &"org.eruption.Canvas".into(),
                &[lightness],
            ))
            .map_err(|_| error!("D-Bus error during send call"));

        Ok(())
    }

    pub fn notify_active_slot_changed(&self) -> Result<()> {
        let active_slot = crate::ACTIVE_SLOT.load(Ordering::SeqCst);

        let _ = self
            .connection
            .as_ref()
            .unwrap()
            .send(
                self.active_slot_changed
                    .emit(&SLOT_PATH, &SLOT_FACE, &[active_slot as u64]),
            )
            .map_err(|_| error!("D-Bus error during send call"));

        Ok(())
    }

    pub fn notify_active_profile_changed(&self) -> Result<()> {
        let active_profile = crate::ACTIVE_PROFILE.read();

        let active_profile = active_profile
            .as_ref()
            .unwrap()
            .profile_file
            .to_str()
            .unwrap();

        let _ = self
            .connection
            .as_ref()
            .unwrap()
            .send(
                self.active_profile_changed
                    .emit(&PROFILE_PATH, &PROFILE_FACE, &[active_profile]),
            )
            .map_err(|_| error!("D-Bus error during send call"));

        Ok(())
    }

    pub fn notify_profiles_changed(&self) -> Result<()> {
        let _ = self
            .connection
            .as_ref()
            .unwrap()
            .send(self.profiles_changed.msg(&PROFILE_PATH, &PROFILE_FACE))
            .map_err(|_| error!("D-Bus error during send call"));

        Ok(())
    }

    /// Returns true if an event is pending on the D-Bus connection
    #[allow(dead_code)]
    pub fn has_pending_event(&self) -> Result<bool> {
        match self.connection {
            Some(ref connection) => {
                let count = connection.incoming(0).peekable().count();

                if count > 0 {
                    Ok(true)
                } else {
                    Ok(false)
                }
            }

            None => Err(DbusApiError::BusNotConnected {}.into()),
        }
    }

    /// Get the next event from D-Bus
    #[allow(dead_code)]
    pub fn get_next_event(&self) -> Result<bool> {
        match self.connection {
            Some(ref connection) => {
                if let Some(item) = connection.incoming(0).next() {
                    // For the actual event handler code please see
                    // implementation of `struct DbusApi`
                    debug!("Message: {:?}", item);

                    Ok(true)
                } else {
                    trace!("Received a timeout message");

                    Ok(false)
                }
            }

            None => Err(DbusApiError::BusNotConnected {}.into()),
        }
    }

    pub fn get_next_event_timeout(&self, timeout_ms: u32) -> Result<bool> {
        match self.connection {
            Some(ref connection) => {
                if let Some(item) = connection.incoming(timeout_ms).next() {
                    // For the actual event handler code please see
                    // implementation of `struct DbusApi`
                    debug!("Message: {:?}", item);

                    Ok(true)
                } else {
                    trace!("Received a timeout message");

                    Ok(false)
                }
            }

            None => Err(DbusApiError::BusNotConnected {}.into()),
        }
    }
}
