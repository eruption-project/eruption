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

use std::collections::{HashMap, HashSet};
use std::ffi::OsStr;
use std::fmt::{self, Display, Formatter};
use std::u8;
use std::{any::Any, sync::Arc, thread};
use std::{path::PathBuf, time::Duration};

#[cfg(not(target_os = "windows"))]
use evdev_rs::enums::EV_KEY;
use eyre::eyre;
use hidapi::HidApi;
use lazy_static::lazy_static;
use libc::wchar_t;
use mlua::prelude::*;
use serde::{self, Deserialize};
use tracing::*;
use tracing_mutex::stdsync::RwLock;
#[cfg(not(target_os = "windows"))]
use udev::Enumerator;

use crate::constants;
use crate::hwdevices::{keyboards::*, mice::*, misc::*};

mod util;

mod keyboards;
mod mice;
mod misc;

pub type Device = Arc<RwLock<Box<dyn DeviceExt + Sync + Send>>>;
pub type MiscSerialDevice = Arc<RwLock<Box<dyn MiscDeviceExt + Sync + Send>>>;

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub struct DeviceHandle(u64);

impl DeviceHandle {
    pub fn from(index: u64) -> Self {
        Self(index)
    }
}

impl From<DeviceHandle> for u64 {
    fn from(val: DeviceHandle) -> Self {
        val.0
    }
}

impl From<DeviceHandle> for usize {
    fn from(val: DeviceHandle) -> Self {
        val.0 as usize
    }
}

impl<'lua> IntoLua<'lua> for DeviceHandle {
    fn into_lua(self, _lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        Ok(LuaValue::Integer(self.0 as i64))
    }
}

impl fmt::Display for DeviceHandle {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "[{:02}]", self.0)?;

        /* find_device_by_handle(self)
        .and_then(|device| {
            device
                .try_read()
                .and_then(|device| {
                    let device_identifier = device.get_support_script_file();

                let _ = write!(f, "[{:02}:{device_identifier}]", self.0);

                    Ok(device)
                })
                .or_else(|e| {
                    let _ = write!(f, "[{:02}:<unknown device>]", self.0);

                    Err(e)
                });

            Some(device)
            })
            .or_else(|| {
                let _ = write!(f, "[{:02}:<invalid device>]", self.0);

                None
        }); */

        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum MaturityLevel {
    /// The driver is considered to be stable
    #[serde(rename = "stable")]
    Stable,

    /// The driver may contain bugs, but is enabled by default
    #[serde(rename = "testing")]
    Testing,

    /// The driver may contain serious bugs, therefor it is disabled by default
    #[serde(rename = "experimental")]
    Experimental,
}

pub type Result<T> = std::result::Result<T, eyre::Error>;

pub struct Interface(i32);
pub struct UsagePage(i32);

#[rustfmt::skip]
lazy_static! {
    // List of supported devices
    pub static ref DRIVERS: Arc<RwLock<[Box<(dyn DriverMetadataExt + Sync + Send + 'static)>; 31]>> = Arc::new(RwLock::new([
        // Supported keyboards

        // Wooting

        // Wooting Two HE (ARM) series
        KeyboardDriver::register("Wooting", "Two HE (ARM)",  0x31e3, 0x1230,&[(Interface(0x00), UsagePage(0x00))], &keyboards::wooting_two_he_arm::bind_hiddev, MaturityLevel::Testing),

        // ROCCAT

        // Vulcan II Max
        KeyboardDriver::register("ROCCAT", "Vulcan II Max",  0x1e7d, 0x2ee2, &[(Interface(0x01), UsagePage(0x01))], &keyboards::roccat_vulcan_2_max::bind_hiddev, MaturityLevel::Experimental),

        // Vulcan 100/12x/Pro (TKL) series
        KeyboardDriver::register("ROCCAT", "Vulcan 100/12x", 0x1e7d, 0x3098,&[(Interface(0x00), UsagePage(0x00))], &keyboards::roccat_vulcan_1xx::bind_hiddev, MaturityLevel::Stable),
        KeyboardDriver::register("ROCCAT", "Vulcan 100/12x", 0x1e7d, 0x307a,&[(Interface(0x00), UsagePage(0x00))], &keyboards::roccat_vulcan_1xx::bind_hiddev, MaturityLevel::Stable),

        KeyboardDriver::register("ROCCAT", "Vulcan Pro",     0x1e7d, 0x30f7,&[(Interface(0x00), UsagePage(0x00))], &keyboards::roccat_vulcan_pro::bind_hiddev, MaturityLevel::Experimental),

        KeyboardDriver::register("ROCCAT", "Vulcan TKL",     0x1e7d, 0x2fee,&[(Interface(0x00), UsagePage(0x00))], &keyboards::roccat_vulcan_tkl::bind_hiddev, MaturityLevel::Experimental),

        KeyboardDriver::register("ROCCAT", "Vulcan Pro TKL", 0x1e7d, 0x311a, &[(Interface(0x01), UsagePage(0x01)), (Interface(0x03), UsagePage(0xff00))], &keyboards::roccat_vulcan_pro_tkl::bind_hiddev, MaturityLevel::Testing),

        KeyboardDriver::register("ROCCAT", "Magma",          0x1e7d, 0x3124,&[(Interface(0x00), UsagePage(0x00))], &keyboards::roccat_magma::bind_hiddev, MaturityLevel::Experimental),

        KeyboardDriver::register("ROCCAT", "Pyro",           0x1e7d, 0x314C,&[(Interface(0x00), UsagePage(0x00))], &keyboards::roccat_pyro::bind_hiddev, MaturityLevel::Experimental),

        // CORSAIR

        // Corsair STRAFE Gaming Keyboard
        KeyboardDriver::register("Corsair", "Corsair STRAFE Gaming Keyboard", 0x1b1c, 0x1b15,&[(Interface(0x00), UsagePage(0x00))], &keyboards::corsair_strafe::bind_hiddev, MaturityLevel::Experimental),


        // Supported mice

        // ROCCAT
        MouseDriver::register("ROCCAT", "Kone Aimo",         0x1e7d, 0x2e27,&[(Interface(0x00), UsagePage(0x00))], &mice::roccat_kone_aimo::bind_hiddev, MaturityLevel::Experimental),

        MouseDriver::register("ROCCAT", "Kone Aimo Remastered", 0x1e7d, 0x2e2c,&[(Interface(0x00), UsagePage(0x00))], &mice::roccat_kone_aimo_remastered::bind_hiddev, MaturityLevel::Experimental),

        MouseDriver::register("ROCCAT", "Kone XTD Mouse",    0x1e7d, 0x2e22,&[(Interface(0x00), UsagePage(0x00))], &mice::roccat_kone_xtd::bind_hiddev, MaturityLevel::Experimental),

        MouseDriver::register("ROCCAT", "Kone Pure Ultra",   0x1e7d, 0x2dd2, &[(Interface(0x02), UsagePage(0x01))], &mice::roccat_kone_pure_ultra::bind_hiddev, MaturityLevel::Stable),

        MouseDriver::register("ROCCAT", "Burst Pro",         0x1e7d, 0x2de1,&[(Interface(0x00), UsagePage(0x00))], &mice::roccat_burst_pro::bind_hiddev, MaturityLevel::Testing),

        MouseDriver::register("ROCCAT", "Kone XP",           0x1e7d, 0x2c8b,&[(Interface(0x00), UsagePage(0x00))], &mice::roccat_kone_xp::bind_hiddev, MaturityLevel::Experimental),

        MouseDriver::register("ROCCAT", "Kone Pro",          0x1e7d, 0x2c88,&[(Interface(0x00), UsagePage(0x00))], &mice::roccat_kone_pro::bind_hiddev, MaturityLevel::Experimental),

        MouseDriver::register("ROCCAT", "Kone Pro Air Dongle", 0x1e7d, 0x2c8e,&[(Interface(0x00), UsagePage(0x00))], &mice::roccat_kone_pro_air::bind_hiddev, MaturityLevel::Testing),
        MouseDriver::register("ROCCAT", "Kone Pro Air",        0x1e7d, 0x2c92,&[(Interface(0x00), UsagePage(0x00))], &mice::roccat_kone_pro_air::bind_hiddev, MaturityLevel::Testing),

        MouseDriver::register("ROCCAT", "Kain 100 AIMO",     0x1e7d, 0x2d00,&[(Interface(0x00), UsagePage(0x00))], &mice::roccat_kain_100::bind_hiddev, MaturityLevel::Experimental),

        MouseDriver::register("ROCCAT", "Kain 200 AIMO",     0x1e7d, 0x2d5f,&[(Interface(0x00), UsagePage(0x00))], &mice::roccat_kain_2xx::bind_hiddev, MaturityLevel::Testing),
        MouseDriver::register("ROCCAT", "Kain 200 AIMO",     0x1e7d, 0x2d60,&[(Interface(0x00), UsagePage(0x00))], &mice::roccat_kain_2xx::bind_hiddev, MaturityLevel::Testing),
        // MouseDriver::register("ROCCAT", "Kain 202 AIMO",     0x1e7d, 0x2d60, &roccat_kain_2xx::bind_hiddev, Status::Experimental),

        MouseDriver::register("ROCCAT", "Kova AIMO",         0x1e7d, 0x2cf1,&[(Interface(0x00), UsagePage(0x00))], &mice::roccat_kova_aimo::bind_hiddev, MaturityLevel::Testing),
        MouseDriver::register("ROCCAT", "Kova AIMO",         0x1e7d, 0x2cf3,&[(Interface(0x00), UsagePage(0x00))], &mice::roccat_kova_aimo::bind_hiddev, MaturityLevel::Testing),

        MouseDriver::register("ROCCAT", "Kova 2016",         0x1e7d, 0x2cee,&[(Interface(0x00), UsagePage(0x00))], &mice::roccat_kova_2016::bind_hiddev, MaturityLevel::Testing),
        MouseDriver::register("ROCCAT", "Kova 2016",         0x1e7d, 0x2cef,&[(Interface(0x00), UsagePage(0x00))], &mice::roccat_kova_2016::bind_hiddev, MaturityLevel::Testing),
        MouseDriver::register("ROCCAT", "Kova 2016",         0x1e7d, 0x2cf0,&[(Interface(0x00), UsagePage(0x00))], &mice::roccat_kova_2016::bind_hiddev, MaturityLevel::Testing),

        MouseDriver::register("ROCCAT", "Nyth",              0x1e7d, 0x2e7c,&[(Interface(0x00), UsagePage(0x00))], &mice::roccat_nyth::bind_hiddev, MaturityLevel::Experimental),
        MouseDriver::register("ROCCAT", "Nyth",              0x1e7d, 0x2e7d,&[(Interface(0x00), UsagePage(0x00))], &mice::roccat_nyth::bind_hiddev, MaturityLevel::Experimental),


        // Supported miscellaneous devices

        // ROCCAT/Turtle Beach
        MiscDriver::register("ROCCAT/Turtle Beach", "Elo 7.1 Air", 0x1e7d, 0x3a37,&[(Interface(0x00), UsagePage(0x00))], &misc::roccat_elo_71_air::bind_hiddev, MaturityLevel::Testing),

        MiscDriver::register("ROCCAT", "Aimo Pad Wide", 0x1e7d, 0x343b, &[(Interface(0x00), UsagePage(0x01))], &misc::roccat_aimo_pad::bind_hiddev, MaturityLevel::Stable),


        // Misc Serial devices

        // Eruption Custom Hardware
        // MiscSerialDriver::register("Eruption", "Custom Serial LEDs", &custom_serial_leds::bind_serial, Status::Testing),
    ]));
}

