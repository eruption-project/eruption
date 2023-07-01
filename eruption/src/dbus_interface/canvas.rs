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

use dbus::{
    arg::{Iter, IterAppend},
    MethodErr,
};
use dbus_tree::{Access, EmitsChangedSignal, MethodResult, Signal};
use std::sync::Arc;

use crate::{constants, hwdevices::Zone};

use super::{
    convenience::PropertyWithPermission,
    convenience::{FactoryWithPermission, InterfaceAddend},
    perms::Permission,
    Factory, Interface, Property,
};

// pub type Result<T> = std::result::Result<T, eyre::Error>;

pub struct CanvasInterface {
    pub hue_changed_signal: Arc<Signal<()>>,
    pub saturation_changed_signal: Arc<Signal<()>>,
    pub lightness_changed_signal: Arc<Signal<()>>,
    hue_property: Arc<Property>,
    saturation_property: Arc<Property>,
    lightness_property: Arc<Property>,
}

impl CanvasInterface {
    pub fn new(f: &Factory) -> Self {
        let hue_changed_signal = Arc::new(f.signal("HueChanged", ()).sarg::<f64, _>("hue"));

        let saturation_changed_signal = Arc::new(
            f.signal("SaturationChanged", ())
                .sarg::<f64, _>("saturation"),
        );

        let lightness_changed_signal =
            Arc::new(f.signal("LightnessChanged", ()).sarg::<f64, _>("lightness"));

        let hue_property = Arc::new(
            f.property::<f64, _>("Hue", ())
                .emits_changed(EmitsChangedSignal::True)
                .access(Access::ReadWrite)
                .auto_emit_on_set(true)
                .on_get_with_permission(Permission::Monitor, get_hue)
                .on_set_with_permission(Permission::Settings, set_hue),
        );

        let saturation_property = Arc::new(
            f.property::<f64, _>("Saturation", ())
                .emits_changed(EmitsChangedSignal::True)
                .access(Access::ReadWrite)
                .auto_emit_on_set(true)
                .on_get_with_permission(Permission::Monitor, get_saturation)
                .on_set_with_permission(Permission::Settings, set_saturation),
        );

        let lightness_property = Arc::new(
            f.property::<f64, _>("Lightness", ())
                .emits_changed(EmitsChangedSignal::True)
                .access(Access::ReadWrite)
                .auto_emit_on_set(true)
                .on_get_with_permission(Permission::Monitor, get_lightness)
                .on_set_with_permission(Permission::Settings, set_lightness),
        );

        Self {
            hue_changed_signal,
            saturation_changed_signal,
            lightness_changed_signal,
            hue_property,
            saturation_property,
            lightness_property,
        }
    }
}

impl InterfaceAddend for CanvasInterface {
    fn add_to_interface(&self, f: &Factory, interface: Interface) -> Interface {
        interface
            .add_s(self.hue_changed_signal.clone())
            .add_s(self.saturation_changed_signal.clone())
            .add_s(self.lightness_changed_signal.clone())
            .add_p(self.hue_property.clone())
            .add_p(self.saturation_property.clone())
            .add_p(self.lightness_property.clone())
            .add_m(
                f.method_with_permission(
                    "GetDevicesZoneAllocations",
                    Permission::Monitor,
                    get_devices_zone_allocations,
                )
                // .inarg::<u64, _>("device")
                .outarg::<Vec<(u64, Zone)>, _>("zones"),
            )
            .add_m(
                f.method_with_permission(
                    "SetDevicesZoneAllocations",
                    Permission::Settings,
                    set_devices_zone_allocations,
                )
                .inarg::<Vec<(u64, Zone)>, _>("zones"),
            )
            .add_m(
                f.method_with_permission(
                    "SetDeviceZoneAllocation",
                    Permission::Settings,
                    set_device_zone_allocation,
                )
                .inarg::<u64, _>("device")
                .inarg::<Zone, _>("zone"),
            )
    }
}

