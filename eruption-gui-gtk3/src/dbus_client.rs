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

#![allow(dead_code)]

use crate::color_scheme::{ColorScheme, ColorSchemeExt};
use crate::zone::Zone;
use crate::{constants, util::RGBA};
use dbus::arg::RefArg;
use dbus::blocking::stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged;
use dbus::blocking::Connection;
use glib::clone;
use std::time::Duration;
use std::{path::Path, thread};

/// Messages received via D-Bus
#[derive(Debug)]
pub enum Message {
    /// Slot has been changed
    SlotChanged(usize),

    /// Slot name has been changed
    SlotNamesChanged(Vec<String>),

    /// Profile has been changed
    ProfileChanged(String),

    /// A device has been hotplugged
    DeviceHotplug((u16, u16, bool)),

    /// A device's status has been updated
    DeviceStatusChanged(String),

    /// Brightness has been changed
    BrightnessChanged(i64),

    /// Hue has been changed
    HueChanged(f64),

    /// Saturation has been changed
    SaturationChanged(f64),

    /// Lightness has been changed
    LightnessChanged(f64),

    /// Ambient effect has been toggled
    AmbientEffectChanged(bool),

    /// SoundFX has been toggled
    SoundFxChanged(bool),

    /// Process-monitor rules changed
    RulesChanged,
}

type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum DbusClientError {
    #[error("Unknown error: {description}")]
    UnknownError { description: String },

    #[error("Authentication failed: {description}")]
    AuthError { description: String },

    #[error("Method call failed: {description}")]
    MethodFailed { description: String },
}

type CallbackFn = dyn Fn(&gtk::Builder, &Message) -> crate::Result<()>;

/// Spawn a thread that listens for events on D-Bus
pub fn spawn_dbus_event_loop_system(
    builder: &gtk::Builder,
    callback: &'static CallbackFn,
) -> Result<()> {
    let (tx, rx) = glib::MainContext::channel(glib::Priority::DEFAULT);

    thread::spawn(move || -> Result<()> {
        let conn = Connection::new_system().unwrap();
        let slot_proxy = conn.with_proxy(
            "org.eruption",
            "/org/eruption/slot",
            Duration::from_millis(4000),
        );

        let profile_proxy = conn.with_proxy(
            "org.eruption",
            "/org/eruption/profile",
            Duration::from_millis(4000),
        );

        let config_proxy = conn.with_proxy(
            "org.eruption",
            "/org/eruption/config",
            Duration::from_millis(4000),
        );

        let devices_proxy = conn.with_proxy(
            "org.eruption",
            "/org/eruption/devices",
            Duration::from_millis(4000),
        );

        let canvas_proxy = conn.with_proxy(
            "org.eruption",
            "/org/eruption/canvas",
            Duration::from_millis(4000),
        );

        let _id1 = slot_proxy.match_signal(
            clone!(@strong tx => move |h: slot::OrgEruptionSlotActiveSlotChanged,
                  _: &Connection,
                  _message: &dbus::Message| {
                tx.send(Message::SlotChanged(h.slot as usize)).unwrap();

                true
            }),
        )?;

        let _id1_1 = slot_proxy
            .match_signal(
                clone!(@strong tx => move |h: slot::OrgFreedesktopDBusPropertiesPropertiesChanged,
                      _: &Connection,
                      _message: &dbus::Message| {

                    // slot names have been changed
                    if let Some(args) = h.changed_properties.get("SlotNames") {
                        let slot_names = args.0.as_iter().unwrap().map(|v| v.as_str().unwrap().to_string()).collect::<Vec<String>>();
                        tx.send(Message::SlotNamesChanged(slot_names)).unwrap();
                    }

                    true
                }),
            )?;

        let _id2 = profile_proxy.match_signal(
            clone!(@strong tx => move |h: profile::OrgEruptionProfileActiveProfileChanged,
                  _: &Connection,
                  _message: &dbus::Message| {
                tx.send(Message::ProfileChanged(h.profile_name))
                    .unwrap();

                true
            }),
        )?;

        let _id3 = config_proxy.match_signal(
            clone!(@strong tx => move |h: config::OrgEruptionConfigBrightnessChanged,
                  _: &Connection,
                  _message: &dbus::Message| {
                tx.send(Message::BrightnessChanged(h.brightness))
                    .unwrap();

                true
            }),
        )?;

        let _id3_1 = config_proxy.match_signal(
            clone!(@strong tx => move |h: PropertiesPropertiesChanged,
                  _: &Connection,
                  _message: &dbus::Message| {

                if let Some(brightness) = h.changed_properties.get("Brightness") {
                    let brightness = brightness.0.as_i64().unwrap();

                    tx.send(Message::BrightnessChanged(brightness))
                        .unwrap();
                }

                if let Some(result) = h.changed_properties.get("EnableSfx") {
                    let enabled = result.0.as_u64().unwrap() != 0;

                    tx.send(Message::SoundFxChanged(enabled))
                        .unwrap();
                }

                true
            }),
        )?;

        let _id4 = devices_proxy.match_signal(
            clone!(@strong tx => move |h: devices::OrgEruptionDeviceDeviceHotplug,
                  _: &Connection,
                  _message: &dbus::Message| {

                tx.send(Message::DeviceHotplug(h.device_info))
                    .unwrap();

                true
            }),
        )?;

        let _id4_1 = devices_proxy.match_signal(
            clone!(@strong tx => move |h: devices::OrgEruptionDeviceDeviceStatusChanged,
                  _: &Connection,
                  _message: &dbus::Message| {

                tx.send(Message::DeviceStatusChanged(h.status))
                    .unwrap();

                true
            }),
        )?;

        let _id4_2 = devices_proxy.match_signal(
            clone!(@strong tx => move |_h: PropertiesPropertiesChanged,
                  _: &Connection,
                  _message: &dbus::Message| {

                true
            }),
        )?;

        let _id5 = canvas_proxy.match_signal(
            clone!(@strong tx => move |h: canvas::OrgEruptionCanvasHueChanged,
                  _: &Connection,
                  _message: &dbus::Message| {

                tx.send(Message::HueChanged(h.hue))
                    .unwrap();

                true
            }),
        )?;

        let _id5_1 = canvas_proxy.match_signal(
            clone!(@strong tx => move |h: canvas::OrgEruptionCanvasSaturationChanged,
                  _: &Connection,
                  _message: &dbus::Message| {

                    tx.send(Message::SaturationChanged(h.saturation))
                    .unwrap();

                true
            }),
        )?;

        let _id5_2 = canvas_proxy.match_signal(
            clone!(@strong tx => move |h: canvas::OrgEruptionCanvasLightnessChanged,
                  _: &Connection,
                  _message: &dbus::Message| {

                    tx.send(Message::LightnessChanged(h.lightness))
                    .unwrap();

                true
            }),
        )?;

        loop {
            conn.process(Duration::from_millis(4000))?;
        }
    });

    rx.attach(
        None,
        clone!(@strong builder, @strong callback => @default-return glib::ControlFlow::Continue, move |event| {
            callback(&builder, &event).unwrap();
            // thread::yield_now();

            glib::ControlFlow::Continue
        }),
    );

    Ok(())
}