#[derive(Debug, thiserror::Error)]
pub enum HwDeviceError {
    #[error("No compatible devices found")]
    NoDevicesFound {},

    #[error("An error occurred during device enumeration")]
    EnumerationError {},

    #[error("Operation not supported")]
    OpNotSupported {},

    #[error("Could not enumerate udev devices")]
    UdevError {},

    #[error("Could not open the device file")]
    DeviceOpenError {},

    #[error("Device not bound")]
    DeviceNotBound {},

    #[error("Device not opened")]
    DeviceNotOpened {},

    #[error("Device not initialized")]
    DeviceNotInitialized {},

    #[error("Invalid status code")]
    InvalidStatusCode {},

    #[error("Invalid result")]
    InvalidResult {},

    #[error("No result")]
    NoOpResult {},

    #[error("Write error")]
    WriteError {},

    #[error("LED map has an invalid size")]
    LedMapError {},

    #[error("Could not map an evdev event code to a key or button")]
    MappingError {},
}

pub trait DriverMetadataExt {
    fn get_usb_vid(&self) -> u16;
    fn get_usb_pid(&self) -> u16;

    fn get_device_make(&self) -> &'static str;
    fn get_device_model(&self) -> &'static str;

    fn get_device_class(&self) -> DeviceClass;

    fn get_maturity_level(&self) -> MaturityLevel;

    fn bind(
        &self,
        hidapi: &HidApi,
        usb_id: (u16, u16),
        serial: &[wchar_t],
    ) -> Result<Box<dyn DeviceExt + Sync + Send>>;

    fn as_any(&self) -> &(dyn Any);
}

pub trait SerialDriverMetadataExt: DriverMetadataExt {
    fn get_serial_port(&self) -> Option<&str>;
}

pub struct KeyboardDriver<'a> {
    pub device_make: &'a str,
    pub device_name: &'a str,

    pub device_class: DeviceClass,

    pub usb_vid: u16,
    pub usb_pid: u16,

    pub endpoints: &'a [(Interface, UsagePage)],

    pub bind_fn: &'a (dyn Fn(&HidApi, u16, u16, &[wchar_t]) -> Result<Box<dyn DeviceExt + Sync + Send>>
             + Sync
             + Send),

    pub status: MaturityLevel,
}

impl KeyboardDriver<'static> {
    pub fn register(
        device_make: &'static str,
        device_name: &'static str,
        usb_vid: u16,
        usb_pid: u16,
        endpoints: &'static [(Interface, UsagePage)],
        bind_fn: &'static (dyn Fn(&HidApi, u16, u16, &[wchar_t]) -> Result<Box<dyn DeviceExt + Sync + Send>>
                      + Sync
                      + Send),
        status: MaturityLevel,
    ) -> Box<(dyn DriverMetadataExt + Sync + Send + 'static)> {
        Box::new(KeyboardDriver {
            device_make,
            device_name,
            device_class: DeviceClass::Keyboard,
            usb_vid,
            usb_pid,
            endpoints,
            bind_fn,
            status,
        })
    }
}

