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

use crossbeam::channel::Sender;
use dbus::{ffidisp::BusType, ffidisp::Connection, ffidisp::NameFlag, message::SignalArgs};
use dbus_tree::{
    Access, MethodErr, Signal, {EmitsChangedSignal, Factory},
};
use log::*;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::plugins::audio;
use crate::profiles;
use crate::script;
use crate::CONFIG;
use crate::{constants, plugins};

/// D-Bus messages and signals that are processed by the main thread
#[derive(Debug, Clone)]
pub enum Message {
    SwitchSlot(usize),
    SwitchProfile(PathBuf),
}

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum DbusApiError {
    #[error("D-Bus not connected")]
    BusNotConnected {},
}

/// D-Bus API support
pub struct DbusApi {
    connection: Option<Arc<Connection>>,

    active_slot_changed: Arc<Signal<()>>,
    active_profile_changed: Arc<Signal<()>>,
    profiles_changed: Arc<Signal<()>>,
    brightness_changed: Arc<Signal<()>>,
}

#[allow(dead_code)]
impl DbusApi {
    /// Initialize the D-Bus API
    pub fn new(dbus_tx: Sender<Message>) -> Result<Self> {
        let dbus_tx_clone = dbus_tx.clone();

        let c = Connection::get_private(BusType::System)?;
        c.register_name("org.eruption", NameFlag::ReplaceExisting as u32)?;

        let c_clone = Arc::new(c);
        let c_clone2 = c_clone.clone();
        let c_clone3 = c_clone.clone();

        let f = Factory::new_fn::<()>();

        let active_slot_changed_signal =
            Arc::new(f.signal("ActiveSlotChanged", ()).sarg::<u64, _>("new slot"));
        let active_slot_changed_signal_clone = active_slot_changed_signal.clone();

        // let slot_names_changed_signal = Arc::new(
        //     f.signal("SlotNamesChanged", ())
        //         .sarg::<String, _>("new slot names"),
        // );
        // let slot_names_changed_signal_clone = slot_names_changed_signal.clone();

        let active_profile_changed_signal = Arc::new(
            f.signal("ActiveProfileChanged", ())
                .sarg::<String, _>("new profile name"),
        );
        let active_profile_changed_signal_clone = active_profile_changed_signal.clone();

        let profiles_changed_signal = Arc::new(f.signal("ProfilesChanged", ()));
        let profiles_changed_signal_clone = profiles_changed_signal.clone();

        let brightness_changed_signal = Arc::new(
            f.signal("BrightnessChanged", ())
                .sarg::<i64, _>("current brightness"),
        );
        let brightness_changed_signal_clone = brightness_changed_signal.clone();

        let active_slot_property = f
            .property::<u64, _>("ActiveSlot", ())
            .emits_changed(EmitsChangedSignal::Const)
            .on_get(|i, m| {
                if perms::has_monitor_permission(&m.msg.sender().unwrap().to_string())
                    .unwrap_or(false)
                {
                    let result = crate::ACTIVE_SLOT.load(Ordering::SeqCst) as u64;
                    i.append(result);

                    Ok(())
                } else {
                    Err(MethodErr::failed("Authentication failed"))
                }
            });

        let active_slot_property_clone = Arc::new(active_slot_property);

        let active_profile_property = f
            .property::<String, _>("ActiveProfile", ())
            .emits_changed(EmitsChangedSignal::Const)
            .on_get(|i, m| {
                if perms::has_monitor_permission(&m.msg.sender().unwrap().to_string())
                    .unwrap_or(false)
                {
                    let result = crate::ACTIVE_PROFILE.lock();

                    result
                        .as_ref()
                        .and_then(|p| {
                            p.profile_file.file_name().map(|v| {
                                i.append(&*v.to_string_lossy());
                            })
                        })
                        .ok_or_else(|| MethodErr::failed("Method failed"))
                } else {
                    Err(MethodErr::failed("Authentication failed"))
                }
            });

        let active_profile_property_clone = Arc::new(active_profile_property);

        let enable_sfx_property = f
            .property::<bool, _>("EnableSfx", ())
            .emits_changed(EmitsChangedSignal::True)
            .access(Access::ReadWrite)
            .auto_emit_on_set(true)
            .on_get(|i, m| {
                if perms::has_monitor_permission(&m.msg.sender().unwrap().to_string())
                    .unwrap_or(false)
                {
                    i.append(audio::ENABLE_SFX.load(Ordering::SeqCst));

                    Ok(())
                } else {
                    Err(MethodErr::failed("Authentication failed"))
                }
            })
            .on_set(|i, m| {
                if perms::has_settings_permission(&m.msg.sender().unwrap().to_string())
                    .unwrap_or(false)
                {
                    audio::ENABLE_SFX.store(i.read::<bool>()?, Ordering::SeqCst);

                    Ok(())
                } else {
                    Err(MethodErr::failed("Authentication failed"))
                }
            });

        let enable_sfx_property_clone = Arc::new(enable_sfx_property);

        let brightness_property = f
            .property::<i64, _>("Brightness", ())
            .emits_changed(EmitsChangedSignal::True)
            .access(Access::ReadWrite)
            .auto_emit_on_set(true)
            .on_get(|i, m| {
                if perms::has_monitor_permission(&m.msg.sender().unwrap().to_string())
                    .unwrap_or(false)
                {
                    let result = crate::BRIGHTNESS.load(Ordering::SeqCst) as i64;
                    i.append(result);

                    Ok(())
                } else {
                    Err(MethodErr::failed("Authentication failed"))
                }
            })
            .on_set(|i, m| {
                if perms::has_settings_permission(&m.msg.sender().unwrap().to_string())
                    .unwrap_or(false)
                {
                    crate::BRIGHTNESS.store(i.read::<i64>()? as isize, Ordering::SeqCst);
                    script::FRAME_GENERATION_COUNTER.fetch_add(1, Ordering::SeqCst);

                    Ok(())
                } else {
                    Err(MethodErr::failed("Authentication failed"))
                }
            });

        let brightness_property_clone = Arc::new(brightness_property);

        let tree = f
            .tree(())
            .add(
                f.object_path("/org/eruption/status", ())
                    .introspectable()
                    .add(
                        f.interface("org.eruption.Status", ())
                            .add_p(
                                f.property::<bool, _>("Running", ())
                                    .emits_changed(EmitsChangedSignal::True)
                                    .on_get(|i, m| {
                                        if perms::has_monitor_permission(
                                            &m.msg.sender().unwrap().to_string(),
                                        )
                                        .unwrap_or(false)
                                        {
                                            i.append(true);
                                            Ok(())
                                        } else {
                                            Err(MethodErr::failed("Authentication failed"))
                                        }
                                    })
                                    .on_set(|i, m| {
                                        if perms::has_settings_permission(
                                            &m.msg.sender().unwrap().to_string(),
                                        )
                                        .unwrap_or(false)
                                        {
                                            let _b: bool = i.read()?;

                                            //TODO: Implement this
                                            warn!("Not implemented");

                                            Ok(())
                                        } else {
                                            Err(MethodErr::failed("Authentication failed"))
                                        }
                                    }),
                            )
                            .add_m(
                                f.method("GetLedColors", (), move |m| {
                                    if perms::has_monitor_permission(
                                        &m.msg.sender().unwrap().to_string(),
                                    )
                                    .unwrap_or(false)
                                    {
                                        let s = script::LAST_RENDERED_LED_MAP
                                            .read()
                                            .iter()
                                            .map(|v| (v.r, v.g, v.b, v.a))
                                            .collect::<Vec<(u8, u8, u8, u8)>>();

                                        Ok(vec![m.msg.method_return().append1(s)])
                                    } else {
                                        Err(MethodErr::failed("Authentication failed"))
                                    }
                                })
                                .outarg::<Vec<(u8, u8, u8, u8)>, _>("values"),
                            )
                            // .add_m(
                            //     f.method("SetLedColors", (), move |m| {
                            //         *crate::LAST_DBUS_EVENT_TIME.lock() = Instant::now();
                            //         let s = *script::LED_MAP.read();
                            //         Ok(vec![m.msg.method_return().append_all(s)])
                            //     }), // .outarg::<Vec<RGBA>, _>("values"),
                            // )
                            .add_m(
                                f.method("GetManagedDevices", (), move |m| {
                                    if perms::has_monitor_permission(
                                        &m.msg.sender().unwrap().to_string(),
                                    )
                                    .unwrap_or(false)
                                    {
                                        let mut s: Vec<(u16, u16)> = Vec::new();

                                        s.extend(crate::KEYBOARD_DEVICES.lock().iter().map(
                                            |device| {
                                                (
                                                    device.read().get_usb_vid(),
                                                    device.read().get_usb_pid(),
                                                )
                                            },
                                        ));

                                        s.extend(crate::MOUSE_DEVICES.lock().iter().map(
                                            |device| {
                                                (
                                                    device.read().get_usb_vid(),
                                                    device.read().get_usb_pid(),
                                                )
                                            },
                                        ));

                                        Ok(vec![m.msg.method_return().append1(s)])
                                    } else {
                                        Err(MethodErr::failed("Authentication failed"))
                                    }
                                })
                                .outarg::<Vec<(u16, u16)>, _>("values"),
                            ),
                    ),
            )
            .add(
                f.object_path("/org/eruption/config", ())
                    .introspectable()
                    .add(
                        f.interface("org.eruption.Config", ())
                            .add_s(brightness_changed_signal_clone)
                            .add_p(enable_sfx_property_clone)
                            .add_p(brightness_property_clone)
                            .add_m(
                                f.method("WriteFile", (), move |m| {
                                    if perms::has_manage_permission(
                                        &m.msg.sender().unwrap().to_string(),
                                    )
                                    .unwrap_or(false)
                                    {
                                        let s = true;
                                        Ok(vec![m.msg.method_return().append1(s)])
                                    } else {
                                        Err(MethodErr::failed("Authentication failed"))
                                    }
                                })
                                .inarg::<String, _>("filename")
                                .inarg::<String, _>("data")
                                .outarg::<bool, _>("status"),
                            ),
                    ),
            )
            .add(
                f.object_path("/org/eruption/slot", ())
                    .introspectable()
                    .add(
                        f.interface("org.eruption.Slot", ())
                            .add_s(active_slot_changed_signal_clone)
                            .add_p(active_slot_property_clone.clone())
                            .add_m(
                                f.method("SwitchSlot", (), move |m| {
                                    if perms::has_settings_permission(
                                        &m.msg.sender().unwrap().to_string(),
                                    )
                                    .unwrap_or(false)
                                    {
                                        let n: u64 = m.msg.read1()?;

                                        if n as usize >= constants::NUM_SLOTS {
                                            Err(MethodErr::failed("Slot index out of bounds"))
                                        } else {
                                            dbus_tx
                                                .send(Message::SwitchSlot(n as usize))
                                                .unwrap_or_else(|e| {
                                                    error!(
                                                        "Could not send a pending D-Bus event: {}",
                                                        e
                                                    )
                                                });

                                            // reset the audio backend, it will be enabled again if needed
                                            plugins::audio::reset_audio_backend();

                                            let mut changed_properties = Vec::new();
                                            active_slot_property_clone.add_propertieschanged(
                                                &mut changed_properties,
                                                &"org.eruption".into(),
                                                || Box::new(n),
                                            );
                                            if !changed_properties.is_empty() {
                                                let msg = changed_properties
                                                    .first()
                                                    .unwrap()
                                                    .to_emit_message(&"/org/eruption/slot".into());
                                                c_clone2.clone().send(msg).unwrap();
                                            }
                                            let s = true;
                                            Ok(vec![m.msg.method_return().append1(s)])
                                        }
                                    } else {
                                        Err(MethodErr::failed("Authentication failed"))
                                    }
                                })
                                .inarg::<u64, _>("slot")
                                .outarg::<bool, _>("status"),
                            )
                            .add_m(
                                f.method("GetSlotProfiles", (), move |m| {
                                    if perms::has_monitor_permission(
                                        &m.msg.sender().unwrap().to_string(),
                                    )
                                    .unwrap_or(false)
                                    {
                                        let s: Vec<String> = crate::SLOT_PROFILES
                                            .lock()
                                            .as_ref()
                                            .unwrap()
                                            .iter()
                                            .map(|p| p.to_string_lossy().to_string())
                                            .collect();

                                        Ok(vec![m.msg.method_return().append1(s)])
                                    } else {
                                        Err(MethodErr::failed("Authentication failed"))
                                    }
                                })
                                .outarg::<Vec<String>, _>("values"),
                            )
                            .add_p(
                                f.property::<Vec<String>, _>("SlotNames", ())
                                    .access(Access::ReadWrite)
                                    .emits_changed(EmitsChangedSignal::True)
                                    .auto_emit_on_set(true)
                                    .on_get(|i, m| {
                                        if perms::has_monitor_permission(
                                            &m.msg.sender().unwrap().to_string(),
                                        )
                                        .unwrap_or(false)
                                        {
                                            let s = crate::SLOT_NAMES.lock();
                                            i.append(&*s);

                                            Ok(())
                                        } else {
                                            Err(MethodErr::failed("Authentication failed"))
                                        }
                                    })
                                    .on_set(|i, m| {
                                        if perms::has_settings_permission(
                                            &m.msg.sender().unwrap().to_string(),
                                        )
                                        .unwrap_or(false)
                                        {
                                            let n: Vec<String> = i.read()?;

                                            if n.len() >= constants::NUM_SLOTS {
                                                *crate::SLOT_NAMES.lock() = n;

                                                Ok(())
                                            } else {
                                                Err(MethodErr::failed("Invalid number of elements"))
                                            }
                                        } else {
                                            Err(MethodErr::failed("Authentication failed"))
                                        }
                                    }),
                            ),
                    ),
            )
            .add(
                f.object_path("/org/eruption/profile", ())
                    .introspectable()
                    .add(
                        f.interface("org.eruption.Profile", ())
                            .add_s(profiles_changed_signal_clone)
                            .add_s(active_profile_changed_signal_clone)
                            .add_p(active_profile_property_clone.clone())
                            .add_m(
                                f.method("SwitchProfile", (), move |m| {
                                    if perms::has_settings_permission(
                                        &m.msg.sender().unwrap().to_string(),
                                    )
                                    .unwrap_or(false)
                                    {
                                        let n: &str = m.msg.read1()?;

                                        dbus_tx_clone
                                            .send(Message::SwitchProfile(PathBuf::from(n)))
                                            .unwrap_or_else(|e| {
                                                error!(
                                                    "Could not send a pending D-Bus event: {}",
                                                    e
                                                )
                                            });

                                        // reset the audio backend, it will be enabled again if needed
                                        plugins::audio::reset_audio_backend();

                                        let mut changed_properties = Vec::new();
                                        active_profile_property_clone.add_propertieschanged(
                                            &mut changed_properties,
                                            &"org.eruption".into(),
                                            || Box::new(n.to_owned()),
                                        );

                                        if !changed_properties.is_empty() {
                                            let msg = changed_properties
                                                .first()
                                                .unwrap()
                                                .to_emit_message(&"/org/eruption/profile".into());
                                            c_clone3.clone().send(msg).unwrap();
                                        }

                                        let s = true;
                                        Ok(vec![m.msg.method_return().append1(s)])
                                    } else {
                                        Err(MethodErr::failed("Authentication failed"))
                                    }
                                })
                                .inarg::<&str, _>("filename")
                                .outarg::<bool, _>("status"),
                            )
                            .add_m(
                                f.method("EnumProfiles", (), move |m| {
                                    if perms::has_monitor_permission(
                                        &m.msg.sender().unwrap().to_string(),
                                    )
                                    .unwrap_or(false)
                                    {
                                        let profile_dir = PathBuf::from(
                                            CONFIG
                                                .lock()
                                                .as_ref()
                                                .unwrap()
                                                .get_str("global.profile_dir")
                                                .unwrap_or_else(|_| {
                                                    constants::DEFAULT_PROFILE_DIR.to_string()
                                                }),
                                        );

                                        let mut s: Vec<(String, String)> =
                                            profiles::get_profiles(&profile_dir)
                                                .unwrap()
                                                .iter()
                                                .map(|profile| {
                                                    (
                                                        profile.name.clone(),
                                                        profile
                                                            .profile_file
                                                            .file_name()
                                                            .unwrap()
                                                            .to_string_lossy()
                                                            .to_string(),
                                                    )
                                                })
                                                .collect();

                                        s.sort_by(|lhs, rhs| lhs.0.cmp(&rhs.0));

                                        Ok(vec![m.msg.method_return().append1(s)])
                                    } else {
                                        Err(MethodErr::failed("Authentication failed"))
                                    }
                                })
                                .outarg::<Vec<(String, String)>, _>("profiles"),
                            ),
                    ),
            );

        tree.set_registered(&*c_clone, true)
            .unwrap_or_else(|e| error!("Could not register the tree: {}", e));
        c_clone.add_handler(tree);

        Ok(Self {
            connection: Some(c_clone),
            active_slot_changed: active_slot_changed_signal,
            active_profile_changed: active_profile_changed_signal,
            profiles_changed: profiles_changed_signal,
            brightness_changed: brightness_changed_signal,
        })
    }

