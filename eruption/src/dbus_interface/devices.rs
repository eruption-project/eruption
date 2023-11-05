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

use crate::{
    hwdevices::{DeviceClass, DeviceHandle},
    scripting::script,
};

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
            .add_m(
                f.method_with_permission(
                    "IsDeviceEnabled",
                    Permission::Settings,
                    is_device_enabled,
                )
                .inarg::<u64, _>("device")
                .outarg::<bool, _>("status"),
            )
            .add_m(
                f.method_with_permission(
                    "SetDeviceEnabled",
                    Permission::Settings,
                    set_device_enabled,
                )
                .inarg::<u64, _>("device")
                .inarg::<bool, _>("enabled")
                .outarg::<bool, _>("status"),
            )
            .add_p(self.device_status_property.clone())
    }
}

pub fn get_device_specific_ids(handle: &DeviceHandle) -> Result<(u16, u16)> {
    if let Some(device) = crate::DEVICES.read().unwrap().get(handle) {
        let device = device.read().unwrap();

        let usb_vid = device.get_usb_vid();
        let usb_pid = device.get_usb_pid();

        Ok((usb_vid, usb_pid))
    } else {
        Err(DbusApiError::InvalidDevice {}.into())
    }
}

fn get_device_status_property(
    i: &mut IterAppend,
    _m: &super::PropertyInfo,
) -> super::PropertyResult {
    let device_status = &*crate::DEVICE_STATUS.as_ref().read().unwrap();

    let device_status = device_status
        .iter()
        .map(|(k, v)| {
            let (usb_vid, usb_pid) =
                get_device_specific_ids(&DeviceHandle::from(*k)).unwrap_or_default();

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
    let device_status = crate::DEVICE_STATUS.as_ref().read().unwrap();

    match device_status.get(&device) {
        Some(status) => Ok(serde_json::to_string_pretty(&status.0)?),
        None => Err(DbusApiError::InvalidDevice {}.into()),
    }
}

fn get_managed_devices(m: &MethodInfo) -> MethodResult {
    if crate::QUIT.load(Ordering::SeqCst) {
        return Err(MethodErr::failed("Eruption is shutting down"));
    }

    let keyboards = crate::DEVICES
        .read()
        .unwrap()
        .iter()
        .filter_map(|(_handle, device)| {
            let device = device.read().unwrap();

            if device.get_device_class() == crate::hwdevices::DeviceClass::Keyboard {
                Some((device.get_usb_vid(), device.get_usb_pid()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let mice = crate::DEVICES
        .read()
        .unwrap()
        .iter()
        .filter_map(|(_handle, device)| {
            let device = device.read().unwrap();

            if device.get_device_class() == crate::hwdevices::DeviceClass::Mouse {
                Some((device.get_usb_vid(), device.get_usb_pid()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    let misc = crate::DEVICES
        .read()
        .unwrap()
        .iter()
        .filter_map(|(_handle, device)| {
            let device = device.read().unwrap();

            if device.get_device_class() == crate::hwdevices::DeviceClass::Misc {
                Some((device.get_usb_vid(), device.get_usb_pid()))
            } else {
                None
            }
        })
        .collect::<Vec<_>>();

    Ok(vec![m.msg.method_return().append1((keyboards, mice, misc))])
}

fn set_device_enabled(m: &MethodInfo) -> MethodResult {
    let (device, enabled): (u64, bool) = m.msg.read2()?;

    debug!("Setting device [{}] enabled: '{}'", device, enabled);

    Ok(vec![m.msg.method_return().append1(true)])
}

fn is_device_enabled(m: &MethodInfo) -> MethodResult {
    let device: u64 = m.msg.read1()?;

    trace!("Querying device [{}] is enabled", device);

    let result = true;

    Ok(vec![m.msg.method_return().append1(result)])
}

fn apply_device_specific_configuration(index: u64, param: &str, value: &str) -> Result<()> {
    let handle = DeviceHandle::from(index);

    if let Some(device) = crate::DEVICES.read().unwrap().get(&handle) {
        let mut device = device
            .write()
            .map_err(|_e| DbusApiError::LockingFailed {})?;

        match device.get_device_class() {
            DeviceClass::Keyboard => {
                let device = device.as_keyboard_device_mut().unwrap();

                match param {
                    "brightness" => {
                        let brightness = value.parse::<i32>()?;
                        device.set_brightness(brightness)?;

                        script::FRAME_GENERATION_COUNTER.fetch_add(1, Ordering::SeqCst);

                        Ok(())
                    }

                    _ => Err(DbusApiError::InvalidParameter {}.into()),
                }
            }

            DeviceClass::Mouse => {
                let device = device.as_mouse_device_mut().unwrap();

                match param {
                    "brightness" => {
                        let brightness = value.parse::<i32>()?;
                        device.set_brightness(brightness)?;

                        script::FRAME_GENERATION_COUNTER.fetch_add(1, Ordering::SeqCst);

                        Ok(())
                    }

                    "profile" => {
                        let profile = value.parse::<i32>()?;
                        device.set_profile(profile)?;

                        Ok(())
                    }

                    "dpi" => {
                        let dpi = value.parse::<i32>()?;
                        device.set_dpi(dpi)?;

                        Ok(())
                    }

                    "rate" => {
                        let rate = value.parse::<i32>()?;
                        device.set_rate(rate)?;

                        Ok(())
                    }

                    "dcu" => {
                        let dcu_config = value.parse::<i32>()?;
                        device.set_dcu_config(dcu_config)?;

                        Ok(())
                    }

                    "angle-snapping" => {
                        let angle_snapping = value.parse::<bool>()?;
                        device.set_angle_snapping(angle_snapping)?;

                        Ok(())
                    }

                    "debounce" => {
                        let debounce = value.parse::<bool>()?;
                        device.set_debounce(debounce)?;

                        Ok(())
                    }

                    _ => Err(DbusApiError::InvalidParameter {}.into()),
                }
            }

            DeviceClass::Misc => {
                let device = device.as_misc_device_mut().unwrap();

                match param {
                    "brightness" => {
                        let brightness = value.parse::<i32>()?;
                        device.set_brightness(brightness)?;

                        script::FRAME_GENERATION_COUNTER.fetch_add(1, Ordering::SeqCst);

                        Ok(())
                    }

                    _ => Err(DbusApiError::InvalidParameter {}.into()),
                }
            }

            _ => Err(DbusApiError::InvalidDeviceClass {}.into()),
        }
    } else {
        Err(DbusApiError::InvalidDevice {}.into())
    }
}

fn query_device_specific_configuration(index: u64, param: &str) -> Result<String> {
    let handle = DeviceHandle::from(index);

    if let Some(device) = crate::DEVICES.read().unwrap().get(&handle) {
        let device = device.read().unwrap();

        match device.get_device_class() {
            DeviceClass::Keyboard => {
                let device = device.as_keyboard_device().unwrap();

                match param {
                    "info" => {
                        let device_info = device.get_device_info()?;
                        let info = format!(
                            "Firmware revision: {}.{:02}",
                            device_info.firmware_version / 100,
                            device_info.firmware_version % 100
                        );

                        Ok(info)
                    }

                    "firmware" => {
                        let device_info = device.get_device_info()?;
                        let info = format!(
                            "{}.{:02}",
                            device_info.firmware_version / 100,
                            device_info.firmware_version % 100
                        );

                        Ok(info)
                    }

                    "brightness" => {
                        let brightness = device.get_brightness()?;

                        Ok(format!("{brightness}"))
                    }

                    _ => Err(DbusApiError::InvalidParameter {}.into()),
                }
            }

            DeviceClass::Mouse => {
                let device = device.as_mouse_device().unwrap();

                match param {
                    "info" => {
                        let device_info = device.get_device_info()?;
                        let info = format!(
                            "Firmware revision: {}.{:02}",
                            device_info.firmware_version / 100,
                            device_info.firmware_version % 100
                        );

                        Ok(info)
                    }

                    "firmware" => {
                        let device_info = device.get_device_info()?;
                        let info = format!(
                            "{}.{:02}",
                            device_info.firmware_version / 100,
                            device_info.firmware_version % 100
                        );

                        Ok(info)
                    }

                    "brightness" => {
                        let brightness = device.get_brightness()?;

                        Ok(format!("{brightness}"))
                    }

                    "profile" => {
                        let profile = device.get_profile()?;

                        Ok(format!("{profile}"))
                    }

                    "dpi" => {
                        let dpi = device.get_dpi()?;

                        Ok(format!("{dpi}"))
                    }

                    "rate" => {
                        let rate = device.get_rate()?;

                        Ok(format!("{rate}"))
                    }

                    "dcu" => {
                        let dcu_config = device.get_dcu_config()?;

                        Ok(format!("{dcu_config}"))
                    }

                    "angle-snapping" => {
                        let angle_snapping = device.get_angle_snapping()?;

                        Ok(format!("{angle_snapping}"))
                    }

                    "debounce" => {
                        let debounce = device.get_debounce()?;

                        Ok(format!("{debounce}"))
                    }

                    _ => Err(DbusApiError::InvalidParameter {}.into()),
                }
            }

            DeviceClass::Misc => {
                let device = device.as_misc_device().unwrap();

                match param {
                    "info" => {
                        let device_info = device.get_device_info()?;
                        let info = format!(
                            "Firmware revision: {}.{:02}",
                            device_info.firmware_version / 100,
                            device_info.firmware_version % 100
                        );

                        Ok(info)
                    }

                    "firmware" => {
                        let device_info = device.get_device_info()?;
                        let info = format!(
                            "{}.{:02}",
                            device_info.firmware_version / 100,
                            device_info.firmware_version % 100
                        );

                        Ok(info)
                    }

                    "brightness" => {
                        let brightness = device.get_brightness()?;

                        Ok(format!("{brightness}"))
                    }

                    _ => Err(DbusApiError::InvalidParameter {}.into()),
                }
            }

            _ => Err(DbusApiError::InvalidDeviceClass {}.into()),
        }
    } else {
        Err(DbusApiError::InvalidDevice {}.into())
    }
}