impl DriverMetadataExt for KeyboardDriver<'static> {
    fn get_device_class(&self) -> DeviceClass {
        self.device_class
    }

    fn get_device_make(&self) -> &'static str {
        self.device_make
    }

    fn get_device_model(&self) -> &'static str {
        self.device_name
    }

    fn as_any(&self) -> &(dyn Any) {
        self
    }

    fn get_usb_vid(&self) -> u16 {
        self.usb_vid
    }

    fn get_usb_pid(&self) -> u16 {
        self.usb_pid
    }

    fn get_maturity_level(&self) -> MaturityLevel {
        self.status
    }

    fn bind(
        &self,
        hidapi: &HidApi,
        usb_id: (u16, u16),
        serial: &[wchar_t],
    ) -> Result<Box<dyn DeviceExt + Sync + Send>> {
        (self.bind_fn)(hidapi, usb_id.0, usb_id.1, serial)
    }
}

pub struct MouseDriver<'a> {
    pub device_make: &'a str,
    pub device_name: &'a str,

    pub device_class: DeviceClass,

    pub usb_vid: u16,
    pub usb_pid: u16,

    pub endpoints: &'a [(Interface, UsagePage)],

    pub bind_fn: &'a (dyn Fn(&HidApi, u16, u16, &[wchar_t]) -> Result<Box<dyn DeviceExt + Sync + Send>>
             + Sync
             + Send),

    pub status: MaturityLevel,
}

impl MouseDriver<'static> {
    pub fn register(
        device_make: &'static str,
        device_name: &'static str,
        usb_vid: u16,
        usb_pid: u16,
        endpoints: &'static [(Interface, UsagePage)],
        bind_fn: &'static (dyn Fn(&HidApi, u16, u16, &[wchar_t]) -> Result<Box<dyn DeviceExt + Sync + Send>>
                      + Sync
                      + Send),
        status: MaturityLevel,
    ) -> Box<(dyn DriverMetadataExt + Sync + Send + 'static)> {
        Box::new(MouseDriver {
            device_make,
            device_name,
            device_class: DeviceClass::Mouse,
            usb_vid,
            usb_pid,
            endpoints,
            bind_fn,
            status,
        })
    }
}

impl DriverMetadataExt for MouseDriver<'static> {
    fn get_device_class(&self) -> DeviceClass {
        self.device_class
    }

    fn get_device_make(&self) -> &'static str {
        self.device_make
    }

    fn get_device_model(&self) -> &'static str {
        self.device_name
    }

    fn as_any(&self) -> &(dyn Any) {
        self
    }

    fn get_usb_vid(&self) -> u16 {
        self.usb_vid
    }

    fn get_usb_pid(&self) -> u16 {
        self.usb_pid
    }

    fn get_maturity_level(&self) -> MaturityLevel {
        self.status
    }

    fn bind(
        &self,
        hidapi: &HidApi,
        usb_id: (u16, u16),
        serial: &[wchar_t],
    ) -> Result<Box<dyn DeviceExt + Sync + Send>> {
        (self.bind_fn)(hidapi, usb_id.0, usb_id.1, serial)
    }
}

pub struct MiscDriver<'a> {
    pub device_make: &'a str,
    pub device_name: &'a str,

    pub device_class: DeviceClass,

    pub usb_vid: u16,
    pub usb_pid: u16,

    pub endpoints: &'a [(Interface, UsagePage)],

    pub bind_fn: &'a (dyn Fn(&HidApi, u16, u16, &[wchar_t]) -> Result<Box<dyn DeviceExt + Sync + Send>>
             + Sync
             + Send),

    pub status: MaturityLevel,
}

impl MiscDriver<'static> {
    #[allow(dead_code)]
    pub fn register(
        device_make: &'static str,
        device_name: &'static str,
        usb_vid: u16,
        usb_pid: u16,
        endpoints: &'static [(Interface, UsagePage)],
        bind_fn: &'static (dyn Fn(&HidApi, u16, u16, &[wchar_t]) -> Result<Box<dyn DeviceExt + Sync + Send>>
                      + Sync
                      + Send),
        status: MaturityLevel,
    ) -> Box<(dyn DriverMetadataExt + Sync + Send + 'static)> {
        Box::new(MiscDriver {
            device_make,
            device_name,
            device_class: DeviceClass::Misc,
            usb_vid,
            usb_pid,
            endpoints,
            bind_fn,
            status,
        })
    }
}

impl DriverMetadataExt for MiscDriver<'static> {
    fn get_device_class(&self) -> DeviceClass {
        self.device_class
    }

    fn get_device_make(&self) -> &'static str {
        self.device_make
    }

    fn get_device_model(&self) -> &'static str {
        self.device_name
    }

    fn as_any(&self) -> &(dyn Any) {
        self
    }

    fn get_usb_vid(&self) -> u16 {
        self.usb_vid
    }

    fn get_usb_pid(&self) -> u16 {
        self.usb_pid
    }

    fn get_maturity_level(&self) -> MaturityLevel {
        self.status
    }

    fn bind(
        &self,
        hidapi: &HidApi,
        usb_id: (u16, u16),
        serial: &[wchar_t],
    ) -> Result<Box<dyn DeviceExt + Sync + Send>> {
        (self.bind_fn)(hidapi, usb_id.0, usb_id.1, serial)
    }
}

pub struct MiscSerialDriver<'a> {
    pub device_make: &'a str,
    pub device_name: &'a str,

    pub device_class: DeviceClass,

    pub serial_port: Option<&'a str>,

    pub bind_fn: &'a (dyn Fn(&str) -> Result<MiscSerialDevice> + Sync + Send),

    pub status: MaturityLevel,
}

impl MiscSerialDriver<'static> {
    #[allow(dead_code)]
    pub fn register(
        device_make: &'static str,
        device_name: &'static str,
        bind_fn: &'static (dyn Fn(&str) -> Result<MiscSerialDevice> + Sync + Send),
        status: MaturityLevel,
    ) -> Box<(dyn DriverMetadataExt + Sync + Send + 'static)> {
        Box::new(MiscSerialDriver {
            device_make,
            device_name,
            device_class: DeviceClass::Misc,
            serial_port: None,
            bind_fn,
            status,
        })
    }
}

impl DriverMetadataExt for MiscSerialDriver<'static> {
    fn get_usb_vid(&self) -> u16 {
        0
    }

    fn get_usb_pid(&self) -> u16 {
        0
    }

    fn get_device_make(&self) -> &'static str {
        self.device_make
    }

    fn get_device_model(&self) -> &'static str {
        self.device_name
    }

    fn get_device_class(&self) -> DeviceClass {
        self.device_class
    }

    fn as_any(&self) -> &(dyn Any) {
        self
    }

    fn get_maturity_level(&self) -> MaturityLevel {
        self.status
    }

    fn bind(
        &self,
        _hidapi: &HidApi,
        _usb_id: (u16, u16),
        _serial: &[wchar_t],
    ) -> Result<Box<dyn DeviceExt + Sync + Send>> {
        Err(HwDeviceError::OpNotSupported {}.into())
    }
}

impl SerialDriverMetadataExt for MiscSerialDriver<'static> {
    fn get_serial_port(&self) -> Option<&str> {
        self.serial_port
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceClass {
    Unknown,
    Keyboard,
    Mouse,
    Misc,
}

/// Represents an RGBA color value
#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl LuaUserData for RGBA {}

impl dbus::arg::Arg for RGBA {
    const ARG_TYPE: dbus::arg::ArgType = dbus::arg::ArgType::Struct;

    fn signature() -> dbus::Signature<'static> {
        dbus::Signature::from("(yyyy)")
    }
}

