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

use dbus::arg::IterAppend;
use dbus_tree::{Access, EmitsChangedSignal, MethodErr, MethodResult, Signal};
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tracing::*;

use crate::script;

use super::{
    convenience::FactoryWithPermission, convenience::InterfaceAddend,
    convenience::PropertyWithPermission, perms::Permission, DbusApiError, DeviceStatus, Factory,
    Interface, MethodInfo, Property,
};

pub type Result<T> = std::result::Result<T, eyre::Error>;

pub struct DevicesInterface {
    pub device_status_changed_signal: Arc<Signal<()>>,
    pub device_hotplug_signal: Arc<Signal<()>>,
    device_status_property: Arc<Property>,
}

impl DevicesInterface {
    pub fn new(f: &Factory) -> Self {
        let device_status_changed_signal = Arc::new(
            f.signal("DeviceStatusChanged", ())
                .sarg::<String, _>("status"),
        );

        let device_hotplug_signal = Arc::new(
            f.signal("DeviceHotplug", ())
                .sarg::<(u16, u16, bool), _>("device_info"),
        );

        let device_status_property = Arc::new(
            f.property::<String, _>("DeviceStatus", ())
                .emits_changed(EmitsChangedSignal::True)
                .access(Access::Read)
                .on_get_with_permission(Permission::Monitor, get_device_status_property),
        );

        Self {
            device_status_changed_signal,
            device_hotplug_signal,
            device_status_property,
        }
    }
}

impl InterfaceAddend for DevicesInterface {
    fn add_to_interface(&self, f: &Factory, interface: Interface) -> Interface {
        interface
            .add_s(self.device_status_changed_signal.clone())
            .add_s(self.device_hotplug_signal.clone())
            .add_m(
                f.method_with_permission(
                    "SetDeviceConfig",
                    Permission::Settings,
                    set_device_config,
                )
                .inarg::<u64, _>("device")
                .inarg::<String, _>("param")
                .inarg::<String, _>("value")
                .outarg::<bool, _>("status"),
            )
            .add_m(
                f.method_with_permission(
                    "GetDeviceConfig",
                    Permission::Settings,
                    get_device_config,
                )
                .inarg::<u64, _>("device")
                .inarg::<String, _>("param")
                .outarg::<String, _>("value"),
            )
            .add_m(
                f.method_with_permission("GetDeviceStatus", Permission::Monitor, get_device_status)
                    .inarg::<u64, _>("device")
                    .outarg::<String, _>("status"),
            )
            .add_m(
                f.method_with_permission(
                    "GetManagedDevices",
                    Permission::Monitor,
                    get_managed_devices,
                )
                .outarg::<(Vec<(u16, u16)>, Vec<(u16, u16)>, Vec<(u16, u16)>), _>("values"),
            )
            .add_p(self.device_status_property.clone())
    }
}

pub fn get_device_specific_ids(device: u64) -> Result<(u16, u16)> {
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

fn get_device_status_property(
    i: &mut IterAppend,
    _m: &super::PropertyInfo,
) -> super::PropertyResult {
    let device_status = &*crate::DEVICE_STATUS.as_ref().read();

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
        .map_err(|e| MethodErr::failed(&format!("{e}")))?;

    i.append(result);

    Ok(())
}

fn set_device_config(m: &MethodInfo) -> MethodResult {
    let (device, param, value): (u64, String, String) = m.msg.read3()?;

    debug!(
        "Setting device [{}] config parameter '{}' to '{}'",
        device, &param, &value
    );

    apply_device_specific_configuration(device, &param, &value)
        .map_err(|_e| MethodErr::invalid_arg(&param))?;

    Ok(vec![m.msg.method_return().append1(true)])
}

fn get_device_config(m: &MethodInfo) -> MethodResult {
    let (device, param): (u64, String) = m.msg.read2()?;

    trace!("Querying device [{}] config parameter '{}'", device, &param);

    let result = query_device_specific_configuration(device, &param)
        .map_err(|_e| MethodErr::invalid_arg(&param))?;

    Ok(vec![m.msg.method_return().append1(result)])
}

fn get_device_status(m: &MethodInfo) -> MethodResult {
    let device: u64 = m.msg.read1()?;

    trace!("Querying device [{}] status", device);

    let result =
        query_device_specific_status(device).map_err(|e| MethodErr::failed(&format!("{e}")))?;

    Ok(vec![m.msg.method_return().append1(result)])
}

/// Query the device specific status from the global status store
fn query_device_specific_status(device: u64) -> Result<String> {
    let device_status = crate::DEVICE_STATUS.as_ref().read();

    match device_status.get(&device) {
        Some(status) => Ok(serde_json::to_string_pretty(&status.0)?),
        None => Err(DbusApiError::InvalidDevice {}.into()),
    }
}

fn get_managed_devices(m: &MethodInfo) -> MethodResult {
    if crate::QUIT.load(Ordering::SeqCst) {
        return Err(MethodErr::failed("Eruption is shutting down"));
    }

    let keyboards = {
        let keyboards = crate::KEYBOARD_DEVICES.read();

        let keyboards: Vec<(u16, u16)> = keyboards
            .iter()
            .map(|device| (device.read().get_usb_vid(), device.read().get_usb_pid()))
            .collect();

        keyboards
    };

    let mice = {
        let mice = crate::MOUSE_DEVICES.read();

        let mice: Vec<(u16, u16)> = mice
            .iter()
            .map(|device| (device.read().get_usb_vid(), device.read().get_usb_pid()))
            .collect();

        mice
    };

    let misc = {
        let misc = crate::MISC_DEVICES.read();

        let misc: Vec<(u16, u16)> = misc
            .iter()
            .map(|device| (device.read().get_usb_vid(), device.read().get_usb_pid()))
            .collect();

        misc
    };

    Ok(vec![m.msg.method_return().append1((keyboards, mice, misc))])
}

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

                Ok(format!("{brightness}"))
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

                Ok(format!("{profile}"))
            }

            "dpi" => {
                let dpi = device.read().get_dpi()?;

                Ok(format!("{dpi}"))
            }

            "rate" => {
                let rate = device.read().get_rate()?;

                Ok(format!("{rate}"))
            }

            "dcu" => {
                let dcu_config = device.read().get_dcu_config()?;

                Ok(format!("{dcu_config}"))
            }

            "angle-snapping" => {
                let angle_snapping = device.read().get_angle_snapping()?;

                Ok(format!("{angle_snapping}"))
            }

            "debounce" => {
                let debounce = device.read().get_debounce()?;

                Ok(format!("{debounce}"))
            }

            "brightness" => {
                let brightness = device.read().get_local_brightness()?;

                Ok(format!("{brightness}"))
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

                Ok(format!("{brightness}"))
            }

            _ => Err(DbusApiError::InvalidParameter {}.into()),
        }
    } else {
        Err(DbusApiError::InvalidDevice {}.into())
    }
}
