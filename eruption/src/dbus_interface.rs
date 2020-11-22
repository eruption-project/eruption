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
    //LoadScript(PathBuf),
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
            .on_get(|i, _m| {
                let result = crate::ACTIVE_SLOT.load(Ordering::SeqCst) as u64;
                i.append(result);

                Ok(())
            });

        let active_slot_property_clone = Arc::new(active_slot_property);

        let active_profile_property = f
            .property::<String, _>("ActiveProfile", ())
            .emits_changed(EmitsChangedSignal::Const)
            .on_get(|i, _m| {
                let result = crate::ACTIVE_PROFILE.lock();

                result
                    .as_ref()
                    .and_then(|p| {
                        p.profile_file.file_name().map(|v| {
                            i.append(&*v.to_string_lossy());
                        })
                    })
                    .ok_or_else(|| MethodErr::failed("Method failed"))
            });

        let active_profile_property_clone = Arc::new(active_profile_property);

        let enable_sfx_property = f
            .property::<bool, _>("EnableSfx", ())
            .emits_changed(EmitsChangedSignal::True)
            .access(Access::ReadWrite)
            .auto_emit_on_set(true)
            .on_get(|i, _m| {
                i.append(audio::ENABLE_SFX.load(Ordering::SeqCst));

                Ok(())
            })
            .on_set(|i, _m| {
                audio::ENABLE_SFX.store(i.read::<bool>()?, Ordering::SeqCst);

                Ok(())
            });

        let enable_sfx_property_clone = Arc::new(enable_sfx_property);

        let brightness_property = f
            .property::<i64, _>("Brightness", ())
            .emits_changed(EmitsChangedSignal::True)
            .access(Access::ReadWrite)
            .auto_emit_on_set(true)
            .on_get(|i, _m| {
                let result = crate::BRIGHTNESS.load(Ordering::SeqCst) as i64;
                i.append(result);

                Ok(())
            })
            .on_set(|i, _m| {
                crate::BRIGHTNESS.store(i.read::<i64>()? as isize, Ordering::SeqCst);
                script::FRAME_GENERATION_COUNTER.fetch_add(1, Ordering::SeqCst);

                Ok(())
            });

        let brightness_property_clone = Arc::new(brightness_property);

        let tree = f
            .tree(())
            .add(
                f.object_path("/org/eruption/status", ())
                    .introspectable()
                    .add(
                        f.interface("org.eruption.Status", ()).add_p(
                            f.property::<bool, _>("Running", ())
                                .emits_changed(EmitsChangedSignal::True)
                                .on_get(|i, _m| {
                                    i.append(true);
                                    Ok(())
                                })
                                .on_set(|i, _m| {
                                    let _b: bool = i.read()?;
                                    Ok(())
                                }),
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
                            .add_p(brightness_property_clone),
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
                                })
                                .inarg::<u64, _>("slot")
                                .outarg::<bool, _>("status"),
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
                                    let n: &str = m.msg.read1()?;

                                    dbus_tx_clone
                                        .send(Message::SwitchProfile(PathBuf::from(n)))
                                        .unwrap_or_else(|e| {
                                            error!("Could not send a pending D-Bus event: {}", e)
                                        });

                                    // reset the audio backend, it will be enabled again if needed
                                    plugins::audio::reset_audio_backend();

                                    //c_clone2
                                    //.send(active_profile_changed_signal_clone.emit(
                                    //&"/org/eruption/profile".into(),
                                    //&"org.eruption.Profile".into(),
                                    //&[n],
                                    //))
                                    //.unwrap();

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
                                })
                                .inarg::<&str, _>("filename")
                                .outarg::<bool, _>("status"),
                            )
                            .add_m(
                                f.method("EnumProfiles", (), move |m| {
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

    /// Get the next event from D-Bus
    pub fn get_next_event(&self) -> Result<()> {
        match self.connection {
            Some(ref connection) => {
                if let Some(item) = connection.incoming(constants::DBUS_TIMEOUT_MILLIS).next() {
                    debug!("Message: {:?}", item);
                } else {
                    trace!("Received a timeout message");
                }

                Ok(())
            }

            None => Err(DbusApiError::BusNotConnected {}.into()),
        }
    }
}

/// Initialize the Eruption D-Bus API support
pub fn initialize(dbus_tx: Sender<Message>) -> Result<DbusApi> {
    DbusApi::new(dbus_tx)
}