impl dbus::arg::Append for RGBA {
    fn append_by_ref(&self, i: &mut dbus::arg::IterAppend) {
        i.append((self.r, self.g, self.b, self.a));
    }
}

/// A Keyboard HID event
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KeyboardHidEvent {
    Unknown,

    // Keyboard events
    KeyDown { code: KeyboardHidEventCode },
    KeyUp { code: KeyboardHidEventCode },

    // Slot switching
    NextSlot,
    PreviousSlot,

    // Brightness related
    BrightnessUp,
    BrightnessDown,
    SetBrightness(u8),

    // Audio related
    MuteDown,
    MuteUp,
    VolumeDown,
    VolumeUp,
}

/// Event code of a device HID message
#[allow(non_camel_case_types)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum KeyboardHidEventCode {
    Unknown(u8),

    KEY_F1,
    KEY_F2,
    KEY_F3,
    KEY_F4,

    KEY_F5,
    KEY_F6,
    KEY_F7,
    KEY_F8,

    KEY_ESC,
    KEY_CAPS_LOCK,
    KEY_FN,
    KEY_EASY_SHIFT,
}

/// A Mouse HID event
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum MouseHidEvent {
    Unknown,

    ButtonDown(u8),
    ButtonUp(u8),

    // Button events
    DpiChange(u8),
}

/// Status LEDs
#[allow(dead_code)]
pub enum LedKind {
    Unknown,
    AudioMute,
    Fx,
    Volume,
    NumLock,
    CapsLock,
    ScrollLock,
    GameMode,
}

// impl LedKind {
//     /// Instantiate a LedKind using an integer constant
//     pub fn from_id(id: u8) -> Result<Self> {
//         match id {
//             0 => Ok(Self::Unknown),
//             1 => Ok(Self::AudioMute),
//             2 => Ok(Self::Fx),
//             3 => Ok(Self::Volume),
//             4 => Ok(Self::NumLock),
//             5 => Ok(Self::CapsLock),
//             6 => Ok(Self::ScrollLock),
//             7 => Ok(Self::GameMode),

//             _ => Err(HwDeviceError::ValueError {
//                 description: "Invalid LED identifier".to_owned(),
//             }
//             .into()),
//         }
//     }
// }

impl From<LedKind> for u8 {
    /// Convert a LedKind to an integer constant
    fn from(val: LedKind) -> Self {
        match val {
            LedKind::Unknown => 0,
            LedKind::AudioMute => 1,
            LedKind::Fx => 2,
            LedKind::Volume => 3,
            LedKind::NumLock => 4,
            LedKind::CapsLock => 5,
            LedKind::ScrollLock => 6,
            LedKind::GameMode => 7,
        }
    }
}

/// Generic Device info
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub firmware_version: i32,
}

impl DeviceInfo {
    pub fn new(firmware_version: i32) -> Self {
        DeviceInfo { firmware_version }
    }
}

/// Generic Device status information, like e.g.: 'signal strength' or 'battery level'
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct DeviceStatus(pub HashMap<String, String>);

impl std::ops::Deref for DeviceStatus {
    type Target = HashMap<String, String>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Default for DeviceStatus {
    fn default() -> Self {
        let map = HashMap::new();

        // fill in default values
        // map.insert("connected".to_owned(), format!("{}", true));

        Self(map)
    }
}

/// Non 'Plug and Play' device, may be declared in .config file
#[derive(Debug, Clone)]
pub struct NonPnPDevice {
    pub class: String,
    pub name: String,
    pub device_file: PathBuf,
}

/// Represents the capabilities of a hardware device
#[derive(Debug, Clone)]
pub struct DeviceCapabilities(HashSet<Capability>);

impl<const N: usize> From<[Capability; N]> for DeviceCapabilities {
    fn from(caps: [Capability; N]) -> Self {
        DeviceCapabilities(HashSet::from(caps))
    }
}

/// Capabilities that hardware may have
#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum Capability {
    // Categorization
    Keyboard,
    Mouse,
    Misc,
    Headset,
    MousePad,

    // Features
    RgbLighting,
    HardwareProfiles,
    PowerManagement,

    DpiSelection,
    Debounce,
    DebounceTimeSelection,
    AngleSnapping,
}

/// Information about a generic device
pub trait DeviceInfoExt {
    /// Get device capabilities
    fn get_device_capabilities(&self) -> DeviceCapabilities;

    /// Get device specific information
    fn get_device_info(&self) -> Result<DeviceInfo>;

    /// Get device firmware revision suitable for display to the user
    fn get_firmware_revision(&self) -> String;
}

#[allow(unused)]
#[derive(Debug, Clone)]
pub enum DeviceConfig {
    NoOp,
}

#[allow(unused)]
pub trait GenericConfigurationExt {
    /// Get device specific configuration
    fn get_device_config(&self, param: &DeviceConfig) -> Result<Box<dyn Any>>;

    /// Set device specific configuration
    fn set_device_config(&self, param: &DeviceConfig, value: &dyn Any) -> Result<()>;
}

/// Represents a rectangular zone on the canvas that is allocated to a device
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Zone {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub enabled: bool,
}

impl mlua::UserData for Zone {
    fn add_fields<'lua, F: mlua::UserDataFields<'lua, Self>>(fields: &mut F) {
        fields.add_field_method_get("x", |_, this| Ok(this.x));
        fields.add_field_method_get("y", |_, this| Ok(this.y));
        fields.add_field_method_get("x2", |_, this| Ok(this.x2()));
        fields.add_field_method_get("y2", |_, this| Ok(this.y2()));

        fields.add_field_method_get("width", |_, this| Ok(this.width));
        fields.add_field_method_get("height", |_, this| Ok(this.height));

        fields.add_field_method_get("enabled", |_, this| Ok(this.enabled));
    }
}

impl dbus::arg::Arg for Zone {
    const ARG_TYPE: dbus::arg::ArgType = dbus::arg::ArgType::Struct;

    fn signature() -> dbus::Signature<'static> {
        dbus::Signature::from("(iiiib)")
    }
}

impl dbus::arg::Append for Zone {
    fn append_by_ref(&self, i: &mut dbus::arg::IterAppend) {
        i.append((self.x, self.y, self.width, self.height, self.enabled));
    }
}

#[allow(unused)]
impl Zone {
    #[inline]
    pub fn new(x: i32, y: i32, width: i32, height: i32, enabled: bool) -> Self {
        Self {
            x,
            y,
            width,
            height,
            enabled,
        }
    }

    pub fn defaults_for(device_class: DeviceClass) -> Self {
        const SCALE_FACTOR: i32 = 1;

        match device_class {
            DeviceClass::Keyboard => Self {
                x: 10,
                y: constants::CANVAS_HEIGHT as i32 / 2,
                width: constants::CANVAS_WIDTH as i32 / 2,
                height: constants::CANVAS_HEIGHT as i32 / 2,
                enabled: true,
            },

            DeviceClass::Mouse => Self {
                x: constants::CANVAS_WIDTH as i32 - 6 * SCALE_FACTOR,
                y: constants::CANVAS_HEIGHT as i32 / 2 - 2 * SCALE_FACTOR,
                width: 5 * SCALE_FACTOR,
                height: 5 * SCALE_FACTOR,
                enabled: true,
            },

            DeviceClass::Misc => Self {
                x: constants::CANVAS_WIDTH as i32 / 2 - 4 * SCALE_FACTOR,
                y: constants::CANVAS_HEIGHT as i32 / 2 - 10 * SCALE_FACTOR,
                width: 8 * SCALE_FACTOR,
                height: SCALE_FACTOR,
                enabled: true,
            },

            DeviceClass::Unknown => Self::empty(),
        }
    }

