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

    Copyright (c) 2019-2022, The Eruption Development Team
*/

use colorgrad::Color;
use dbus::{ffidisp::BusType, ffidisp::Connection, ffidisp::NameFlag, message::SignalArgs};
use dbus_tree::{
    Access, MethodErr, Signal, {EmitsChangedSignal, Factory},
};
use flume::Sender;
use log::*;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::Arc;

use crate::{
    color_scheme::ColorScheme,
    constants, hwdevices,
    plugins::{self, audio},
    profiles, script,
    scripting::parameters,
    scripting::parameters_util,
};

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

    #[error("Invalid device")]
    InvalidDevice {},

    #[error("Invalid parameter")]
    InvalidParameter {},
    // #[error("Operation not supported")]
    // OpNotSupported {},
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
            Arc::new(f.signal("ActiveSlotChanged", ()).sarg::<u64, _>("slot"));
        let active_slot_changed_signal_clone = active_slot_changed_signal.clone();

        // let slot_names_changed_signal = Arc::new(
        //     f.signal("SlotNamesChanged", ())
        //         .sarg::<String, _>("new slot names"),
        // );
        // let slot_names_changed_signal_clone = slot_names_changed_signal.clone();

        let active_profile_changed_signal = Arc::new(
            f.signal("ActiveProfileChanged", ())
                .sarg::<String, _>("profile_name"),
        );
        let active_profile_changed_signal_clone = active_profile_changed_signal.clone();

        let profiles_changed_signal = Arc::new(f.signal("ProfilesChanged", ()));
        let profiles_changed_signal_clone = profiles_changed_signal.clone();

        let brightness_changed_signal = Arc::new(
            f.signal("BrightnessChanged", ())
                .sarg::<i64, _>("brightness"),
        );
        let brightness_changed_signal_clone = brightness_changed_signal.clone();

        let hue_changed_signal = Arc::new(f.signal("HueChanged", ()).sarg::<f64, _>("hue"));
        let hue_changed_signal_clone = hue_changed_signal.clone();

        let saturation_changed_signal = Arc::new(
            f.signal("SaturationChanged", ())
                .sarg::<f64, _>("saturation"),
        );
        let saturation_changed_signal_clone = saturation_changed_signal.clone();

        let lightness_changed_signal =
            Arc::new(f.signal("LightnessChanged", ()).sarg::<f64, _>("lightness"));
        let lightness_changed_signal_clone = lightness_changed_signal.clone();

        let device_status_changed_signal = Arc::new(
            f.signal("DeviceStatusChanged", ())
                .sarg::<String, _>("status"),
        );
        let device_status_changed_signal_clone = device_status_changed_signal.clone();

        let device_hotplug_signal = Arc::new(
            f.signal("DeviceHotplug", ())
                .sarg::<(u16, u16, bool), _>("device_info"),
        );
        let device_hotplug_signal_clone = device_hotplug_signal.clone();

        let active_slot_property = f
            .property::<u64, _>("ActiveSlot", ())
            .emits_changed(EmitsChangedSignal::Const)
            .on_get(|i, m| {
                if perms::has_monitor_permission_cached(&m.msg.sender().unwrap()).unwrap_or(false) {
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
                if perms::has_monitor_permission_cached(&m.msg.sender().unwrap()).unwrap_or(false) {
                    let result = crate::ACTIVE_PROFILE.lock();

                    result
                        .as_ref()
                        .map(|p| {
                            i.append(&*p.profile_file.to_string_lossy());
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
                if perms::has_monitor_permission_cached(&m.msg.sender().unwrap()).unwrap_or(false) {
                    i.append(audio::ENABLE_SFX.load(Ordering::SeqCst));

                    Ok(())
                } else {
                    Err(MethodErr::failed("Authentication failed"))
                }
            })
            .on_set(|i, m| {
                if perms::has_settings_permission_cached(&m.msg.sender().unwrap()).unwrap_or(false)
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
                if perms::has_monitor_permission_cached(&m.msg.sender().unwrap()).unwrap_or(false) {
                    let result = crate::BRIGHTNESS.load(Ordering::SeqCst) as i64;
                    i.append(result);

                    Ok(())
                } else {
                    Err(MethodErr::failed("Authentication failed"))
                }
            })
            .on_set(|i, m| {
                if perms::has_settings_permission_cached(&m.msg.sender().unwrap()).unwrap_or(false)
                {
                    crate::BRIGHTNESS.store(i.read::<i64>()? as isize, Ordering::SeqCst);
                    script::FRAME_GENERATION_COUNTER.fetch_add(1, Ordering::SeqCst);

                    Ok(())
                } else {
                    Err(MethodErr::failed("Authentication failed"))
                }
            });

        let brightness_property_clone = Arc::new(brightness_property);

        let hue_property = f
            .property::<f64, _>("Hue", ())
            .emits_changed(EmitsChangedSignal::True)
            .access(Access::ReadWrite)
            .auto_emit_on_set(true)
            .on_get(|i, m| {
                if perms::has_monitor_permission_cached(&m.msg.sender().unwrap()).unwrap_or(false) {
                    let result = crate::CANVAS_HSL.lock().0;
                    i.append(result);

                    Ok(())
                } else {
                    Err(MethodErr::failed("Authentication failed"))
                }
            })
            .on_set(|i, m| {
                if perms::has_settings_permission_cached(&m.msg.sender().unwrap()).unwrap_or(false)
                {
                    crate::CANVAS_HSL.lock().0 = i.read::<f64>()?;
                    script::FRAME_GENERATION_COUNTER.fetch_add(1, Ordering::SeqCst);

                    Ok(())
                } else {
                    Err(MethodErr::failed("Authentication failed"))
                }
            });

        let hue_property_clone = Arc::new(hue_property);

        let saturation_property = f
            .property::<f64, _>("Saturation", ())
            .emits_changed(EmitsChangedSignal::True)
            .access(Access::ReadWrite)
            .auto_emit_on_set(true)
            .on_get(|i, m| {
                if perms::has_monitor_permission_cached(&m.msg.sender().unwrap()).unwrap_or(false) {
                    let result = crate::CANVAS_HSL.lock().1;
                    i.append(result);

                    Ok(())
                } else {
                    Err(MethodErr::failed("Authentication failed"))
                }
            })
            .on_set(|i, m| {
                if perms::has_settings_permission_cached(&m.msg.sender().unwrap()).unwrap_or(false)
                {
                    crate::CANVAS_HSL.lock().1 = i.read::<f64>()?;
                    script::FRAME_GENERATION_COUNTER.fetch_add(1, Ordering::SeqCst);

                    Ok(())
                } else {
                    Err(MethodErr::failed("Authentication failed"))
                }
            });

        let saturation_property_clone = Arc::new(saturation_property);

        let lightness_property = f
            .property::<f64, _>("Lightness", ())
            .emits_changed(EmitsChangedSignal::True)
            .access(Access::ReadWrite)
            .auto_emit_on_set(true)
            .on_get(|i, m| {
                if perms::has_monitor_permission_cached(&m.msg.sender().unwrap()).unwrap_or(false) {
                    let result = crate::CANVAS_HSL.lock().2;
                    i.append(result);

                    Ok(())
                } else {
                    Err(MethodErr::failed("Authentication failed"))
                }
            })
            .on_set(|i, m| {
                if perms::has_settings_permission_cached(&m.msg.sender().unwrap()).unwrap_or(false)
                {
                    crate::CANVAS_HSL.lock().2 = i.read::<f64>()?;
                    script::FRAME_GENERATION_COUNTER.fetch_add(1, Ordering::SeqCst);

                    Ok(())
                } else {
                    Err(MethodErr::failed("Authentication failed"))
                }
            });

        let lightness_property_clone = Arc::new(lightness_property);

        let device_status_property = f
            .property::<String, _>("DeviceStatus", ())
            .emits_changed(EmitsChangedSignal::True)
            .access(Access::Read)
            // .auto_emit_on_set(true)
            .on_get(|i, m| {
                if perms::has_monitor_permission_cached(&m.msg.sender().unwrap()).unwrap_or(false) {
                    let device_status = &*crate::DEVICE_STATUS.as_ref().lock();

                    let device_status = device_status
                        .iter()
                        .map(|(k, v)| {
                            let (usb_vid, usb_pid) =
                                get_device_specific_ids(*k).unwrap_or_default();

                            DeviceStatus {
                                index: *k,
                                usb_vid,
                                usb_pid,
                                status: v.clone(),
                            }
                        })
                        .collect::<Vec<DeviceStatus>>();

                    let result = serde_json::to_string_pretty(&device_status)
                        .map_err(|e| MethodErr::failed(&format!("{}", e)))?;

                    i.append(result);

                    Ok(())
                } else {
                    Err(MethodErr::failed("Authentication failed"))
                }
            });

        let device_status_property_clone = Arc::new(device_status_property);

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
                                        if perms::has_monitor_permission_cached(
                                            &m.msg.sender().unwrap(),
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
                                        if perms::has_settings_permission_cached(
                                            &m.msg.sender().unwrap(),
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
                                    if perms::has_monitor_permission_cached(
                                        &m.msg.sender().unwrap(),
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
                                    if perms::has_monitor_permission_cached(
                                        &m.msg.sender().unwrap(),
                                    )
                                    .unwrap_or(false)
                                    {
                                        if crate::QUIT.load(Ordering::SeqCst) {
                                            return Err(MethodErr::failed(
                                                "Eruption is shutting down",
                                            ));
                                        }

                                        let mut keyboards: Vec<(u16, u16)> = Vec::new();
                                        let mut mice: Vec<(u16, u16)> = Vec::new();
                                        let mut misc: Vec<(u16, u16)> = Vec::new();

                                        {
                                            keyboards.extend(
                                                crate::KEYBOARD_DEVICES.read().iter().map(
                                                    |device| {
                                                        (
                                                            device.read().get_usb_vid(),
                                                            device.read().get_usb_pid(),
                                                        )
                                                    },
                                                ),
                                            );
                                        }

                                        {
                                            mice.extend(crate::MOUSE_DEVICES.read().iter().map(
                                                |device| {
                                                    (
                                                        device.read().get_usb_vid(),
                                                        device.read().get_usb_pid(),
                                                    )
                                                },
                                            ));
                                        }

                                        {
                                            misc.extend(crate::MISC_DEVICES.read().iter().map(
                                                |device| {
                                                    (
                                                        device.read().get_usb_vid(),
                                                        device.read().get_usb_pid(),
                                                    )
                                                },
                                            ));
                                        }

                                        Ok(vec![m
                                            .msg
                                            .method_return()
                                            .append1((keyboards, mice, misc))])
                                    } else {
                                        Err(MethodErr::failed("Authentication failed"))
                                    }
                                })
                                .outarg::<(
                                    Vec<(u16, u16)>,
                                    Vec<(u16, u16)>,
                                    Vec<(u16, u16)>,
                                ), _>(
                                    "values"
                                ),
                            ),
                    ),
            )
            .add(
                f.object_path("/org/eruption/devices", ())
                    .introspectable()
                    .add(
                        f.interface("org.eruption.Device", ())
                            .add_s(device_status_changed_signal_clone)
                            .add_s(device_hotplug_signal_clone)
                            .add_m(
                                f.method("SetDeviceConfig", (), move |m| {
                                    if perms::has_settings_permission_cached(
                                        &m.msg.sender().unwrap(),
                                    )
                                    .unwrap_or(false)
                                    {
                                        let (device, param, value): (u64, String, String) =
                                            m.msg.read3()?;

                                        debug!(
                                            "Setting device [{}] config parameter '{}' to '{}'",
                                            device, &param, &value
                                        );

                                        apply_device_specific_configuration(device, &param, &value)
                                            .map_err(|_e| MethodErr::invalid_arg(&param))?;

                                        Ok(vec![m.msg.method_return().append1(true)])
                                    } else {
                                        Err(MethodErr::failed("Authentication failed"))
                                    }
                                })
                                .inarg::<u64, _>("device")
                                .inarg::<String, _>("param")
                                .inarg::<String, _>("value")
                                .outarg::<bool, _>("status"),
                            )
                            .add_m(
                                f.method("GetDeviceConfig", (), move |m| {
                                    if perms::has_settings_permission_cached(
                                        &m.msg.sender().unwrap(),
                                    )
                                    .unwrap_or(false)
                                    {
                                        let (device, param): (u64, String) = m.msg.read2()?;

                                        trace!(
                                            "Querying device [{}] config parameter '{}'",
                                            device,
                                            &param
                                        );

                                        let result =
                                            query_device_specific_configuration(device, &param)
                                                .map_err(|_e| MethodErr::invalid_arg(&param))?;

                                        Ok(vec![m.msg.method_return().append1(result)])
                                    } else {
                                        Err(MethodErr::failed("Authentication failed"))
                                    }
                                })
                                .inarg::<u64, _>("device")
                                .inarg::<String, _>("param")
                                .outarg::<String, _>("value"),
                            )
                            .add_m(
                                f.method("GetDeviceStatus", (), move |m| {
                                    if perms::has_monitor_permission_cached(
                                        &m.msg.sender().unwrap(),
                                    )
                                    .unwrap_or(false)
                                    {
                                        let device: u64 = m.msg.read1()?;

                                        trace!("Querying device [{}] status", device);

                                        let result = query_device_specific_status(device)
                                            .map_err(|e| MethodErr::failed(&format!("{}", e)))?;

                                        Ok(vec![m.msg.method_return().append1(result)])
                                    } else {
                                        Err(MethodErr::failed("Authentication failed"))
                                    }
                                })
                                .inarg::<u64, _>("device")
                                .outarg::<String, _>("status"),
                            )
                            .add_m(
                                f.method("GetManagedDevices", (), move |m| {
                                    if perms::has_monitor_permission_cached(
                                        &m.msg.sender().unwrap(),
                                    )
                                    .unwrap_or(false)
                                    {
                                        if crate::QUIT.load(Ordering::SeqCst) {
                                            return Err(MethodErr::failed(
                                                "Eruption is shutting down",
                                            ));
                                        }

                                        let keyboards = {
                                            let keyboards = crate::KEYBOARD_DEVICES.read();

                                            let keyboards: Vec<(u16, u16)> = keyboards
                                                .iter()
                                                .map(|device| {
                                                    (
                                                        device.read().get_usb_vid(),
                                                        device.read().get_usb_pid(),
                                                    )
                                                })
                                                .collect();

                                            keyboards
                                        };

                                        let mice = {
                                            let mice = crate::MOUSE_DEVICES.read();

                                            let mice: Vec<(u16, u16)> = mice
                                                .iter()
                                                .map(|device| {
                                                    (
                                                        device.read().get_usb_vid(),
                                                        device.read().get_usb_pid(),
                                                    )
                                                })
                                                .collect();

                                            mice
                                        };

                                        let misc = {
                                            let misc = crate::MISC_DEVICES.read();

                                            let misc: Vec<(u16, u16)> = misc
                                                .iter()
                                                .map(|device| {
                                                    (
                                                        device.read().get_usb_vid(),
                                                        device.read().get_usb_pid(),
                                                    )
                                                })
                                                .collect();

                                            misc
                                        };

                                        Ok(vec![m
                                            .msg
                                            .method_return()
                                            .append1((keyboards, mice, misc))])
                                    } else {
                                        Err(MethodErr::failed("Authentication failed"))
                                    }
                                })
                                .outarg::<(
                                    Vec<(u16, u16)>,
                                    Vec<(u16, u16)>,
                                    Vec<(u16, u16)>,
                                ), _>(
                                    "values"
                                ),
                            )
                            .add_p(device_status_property_clone),
                    ),
            )
            .add(
                f.object_path("/org/eruption/canvas", ())
                    .introspectable()
                    .add(
                        f.interface("org.eruption.Canvas", ())
                            .add_s(hue_changed_signal_clone)
                            .add_s(saturation_changed_signal_clone)
                            .add_s(lightness_changed_signal_clone)
                            .add_p(hue_property_clone)
                            .add_p(saturation_property_clone)
                            .add_p(lightness_property_clone),
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
                                    if perms::has_manage_permission_cached(&m.msg.sender().unwrap())
                                        .unwrap_or(false)
                                    {
                                        let (filename, data): (String, String) = m.msg.read2()?;

                                        crate::util::write_file(&PathBuf::from(filename), &data)
                                            .map_err(|e| {
                                                MethodErr::failed(&format!(
                                                    "Error writing file: {}",
                                                    e
                                                ))
                                            })?;

                                        let s = true;
                                        Ok(vec![m.msg.method_return().append1(s)])
                                    } else {
                                        Err(MethodErr::failed("Authentication failed"))
                                    }
                                })
                                .inarg::<String, _>("filename")
                                .inarg::<String, _>("data")
                                .outarg::<bool, _>("status"),
                            )
                            .add_m(
                                f.method("Ping", (), move |m| {
                                    if perms::has_monitor_permission_cached(
                                        &m.msg.sender().unwrap(),
                                    )
                                    .unwrap_or(false)
                                    {
                                        let s = true;
                                        Ok(vec![m.msg.method_return().append1(s)])
                                    } else {
                                        Err(MethodErr::failed("Authentication failed"))
                                    }
                                })
                                .outarg::<bool, _>("status"),
                            )
                            .add_m(
                                f.method("PingPrivileged", (), move |m| {
                                    if perms::has_manage_permission_cached(&m.msg.sender().unwrap())
                                        .unwrap_or(false)
                                    {
                                        let s = true;
                                        Ok(vec![m.msg.method_return().append1(s)])
                                    } else {
                                        Err(MethodErr::failed("Authentication failed"))
                                    }
                                })
                                .outarg::<bool, _>("status"),
                            )
                            .add_m(
                                f.method("GetColorSchemes", (), move |m| {
                                    if perms::has_monitor_permission_cached(
                                        &m.msg.sender().unwrap(),
                                    )
                                    .unwrap_or(false)
                                    {
                                        let color_schemes: Vec<String> = crate::NAMED_COLOR_SCHEMES
                                            .read()
                                            .keys()
                                            .cloned()
                                            .collect();

                                        Ok(vec![m.msg.method_return().append1(color_schemes)])
                                    } else {
                                        Err(MethodErr::failed("Authentication failed"))
                                    }
                                })
                                .outarg::<Vec<String>, _>("color_schemes"),
                            )
                            .add_m(
                                f.method("SetColorScheme", (), move |m| {
                                    if perms::has_settings_permission_cached(
                                        &m.msg.sender().unwrap(),
                                    )
                                    .unwrap_or(false)
                                    {
                                        let (name, data): (String, Vec<u8>) = m.msg.read2()?;

                                        if name.chars().take(1).all(char::is_numeric)
                                            || !name.chars().all(|c| {
                                                c == '_' || char::is_ascii_alphanumeric(&c)
                                            })
                                        {
                                            Err(MethodErr::failed("Invalid identifier name"))
                                        } else {
                                            let mut color_schemes =
                                                crate::NAMED_COLOR_SCHEMES.write();
                                            let mut colors = Vec::new();

                                            for chunk in data.chunks(4) {
                                                let r = chunk[0];
                                                let g = chunk[1];
                                                let b = chunk[2];
                                                let a = chunk[3];

                                                let color = Color::from_linear_rgba8(r, g, b, a);

                                                colors.push(color);
                                            }

                                            color_schemes.insert(name, ColorScheme { colors });

                                            crate::REQUEST_PROFILE_RELOAD
                                                .store(true, Ordering::SeqCst);

                                            let s = true;
                                            Ok(vec![m.msg.method_return().append1(s)])
                                        }
                                    } else {
                                        Err(MethodErr::failed("Authentication failed"))
                                    }
                                })
                                .inarg::<String, _>("name")
                                .inarg::<Vec<u8>, _>("data")
                                .outarg::<bool, _>("status"),
                            )
                            .add_m(
                                f.method("RemoveColorScheme", (), move |m| {
                                    if perms::has_settings_permission_cached(
                                        &m.msg.sender().unwrap(),
                                    )
                                    .unwrap_or(false)
                                    {
                                        let name: String = m.msg.read1()?;

                                        let s = crate::NAMED_COLOR_SCHEMES
                                            .write()
                                            .remove(&name)
                                            .is_some();

                                        if s {
                                            crate::REQUEST_PROFILE_RELOAD
                                                .store(true, Ordering::SeqCst);
                                        }

                                        Ok(vec![m.msg.method_return().append1(s)])
                                    } else {
                                        Err(MethodErr::failed("Authentication failed"))
                                    }
                                })
                                .inarg::<String, _>("name")
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
                                    if perms::has_settings_permission_cached(
                                        &m.msg.sender().unwrap(),
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
                                    if perms::has_monitor_permission_cached(
                                        &m.msg.sender().unwrap(),
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
                                        if perms::has_monitor_permission_cached(
                                            &m.msg.sender().unwrap(),
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
                                        if perms::has_settings_permission_cached(
                                            &m.msg.sender().unwrap(),
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
                                    if perms::has_settings_permission_cached(
                                        &m.msg.sender().unwrap(),
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
                                    if perms::has_monitor_permission_cached(
                                        &m.msg.sender().unwrap(),
                                    )
                                    .unwrap_or(false)
                                    {
                                        let mut s: Vec<(String, String)> = profiles::get_profiles()
                                            .unwrap_or_else(|_| vec![])
                                            .iter()
                                            .map(|profile| {
                                                (
                                                    profile.name.clone(),
                                                    profile
                                                        .profile_file
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
                            )
                            .add_m(
                                f.method("SetParameter", (), move |m| {
                                    if perms::has_settings_permission_cached(
                                        &m.msg.sender().unwrap(),
                                    )
                                    .unwrap_or(false)
                                    {
                                        let (profile_file, script_file, param_name, value): (
                                            &str,
                                            &str,
                                            &str,
                                            &str,
                                        ) = m.msg.read4()?;

                                        debug!(
                                            "Setting parameter {}:{} {} to '{}'",
                                            &profile_file, &script_file, &param_name, &value
                                        );

                                        let applied = apply_parameter(
                                            profile_file,
                                            script_file,
                                            param_name,
                                            value,
                                        );
                                        match applied {
                                            Ok(()) => Ok(vec![m.msg.method_return().append1(true)]),
                                            Err(err) => {
                                                debug!("Could not set parameter: {}", err);
                                                Err(MethodErr::invalid_arg(&value))
                                            }
                                        }
                                    } else {
                                        Err(MethodErr::failed("Authentication failed"))
                                    }
                                })
                                .inarg::<&str, _>("profile_file")
                                .inarg::<&str, _>("script_file")
                                .inarg::<&str, _>("param_name")
                                .inarg::<&str, _>("value")
                                .outarg::<bool, _>("status"),
                            ),
                    ),
            );

        tree.set_registered(&c_clone, true)
            .unwrap_or_else(|e| error!("Could not register the tree: {}", e));
        c_clone.add_handler(tree);

        Ok(Self {
            connection: Some(c_clone),
            active_slot_changed: active_slot_changed_signal,
            active_profile_changed: active_profile_changed_signal,
            profiles_changed: profiles_changed_signal,
            brightness_changed: brightness_changed_signal,

            hue_changed: hue_changed_signal,
            saturation_changed: saturation_changed_signal,
            lightness_changed: lightness_changed_signal,

            device_status_changed: device_status_changed_signal,
            device_hotplug: device_hotplug_signal,
        })
    }

    pub fn notify_device_status_changed(&self) -> Result<()> {
        let device_status = &*crate::DEVICE_STATUS.as_ref().lock();

        let device_status = device_status
            .iter()
            .map(|(k, v)| {
                let (usb_vid, usb_pid) = get_device_specific_ids(*k).unwrap_or_default();

                DeviceStatus {
                    index: *k,
                    usb_vid,
                    usb_pid,
                    status: v.clone(),
                }
            })
            .collect::<Vec<DeviceStatus>>();

        let result = serde_json::to_string_pretty(&device_status)
            .map_err(|e| MethodErr::failed(&format!("{}", e)))?;

        let _ = self
            .connection
            .as_ref()
            .unwrap()
            .send(self.device_status_changed.emit(
                &"/org/eruption/devices".into(),
                &"org.eruption.Device".into(),
                &[result],
            ))
            .map_err(|_| error!("D-Bus error during send call"));

        Ok(())
    }

    pub fn notify_device_hotplug(&self, device_info: (u16, u16), removed: bool) -> Result<()> {
        let _ = self
            .connection
            .as_ref()
            .unwrap()
            .send(self.device_hotplug.emit(
                &"/org/eruption/devices".into(),
                &"org.eruption.Device".into(),
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
            .send(self.brightness_changed.emit(
                &"/org/eruption/config".into(),
                &"org.eruption.Config".into(),
                &[brightness as i64],
            ))
            .map_err(|_| error!("D-Bus error during send call"));

        Ok(())
    }

    pub fn notify_hue_changed(&self) -> Result<()> {
        let hue = crate::CANVAS_HSL.lock().0;

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
        let saturation = crate::CANVAS_HSL.lock().1;

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
        let lightness = crate::CANVAS_HSL.lock().2;

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
            .send(self.active_slot_changed.emit(
                &"/org/eruption/slot".into(),
                &"org.eruption.Slot".into(),
                &[active_slot as u64],
            ))
            .map_err(|_| error!("D-Bus error during send call"));

        Ok(())
    }

    pub fn notify_active_profile_changed(&self) -> Result<()> {
        let active_profile = crate::ACTIVE_PROFILE.lock();

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
            .send(self.active_profile_changed.emit(
                &"/org/eruption/profile".into(),
                &"org.eruption.Profile".into(),
                &[active_profile],
            ))
            .map_err(|_| error!("D-Bus error during send call"));

        Ok(())
    }

    pub fn notify_profiles_changed(&self) -> Result<()> {
        let _ = self
            .connection
            .as_ref()
            .unwrap()
            .send(self.profiles_changed.msg(
                &"/org/eruption/profile".into(),
                &"org.eruption.Profile".into(),
            ))
            .map_err(|_| error!("D-Bus error during send call"));

        Ok(())
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

fn apply_parameter(
    profile_file: &str,
    script_file: &str,
    param_name: &str,
    value: &str,
) -> Result<()> {
    parameters_util::apply_parameters(
        profile_file,
        script_file,
        &[parameters::UntypedParameter {
            name: param_name.to_string(),
            value: value.to_string(),
        }],
    )
}

/// Query the device specific status from the global status store
fn query_device_specific_status(device: u64) -> Result<String> {
    let device_status = crate::DEVICE_STATUS.as_ref().lock();

    match device_status.get(&device) {
        Some(status) => Ok(serde_json::to_string_pretty(&status.0)?),
        None => Err(DbusApiError::InvalidDevice {}.into()),
    }
}

/// Query the device driver for status information
/// this will likely cause stuttering when not synchronized with the main loop
// fn query_device_specific_status_no_cache(device: u64) -> Result<String> {
//     let json = if (device as usize) < crate::KEYBOARD_DEVICES.read().len() {
//         let device = &crate::KEYBOARD_DEVICES.read()[device as usize];

//         let status = device.read().device_status()?;
//         let result = serde_json::to_string_pretty(&*status)?;

//         result
//     } else if (device as usize)
//         < (crate::KEYBOARD_DEVICES.read().len() + crate::MOUSE_DEVICES.read().len())
//     {
//         let index = device as usize - crate::KEYBOARD_DEVICES.read().len();
//         let device = &crate::MOUSE_DEVICES.read()[index];

//         let status = device.read().device_status()?;
//         let result = serde_json::to_string_pretty(&*status)?;

//         result
//     } else if (device as usize)
//         < (crate::KEYBOARD_DEVICES.read().len()
//             + crate::MOUSE_DEVICES.read().len()
//             + crate::MISC_DEVICES.read().len())
//     {
//         let index = device as usize
//             - (crate::KEYBOARD_DEVICES.read().len() + crate::MOUSE_DEVICES.read().len());
//         let device = &crate::MISC_DEVICES.read()[index];

//         let status = device.read().device_status()?;
//         let result = serde_json::to_string_pretty(&*status)?;

//         result
//     } else {
//         return Err(DbusApiError::InvalidDevice {}.into());
//     };

//     Ok(json)
// }

fn apply_device_specific_configuration(device: u64, param: &str, value: &str) -> Result<()> {
    if (device as usize) < crate::KEYBOARD_DEVICES.read().len() {
        let device = &crate::KEYBOARD_DEVICES.read()[device as usize];

        match param {
            "brightness" => {
                let brightness = value.parse::<i32>()?;
                device.write().set_local_brightness(brightness)?;

                script::FRAME_GENERATION_COUNTER.fetch_add(1, Ordering::SeqCst);

                Ok(())
            }

            _ => Err(DbusApiError::InvalidParameter {}.into()),
        }
    } else if (device as usize)
        < (crate::KEYBOARD_DEVICES.read().len() + crate::MOUSE_DEVICES.read().len())
    {
        let index = device as usize - crate::KEYBOARD_DEVICES.read().len();
        let device = &crate::MOUSE_DEVICES.read()[index];

        match param {
            "profile" => {
                let profile = value.parse::<i32>()?;
                device.write().set_profile(profile)?;

                Ok(())
            }

            "dpi" => {
                let dpi = value.parse::<i32>()?;
                device.write().set_dpi(dpi)?;

                Ok(())
            }

            "rate" => {
                let rate = value.parse::<i32>()?;
                device.write().set_rate(rate)?;

                Ok(())
            }

            "dcu" => {
                let dcu_config = value.parse::<i32>()?;
                device.write().set_dcu_config(dcu_config)?;

                Ok(())
            }

            "angle-snapping" => {
                let angle_snapping = value.parse::<bool>()?;
                device.write().set_angle_snapping(angle_snapping)?;

                Ok(())
            }

            "debounce" => {
                let debounce = value.parse::<bool>()?;
                device.write().set_debounce(debounce)?;

                Ok(())
            }

            "brightness" => {
                let brightness = value.parse::<i32>()?;
                device.write().set_local_brightness(brightness)?;

                script::FRAME_GENERATION_COUNTER.fetch_add(1, Ordering::SeqCst);

                Ok(())
            }

            _ => Err(DbusApiError::InvalidParameter {}.into()),
        }
    } else if (device as usize)
        < (crate::KEYBOARD_DEVICES.read().len()
            + crate::MOUSE_DEVICES.read().len()
            + crate::MISC_DEVICES.read().len())
    {
        let index = device as usize
            - (crate::KEYBOARD_DEVICES.read().len() + crate::MOUSE_DEVICES.read().len());
        let device = &crate::MISC_DEVICES.read()[index];

        match param {
            "brightness" => {
                let brightness = value.parse::<i32>()?;
                device.write().set_local_brightness(brightness)?;

                script::FRAME_GENERATION_COUNTER.fetch_add(1, Ordering::SeqCst);

                Ok(())
            }

            _ => Err(DbusApiError::InvalidParameter {}.into()),
        }
    } else {
        Err(DbusApiError::InvalidDevice {}.into())
    }
}

fn query_device_specific_configuration(device: u64, param: &str) -> Result<String> {
    if (device as usize) < crate::KEYBOARD_DEVICES.read().len() {
        let device = &crate::KEYBOARD_DEVICES.read()[device as usize];

        match param {
            "info" => {
                let device_info = device.read().get_device_info()?;
                let info = format!(
                    "Firmware revision: {}.{:02}",
                    device_info.firmware_version / 100,
                    device_info.firmware_version % 100
                );

                Ok(info)
            }

            "firmware" => {
                let device_info = device.read().get_device_info()?;
                let info = format!(
                    "{}.{:02}",
                    device_info.firmware_version / 100,
                    device_info.firmware_version % 100
                );

                Ok(info)
            }

            "brightness" => {
                let brightness = device.read().get_local_brightness()?;

                Ok(format!("{}", brightness))
            }

            _ => Err(DbusApiError::InvalidParameter {}.into()),
        }
    } else if (device as usize)
        < (crate::KEYBOARD_DEVICES.read().len() + crate::MOUSE_DEVICES.read().len())
    {
        let index = device as usize - crate::KEYBOARD_DEVICES.read().len();
        let device = &crate::MOUSE_DEVICES.read()[index];

        match param {
            "info" => {
                let device_info = device.read().get_device_info()?;
                let info = format!(
                    "Firmware revision: {}.{:02}",
                    device_info.firmware_version / 100,
                    device_info.firmware_version % 100
                );

                Ok(info)
            }

            "firmware" => {
                let device_info = device.read().get_device_info()?;
                let info = format!(
                    "{}.{:02}",
                    device_info.firmware_version / 100,
                    device_info.firmware_version % 100
                );

                Ok(info)
            }

            "profile" => {
                let profile = device.read().get_profile()?;

                Ok(format!("{}", profile))
            }

            "dpi" => {
                let dpi = device.read().get_dpi()?;

                Ok(format!("{}", dpi))
            }

            "rate" => {
                let rate = device.read().get_rate()?;

                Ok(format!("{}", rate))
            }

            "dcu" => {
                let dcu_config = device.read().get_dcu_config()?;

                Ok(format!("{}", dcu_config))
            }

            "angle-snapping" => {
                let angle_snapping = device.read().get_angle_snapping()?;

                Ok(format!("{}", angle_snapping))
            }

            "debounce" => {
                let debounce = device.read().get_debounce()?;

                Ok(format!("{}", debounce))
            }

            "brightness" => {
                let brightness = device.read().get_local_brightness()?;

                Ok(format!("{}", brightness))
            }

            _ => Err(DbusApiError::InvalidParameter {}.into()),
        }
    } else if (device as usize)
        < (crate::KEYBOARD_DEVICES.read().len()
            + crate::MOUSE_DEVICES.read().len()
            + crate::MISC_DEVICES.read().len())
    {
        let index = device as usize
            - (crate::KEYBOARD_DEVICES.read().len() + crate::MOUSE_DEVICES.read().len());
        let device = &crate::MISC_DEVICES.read()[index];

        match param {
            "info" => {
                let device_info = device.read().get_device_info()?;
                let info = format!(
                    "Firmware revision: {}.{:02}",
                    device_info.firmware_version / 100,
                    device_info.firmware_version % 100
                );

                Ok(info)
            }

            "firmware" => {
                let device_info = device.read().get_device_info()?;
                let info = format!(
                    "{}.{:02}",
                    device_info.firmware_version / 100,
                    device_info.firmware_version % 100
                );

                Ok(info)
            }

            "brightness" => {
                let brightness = device.read().get_local_brightness()?;

                Ok(format!("{}", brightness))
            }

            _ => Err(DbusApiError::InvalidParameter {}.into()),
        }
    } else {
        Err(DbusApiError::InvalidDevice {}.into())
    }
}

fn get_device_specific_ids(device: u64) -> Result<(u16, u16)> {
    if (device as usize) < crate::KEYBOARD_DEVICES.read().len() {
        let device = &crate::KEYBOARD_DEVICES.read()[device as usize];

        let usb_vid = device.read().get_usb_vid();
        let usb_pid = device.read().get_usb_pid();

        Ok((usb_vid, usb_pid))
    } else if (device as usize)
        < (crate::KEYBOARD_DEVICES.read().len() + crate::MOUSE_DEVICES.read().len())
    {
        let index = device as usize - crate::KEYBOARD_DEVICES.read().len();
        let device = &crate::MOUSE_DEVICES.read()[index];

        let usb_vid = device.read().get_usb_vid();
        let usb_pid = device.read().get_usb_pid();

        Ok((usb_vid, usb_pid))
    } else if (device as usize)
        < (crate::KEYBOARD_DEVICES.read().len()
            + crate::MOUSE_DEVICES.read().len()
            + crate::MISC_DEVICES.read().len())
    {
        let index = device as usize
            - (crate::KEYBOARD_DEVICES.read().len() + crate::MOUSE_DEVICES.read().len());
        let device = &crate::MISC_DEVICES.read()[index];

        let usb_vid = device.read().get_usb_vid();
        let usb_pid = device.read().get_usb_pid();

        Ok((usb_vid, usb_pid))
    } else {
        Err(DbusApiError::InvalidDevice {}.into())
    }
}

mod perms {
    use dbus::{arg::RefArg, arg::Variant, blocking::Connection};
    use lazy_static::lazy_static;
    use parking_lot::RwLock;
    use std::sync::Arc;
    use std::{collections::HashMap, time::Duration};

    use crate::constants;

    pub type Result<T> = std::result::Result<T, eyre::Error>;

    // cached permissions
    lazy_static! {
        static ref HAS_MONITOR_PERMISSION: Arc<RwLock<Option<bool>>> = Arc::new(RwLock::new(None));
        static ref HAS_SETTINGS_PERMISSION: Arc<RwLock<Option<bool>>> = Arc::new(RwLock::new(None));
        static ref HAS_MANAGE_PERMISSION: Arc<RwLock<Option<bool>>> = Arc::new(RwLock::new(None));
    }

    pub fn has_monitor_permission_cached(sender: &str) -> Result<bool> {
        if HAS_MONITOR_PERMISSION.read().is_some() {
            // cache is valid
            Ok(HAS_MONITOR_PERMISSION.read().unwrap())
        } else {
            // cache is invalid, we need to call out to PolKit
            let result = has_monitor_permission(sender)?;

            if !result.1 {
                if result.0 {
                    // call succeeded, update cached state
                    HAS_MONITOR_PERMISSION.write().replace(result.0);
                }

                Ok(result.0)
            } else {
                // user pressed cancel in authentication dialog
                Ok(false)
            }
        }
    }

    pub fn has_settings_permission_cached(sender: &str) -> Result<bool> {
        if HAS_SETTINGS_PERMISSION.read().is_some() {
            // cache is valid
            Ok(HAS_SETTINGS_PERMISSION.read().unwrap())
        } else {
            // cache is invalid, we need to call out to PolKit
            let result = has_settings_permission(sender)?;

            if !result.1 {
                if result.0 {
                    // call succeeded, update cached state
                    HAS_SETTINGS_PERMISSION.write().replace(result.0);
                }

                Ok(result.0)
            } else {
                // user pressed cancel in authentication dialog
                Ok(false)
            }
        }
    }

    pub fn has_manage_permission_cached(sender: &str) -> Result<bool> {
        if HAS_MANAGE_PERMISSION.read().is_some() {
            // cache is valid
            Ok(HAS_MANAGE_PERMISSION.read().unwrap())
        } else {
            // cache is invalid, we need to call out to PolKit
            let result = has_manage_permission(sender)?;

            if !result.1 {
                if result.0 {
                    // call succeeded, update cached state
                    HAS_MANAGE_PERMISSION.write().replace(result.0);
                }

                Ok(result.0)
            } else {
                // user pressed cancel in authentication dialog
                Ok(false)
            }
        }
    }

    pub fn has_monitor_permission(sender: &str) -> Result<(bool, bool)> {
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
            map.insert("start-time", Variant(Box::new(0_u64) as Box<dyn RefArg>));
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
                break 'AUTH_LOOP (result, dismissed);
            }
        };

        Ok((result.0 .0, false))
    }

    pub fn has_settings_permission(sender: &str) -> Result<(bool, bool)> {
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
            map.insert("start-time", Variant(Box::new(0_u64) as Box<dyn RefArg>));
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
                break 'AUTH_LOOP (result, dismissed);
            }
        };

        Ok((result.0 .0, false))
    }

    pub fn has_manage_permission(sender: &str) -> Result<(bool, bool)> {
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
            map.insert("start-time", Variant(Box::new(0_u64) as Box<dyn RefArg>));
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
                break 'AUTH_LOOP (result, dismissed);
            }
        };

        Ok((result.0 .0, false))
    }

    mod bus {
        // This code was autogenerated with `dbus-codegen-rust -s -d org.freedesktop.DBus -p /org/freedesktop/DBus/Bus -m None`, see https://github.com/diwic/dbus-rs

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
                    .map(|r: (String,)| r.0)
            }

            fn request_name(&self, arg0: &str, arg1: u32) -> Result<u32, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "RequestName", (arg0, arg1))
                    .map(|r: (u32,)| r.0)
            }

            fn release_name(&self, arg0: &str) -> Result<u32, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "ReleaseName", (arg0,))
                    .map(|r: (u32,)| r.0)
            }

            fn start_service_by_name(&self, arg0: &str, arg1: u32) -> Result<u32, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "StartServiceByName", (arg0, arg1))
                    .map(|r: (u32,)| r.0)
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
                    .map(|r: (bool,)| r.0)
            }

            fn list_names(&self) -> Result<Vec<String>, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "ListNames", ())
                    .map(|r: (Vec<String>,)| r.0)
            }

            fn list_activatable_names(&self) -> Result<Vec<String>, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "ListActivatableNames", ())
                    .map(|r: (Vec<String>,)| r.0)
            }

            fn add_match(&self, arg0: &str) -> Result<(), dbus::Error> {
                self.method_call("org.freedesktop.DBus", "AddMatch", (arg0,))
            }

            fn remove_match(&self, arg0: &str) -> Result<(), dbus::Error> {
                self.method_call("org.freedesktop.DBus", "RemoveMatch", (arg0,))
            }

            fn get_name_owner(&self, arg0: &str) -> Result<String, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "GetNameOwner", (arg0,))
                    .map(|r: (String,)| r.0)
            }

            fn list_queued_owners(&self, arg0: &str) -> Result<Vec<String>, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "ListQueuedOwners", (arg0,))
                    .map(|r: (Vec<String>,)| r.0)
            }

            fn get_connection_unix_user(&self, arg0: &str) -> Result<u32, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "GetConnectionUnixUser", (arg0,))
                    .map(|r: (u32,)| r.0)
            }

            fn get_connection_unix_process_id(&self, arg0: &str) -> Result<u32, dbus::Error> {
                self.method_call(
                    "org.freedesktop.DBus",
                    "GetConnectionUnixProcessID",
                    (arg0,),
                )
                .map(|r: (u32,)| r.0)
            }

            fn get_adt_audit_session_data(&self, arg0: &str) -> Result<Vec<u8>, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "GetAdtAuditSessionData", (arg0,))
                    .map(|r: (Vec<u8>,)| r.0)
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
                .map(|r: (Vec<u8>,)| r.0)
            }

            fn reload_config(&self) -> Result<(), dbus::Error> {
                self.method_call("org.freedesktop.DBus", "ReloadConfig", ())
            }

            fn get_id(&self) -> Result<String, dbus::Error> {
                self.method_call("org.freedesktop.DBus", "GetId", ())
                    .map(|r: (String,)| r.0)
            }

            fn get_connection_credentials(
                &self,
                arg0: &str,
            ) -> Result<
                ::std::collections::HashMap<String, arg::Variant<Box<dyn arg::RefArg + 'static>>>,
                dbus::Error,
            > {
                self.method_call("org.freedesktop.DBus", "GetConnectionCredentials", (arg0,))
                    .map(
                        |r: (
                            ::std::collections::HashMap<
                                String,
                                arg::Variant<Box<dyn arg::RefArg + 'static>>,
                            >,
                        )| r.0,
                    )
            }

            fn features(&self) -> Result<Vec<String>, dbus::Error> {
                <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                    self,
                    "org.freedesktop.DBus",
                    "Features",
                )
            }

            fn interfaces(&self) -> Result<Vec<String>, dbus::Error> {
                <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                    self,
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
                    .map(|r: (String,)| r.0)
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
                    .map(|r: (String,)| r.0)
            }

            fn ping(&self) -> Result<(), dbus::Error> {
                self.method_call("org.freedesktop.DBus.Peer", "Ping", ())
            }
        }
    }

    mod polkit {
        // This code was autogenerated with `dbus-codegen-rust -s -d org.freedesktop.PolicyKit1 -p /org/freedesktop/PolicyKit1/Authority -m None`, see https://github.com/diwic/dbus-rs

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
                .map(|r: (arg::Variant<Box<dyn arg::RefArg + 'static>>,)| r.0)
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
                .map(
                    |r: (
                        ::std::collections::HashMap<
                            String,
                            arg::Variant<Box<dyn arg::RefArg + 'static>>,
                        >,
                    )| r.0,
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
                    .map(|r: (String,)| r.0)
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
                    .map(|r: (String,)| r.0)
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
                .map(
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
                    )| r.0,
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
                .map(|r: ((bool, bool, ::std::collections::HashMap<String, String>),)| r.0)
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
                .map(
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
                    )| r.0,
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
                    self,
                    "org.freedesktop.PolicyKit1.Authority",
                    "BackendName",
                )
            }

            fn backend_version(&self) -> Result<String, dbus::Error> {
                <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                    self,
                    "org.freedesktop.PolicyKit1.Authority",
                    "BackendVersion",
                )
            }

            fn backend_features(&self) -> Result<u32, dbus::Error> {
                <Self as blocking::stdintf::org_freedesktop_dbus::Properties>::get(
                    self,
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
