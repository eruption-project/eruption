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

use crate::{
    color_scheme::{ColorScheme, ColorSchemeExt},
    constants,
};
use dbus::blocking::Connection;
use dbus::nonblock;
use dbus_tokio::connection;
use std::{sync::Arc, time::Duration};

type Result<T> = std::result::Result<T, eyre::Error>;

/// Returns a connection to the D-Bus system bus using the specified `path`
pub async fn dbus_system_bus(
    path: &str,
) -> Result<dbus::nonblock::Proxy<'_, Arc<dbus::nonblock::SyncConnection>>> {
    let (resource, conn) = connection::new_system_sync()?;

    tokio::spawn(async {
        let err = resource.await;
        panic!("Lost connection to D-Bus: {err}");
    });

    let proxy = nonblock::Proxy::new(
        "org.eruption",
        path,
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
        conn,
    );

    Ok(proxy)
}

/// Returns a connection to the D-Bus session bus using the specified `path`
pub async fn dbus_session_bus<'a>(
    dest: &'a str,
    path: &'a str,
) -> Result<dbus::nonblock::Proxy<'a, Arc<dbus::nonblock::SyncConnection>>> {
    let (resource, conn) = connection::new_session_sync()?;

    tokio::spawn(async {
        let err = resource.await;
        panic!("Lost connection to D-Bus: {err}");
    });

    let proxy = nonblock::Proxy::new(
        dest,
        path,
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
        conn,
    );

    Ok(proxy)
}

pub fn enable_ambient_effect() -> Result<()> {
    use fx_proxy::OrgEruptionFxProxyEffects;

    let conn = Connection::new_session()?;
    let proxy = conn.with_proxy(
        "org.eruption.fx_proxy",
        "/org/eruption/fx_proxy/effects",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    proxy.enable_ambient_effect()?;

    Ok(())
}

pub fn disable_ambient_effect() -> Result<()> {
    use fx_proxy::OrgEruptionFxProxyEffects;

    let conn = Connection::new_session()?;
    let proxy = conn.with_proxy(
        "org.eruption.fx_proxy",
        "/org/eruption/fx_proxy/effects",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    proxy.disable_ambient_effect()?;

    Ok(())
}

pub fn is_ambient_effect_enabled() -> Result<bool> {
    use fx_proxy::OrgEruptionFxProxyEffects;

    let conn = Connection::new_session()?;
    let proxy = conn.with_proxy(
        "org.eruption.fx_proxy",
        "/org/eruption/fx_proxy/effects",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS as u64),
    );

    let result = proxy.ambient_effect()?;

    Ok(result)
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