/// Spawn a thread that listens for events on the D-Bus session bus
pub fn spawn_dbus_event_loop_session(
    builder: &gtk::Builder,
    callback: &'static CallbackFn,
) -> Result<()> {
    // use process_monitor::OrgEruptionProcessMonitorRules;

    let (tx, rx) = glib::MainContext::channel(glib::Priority::DEFAULT);

    thread::spawn(move || -> Result<()> {
        let conn = Connection::new_session()?;

        let rules_proxy = conn.with_proxy(
            "org.eruption.process_monitor",
            "/org/eruption/process_monitor/rules",
            Duration::from_millis(4000),
        );

        let _id1 = rules_proxy
            .match_signal(
                clone!(@strong tx => move |_h: process_monitor::OrgEruptionProcessMonitorRulesRulesChanged,
                      _: &Connection,
                      _message: &dbus::Message| {

                        tx.send(Message::RulesChanged).unwrap();

                        false
                }),
            )?;

        // let _id1_1 = rules_proxy.match_signal(
        //     move |h: PropertiesPropertiesChanged, _: &Connection, _message: &dbus::Message| {
        //         tracing::info!("{:?}", h);

        //         true
        //     },
        // )?;

        let fx_proxy_proxy = conn.with_proxy(
            "org.eruption.fx_proxy",
            "/org/eruption/fx_proxy/effects",
            Duration::from_millis(4000),
        );

        let _id2_1 = fx_proxy_proxy.match_signal(
            clone!(@strong tx => move |h: PropertiesPropertiesChanged,
                  _: &Connection,
                  _message: &dbus::Message| {

                if let Some(ambient_effect) = h.changed_properties.get("AmbientEffect") {
                    let enabled = ambient_effect.0.as_u64().unwrap() != 0;

                    tx.send(Message::AmbientEffectChanged(enabled))
                        .unwrap();
                }

                true
            }),
        )?;

        loop {
            conn.process(Duration::from_millis(4000))?;
        }
    });

    rx.attach(
        None,
        clone!(@strong builder, @strong callback => @default-return glib::ControlFlow::Continue, move |event| {
            callback(&builder, &event).unwrap();
            // thread::yield_now();

            glib::ControlFlow::Continue

        }),
    );

    Ok(())
}

