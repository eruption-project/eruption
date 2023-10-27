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

use dbus::arg::{Iter, IterAppend};
use dbus_tree::{EmitsChangedSignal, MethodErr, MethodResult};

use std::sync::atomic::Ordering;
use tracing::*;

use crate::script;

use super::{
    convenience::FactoryWithPermission, convenience::InterfaceAddend,
    convenience::PropertyWithPermission, perms::Permission, Factory, Interface, MethodInfo,
};

pub struct StatusInterface {}

impl StatusInterface {
    pub fn new() -> Self {
        Self {}
    }
}

impl InterfaceAddend for StatusInterface {
    fn add_to_interface(&self, f: &Factory, interface: Interface) -> Interface {
        interface
            .add_p(
                f.property::<bool, _>("Running", ())
                    .emits_changed(EmitsChangedSignal::True)
                    .on_get_with_permission(Permission::Monitor, get_running)
                    .on_set_with_permission(Permission::Settings, set_running),
            )
            .add_m(
                f.method_with_permission("GetLedColors", Permission::Monitor, get_led_colors)
                    .outarg::<Vec<(u8, u8, u8, u8)>, _>("values"),
            )
            .add_m(
                f.method_with_permission(
                    "GetManagedDevices",
                    Permission::Monitor,
                    get_managed_devices,
                )
                .outarg::<(Vec<(u16, u16)>, Vec<(u16, u16)>, Vec<(u16, u16)>), _>("values"),
            )
    }
}

fn get_running(i: &mut IterAppend, _m: &super::PropertyInfo) -> super::PropertyResult {
    i.append(true);
    Ok(())
}

fn set_running(i: &mut Iter, _m: &super::PropertyInfo) -> super::PropertyResult {
    let _b: bool = i.read()?;

    //TODO: Implement this
    warn!("Not implemented");

    Ok(())
}

fn get_led_colors(m: &MethodInfo) -> MethodResult {
    let s = script::LAST_RENDERED_LED_MAP
        .read()
        .iter()
        .map(|v| (v.r, v.g, v.b, v.a))
        .collect::<Vec<(u8, u8, u8, u8)>>();

    Ok(vec![m.msg.method_return().append1(s)])
}

fn get_managed_devices(m: &MethodInfo) -> MethodResult {
    if crate::QUIT.load(Ordering::SeqCst) {
        return Err(MethodErr::failed("Eruption is shutting down"));
    }

    let keyboards = crate::DEVICES
        .read()
        .iter()
        .filter_map(|(_handle, device)| {
            let device = device.read();

            if device.get_device_class() == crate::hwdevices::DeviceClass::Keyboard {
                Some((device.get_usb_vid(), device.get_usb_pid()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let mice = crate::DEVICES
        .read()
        .iter()
        .filter_map(|(_handle, device)| {
            let device = device.read();

            if device.get_device_class() == crate::hwdevices::DeviceClass::Mouse {
                Some((device.get_usb_vid(), device.get_usb_pid()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let misc = crate::DEVICES
        .read()
        .iter()
        .filter_map(|(_handle, device)| {
            let device = device.read();

            if device.get_device_class() == crate::hwdevices::DeviceClass::Misc {
                Some((device.get_usb_vid(), device.get_usb_pid()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(vec![m.msg.method_return().append1((keyboards, mice, misc))])
}