    #[inline]
    pub fn empty() -> Self {
        Self {
            x: 0,
            y: 0,
            width: 0,
            height: 0,
            enabled: false,
        }
    }

    #[inline]
    pub fn cell_count(&self) -> usize {
        (self.width * self.height).unsigned_abs() as usize
    }

    #[inline]
    pub fn x2(&self) -> i32 {
        self.x + self.width
    }

    #[inline]
    pub fn y2(&self) -> i32 {
        self.y + self.height
    }
}

impl Default for Zone {
    fn default() -> Self {
        Self::empty()
    }
}

impl Display for Zone {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}x{}:{}x{}", self.x, self.y, self.width, self.height)
    }
}

/// Zone allocation on the unified canvas for a supported device
pub trait DeviceZoneAllocationExt {
    /// Returns the size of the zone (number of LEDs) that the device is able to display
    fn get_zone_size_hint(&self) -> usize;

    /// Returns the allocated rectangular area on the canvas
    fn get_allocated_zone(&self) -> Zone;

    /// Sets the rectangular area on the canvas that is allocated to be displayed on the device
    fn set_zone_allocation(&mut self, zone: Zone);
}

/// Generic device trait
pub trait DeviceExt: DeviceInfoExt + DeviceZoneAllocationExt {
    /// Returns the path(s) of the bound (sub-) device(s)
    fn get_dev_paths(&self) -> Vec<String>;

    /// Returns the USB vendor ID of the device
    fn get_usb_vid(&self) -> u16;

    /// Returns the USB product ID of the device
    fn get_usb_pid(&self) -> u16;

    /// Returns a device specific serial number/identifier
    fn get_serial(&self) -> Option<&str>;

    /// Returns the file name of the Lua support script for the device
    fn get_support_script_file(&self) -> String;

    /// Opens the sub-devices, should be called after `bind()`ing a driver
    fn open(&mut self, api: &hidapi::HidApi) -> Result<()>;

    /// Close the device files
    fn close_all(&mut self) -> Result<()>;

    /// Send a device specific initialization sequence to set the device
    /// to a known good state. Should be called after `open()`ing the device
    fn send_init_sequence(&mut self) -> Result<()>;

    /// Send a device specific shutdown sequence to set the device
    /// to a known good state. Should be called before `close_all()`
    fn send_shutdown_sequence(&mut self) -> Result<()>;

    /// Set the device specific brightness
    fn set_brightness(&mut self, brightness: i32) -> Result<()>;

    /// Get the device specific brightness
    fn get_brightness(&self) -> Result<i32>;

    /// Send RGBA LED map to the device
    fn send_led_map(&mut self, led_map: &[RGBA]) -> Result<()>;

    /// Send the LED init pattern to the device. This should be used to initialize
    /// all LEDs and set them to a known good state
    fn set_led_init_pattern(&mut self) -> Result<()>;

    /// Send a LED finalization pattern to the device. This should normally be used,
    /// to set the device to a known good state, on exit of the daemon
    fn set_led_off_pattern(&mut self) -> Result<()>;

    /// Returns `true` if the device has been initialized
    fn is_initialized(&self) -> Result<bool>;

    /// Returns `true` if the device has failed or has been disconnected
    fn has_failed(&self) -> Result<bool>;

    /// Mark the device as `failed`
    fn fail(&mut self) -> Result<()>;

    /// Send raw data to the control device
    fn write_data_raw(&self, buf: &[u8]) -> Result<()>;

    /// Read raw data from the control device
    fn read_data_raw(&self, size: usize) -> Result<Vec<u8>>;

    /// Get the device status
    fn device_status(&self) -> Result<DeviceStatus>;

    fn get_evdev_input_rx(&self) -> &Option<flume::Receiver<Option<evdev_rs::InputEvent>>>;
    fn set_evdev_input_rx(&mut self, rx: Option<flume::Receiver<Option<evdev_rs::InputEvent>>>);

    fn get_device_class(&self) -> DeviceClass;

    fn as_device(&self) -> &(dyn DeviceExt + Send + Sync);
    fn as_device_mut(&mut self) -> &mut (dyn DeviceExt + Send + Sync);

    fn as_keyboard_device(&self) -> Option<&(dyn KeyboardDeviceExt + Send + Sync)>;
    fn as_keyboard_device_mut(&mut self) -> Option<&mut (dyn KeyboardDeviceExt + Send + Sync)>;

    fn as_mouse_device(&self) -> Option<&(dyn MouseDeviceExt + Send + Sync)>;
    fn as_mouse_device_mut(&mut self) -> Option<&mut (dyn MouseDeviceExt + Send + Sync)>;

    fn as_misc_device(&self) -> Option<&(dyn MiscDeviceExt + Send + Sync)>;
    fn as_misc_device_mut(&mut self) -> Option<&mut (dyn MiscDeviceExt + Send + Sync)>;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

// Generic device trait
pub trait SerialDeviceExt: DeviceInfoExt {
    /// Returns the file name of the Lua support script for the device
    fn get_support_script_file(&self) -> String;

    /// Opens the sub-devices, should be called after `bind()`ing a driver
    fn open(&mut self, api: &hidapi::HidApi) -> Result<()>;

    /// Close the device files
    fn close_all(&mut self) -> Result<()>;

    /// Send a device specific initialization sequence to set the device
    /// to a known good state. Should be called after `open()`ing the device
    fn send_init_sequence(&mut self) -> Result<()>;

    /// Send raw data to the control device
    fn write_data_raw(&self, buf: &[u8]) -> Result<()>;

    /// Read raw data from the control device
    fn read_data_raw(&self, size: usize) -> Result<Vec<u8>>;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Devices like e.g. a supported keyboard
pub trait KeyboardDeviceExt: DeviceExt {
    /// Set the state of a device status LED, like e.g. Num Lock, etc...
    fn set_status_led(&self, led_kind: LedKind, on: bool) -> Result<()>;

    /// Get the next HID event from the control device (blocking)
    fn get_next_event(&self) -> Result<KeyboardHidEvent>;

    /// Get the next HID event from the control device (with timeout)
    fn get_next_event_timeout(&self, millis: i32) -> Result<KeyboardHidEvent>;

    /// Convert an EV_KEY to an index value
    #[cfg(not(target_os = "windows"))]
    fn ev_key_to_key_index(&self, key: EV_KEY) -> u8;

    /// Convert a HID event code to a key index
    fn hid_event_code_to_key_index(&self, code: &KeyboardHidEventCode) -> u8;

    /// Convert a HID event code back to a report code
    fn hid_event_code_to_report(&self, code: &KeyboardHidEventCode) -> u8;

    /// Returns the number of keys
    fn get_num_keys(&self) -> usize;

    /// Returns the number of rows (vertical number of keys)
    fn get_num_rows(&self) -> usize;

    /// Returns the number of columns (horizontal number of keys)
    fn get_num_cols(&self) -> usize;

    /// Returns the indices of the keys in row `row`
    fn get_row_topology(&self, row: usize) -> &'static [u8];

    /// Returns the indices of the keys in column `col`
    fn get_col_topology(&self, col: usize) -> &'static [u8];
}

/// Device like e.g. a supported mouse
pub trait MouseDeviceExt: DeviceExt {
    fn get_profile(&self) -> Result<i32>;