/// Instruct the daemon to write a .profile file or a Lua script or manifest
pub fn write_file<P: AsRef<Path>>(path: &P, data: &str) -> Result<()> {
    use self::config::OrgEruptionConfig;

    let conn = Connection::new_system()?;
    let proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/config",
        Duration::from_secs(35),
    );

    if let Err(e) = proxy.write_file(&path.as_ref().to_string_lossy(), data) {
        tracing::error!("{}", e);

        Err(DbusClientError::MethodFailed {
            description: format!("{e}"),
        }
        .into())
    } else {
        Ok(())
    }
}

pub fn ping() -> Result<()> {
    use self::config::OrgEruptionConfig;

    let conn = Connection::new_system()?;
    let proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/config",
        Duration::from_secs(35),
    );

    if let Err(e) = proxy.ping() {
        tracing::error!("{}", e);

        Err(DbusClientError::MethodFailed {
            description: format!("{e}"),
        }
        .into())
    } else {
        Ok(())
    }
}

pub fn ping_privileged() -> Result<()> {
    use self::config::OrgEruptionConfig;

    let conn = Connection::new_system()?;
    let proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/config",
        Duration::from_secs(35),
    );

    if let Err(e) = proxy.ping_privileged() {
        tracing::error!("{}", e);

        Err(DbusClientError::MethodFailed {
            description: format!("{e}"),
        }
        .into())
    } else {
        Ok(())
    }
}

pub fn set_parameter(
    profile_file: &str,
    script_file: &str,
    param_name: &str,
    value: &str,
) -> Result<()> {
    use profile::OrgEruptionProfile;

    let conn = Connection::new_system()?;
    let proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/profile",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let _result = proxy.set_parameter(profile_file, script_file, param_name, value)?;

    Ok(())
}

// TODO: This currently fails with a dbus error, use util::get_slot_names() for now
/// Fetches all slot names
// pub fn get_slot_names() -> Result<Vec<String>> {
//     use slot::OrgEruptionSlot;

//     let conn = Connection::new_system()?;
//     let slot_proxy = conn.with_proxy(
//         "org.eruption",
//         "/org/eruption/slots",
//         Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
//     );

//     let result = slot_proxy.slot_names()?;

//     Ok(result)
// }

pub fn get_color_schemes() -> Result<Vec<String>> {
    use self::config::OrgEruptionConfig;

    let conn = Connection::new_system()?;
    let config_proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/config",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let result = config_proxy.get_color_schemes()?;

    Ok(result)
}

pub fn get_color_scheme(name: &str) -> Result<ColorScheme> {
    use self::config::OrgEruptionConfig;

    let conn = Connection::new_system()?;
    let config_proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/config",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    config_proxy.get_color_scheme(name)?.try_into()
}

pub fn set_color_scheme(name: &str, color_scheme: &ColorScheme) -> Result<()> {
    use self::config::OrgEruptionConfig;

    let conn = Connection::new_system()?;
    let proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/config",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let mut data = Vec::new();

    for index in 0..color_scheme.num_colors() {
        let result = color_scheme.color_rgba_at(index)?.to_linear_rgba_u8();

        data.push(result.0);
        data.push(result.1);
        data.push(result.2);
        data.push(result.3);
    }

    let _result = proxy.set_color_scheme(name, data)?;

    Ok(())
}

pub fn remove_color_scheme(name: &str) -> Result<bool> {
    use self::config::OrgEruptionConfig;

    let conn = Connection::new_system()?;
    let config_proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/config",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let result = config_proxy.remove_color_scheme(name)?;

    Ok(result)
}