    pub fn notify_brightness_changed(&self) {
        let brightness = crate::BRIGHTNESS.load(Ordering::SeqCst);

        self.connection
            .as_ref()
            .unwrap()
            .send(self.brightness_changed.emit(
                &"/org/eruption/config".into(),
                &"org.eruption.Config".into(),
                &[brightness as i64],
            ))
            .unwrap();
    }

    pub fn notify_active_slot_changed(&self) {
        let active_slot = crate::ACTIVE_SLOT.load(Ordering::SeqCst);

        self.connection
            .as_ref()
            .unwrap()
            .send(self.active_slot_changed.emit(
                &"/org/eruption/slot".into(),
                &"org.eruption.Slot".into(),
                &[active_slot as u64],
            ))
            .unwrap();
    }

    pub fn notify_active_profile_changed(&self) {
        let active_profile = crate::ACTIVE_PROFILE.lock();

        let active_profile = active_profile
            .as_ref()
            .unwrap()
            .profile_file
            .file_name()
            .unwrap()
            .to_str()
            .unwrap();

        self.connection
            .as_ref()
            .unwrap()
            .send(self.active_profile_changed.emit(
                &"/org/eruption/profile".into(),
                &"org.eruption.Profile".into(),
                &[active_profile],
            ))
            .unwrap();
    }