    fn set_profile(&mut self, profile: i32) -> Result<()>;

    fn get_dpi(&self) -> Result<i32>;

    fn set_dpi(&mut self, dpi: i32) -> Result<()>;

    fn get_rate(&self) -> Result<i32>;

    fn set_rate(&mut self, rate: i32) -> Result<()>;

    fn get_dcu_config(&self) -> Result<i32>;

    fn set_dcu_config(&mut self, dcu: i32) -> Result<()>;

    fn get_angle_snapping(&self) -> Result<bool>;

    fn set_angle_snapping(&mut self, angle_snapping: bool) -> Result<()>;

    fn get_debounce(&self) -> Result<bool>;

    fn set_debounce(&mut self, debounce: bool) -> Result<()>;

    /// Get the next HID event from the control device (blocking)
    fn get_next_event(&self) -> Result<MouseHidEvent>;

    /// Get the next HID event from the control device (with timeout)
    fn get_next_event_timeout(&self, millis: i32) -> Result<MouseHidEvent>;

    /// Converts an EV_KEY value to a button index
    #[cfg(not(target_os = "windows"))]
    fn ev_key_to_button_index(&self, code: EV_KEY) -> Result<u8>;

    /// Converts a button index to an EV_KEY value
    #[cfg(not(target_os = "windows"))]
    fn button_index_to_ev_key(&self, index: u32) -> Result<EV_KEY>;
}

/// Misc Devices
pub trait MiscDeviceExt: DeviceExt {
    /// Returns true when the device supports an input sub-device like e.g. a dial or volume wheel on a headset
    fn has_input_device(&self) -> bool;
}

/// Misc Serial Devices
pub trait MiscSerialDeviceExt: SerialDeviceExt {
    /// Send RGBA LED map to the device
    fn send_led_map(&mut self, led_map: &[RGBA]) -> Result<()>;

    /// Send the LED init pattern to the device. This should be used to initialize
    /// all LEDs and set them to a known good state
    fn set_led_init_pattern(&mut self) -> Result<()>;