pub fn enumerate_process_monitor_rules() -> Result<Vec<(String, String, String, String)>> {
    use process_monitor::OrgEruptionProcessMonitorRules;

    let conn = Connection::new_session()?;
    let proxy = conn.with_proxy(
        "org.eruption.process_monitor",
        "/org/eruption/process_monitor/rules",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let result = proxy.enum_rules()?;

    Ok(result)
}

pub fn transmit_process_monitor_rules(rules: &[(&str, &str, &str, &str)]) -> Result<()> {
    use process_monitor::OrgEruptionProcessMonitorRules;

    let conn = Connection::new_session()?;
    let proxy = conn.with_proxy(
        "org.eruption.process_monitor",
        "/org/eruption/process_monitor/rules",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    proxy.set_rules(rules.to_vec())?;

    Ok(())
}

/// Fetches all LED color values from the eruption daemon
pub fn get_led_colors() -> Result<Vec<RGBA>> {
    use status::OrgEruptionStatus;

    let conn = Connection::new_system()?;
    let status_proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/status",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let result = status_proxy.get_led_colors()?;

    let result = result
        .iter()
        .map(|v| RGBA {
            r: v.0,
            g: v.1,
            b: v.2,
            a: v.3,
        })
        .collect::<Vec<RGBA>>();

    Ok(result)
}

/// Get managed devices USB IDs from the eruption daemon
pub fn get_managed_devices() -> Result<(Vec<(u16, u16)>, Vec<(u16, u16)>, Vec<(u16, u16)>)> {
    use status::OrgEruptionStatus;

    let conn = Connection::new_system()?;
    let status_proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/status",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let result = status_proxy.get_managed_devices()?;

    Ok(result)
}

/// Fetches all allocated zones from the eruption daemon
pub fn get_devices_zone_allocations() -> Result<Vec<Zone>> {
    use canvas::OrgEruptionCanvas;

    let conn = Connection::new_system()?;
    let status_proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/canvas",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let result = status_proxy.get_devices_zone_allocations()?;

    let result = result
        .iter()
        .map(|v| Zone {
            x: v.1 .0,
            y: v.1 .1,
            width: v.1 .2,
            height: v.1 .3,
            enabled: v.1 .4,
            device: Some(v.0),
        })
        .collect::<Vec<Zone>>();

    Ok(result)
}

/// Update all allocated zones in the eruption daemon
pub fn set_devices_zone_allocations(zones: &[(u64, Zone)]) -> Result<()> {
    use canvas::OrgEruptionCanvas;

    let conn = Connection::new_system()?;
    let status_proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/canvas",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let zones = zones
        .iter()
        .map(|(device, zone)| {
            (
                *device,
                (zone.x, zone.y, zone.width, zone.height, zone.enabled),
            )
        })
        .collect::<Vec<(u64, (i32, i32, i32, i32, bool))>>();

    status_proxy.set_devices_zone_allocations(zones)?;

    Ok(())
}

/// Update all allocated zones in the eruption daemon
pub fn set_device_zone_allocation(device: u64, zone: &Zone) -> Result<()> {
    use canvas::OrgEruptionCanvas;

    let conn = Connection::new_system()?;
    let status_proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/canvas",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let zone = (zone.x, zone.y, zone.width, zone.height, zone.enabled);
    status_proxy.set_device_zone_allocation(device, zone)?;

    Ok(())
}

/// Enable or disable a managed device
pub fn set_device_enabled(device_index: u64, enabled: bool) -> Result<()> {
    use devices::OrgEruptionDevice;

    let conn = Connection::new_system()?;
    let devices_proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/devices",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    devices_proxy.set_device_enabled(device_index, enabled)?;

    Ok(())
}

/// Is a managed device enabled or disabled
pub fn is_device_enabled(device_index: u64) -> Result<bool> {
    use devices::OrgEruptionDevice;

    let conn = Connection::new_system()?;
    let devices_proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/devices",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let result = devices_proxy.is_device_enabled(device_index)?;

    Ok(result)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceClass {
    Unknown,
    Keyboard,
    Mouse,
    Misc,
}

// import D-Bus to Rust glue code modules, autogenerated with: `support/shell/update-dbus-interfaces.sh`
#[path = "../../support/dbus/interfaces/rust/org.eruption/canvas.rs"]
pub mod canvas;

#[path = "../../support/dbus/interfaces/rust/org.eruption/config.rs"]
pub mod config;

#[path = "../../support/dbus/interfaces/rust/org.eruption/devices.rs"]
pub mod devices;

#[path = "../../support/dbus/interfaces/rust/org.eruption/profile.rs"]
pub mod profile;

#[path = "../../support/dbus/interfaces/rust/org.eruption/slot.rs"]
pub mod slot;

#[path = "../../support/dbus/interfaces/rust/org.eruption/status.rs"]
pub mod status;

#[path = "../../support/dbus/interfaces/rust/org.eruption.process_monitor/rules.rs"]
pub mod process_monitor;