    pub fn notify_profiles_changed(&self) {
        self.connection
            .as_ref()
            .unwrap()
            .send(self.profiles_changed.msg(
                &"/org/eruption/profile".into(),
                &"org.eruption.Profile".into(),
            ))
            .unwrap();
    }

    /// Returns true if an event is pending on the D-Bus connection
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

/// Initialize the Eruption D-Bus API support
pub fn initialize(dbus_tx: Sender<Message>) -> Result<DbusApi> {
    DbusApi::new(dbus_tx)
}

mod perms {
    use dbus::{arg::RefArg, arg::Variant, blocking::Connection};
    use std::{collections::HashMap, time::Duration};

    use crate::constants;

    pub type Result<T> = std::result::Result<T, eyre::Error>;

    pub fn has_monitor_permission(sender: &str) -> Result<bool> {
        use bus::OrgFreedesktopDBus;
        use polkit::OrgFreedesktopPolicyKit1Authority;

        let conn = Connection::new_system().unwrap();

        let dbus_proxy = conn.with_proxy(
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus/Bus",
            Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
        );

        let pid: u32 = dbus_proxy.get_connection_unix_process_id(sender)?;
        let uid: u32 = dbus_proxy.get_connection_unix_user(sender)?;

        let polkit_proxy = conn.with_proxy(
            "org.freedesktop.PolicyKit1",
            "/org/freedesktop/PolicyKit1/Authority",
            Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS_INTERACTIVE as u64),
        );