    /// Send a LED finalization pattern to the device. This should normally be used,
    /// to set the device to a known good state, on exit of the daemon
    fn set_led_off_pattern(&mut self) -> Result<()>;
}

/// Returns true if the USB device is blacklisted in the global configuration
pub fn is_device_blacklisted(vid: u16, pid: u16) -> Result<bool> {
    let config = crate::CONFIG.read();
    let config = config.as_ref().unwrap();

    if let Some(config) = config.as_ref() {
        let devices = config.get_array("devices").unwrap_or_else(|_e| vec![]);

        for entry in devices.iter() {
            let table = entry.clone().into_table()?;

            if table["entry_type"].clone().into_string()? == "blacklist" {
                let usb_vid = table["vendor_id"].clone().into_int()?;
                let usb_pid = table["product_id"].clone().into_int()?;

                if usb_vid == vid as i64 && usb_pid == pid as i64 {
                    // specified vid/pid is blacklisted
                    return Ok(true);
                }
            } else if table["entry_type"].clone().into_string()? == "device" {
                /* skip device declarations */
            } else {
                error!("Invalid 'entry_type' specified in the configuration file");
            }
        }

        // specified vid/pid not in blacklisted entries
        Ok(false)
    } else {
        // no config available
        Ok(false)
    }
}

/// Returns a Vec of non plug and play devices declared in eruption.conf
pub fn get_non_pnp_devices() -> Result<Vec<NonPnPDevice>> {
    let mut result = vec![];

    let config = crate::CONFIG.read().unwrap();

    if let Some(config) = config.as_ref() {
        let devices = config.get_array("devices").unwrap_or_else(|_e| vec![]);

        for entry in devices.iter() {
            let table = entry.clone().into_table()?;

            if table["entry_type"].clone().into_string()? == "device" {
                let class = table["device_class"].clone().into_string()?;
                let name = table["device_name"].clone().into_string()?;
                let device_file = PathBuf::from(&table["device_file"].clone().into_string()?);

                let device = NonPnPDevice {
                    class,
                    name,
                    device_file,
                };

                result.push(device);
            } else if table["entry_type"].clone().into_string()? == "blacklist" {
                /* skip blacklist entries */
            } else {
                error!("Invalid 'entry_type' specified in the configuration file");
            }
        }

        Ok(result)
    } else {
        // no config available, result will be empty
        Ok(result)
    }
}

/// Enumerates all HID devices on the system (and static device declarations
/// from the .conf file as well). Returns a [Vec] of detected devices,
/// (devices are bound but not initialized)
pub fn probe_devices() -> Result<Vec<Device>> {
    let mut devices = vec![];

    // bind all declared non-pnp devices from configuration file
    let declared_devices = get_non_pnp_devices()?;

    for device in declared_devices {
        if device.class == "serial" {
            info!(
                "Binding non-pnp serial LEDs device: {} ({})",
                device.name,
                device.device_file.display()
            );

            {
                let mut pending_devices = crate::DEVICES_PENDING_INIT.0.lock().unwrap();
                *pending_devices += 1;

                crate::DEVICES_PENDING_INIT.1.notify_all();
            }

            let serial_leds = custom_serial_leds::CustomSerialLeds::bind(device.device_file);

            // non pnp devices are currently always 'misc' devices
            devices.push(Arc::new(RwLock::new(
                Box::new(serial_leds) as Box<dyn DeviceExt + Sync + Send>
            )));
        } else {
            error!("Unknown device class specified in the configuration file");
        }
    }

    let mut bound_devices = Vec::new();

    // for (_handle, device) in crate::DEVICES.read().unwrap().iter() {
    //     bound_devices.extend(device.read().unwrap().get_dev_paths());
    // }

    let mut hidapi = crate::HIDAPI.write().unwrap();
    let hidapi = hidapi.as_mut().unwrap();

    hidapi.refresh_devices()?;

    for device_info in hidapi.device_list() {
        if !is_device_blacklisted(device_info.vendor_id(), device_info.product_id())? {
            if let Some(driver) = DRIVERS.read().unwrap().iter().find(|&driver| {
                driver.get_usb_vid() == device_info.vendor_id()
                    && driver.get_usb_pid() == device_info.product_id()
            }) {
                // info!(
                //     "Found supported device: 0x{:x}:0x{:x} iface: {}:{:x} - {} {}",
                //     device_info.vendor_id(),
                //     device_info.product_id(),
                //     device_info.interface_number(),
                //     device_info.usage_page(),
                //     device_info
                //         .manufacturer_string()
                //         .unwrap_or("<unknown>")
                //         .to_string(),
                //     device_info
                //         .product_string()
                //         .unwrap_or("<unknown>")
                //         .to_string(),
                // );

                let serial = device_info.serial_number_raw().unwrap_or(&[]);
                let path = device_info.path().to_string_lossy().into_owned();

                let driver_maturity_level = *crate::DRIVER_MATURITY_LEVEL.read().unwrap();

                if driver.get_maturity_level() <= driver_maturity_level {
                    match driver.get_device_class() {
                        DeviceClass::Keyboard | DeviceClass::Mouse | DeviceClass::Misc => {
                            if let Ok(device) = driver.bind(
                                hidapi,
                                (device_info.vendor_id(), device_info.product_id()),
                                serial,
                            ) {
                                if !bound_devices
                                    .iter()
                                    .any(|d| device.get_dev_paths().contains(d))
                                {
                                    info!(
                                        "Found supported device: 0x{:x}:0x{:x} iface: {}:{:x} - {} {}",
                                        device_info.vendor_id(),
                                        device_info.product_id(),
                                        device_info.interface_number(),
                                        device_info.usage_page(),
                                        device_info
                                            .manufacturer_string()
                                            .unwrap_or("<unknown>")
                                            .to_string(),
                                        device_info
                                            .product_string()
                                            .unwrap_or("<unknown>")
                                            .to_string(),
                                    );

                                    {
                                        let mut pending_devices =
                                            crate::DEVICES_PENDING_INIT.0.lock().unwrap();
                                        *pending_devices += 1;

                                        crate::DEVICES_PENDING_INIT.1.notify_all();
                                    }

                                    bound_devices.extend(device.get_dev_paths());
                                    devices.push(Arc::new(RwLock::new(device)));
                                } else {
                                    trace!(
                                        "Skipping this endpoint since the device '{path}' is already bound by us"
                                    );
                                }
                            } else {
                                error!("Failed to bind the device driver");
                            }
                        }

                        _ => {
                            error!("Failed to bind the device driver, unsupported device class");
                        }
                    }
                } else {
                    warn!("Not binding the device driver because it would require a lesser code maturity level");
                    warn!("To enable this device driver, please change the 'driver_maturity_level' setting in eruption.conf");
                }
            } else {
                // found an unsupported device
                debug!(
                    "Found unsupported device: 0x{:x}:0x{:x} iface: {}:{:x} - {} {}",
                    device_info.vendor_id(),
                    device_info.product_id(),
                    device_info.interface_number(),
                    device_info.usage_page(),
                    device_info
                        .manufacturer_string()
                        .unwrap_or("<unknown>")
                        .to_string(),
                    device_info
                        .product_string()
                        .unwrap_or("<unknown>")
                        .to_string(),
                );

                let serial = device_info.serial_number_raw().unwrap_or(&[]);
                // let path = device_info.path().to_string_lossy().to_string();

                if !bound_devices.contains(&device_info.path().to_string_lossy().into_owned()) {
                    match get_usb_device_class(device_info.vendor_id(), device_info.product_id()) {
                        Ok(DeviceClass::Keyboard) => {
                            if let Ok(device) = generic_keyboard::bind_hiddev(
                                hidapi,
                                device_info.vendor_id(),
                                device_info.product_id(),
                                serial,
                            ) {
                                bound_devices
                                    .push(device_info.path().to_string_lossy().into_owned());
                                devices.push(Arc::new(RwLock::new(device)));
                            } else {
                                error!("Failed to bind the device driver");
                            }
                        }

                        Ok(DeviceClass::Mouse) => {
                            if let Ok(device) = generic_mouse::bind_hiddev(
                                hidapi,
                                device_info.vendor_id(),
                                device_info.product_id(),
                                serial,
                            ) {
                                bound_devices
                                    .push(device_info.path().to_string_lossy().into_owned());
                                devices.push(Arc::new(RwLock::new(device)));
                            } else {
                                error!("Failed to bind the device driver");
                            }
                        }

                        Ok(DeviceClass::Misc) => {
                            /* if let Ok(device) = generic_misc::bind_hiddev(
                                hidapi,
                                device_info.vendor_id(),
                                device_info.product_id(),
                                serial,
                            ) {
                                bound_devices
                                    .push(device_info.path().to_string_lossy().into_owned());
                                devices.push(Arc::new(RwLock::new(device)));
                            } else {
                                error!("Failed to bind the device driver");
                            } */
                        }

                        Err(e) => {
                            error!("Failed to query device class: {}", e);
                        }

                        _ => {
                            warn!("Unknown device class");
                        }
                    }
                }
            }
        } else {
            info!(
                "Skipping blacklisted device: 0x{:x}:0x{:x} iface {}:{:x} - {} {}",
                device_info.vendor_id(),
                device_info.product_id(),
                device_info.interface_number(),
                device_info.usage_page(),
                device_info
                    .manufacturer_string()
                    .unwrap_or("<unknown>")
                    .to_string(),
                device_info
                    .product_string()
                    .unwrap_or("<unknown>")
                    .to_string(),
            );
        }
    }

    Ok(devices)
}

/// Get the path of the USB device from udev
#[cfg(not(target_os = "windows"))]
pub fn get_input_dev_from_udev(usb_vid: u16, usb_pid: u16) -> Result<String> {
    // retry up to n times, in case device enumeration fails
    let mut retry_counter = 3;

    loop {
        match Enumerator::new() {
            Ok(mut enumerator) => {
                enumerator.match_is_initialized().unwrap();

                enumerator.match_subsystem("input").unwrap();
                enumerator.match_property("ID_INPUT_KEYBOARD", "1").unwrap();
                enumerator.match_property("ID_INPUT_MOUSE", "1").unwrap();

                // statically blacklist the following unsupported devices
                let static_blacklist: Vec<String> = vec![/* String::from("Generic X-Box pad") */];

                match enumerator.scan_devices() {
                    Ok(devices) => {
                        for device in devices {
                            if let Some(devname) = device
                                .properties()
                                .find(|e| e.name() == "NAME")
                                .map(|v| v.value().to_string_lossy().into_owned())
                            {
                                if static_blacklist
                                    .iter()
                                    .any(|e| e.eq_ignore_ascii_case(&devname))
                                {
                                    warn!("Skipping statically blacklisted device: {}", devname);
                                    continue;
                                }
                            }

                            let found_dev = device.properties().any(|e| {
                                e.name() == "ID_VENDOR_ID"
                                    && ([usb_vid]
                                        .iter()
                                        .map(|v| format!("{v:04x}"))
                                        .any(|v| v == e.value().to_string_lossy()))
                            }) && device.properties().any(|e| {
                                e.name() == "ID_MODEL_ID"
                                    && ([usb_pid]
                                        .iter()
                                        .map(|v| format!("{v:04x}"))
                                        .any(|v| v == e.value().to_string_lossy()))
                            }) /* && device.devnode().is_some() */;

                            if found_dev {
                                if let Some(devnode) = device.devnode() {
                                    debug!(
                                        "Picking evdev device: {}",
                                        devnode.to_str().unwrap().to_string()
                                    );

                                    return Ok(devnode.to_str().unwrap().to_string());
                                } else if let Some(devname) =
                                    device.properties().find(|e| e.name() == "DEVNAME")
                                {
                                    debug!(
                                        "Picking evdev device: {}",
                                        devname.value().to_str().unwrap().to_string()
                                    );

                                    return Ok(devname.value().to_str().unwrap().to_string());
                                } else {
                                    // give up the search
                                    trace!("Could not query device node path");
                                }
                            }
                        }

                        if retry_counter <= 0 {
                            // give up the search
                            error!("The requested device could not be found");

                            break Err(HwDeviceError::NoDevicesFound {}.into());
                        } else {
                            // wait for the device to be available
                            retry_counter -= 1;
                            thread::sleep(Duration::from_millis(constants::DEVICE_SETTLE_DELAY));
                        }
                    }

                    Err(_e) => {
                        if retry_counter <= 0 {
                            // give up the search
                            break Err(HwDeviceError::EnumerationError {}.into());
                        } else {
                            // wait for the enumerator to be available
                            retry_counter -= 1;
                            thread::sleep(Duration::from_millis(constants::DEVICE_SETTLE_DELAY));
                        }
                    }
                }
            }

            Err(_e) => {
                if retry_counter <= 0 {
                    // give up the search
                    break Err(HwDeviceError::UdevError {}.into());
                } else {
                    // wait for the enumerator to be available
                    retry_counter -= 1;
                    thread::sleep(Duration::from_millis(constants::DEVICE_SETTLE_DELAY));
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
pub fn get_usb_device_class(usb_vid: u16, usb_pid: u16) -> Result<DeviceClass> {
    // use usb_enumeration::{Event, Observer};

    // let sub = Observer::new()
    //     .with_poll_interval(2)
    //     .with_vendor_id(usb_vid)
    //     .with_product_id(usb_pid)
    //     .subscribe();

    // for event in sub.rx_event.iter() {
    //     match event {
    //         Event::Initial(d) => println!("Initial devices: {:?}", d),
    //         Event::Connect(d) => println!("Connected device: {:?}", d),
    //         Event::Disconnect(d) => println!("Disconnected device: {:?}", d),
    //     }
    // }

    // if usb_pid == 0x343b {
    //     Ok(DeviceClass::Misc)
    // } else {
    //     Ok(DeviceClass::Unknown)
    // }

    Ok(DeviceClass::Unknown)
}

/// Queries udev for the device class of an USB input device
#[cfg(not(target_os = "windows"))]
pub fn get_usb_device_class(usb_vid: u16, usb_pid: u16) -> Result<DeviceClass> {
    match Enumerator::new() {
        Ok(mut enumerator) => {
            // enumerator.match_subsystem("input").unwrap();

            match enumerator.scan_devices() {
                Ok(devices) => {
                    for device in devices {
                        let found_dev = device.properties().any(|e| {
                            e.name() == "ID_VENDOR_ID"
                                && ([usb_vid]
                                    .iter()
                                    .map(|v| format!("{v:04x}"))
                                    .any(|v| v == e.value().to_string_lossy()))
                        }) && device.properties().any(|e| {
                            e.name() == "ID_MODEL_ID"
                                && ([usb_pid]
                                    .iter()
                                    .map(|v| format!("{v:04x}"))
                                    .any(|v| v == e.value().to_string_lossy()))
                        });

                        if found_dev {
                            let is_keyboard =
                                device.properties().any(|e| e.name() == "ID_INPUT_KEYBOARD");

                            let is_mouse =
                                device.properties().any(|e| e.name() == "ID_INPUT_MOUSE");

                            if is_keyboard {
                                return Ok(DeviceClass::Keyboard);
                            } else if is_mouse {
                                return Ok(DeviceClass::Mouse);
                            } else {
                                return Ok(DeviceClass::Unknown);
                            }
                        }
                    }

                    Err(HwDeviceError::NoDevicesFound {}.into())
                }

                Err(_e) => Err(HwDeviceError::EnumerationError {}.into()),
            }
        }

        Err(_e) => Err(HwDeviceError::UdevError {}.into()),
    }
}

/// For some devices, such as the Vulcan 1xx, after sending the report to update the LEDs, the device's evdev LED interface
/// goes crazy and starts spewing out KEY_UNKNOWN events.  This is ignored by X and Wayland, but is interpreted as real key
/// stroke inputs on virtual consoles.  As best as I can tell, this behavior is a bug somewhere in udev/evdev/hidraw.  As a
/// workaround, toggling the "inhibited" attribute back and forth as a privileged user silences these events for as long as
/// the device is plugged in.  Not all Roccat devices require this workaround, headphones don't, but I don't know which all
/// do and which don't.  Note that this workaround can also be applied manually by writing to the "inhibited" file found at
/// path "/sys/class/input/eventX/inhibited", where the X in "eventX" is the udev number associated with the LED interface.
#[cfg(not(target_os = "windows"))]
pub fn udev_inhibited_workaround(
    vendor_id: u16,
    product_id: u16,
    interface_num: i32,
) -> Result<()> {
    let mut enumerator = udev::Enumerator::new()?;

    // the following filters seem to not work reliably, using udev crate version 0.7

    // enumerator.match_subsystem("input")?;
    // enumerator.match_property("ID_VENDOR_ID", format!("{vendor_id:04x}"))?;
    // enumerator.match_property("ID_MODEL_ID", format!("{product_id:04x}"))?;
    // enumerator.match_property("ID_USB_INTERFACE_NUM", &interface_num_str)?;
    // enumerator.match_attribute("inhibited", "0")?;

    // ... we have to check them manually in find(..) for now

    enumerator
        .scan_devices()?
        .find(|dev| {
            dev.property_value("ID_VENDOR_ID").map_or(false, |value| {
                value == OsStr::new(&format!("{vendor_id:04x}"))
            }) && dev.property_value("ID_MODEL_ID").map_or(false, |value| {
                value == OsStr::new(&format!("{product_id:04x}"))
            }) && dev
                .property_value("ID_USB_INTERFACE_NUM")
                .map_or(false, |value| {
                    value == OsStr::new(&format!("{interface_num:02}"))
                })
                && dev
                    .attribute_value("inhibited")
                    .map_or(false, |value| value == OsStr::new("0"))
        })
        .map_or_else(
            || Err(eyre!("Udev device not found.")),
            |mut dev| {
                info!("Trying to apply Udev 'inhibited' workaround for device {vendor_id:04x}:{product_id:04x} iface: {interface_num}");

                // Toggling the value on and off is enough to quiet spurious events.
                dev.set_attribute_value("inhibited", "1")?;
                dev.set_attribute_value("inhibited", "0")?;

                Ok(())
            },
        )
}

#[cfg(not(target_os = "windows"))]
pub fn attempt_udev_inhibited_workaround(vendor_id: u16, product_id: u16, interface_num: i32) {
    let workaround_attempt = udev_inhibited_workaround(vendor_id, product_id, interface_num);
    if let Err(err) = workaround_attempt {
        warn!(
            "Udev 'inhibited' workaround for device {vendor_id:04x}:{product_id:04x} iface: {interface_num} failed: {err}");
    } else {
        info!("Udev 'inhibited' workaround succeeded for device {vendor_id:04x}:{product_id:04x} iface: {interface_num}");
    }
}

pub fn get_device_make(usb_vid: u16, usb_pid: u16) -> Option<&'static str> {
    Some(get_device_info(usb_vid, usb_pid)?.0)
}

pub fn get_device_model(usb_vid: u16, usb_pid: u16) -> Option<&'static str> {
    Some(get_device_info(usb_vid, usb_pid)?.1)
}

pub fn get_device_info(usb_vid: u16, usb_pid: u16) -> Option<(&'static str, &'static str)> {
    let drivers = DRIVERS.read().unwrap();
    let metadata = drivers
        .iter()
        .find(|e| e.get_usb_vid() == usb_vid && e.get_usb_pid() == usb_pid);

    metadata.map(|metadata| (metadata.get_device_make(), metadata.get_device_model()))
}

#[allow(dead_code)]
#[inline]
pub fn find_device_by_handle(handle: &DeviceHandle) -> Option<Device> {
    crate::DEVICES.read().unwrap().get(handle).cloned()
}

pub fn get_device_by_index(device_class: DeviceClass, index: usize) -> Option<Device> {
    let mut cntr = 0;

    for (_handle, device) in crate::DEVICES.read().unwrap().iter() {
        if device.read().unwrap().get_device_class() == device_class {
            if index == cntr {
                return Some(device.clone());
            }

            cntr += 1;
        }
    }

    None
}
