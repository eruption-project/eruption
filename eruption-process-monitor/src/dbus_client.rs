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

use crate::constants;
// use dbus::arg::RefArg;
// use dbus::blocking::stdintf::org_freedesktop_dbus::PropertiesPropertiesChanged;
use dbus::blocking::Connection;
use std::time::Duration;

type Result<T> = std::result::Result<T, eyre::Error>;

/// Messages received via D-Bus
#[derive(Debug, Clone)]
pub enum Message {
    /// Slot has been changed
    SlotChanged,

    /// Slot name has been changed
    SlotNamesChanged(Vec<String>),

    /// Profile has been changed
    ProfileChanged(String),

    /// Brightness has been changed
    BrightnessChanged(usize),

    /// SoundFX has been toggled
    SoundFxChanged(bool),
}

/// Switch the currently active profile
pub fn switch_profile(name: &str) -> Result<()> {
    use profile::OrgEruptionProfile;

    let conn = Connection::new_system()?;
    let proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/profile",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS),
    );

    let _result = proxy.switch_profile(name)?;

    Ok(())
}

/// Switch the currently active slot
pub fn switch_slot(index: u64) -> Result<()> {
    use slot::OrgEruptionSlot;

    let conn = Connection::new_system()?;
    let proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/slot",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS),
    );

    let _result = proxy.switch_slot(index)?;

    Ok(())
}

pub fn get_active_profile() -> Result<String> {
    use profile::OrgEruptionProfile;

    let conn = Connection::new_system()?;
    let proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/profile",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS),
    );

    let result = proxy.active_profile()?;

    Ok(result)
}

pub fn get_active_slot() -> Result<u64> {
    use slot::OrgEruptionSlot;

    let conn = Connection::new_system()?;
    let proxy = conn.with_proxy(
        "org.eruption",
        "/org/eruption/slot",
        Duration::from_secs(constants::DBUS_TIMEOUT_MILLIS),
    );

    let result = proxy.active_slot()?;

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
