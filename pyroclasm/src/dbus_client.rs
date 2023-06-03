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

use crate::{constants, util::RGBA};

use dbus::blocking::Connection;
use std::path::Path;
use std::time::Duration;

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

    /// Brightness has been changed
    BrightnessChanged(i64),

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

/// Fetches all LED color values from the eruption daemon
pub fn get_slot_names() -> Result<Vec<RGBA>> {
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

#[path = "../../support/dbus/interfaces/rust/org.eruption.fx_proxy/effects.rs"]
pub mod fx_proxy;