fn get_hue(i: &mut IterAppend, _m: &super::PropertyInfo) -> super::PropertyResult {
    i.append(crate::CANVAS_HSL.write().0);
    Ok(())
}

fn set_hue(i: &mut Iter, _m: &super::PropertyInfo) -> super::PropertyResult {
    crate::CANVAS_HSL.write().0 = i.read::<f64>()?;
    Ok(())
}

fn get_saturation(i: &mut IterAppend, _m: &super::PropertyInfo) -> super::PropertyResult {
    i.append(crate::CANVAS_HSL.write().1);
    Ok(())
}

fn set_saturation(i: &mut Iter, _m: &super::PropertyInfo) -> super::PropertyResult {
    crate::CANVAS_HSL.write().1 = i.read::<f64>()?;
    Ok(())
}

fn get_lightness(i: &mut IterAppend, _m: &super::PropertyInfo) -> super::PropertyResult {
    i.append(crate::CANVAS_HSL.write().2);
    Ok(())
}

fn set_lightness(i: &mut Iter, _m: &super::PropertyInfo) -> super::PropertyResult {
    crate::CANVAS_HSL.write().2 = i.read::<f64>()?;
    Ok(())
}

fn get_devices_zone_allocations(m: &super::MethodInfo) -> MethodResult {
    let mut result: Vec<(u64, Zone)> = Vec::new();
    let mut cntr = 0;

    let keyboards = crate::KEYBOARD_DEVICES.read();

    for device in keyboards.iter() {
        result.push((cntr, device.read().get_allocated_zone()));

        cntr += 1;
    }

    let mice = crate::MOUSE_DEVICES.read();

    for device in mice.iter() {
        result.push((cntr, device.read().get_allocated_zone()));

        cntr += 1;
    }

    let misc = crate::MISC_DEVICES.read();

    for device in misc.iter() {
        result.push((cntr, device.read().get_allocated_zone()));

        cntr += 1;
    }

    Ok(vec![m.msg.method_return().append1(result)])
}

fn set_devices_zone_allocations(m: &super::MethodInfo) -> MethodResult {
    Ok(vec![m.msg.method_return()])
}

fn set_device_zone_allocation(m: &super::MethodInfo) -> MethodResult {
    let (device, zone): (u64, (i32, i32, i32, i32, bool)) = m.msg.read2()?;

    let zone = Zone::new(zone.0, zone.1, zone.2, zone.3, zone.4);

    if zone.x < 0
        || zone.x > constants::CANVAS_WIDTH as i32
        || zone.y < 0
        || zone.y > constants::CANVAS_HEIGHT as i32
        || zone.x2() < 0
        || zone.x2() > constants::CANVAS_WIDTH as i32
        || zone.y2() < 0
        || zone.y2() > constants::CANVAS_HEIGHT as i32
    {
        return Err(MethodErr::failed("Invalid zone dimensions"));
    }

    if (device as usize) < crate::KEYBOARD_DEVICES.read().len() {
        let device = &crate::KEYBOARD_DEVICES.read()[device as usize];

        device.write().set_zone_allocation(zone);

        Ok(vec![m.msg.method_return()])
    } else if (device as usize)
        < (crate::KEYBOARD_DEVICES.read().len() + crate::MOUSE_DEVICES.read().len())
    {
        let index = device as usize - crate::KEYBOARD_DEVICES.read().len();
        let device = &crate::MOUSE_DEVICES.read()[index];

        device.write().set_zone_allocation(zone);

        Ok(vec![m.msg.method_return()])
    } else if (device as usize)
        < (crate::KEYBOARD_DEVICES.read().len()
            + crate::MOUSE_DEVICES.read().len()
            + crate::MISC_DEVICES.read().len())
    {
        let index = device as usize
            - (crate::KEYBOARD_DEVICES.read().len() + crate::MOUSE_DEVICES.read().len());
        let device = &crate::MISC_DEVICES.read()[index];

        device.write().set_zone_allocation(zone);

        Ok(vec![m.msg.method_return()])
    } else {
        Err(MethodErr::failed("Invalid device index"))
    }
}
