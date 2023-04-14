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

use dbus::{arg::RefArg, arg::Variant, blocking::Connection};
use lazy_static::lazy_static;
use parking_lot::RwLock;
use std::sync::Arc;
use std::{collections::HashMap, time::Duration};

use crate::constants;
use crate::dbus_interface::bus;
use crate::dbus_interface::polkit;

pub type Result<T> = std::result::Result<T, eyre::Error>;

// cached permissions
lazy_static! {
    static ref HAS_MONITOR_PERMISSION: Arc<RwLock<Option<bool>>> = Arc::new(RwLock::new(None));
    static ref HAS_SETTINGS_PERMISSION: Arc<RwLock<Option<bool>>> = Arc::new(RwLock::new(None));
    static ref HAS_MANAGE_PERMISSION: Arc<RwLock<Option<bool>>> = Arc::new(RwLock::new(None));
}

#[derive(Clone, Copy)]
pub enum Permission {
    Monitor,
    Settings,
    Manage,
}

pub fn has_permission_cached(permission: Permission, sender: &str) -> Result<bool> {
    return Ok(true);

    match permission {
        Permission::Monitor => has_monitor_permission_cached(sender),
        Permission::Settings => has_settings_permission_cached(sender),
        Permission::Manage => has_manage_permission_cached(sender),
    }
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

#[allow(dead_code)]
pub fn has_permission(permission: Permission, sender: &str) -> Result<(bool, bool)> {
    match permission {
        Permission::Monitor => has_monitor_permission(sender),
        Permission::Settings => has_settings_permission(sender),
        Permission::Manage => has_manage_permission(sender),
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
            "eruption-1",
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
            "eruption-2",
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
            "eruption-3",
        )?;

        let dismissed = result.2.get("polkit.dismissed").is_some();

        if (result.0 && !dismissed) || (!result.0 && dismissed) {
            // we have either been dismissed with 'cancel' or the authentication succeeded
            break 'AUTH_LOOP (result, dismissed);
        }
    };

    Ok((result.0 .0, false))
}