        let result = 'AUTH_LOOP: loop {
            let mut map = HashMap::new();
            map.insert("pid", Variant(Box::new(pid) as Box<dyn RefArg>));
            map.insert("start-time", Variant(Box::new(0 as u64) as Box<dyn RefArg>));
            map.insert("uid", Variant(Box::new(uid) as Box<dyn RefArg>));

            let mut details = HashMap::new();
            details.insert("AllowUserInteraction", "true");
            // details.insert("polkit.Message", "Authenticate");
            // details.insert("polkit.icon_name", "keyboard");

            let result = polkit_proxy.check_authorization(
                ("unix-process", map),
                "org.eruption.monitor",
                details,
                1,
                "",
            )?;

            let dismissed = result.2.get("polkit.dismissed").is_some();

            if (result.0 && !dismissed) || (!result.0 && dismissed) {
                // we have either been dismissed with 'cancel' or the authentication succeeded
                break 'AUTH_LOOP result;
            }
        };

        Ok(result.0)
    }

    pub fn has_settings_permission(sender: &str) -> Result<bool> {
        use bus::OrgFreedesktopDBus;
        use polkit::OrgFreedesktopPolicyKit1Authority;

        let conn = Connection::new_system().unwrap();

        let dbus_proxy = conn.with_proxy(
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus/Bus",
            Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
        );

        let pid: u32 = dbus_proxy.get_connection_unix_process_id(sender)?;
        let uid: u32 = dbus_proxy.get_connection_unix_user(sender)?;

        let polkit_proxy = conn.with_proxy(
            "org.freedesktop.PolicyKit1",
            "/org/freedesktop/PolicyKit1/Authority",
            Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS_INTERACTIVE as u64),
        );

        let result = 'AUTH_LOOP: loop {
            let mut map = HashMap::new();
            map.insert("pid", Variant(Box::new(pid) as Box<dyn RefArg>));
            map.insert("start-time", Variant(Box::new(0 as u64) as Box<dyn RefArg>));
            map.insert("uid", Variant(Box::new(uid) as Box<dyn RefArg>));

            let mut details = HashMap::new();
            details.insert("AllowUserInteraction", "true");
            // details.insert("polkit.Message", "Authenticate");
            // details.insert("polkit.icon_name", "keyboard");

            let result = polkit_proxy.check_authorization(
                ("unix-process", map),
                "org.eruption.settings",
                details,
                1,
                "",
            )?;

            let dismissed = result.2.get("polkit.dismissed").is_some();

            if (result.0 && !dismissed) || (!result.0 && dismissed) {
                // we have either been dismissed with 'cancel' or the authentication succeeded
                break 'AUTH_LOOP result;
            }
        };

        Ok(result.0)
    }

    pub fn has_manage_permission(sender: &str) -> Result<bool> {
        use bus::OrgFreedesktopDBus;
        use polkit::OrgFreedesktopPolicyKit1Authority;

        let conn = Connection::new_system().unwrap();

        let dbus_proxy = conn.with_proxy(
            "org.freedesktop.DBus",
            "/org/freedesktop/DBus/Bus",
            Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
        );

        let pid: u32 = dbus_proxy.get_connection_unix_process_id(sender)?;
        let uid: u32 = dbus_proxy.get_connection_unix_user(sender)?;

        let polkit_proxy = conn.with_proxy(
            "org.freedesktop.PolicyKit1",
            "/org/freedesktop/PolicyKit1/Authority",
            Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS_INTERACTIVE as u64),
        );

        let result = 'AUTH_LOOP: loop {
            let mut map = HashMap::new();
            map.insert("pid", Variant(Box::new(pid) as Box<dyn RefArg>));
            map.insert("start-time", Variant(Box::new(0 as u64) as Box<dyn RefArg>));
            map.insert("uid", Variant(Box::new(uid) as Box<dyn RefArg>));

            let mut details = HashMap::new();
            details.insert("AllowUserInteraction", "true");
            // details.insert("polkit.Message", "Authenticate");
            // details.insert("polkit.icon_name", "keyboard");

            let result = polkit_proxy.check_authorization(
                ("unix-process", map),
                "org.eruption.manage",
                details,
                1,
                "",
            )?;

            let dismissed = result.2.get("polkit.dismissed").is_some();

            if (result.0 && !dismissed) || (!result.0 && dismissed) {
                // we have either been dismissed with 'cancel' or the authentication succeeded
                break 'AUTH_LOOP result;
            }
        };

        Ok(result.0)
    }

    mod bus {
        // This code was autogenerated with `dbus-codegen-rust -s -d org.freedesktop.DBus -p /org/freedesktop/DBus/Bus -m None`, see https://github.com/diwic/dbus-rs
        use dbus;
        use dbus::arg;
        use dbus::blocking;

        pub trait OrgFreedesktopDBus {
            fn hello(&self) -> Result<String, dbus::Error>;
            fn request_name(&self, arg0: &str, arg1: u32) -> Result<u32, dbus::Error>;
            fn release_name(&self, arg0: &str) -> Result<u32, dbus::Error>;
            fn start_service_by_name(&self, arg0: &str, arg1: u32) -> Result<u32, dbus::Error>;
            fn update_activation_environment(
                &self,
                arg0: ::std::collections::HashMap<&str, &str>,
            ) -> Result<(), dbus::Error>;
            fn name_has_owner(&self, arg0: &str) -> Result<bool, dbus::Error>;
            fn list_names(&self) -> Result<Vec<String>, dbus::Error>;
            fn list_activatable_names(&self) -> Result<Vec<String>, dbus::Error>;
            fn add_match(&self, arg0: &str) -> Result<(), dbus::Error>;
            fn remove_match(&self, arg0: &str) -> Result<(), dbus::Error>;
            fn get_name_owner(&self, arg0: &str) -> Result<String, dbus::Error>;
            fn list_queued_owners(&self, arg0: &str) -> Result<Vec<String>, dbus::Error>;
            fn get_connection_unix_user(&self, arg0: &str) -> Result<u32, dbus::Error>;
            fn get_connection_unix_process_id(&self, arg0: &str) -> Result<u32, dbus::Error>;
            fn get_adt_audit_session_data(&self, arg0: &str) -> Result<Vec<u8>, dbus::Error>;
            fn get_connection_selinux_security_context(
                &self,
                arg0: &str,
            ) -> Result<Vec<u8>, dbus::Error>;
            fn reload_config(&self) -> Result<(), dbus::Error>;
            fn get_id(&self) -> Result<String, dbus::Error>;
            fn get_connection_credentials(
                &self,
                arg0: &str,
            ) -> Result<
                ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
                dbus::Error,
            >;
            fn features(&self) -> Result<Vec<String>, dbus::Error>;
            fn interfaces(&self) -> Result<Vec<String>, dbus::Error>;
        }

        impl<'a, C: ::std::ops::Deref<Target = blocking::Connection>> OrgFreedesktopDBus
            for blocking::Proxy<'a, C>
        {
            fn hello(&self) -> Result<String, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "Hello", ())
                    .and_then(|r: (String,)| Ok(r.0))
            }

            fn request_name(&self, arg0: &str, arg1: u32) -> Result<u32, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "RequestName", (arg0, arg1))
                    .and_then(|r: (u32,)| Ok(r.0))
            }

            fn release_name(&self, arg0: &str) -> Result<u32, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "ReleaseName", (arg0,))
                    .and_then(|r: (u32,)| Ok(r.0))
            }

            fn start_service_by_name(&self, arg0: &str, arg1: u32) -> Result<u32, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "StartServiceByName", (arg0, arg1))
                    .and_then(|r: (u32,)| Ok(r.0))
            }

            fn update_activation_environment(
                &self,
                arg0: ::std::collections::HashMap<&str, &str>,
            ) -> Result<(), dbus::Error> {
                self.method_call(
                    "org.freedesktop.DBus",
                    "UpdateActivationEnvironment",
                    (arg0,),
                )
            }

            fn name_has_owner(&self, arg0: &str) -> Result<bool, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "NameHasOwner", (arg0,))
                    .and_then(|r: (bool,)| Ok(r.0))
            }

            fn list_names(&self) -> Result<Vec<String>, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "ListNames", ())
                    .and_then(|r: (Vec<String>,)| Ok(r.0))
            }

            fn list_activatable_names(&self) -> Result<Vec<String>, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "ListActivatableNames", ())
                    .and_then(|r: (Vec<String>,)| Ok(r.0))
            }

            fn add_match(&self, arg0: &str) -> Result<(), dbus::Error> {
                self.method_call("org.freedesktop.DBus", "AddMatch", (arg0,))
            }

            fn remove_match(&self, arg0: &str) -> Result<(), dbus::Error> {
                self.method_call("org.freedesktop.DBus", "RemoveMatch", (arg0,))
            }

            fn get_name_owner(&self, arg0: &str) -> Result<String, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "GetNameOwner", (arg0,))
                    .and_then(|r: (String,)| Ok(r.0))
            }

            fn list_queued_owners(&self, arg0: &str) -> Result<Vec<String>, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "ListQueuedOwners", (arg0,))
                    .and_then(|r: (Vec<String>,)| Ok(r.0))
            }

            fn get_connection_unix_user(&self, arg0: &str) -> Result<u32, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "GetConnectionUnixUser", (arg0,))
                    .and_then(|r: (u32,)| Ok(r.0))
            }

            fn get_connection_unix_process_id(&self, arg0: &str) -> Result<u32, dbus::Error> {
                self.method_call(
                    "org.freedesktop.DBus",
                    "GetConnectionUnixProcessID",
                    (arg0,),
                )
                .and_then(|r: (u32,)| Ok(r.0))
            }

            fn get_adt_audit_session_data(&self, arg0: &str) -> Result<Vec<u8>, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "GetAdtAuditSessionData", (arg0,))
                    .and_then(|r: (Vec<u8>,)| Ok(r.0))
            }

            fn get_connection_selinux_security_context(
                &self,
                arg0: &str,
            ) -> Result<Vec<u8>, dbus::Error> {
                self.method_call(
                    "org.freedesktop.DBus",
                    "GetConnectionSELinuxSecurityContext",
                    (arg0,),
                )
                .and_then(|r: (Vec<u8>,)| Ok(r.0))
            }

            fn reload_config(&self) -> Result<(), dbus::Error> {
                self.method_call("org.freedesktop.DBus", "ReloadConfig", ())
            }

            fn get_id(&self) -> Result<String, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "GetId", ())
                    .and_then(|r: (String,)| Ok(r.0))
            }

            fn get_connection_credentials(
                &self,
                arg0: &str,
            ) -> Result<
                ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
                dbus::Error,
            > {
                self.method_call("org.freedesktop.DBus", "GetConnectionCredentials", (arg0,))
                    .and_then(
                        |r: (
                            ::std::collections::HashMap<
                                String,
                                arg::Variant<Box<dyn arg::RefArg + 'static>>,
                            >,
                        )| Ok(r.0),
                    )
            }

            fn features(&self) -> Result<Vec<String>, dbus::Error> {
                <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                    &self,
                    "org.freedesktop.DBus",
                    "Features",
                )
            }

            fn interfaces(&self) -> Result<Vec<String>, dbus::Error> {
                <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                    &self,
                    "org.freedesktop.DBus",
                    "Interfaces",
                )
            }
        }

        #[derive(Debug)]
        pub struct OrgFreedesktopDBusNameOwnerChanged {
            pub arg0: String,
            pub arg1: String,
            pub arg2: String,
        }

        impl arg::AppendAll for OrgFreedesktopDBusNameOwnerChanged {
            fn append(&self, i: &mut arg::IterAppend) {
                arg::RefArg::append(&self.arg0, i);
                arg::RefArg::append(&self.arg1, i);
                arg::RefArg::append(&self.arg2, i);
            }
        }

        impl arg::ReadAll for OrgFreedesktopDBusNameOwnerChanged {
            fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
                Ok(OrgFreedesktopDBusNameOwnerChanged {
                    arg0: i.read()?,
                    arg1: i.read()?,
                    arg2: i.read()?,
                })
            }
        }

        impl dbus::message::SignalArgs for OrgFreedesktopDBusNameOwnerChanged {
            const NAME: &'static str = "NameOwnerChanged";
            const INTERFACE: &'static str = "org.freedesktop.DBus";
        }

        #[derive(Debug)]
        pub struct OrgFreedesktopDBusNameLost {
            pub arg0: String,
        }

        impl arg::AppendAll for OrgFreedesktopDBusNameLost {
            fn append(&self, i: &mut arg::IterAppend) {
                arg::RefArg::append(&self.arg0, i);
            }
        }

        impl arg::ReadAll for OrgFreedesktopDBusNameLost {
            fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
                Ok(OrgFreedesktopDBusNameLost { arg0: i.read()? })
            }
        }

        impl dbus::message::SignalArgs for OrgFreedesktopDBusNameLost {
            const NAME: &'static str = "NameLost";
            const INTERFACE: &'static str = "org.freedesktop.DBus";
        }

        #[derive(Debug)]
        pub struct OrgFreedesktopDBusNameAcquired {
            pub arg0: String,
        }

        impl arg::AppendAll for OrgFreedesktopDBusNameAcquired {
            fn append(&self, i: &mut arg::IterAppend) {
                arg::RefArg::append(&self.arg0, i);
            }
        }

        impl arg::ReadAll for OrgFreedesktopDBusNameAcquired {
            fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
                Ok(OrgFreedesktopDBusNameAcquired { arg0: i.read()? })
            }
        }

        impl dbus::message::SignalArgs for OrgFreedesktopDBusNameAcquired {
            const NAME: &'static str = "NameAcquired";
            const INTERFACE: &'static str = "org.freedesktop.DBus";
        }

        pub trait OrgFreedesktopDBusIntrospectable {
            fn introspect(&self) -> Result<String, dbus::Error>;
        }

        impl<'a, C: ::std::ops::Deref<Target = blocking::Connection>>
            OrgFreedesktopDBusIntrospectable for blocking::Proxy<'a, C>
        {
            fn introspect(&self) -> Result<String, dbus::Error> {
                self.method_call("org.freedesktop.DBus.Introspectable", "Introspect", ())
                    .and_then(|r: (String,)| Ok(r.0))
            }
        }

        pub trait OrgFreedesktopDBusPeer {
            fn get_machine_id(&self) -> Result<String, dbus::Error>;
            fn ping(&self) -> Result<(), dbus::Error>;
        }

        impl<'a, C: ::std::ops::Deref<Target = blocking::Connection>> OrgFreedesktopDBusPeer
            for blocking::Proxy<'a, C>
        {
            fn get_machine_id(&self) -> Result<String, dbus::Error> {
                self.method_call("org.freedesktop.DBus.Peer", "GetMachineId", ())
                    .and_then(|r: (String,)| Ok(r.0))
            }

            fn ping(&self) -> Result<(), dbus::Error> {
                self.method_call("org.freedesktop.DBus.Peer", "Ping", ())
            }
        }
    }

    mod polkit {
        // This code was autogenerated with `dbus-codegen-rust -s -d org.freedesktop.PolicyKit1 -p /org/freedesktop/PolicyKit1/Authority -m None`, see https://github.com/diwic/dbus-rs
        use dbus;
        use dbus::arg;
        use dbus::blocking;

        pub trait OrgFreedesktopDBusProperties {
            fn get(
                &self,
                interface_name: &str,
                property_name: &str,
            ) -> Result<arg::Variant<Box<dyn arg::RefArg + 'static>>, dbus::Error>;
            fn get_all(
                &self,
                interface_name: &str,
            ) -> Result<
                ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
                dbus::Error,
            >;
            fn set(
                &self,
                interface_name: &str,
                property_name: &str,
                value: arg::Variant<Box<dyn arg::RefArg>>,
            ) -> Result<(), dbus::Error>;
        }

        impl<'a, C: ::std::ops::Deref<Target = blocking::Connection>> OrgFreedesktopDBusProperties
            for blocking::Proxy<'a, C>
        {
            fn get(
                &self,
                interface_name: &str,
                property_name: &str,
            ) -> Result<arg::Variant<Box<dyn arg::RefArg + 'static>>, dbus::Error> {
                self.method_call(
                    "org.freedesktop.DBus.Properties",
                    "Get",
                    (interface_name, property_name),
                )
                .and_then(|r: (arg::Variant<Box<dyn arg::RefArg + 'static>>,)| Ok(r.0))
            }

            fn get_all(
                &self,
                interface_name: &str,
            ) -> Result<
                ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
                dbus::Error,
            > {
                self.method_call(
                    "org.freedesktop.DBus.Properties",
                    "GetAll",
                    (interface_name,),
                )
                .and_then(
                    |r: (
                        ::std::collections::HashMap<
                            String,
                            arg::Variant<Box<dyn arg::RefArg + 'static>>,
                        >,
                    )| Ok(r.0),
                )
            }

            fn set(
                &self,
                interface_name: &str,
                property_name: &str,
                value: arg::Variant<Box<dyn arg::RefArg>>,
            ) -> Result<(), dbus::Error> {
                self.method_call(
                    "org.freedesktop.DBus.Properties",
                    "Set",
                    (interface_name, property_name, value),
                )
            }
        }

        #[derive(Debug)]
        pub struct OrgFreedesktopDBusPropertiesPropertiesChanged {
            pub interface_name: String,
            pub changed_properties:
                ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
            pub invalidated_properties: Vec<String>,
        }

        impl arg::AppendAll for OrgFreedesktopDBusPropertiesPropertiesChanged {
            fn append(&self, i: &mut arg::IterAppend) {
                arg::RefArg::append(&self.interface_name, i);
                arg::RefArg::append(&self.changed_properties, i);
                arg::RefArg::append(&self.invalidated_properties, i);
            }
        }

        impl arg::ReadAll for OrgFreedesktopDBusPropertiesPropertiesChanged {
            fn read(i: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
                Ok(OrgFreedesktopDBusPropertiesPropertiesChanged {
                    interface_name: i.read()?,
                    changed_properties: i.read()?,
                    invalidated_properties: i.read()?,
                })
            }
        }

        impl dbus::message::SignalArgs for OrgFreedesktopDBusPropertiesPropertiesChanged {
            const NAME: &'static str = "PropertiesChanged";
            const INTERFACE: &'static str = "org.freedesktop.DBus.Properties";
        }

        pub trait OrgFreedesktopDBusIntrospectable {
            fn introspect(&self) -> Result<String, dbus::Error>;
        }

        impl<'a, C: ::std::ops::Deref<Target = blocking::Connection>>
            OrgFreedesktopDBusIntrospectable for blocking::Proxy<'a, C>
        {
            fn introspect(&self) -> Result<String, dbus::Error> {
                self.method_call("org.freedesktop.DBus.Introspectable", "Introspect", ())
                    .and_then(|r: (String,)| Ok(r.0))
            }
        }

        pub trait OrgFreedesktopDBusPeer {
            fn ping(&self) -> Result<(), dbus::Error>;
            fn get_machine_id(&self) -> Result<String, dbus::Error>;
        }

        impl<'a, C: ::std::ops::Deref<Target = blocking::Connection>> OrgFreedesktopDBusPeer
            for blocking::Proxy<'a, C>
        {
            fn ping(&self) -> Result<(), dbus::Error> {
                self.method_call("org.freedesktop.DBus.Peer", "Ping", ())
            }

            fn get_machine_id(&self) -> Result<String, dbus::Error> {
                self.method_call("org.freedesktop.DBus.Peer", "GetMachineId", ())
                    .and_then(|r: (String,)| Ok(r.0))
            }
        }

        pub trait OrgFreedesktopPolicyKit1Authority {
            fn enumerate_actions(
                &self,
                locale: &str,
            ) -> Result<
                Vec<(
                    String,
                    String,
                    String,
                    String,
                    String,
                    String,
                    u32,
                    u32,
                    u32,
                    ::std::collections::HashMap<String, String>,
                )>,
                dbus::Error,
            >;
            fn check_authorization(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
                action_id: &str,
                details: ::std::collections::HashMap<&str, &str>,
                flags: u32,
                cancellation_id: &str,
            ) -> Result<(bool, bool, ::std::collections::HashMap<String, String>), dbus::Error>;
            fn cancel_check_authorization(&self, cancellation_id: &str) -> Result<(), dbus::Error>;
            fn register_authentication_agent(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
                locale: &str,
                object_path: &str,
            ) -> Result<(), dbus::Error>;
            fn register_authentication_agent_with_options(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
                locale: &str,
                object_path: &str,
                options: ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
            ) -> Result<(), dbus::Error>;
            fn unregister_authentication_agent(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
                object_path: &str,
            ) -> Result<(), dbus::Error>;
            fn authentication_agent_response(
                &self,
                cookie: &str,
                identity: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
            ) -> Result<(), dbus::Error>;
            fn authentication_agent_response2(
                &self,
                uid: u32,
                cookie: &str,
                identity: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
            ) -> Result<(), dbus::Error>;
            fn enumerate_temporary_authorizations(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
            ) -> Result<
                Vec<(
                    String,
                    String,
                    (
                        String,
                        ::std::collections::HashMap<
                            String,
                            arg::Variant<Box<dyn arg::RefArg + 'static>>,
                        >,
                    ),
                    u64,
                    u64,
                )>,
                dbus::Error,
            >;
            fn revoke_temporary_authorizations(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
            ) -> Result<(), dbus::Error>;
            fn revoke_temporary_authorization_by_id(&self, id: &str) -> Result<(), dbus::Error>;
            fn backend_name(&self) -> Result<String, dbus::Error>;
            fn backend_version(&self) -> Result<String, dbus::Error>;
            fn backend_features(&self) -> Result<u32, dbus::Error>;
        }

        impl<'a, C: ::std::ops::Deref<Target = blocking::Connection>>
            OrgFreedesktopPolicyKit1Authority for blocking::Proxy<'a, C>
        {
            fn enumerate_actions(
                &self,
                locale: &str,
            ) -> Result<
                Vec<(
                    String,
                    String,
                    String,
                    String,
                    String,
                    String,
                    u32,
                    u32,
                    u32,
                    ::std::collections::HashMap<String, String>,
                )>,
                dbus::Error,
            > {
                self.method_call(
                    "org.freedesktop.PolicyKit1.Authority",
                    "EnumerateActions",
                    (locale,),
                )
                .and_then(
                    |r: (
                        Vec<(
                            String,
                            String,
                            String,
                            String,
                            String,
                            String,
                            u32,
                            u32,
                            u32,
                            ::std::collections::HashMap<String, String>,
                        )>,
                    )| Ok(r.0),
                )
            }

            fn check_authorization(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
                action_id: &str,
                details: ::std::collections::HashMap<&str, &str>,
                flags: u32,
                cancellation_id: &str,
            ) -> Result<(bool, bool, ::std::collections::HashMap<String, String>), dbus::Error>
            {
                self.method_call(
                    "org.freedesktop.PolicyKit1.Authority",
                    "CheckAuthorization",
                    (subject, action_id, details, flags, cancellation_id),
                )
                .and_then(|r: ((bool, bool, ::std::collections::HashMap<String, String>),)| Ok(r.0))
            }

            fn cancel_check_authorization(&self, cancellation_id: &str) -> Result<(), dbus::Error> {
                self.method_call(
                    "org.freedesktop.PolicyKit1.Authority",
                    "CancelCheckAuthorization",
                    (cancellation_id,),
                )
            }

            fn register_authentication_agent(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
                locale: &str,
                object_path: &str,
            ) -> Result<(), dbus::Error> {
                self.method_call(
                    "org.freedesktop.PolicyKit1.Authority",
                    "RegisterAuthenticationAgent",
                    (subject, locale, object_path),
                )
            }

            fn register_authentication_agent_with_options(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
                locale: &str,
                object_path: &str,
                options: ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
            ) -> Result<(), dbus::Error> {
                self.method_call(
                    "org.freedesktop.PolicyKit1.Authority",
                    "RegisterAuthenticationAgentWithOptions",
                    (subject, locale, object_path, options),
                )
            }

            fn unregister_authentication_agent(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
                object_path: &str,
            ) -> Result<(), dbus::Error> {
                self.method_call(
                    "org.freedesktop.PolicyKit1.Authority",
                    "UnregisterAuthenticationAgent",
                    (subject, object_path),
                )
            }

            fn authentication_agent_response(
                &self,
                cookie: &str,
                identity: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
            ) -> Result<(), dbus::Error> {
                self.method_call(
                    "org.freedesktop.PolicyKit1.Authority",
                    "AuthenticationAgentResponse",
                    (cookie, identity),
                )
            }

            fn authentication_agent_response2(
                &self,
                uid: u32,
                cookie: &str,
                identity: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
            ) -> Result<(), dbus::Error> {
                self.method_call(
                    "org.freedesktop.PolicyKit1.Authority",
                    "AuthenticationAgentResponse2",
                    (uid, cookie, identity),
                )
            }

            fn enumerate_temporary_authorizations(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
            ) -> Result<
                Vec<(
                    String,
                    String,
                    (
                        String,
                        ::std::collections::HashMap<
                            String,
                            arg::Variant<Box<dyn arg::RefArg + 'static>>,
                        >,
                    ),
                    u64,
                    u64,
                )>,
                dbus::Error,
            > {
                self.method_call(
                    "org.freedesktop.PolicyKit1.Authority",
                    "EnumerateTemporaryAuthorizations",
                    (subject,),
                )
                .and_then(
                    |r: (
                        Vec<(
                            String,
                            String,
                            (
                                String,
                                ::std::collections::HashMap<
                                    String,
                                    arg::Variant<Box<dyn arg::RefArg + 'static>>,
                                >,
                            ),
                            u64,
                            u64,
                        )>,
                    )| Ok(r.0),
                )
            }

            fn revoke_temporary_authorizations(
                &self,
                subject: (
                    &str,
                    ::std::collections::HashMap<&str, arg::Variant<Box<dyn arg::RefArg>>>,
                ),
            ) -> Result<(), dbus::Error> {
                self.method_call(
                    "org.freedesktop.PolicyKit1.Authority",
                    "RevokeTemporaryAuthorizations",
                    (subject,),
                )
            }

            fn revoke_temporary_authorization_by_id(&self, id: &str) -> Result<(), dbus::Error> {
                self.method_call(
                    "org.freedesktop.PolicyKit1.Authority",
                    "RevokeTemporaryAuthorizationById",
                    (id,),
                )
            }

            fn backend_name(&self) -> Result<String, dbus::Error> {
                <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                    &self,
                    "org.freedesktop.PolicyKit1.Authority",
                    "BackendName",
                )
            }

            fn backend_version(&self) -> Result<String, dbus::Error> {
                <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                    &self,
                    "org.freedesktop.PolicyKit1.Authority",
                    "BackendVersion",
                )
            }

            fn backend_features(&self) -> Result<u32, dbus::Error> {
                <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                    &self,
                    "org.freedesktop.PolicyKit1.Authority",
                    "BackendFeatures",
                )
            }
        }

        #[derive(Debug)]
        pub struct OrgFreedesktopPolicyKit1AuthorityChanged {}

        impl arg::AppendAll for OrgFreedesktopPolicyKit1AuthorityChanged {
            fn append(&self, _: &mut arg::IterAppend) {}
        }

        impl arg::ReadAll for OrgFreedesktopPolicyKit1AuthorityChanged {
            fn read(_: &mut arg::Iter) -> Result<Self, arg::TypeMismatchError> {
                Ok(OrgFreedesktopPolicyKit1AuthorityChanged {})
            }
        }

        impl dbus::message::SignalArgs for OrgFreedesktopPolicyKit1AuthorityChanged {
            const NAME: &'static str = "Changed";
            const INTERFACE: &'static str = "org.freedesktop.PolicyKit1.Authority";
        }
    }
}
