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

    Copyright (c) 2019-2022, The Eruption Development Team
*/

use evdev_rs::enums::EV_KEY;
use hidapi::HidApi;
use lazy_static::lazy_static;
use log::*;
use parking_lot::{Mutex, RwLock};
use serde::{self, Deserialize};
use std::collections::HashMap;
use std::u8;
use std::{any::Any, sync::Arc, thread};
use std::{path::PathBuf, time::Duration};
use udev::Enumerator;

mod corsair_strafe;
mod custom_serial_leds;
mod generic_keyboard;
mod generic_mouse;
mod roccat_aimo_pad;
mod roccat_burst_pro;
mod roccat_elo_71_air;
mod roccat_kain_100;
mod roccat_kain_2xx;
mod roccat_kone_aimo;
mod roccat_kone_aimo_remastered;
mod roccat_kone_pro_air;
mod roccat_kone_pure_ultra;
mod roccat_kone_xtd;
mod roccat_kova_2016;
mod roccat_kova_aimo;
mod roccat_magma;
mod roccat_nyth;
mod roccat_vulcan_1xx;
mod roccat_vulcan_pro;
mod roccat_vulcan_pro_tkl;
mod roccat_vulcan_tkl;

pub type KeyboardDevice = Arc<RwLock<Box<dyn KeyboardDeviceTrait + Sync + Send>>>;
pub type MouseDevice = Arc<RwLock<Box<dyn MouseDeviceTrait + Sync + Send>>>;
pub type MiscDevice = Arc<RwLock<Box<dyn MiscDeviceTrait + Sync + Send>>>;
pub type MiscSerialDevice = Arc<RwLock<Box<dyn MiscDeviceTrait + Sync + Send>>>;

#[derive(Debug, Clone, Copy, Deserialize, Eq, PartialEq, PartialOrd, Ord)]
pub enum MaturityLevel {
    #[serde(rename = "stable")]
    Stable,
    #[serde(rename = "testing")]
    Testing,
    #[serde(rename = "experimental")]
    Experimental,
}

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[rustfmt::skip]
lazy_static! {
    // List of supported devices
    pub static ref DRIVERS: Arc<Mutex<[Box<(dyn DriverMetadata + Sync + Send + 'static)>; 26]>> = Arc::new(Mutex::new([
        // Supported keyboards

        // ROCCAT

        // Vulcan 100/12x/Pro (TKL) series
        KeyboardDriver::register("ROCCAT", "Vulcan 100/12x", 0x1e7d, 0x3098, &roccat_vulcan_1xx::bind_hiddev, MaturityLevel::Stable),
        KeyboardDriver::register("ROCCAT", "Vulcan 100/12x", 0x1e7d, 0x307a, &roccat_vulcan_1xx::bind_hiddev, MaturityLevel::Stable),

        KeyboardDriver::register("ROCCAT", "Vulcan Pro",     0x1e7d, 0x30f7, &roccat_vulcan_pro::bind_hiddev, MaturityLevel::Experimental),

        KeyboardDriver::register("ROCCAT", "Vulcan TKL",     0x1e7d, 0x2fee, &roccat_vulcan_tkl::bind_hiddev, MaturityLevel::Experimental),

        KeyboardDriver::register("ROCCAT", "Vulcan Pro TKL", 0x1e7d, 0x311a, &roccat_vulcan_pro_tkl::bind_hiddev, MaturityLevel::Testing),

        KeyboardDriver::register("ROCCAT", "Magma",          0x1e7d, 0x3124, &roccat_magma::bind_hiddev, MaturityLevel::Experimental),

        // CORSAIR

        // Corsair STRAFE Gaming Keyboard
        KeyboardDriver::register("Corsair", "Corsair STRAFE Gaming Keyboard", 0x1b1c, 0x1b15, &corsair_strafe::bind_hiddev, MaturityLevel::Experimental),


        // Supported mice

        // ROCCAT
        MouseDriver::register("ROCCAT", "Kone Aimo",         0x1e7d, 0x2e27, &roccat_kone_aimo::bind_hiddev, MaturityLevel::Experimental),

        MouseDriver::register("ROCCAT", "Kone Aimo Remastered", 0x1e7d, 0x2e2c, &roccat_kone_aimo_remastered::bind_hiddev, MaturityLevel::Experimental),

        MouseDriver::register("ROCCAT", "Kone XTD Mouse",    0x1e7d, 0x2e22, &roccat_kone_xtd::bind_hiddev, MaturityLevel::Experimental),

        MouseDriver::register("ROCCAT", "Kone Pure Ultra",   0x1e7d, 0x2dd2, &roccat_kone_pure_ultra::bind_hiddev, MaturityLevel::Stable),

        MouseDriver::register("ROCCAT", "Burst Pro",         0x1e7d, 0x2de1, &roccat_burst_pro::bind_hiddev, MaturityLevel::Testing),

        MouseDriver::register("ROCCAT", "Kone Pro Air Dongle", 0x1e7d, 0x2c8e, &roccat_kone_pro_air::bind_hiddev, MaturityLevel::Experimental),
        MouseDriver::register("ROCCAT", "Kone Pro Air",        0x1e7d, 0x2c92, &roccat_kone_pro_air::bind_hiddev, MaturityLevel::Experimental),

        MouseDriver::register("ROCCAT", "Kain 100 AIMO",     0x1e7d, 0x2d00, &roccat_kain_100::bind_hiddev, MaturityLevel::Experimental),

        MouseDriver::register("ROCCAT", "Kain 200 AIMO",     0x1e7d, 0x2d5f, &roccat_kain_2xx::bind_hiddev, MaturityLevel::Testing),
        MouseDriver::register("ROCCAT", "Kain 200 AIMO",     0x1e7d, 0x2d60, &roccat_kain_2xx::bind_hiddev, MaturityLevel::Testing),
        // MouseDriver::register("ROCCAT", "Kain 202 AIMO",     0x1e7d, 0x2d60, &roccat_kain_2xx::bind_hiddev, Status::Experimental),

        MouseDriver::register("ROCCAT", "Kova AIMO",         0x1e7d, 0x2cf1, &roccat_kova_aimo::bind_hiddev, MaturityLevel::Testing),
        MouseDriver::register("ROCCAT", "Kova AIMO",         0x1e7d, 0x2cf3, &roccat_kova_aimo::bind_hiddev, MaturityLevel::Testing),

        MouseDriver::register("ROCCAT", "Kova 2016",         0x1e7d, 0x2cee, &roccat_kova_2016::bind_hiddev, MaturityLevel::Testing),
        MouseDriver::register("ROCCAT", "Kova 2016",         0x1e7d, 0x2cef, &roccat_kova_2016::bind_hiddev, MaturityLevel::Testing),
        MouseDriver::register("ROCCAT", "Kova 2016",         0x1e7d, 0x2cf0, &roccat_kova_2016::bind_hiddev, MaturityLevel::Testing),

        MouseDriver::register("ROCCAT", "Nyth",              0x1e7d, 0x2e7c, &roccat_nyth::bind_hiddev, MaturityLevel::Experimental),
        MouseDriver::register("ROCCAT", "Nyth",              0x1e7d, 0x2e7d, &roccat_nyth::bind_hiddev, MaturityLevel::Experimental),


        // Supported miscellaneous devices

        // ROCCAT/Turtle Beach
        MiscDriver::register("ROCCAT/Turtle Beach", "Elo 7.1 Air", 0x1e7d, 0x3a37, &roccat_elo_71_air::bind_hiddev, MaturityLevel::Testing),

        MiscDriver::register("ROCCAT", "Aimo Pad Wide", 0x1e7d, 0x343b, &roccat_aimo_pad::bind_hiddev, MaturityLevel::Stable),


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

pub trait DriverMetadata {
    fn get_usb_vid(&self) -> u16;
    fn get_usb_pid(&self) -> u16;

    fn get_device_class(&self) -> DeviceClass;

    fn as_any(&self) -> &(dyn Any);
}

pub trait SerialDriverMetadata: DriverMetadata {
    fn get_serial_port(&self) -> Option<&str>;
}

pub struct KeyboardDriver<'a> {
    pub device_make: &'a str,
    pub device_name: &'a str,

    pub device_class: DeviceClass,

    pub usb_vid: u16,
    pub usb_pid: u16,

    pub bind_fn: &'a (dyn Fn(&HidApi, u16, u16, &str) -> Result<KeyboardDevice> + Sync + Send),

    pub status: MaturityLevel,
}

impl<'a> KeyboardDriver<'a>
where
    'a: 'static,
{
    pub fn register(
        device_make: &'a str,
        device_name: &'a str,
        usb_vid: u16,
        usb_pid: u16,
        bind_fn: &'a (dyn Fn(&HidApi, u16, u16, &str) -> Result<KeyboardDevice> + Sync + Send),
        status: MaturityLevel,
    ) -> Box<(dyn DriverMetadata + Sync + Send + 'static)> {
        Box::new(KeyboardDriver {
            device_make,
            device_name,
            device_class: DeviceClass::Keyboard,
            usb_vid,
            usb_pid,
            bind_fn,
            status,
        })
    }
}

impl<'a> DriverMetadata for KeyboardDriver<'a>
where
    'a: 'static,
{
    fn get_device_class(&self) -> DeviceClass {
        self.device_class
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
}

pub struct MouseDriver<'a> {
    pub device_make: &'a str,
    pub device_name: &'a str,

    pub device_class: DeviceClass,

    pub usb_vid: u16,
    pub usb_pid: u16,

    pub bind_fn: &'a (dyn Fn(&HidApi, u16, u16, &str) -> Result<MouseDevice> + Sync + Send),

    pub status: MaturityLevel,
}

impl<'a> MouseDriver<'a>
where
    'a: 'static,
{
    pub fn register(
        device_make: &'a str,
        device_name: &'a str,
        usb_vid: u16,
        usb_pid: u16,
        bind_fn: &'a (dyn Fn(&HidApi, u16, u16, &str) -> Result<MouseDevice> + Sync + Send),
        status: MaturityLevel,
    ) -> Box<(dyn DriverMetadata + Sync + Send + 'static)> {
        Box::new(MouseDriver {
            device_make,
            device_name,
            device_class: DeviceClass::Mouse,
            usb_vid,
            usb_pid,
            bind_fn,
            status,
        })
    }
}

impl<'a> DriverMetadata for MouseDriver<'a>
where
    'a: 'static,
{
    fn get_device_class(&self) -> DeviceClass {
        self.device_class
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
}

pub struct MiscDriver<'a> {
    pub device_make: &'a str,
    pub device_name: &'a str,

    pub device_class: DeviceClass,

    pub usb_vid: u16,
    pub usb_pid: u16,

    pub bind_fn: &'a (dyn Fn(&HidApi, u16, u16, &str) -> Result<MiscDevice> + Sync + Send),

    pub status: MaturityLevel,
}

impl<'a> MiscDriver<'a>
where
    'a: 'static,
{
    #[allow(dead_code)]
    pub fn register(
        device_make: &'a str,
        device_name: &'a str,
        usb_vid: u16,
        usb_pid: u16,
        bind_fn: &'a (dyn Fn(&HidApi, u16, u16, &str) -> Result<MiscDevice> + Sync + Send),
        status: MaturityLevel,
    ) -> Box<(dyn DriverMetadata + Sync + Send + 'static)> {
        Box::new(MiscDriver {
            device_make,
            device_name,
            device_class: DeviceClass::Misc,
            usb_vid,
            usb_pid,
            bind_fn,
            status,
        })
    }
}

impl<'a> DriverMetadata for MiscDriver<'a>
where
    'a: 'static,
{
    fn get_device_class(&self) -> DeviceClass {
        self.device_class
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
}

pub struct MiscSerialDriver<'a> {
    pub device_make: &'a str,
    pub device_name: &'a str,

    pub device_class: DeviceClass,

    pub serial_port: Option<&'a str>,

    pub bind_fn: &'a (dyn Fn(&str) -> Result<MiscSerialDevice> + Sync + Send),

    pub status: MaturityLevel,
}

impl<'a> MiscSerialDriver<'a>
where
    'a: 'static,
{
    #[allow(dead_code)]
    pub fn register(
        device_make: &'a str,
        device_name: &'a str,
        bind_fn: &'a (dyn Fn(&str) -> Result<MiscSerialDevice> + Sync + Send),
        status: MaturityLevel,
    ) -> Box<(dyn DriverMetadata + Sync + Send + 'static)> {
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

impl<'a> DriverMetadata for MiscSerialDriver<'a>
where
    'a: 'static,
{
    fn get_device_class(&self) -> DeviceClass {
        self.device_class
    }

    fn as_any(&self) -> &(dyn Any) {
        self
    }

    fn get_usb_vid(&self) -> u16 {
        0
    }

    fn get_usb_pid(&self) -> u16 {
        0
    }
}

impl<'a> SerialDriverMetadata for MiscSerialDriver<'a>
where
    'a: 'static,
{
    fn get_serial_port(&self) -> Option<&str> {
        self.serial_port
    }
}

#[derive(Clone, Copy)]
pub enum DeviceClass {
    Unknown,
    Keyboard,
    Mouse,
    Misc,
}

/// Represents an RGBA color value
#[derive(Debug, Copy, Clone)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// A Keyboard HID event
#[derive(Debug, Copy, Clone, PartialEq)]
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
#[derive(Debug, Copy, Clone, PartialEq)]
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
#[derive(Debug, Copy, Clone, PartialEq)]
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

impl Into<u8> for LedKind {
    /// Convert a LedKind to an integer constant
    fn into(self) -> u8 {
        match self {
            Self::Unknown => 0,
            Self::AudioMute => 1,
            Self::Fx => 2,
            Self::Volume => 3,
            Self::NumLock => 4,
            Self::CapsLock => 5,
            Self::ScrollLock => 6,
            Self::GameMode => 7,
        }
    }
}

/// Generic Device capabilities
#[derive(Debug, Clone)]
pub struct DeviceCapabilities {}

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
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
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

/// Information about a generic device
pub trait DeviceInfoTrait {
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
pub trait GenericConfiguration {
    /// Get device specific configuration
    fn get_device_config(&self, param: &DeviceConfig) -> Result<Box<dyn Any>>;

    /// Set device specific configuration
    fn set_device_config(&self, param: &DeviceConfig, value: &dyn Any) -> Result<()>;
}

/// Generic device trait
pub trait DeviceTrait: DeviceInfoTrait {
    /// Returns the USB path/ID of the device
    fn get_usb_path(&self) -> String;

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

    /// Returns `true` if the device has been initialized
    fn is_initialized(&self) -> Result<bool>;

    /// Returns `true` if the device has failed or has been disconnected
    fn has_failed(&self) -> Result<bool>;

    /// Send raw data to the control device
    fn write_data_raw(&self, buf: &[u8]) -> Result<()>;

    /// Read raw data from the control device
    fn read_data_raw(&self, size: usize) -> Result<Vec<u8>>;

    /// Get the device status
    fn device_status(&self) -> Result<DeviceStatus>;

    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;

    fn as_device(&self) -> &dyn DeviceTrait;
    fn as_device_mut(&mut self) -> &mut dyn DeviceTrait;

    fn as_mouse_device(&self) -> Option<&dyn MouseDeviceTrait>;
    fn as_mouse_device_mut(&mut self) -> Option<&mut dyn MouseDeviceTrait>;
}

// Generic device trait
pub trait SerialDeviceTrait: DeviceInfoTrait {
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
pub trait KeyboardDeviceTrait: DeviceTrait {
    /// Set the state of a device status LED, like e.g. Num Lock, etc...
    fn set_status_led(&self, led_kind: LedKind, on: bool) -> Result<()>;

    /// Set the device specific brightness
    fn set_local_brightness(&mut self, brightness: i32) -> Result<()>;

    /// Get the device specific brightness
    fn get_local_brightness(&self) -> Result<i32>;

    /// Send RGBA LED map to the device
    fn send_led_map(&mut self, led_map: &[RGBA]) -> Result<()>;

    /// Send the LED init pattern to the device. This should be used to initialize
    /// all LEDs and set them to a known good state
    fn set_led_init_pattern(&mut self) -> Result<()>;

    /// Send a LED finalization pattern to the device. This should normally be used,
    /// to set the device to a known good state, on exit of the daemon
    fn set_led_off_pattern(&mut self) -> Result<()>;

    /// Get the next HID event from the control device (blocking)
    fn get_next_event(&self) -> Result<KeyboardHidEvent>;

    /// Get the next HID event from the control device (with timeout)
    fn get_next_event_timeout(&self, millis: i32) -> Result<KeyboardHidEvent>;

    /// Convert an EV_KEY to an index value
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
pub trait MouseDeviceTrait: DeviceTrait {
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

    /// Set the device specific brightness
    fn set_local_brightness(&mut self, brightness: i32) -> Result<()>;

    /// Get the device specific brightness
    fn get_local_brightness(&self) -> Result<i32>;

    /// Get the next HID event from the control device (blocking)
    fn get_next_event(&self) -> Result<MouseHidEvent>;

    /// Get the next HID event from the control device (with timeout)
    fn get_next_event_timeout(&self, millis: i32) -> Result<MouseHidEvent>;

    /// Converts an EV_KEY value to a button index
    fn ev_key_to_button_index(&self, code: EV_KEY) -> Result<u8>;

    /// Converts a button index to an EV_KEY value
    fn button_index_to_ev_key(&self, index: u32) -> Result<EV_KEY>;

    /// Send RGBA LED map to the device
    fn send_led_map(&mut self, led_map: &[RGBA]) -> Result<()>;

    /// Send the LED init pattern to the device. This should be used to initialize
    /// all LEDs and set them to a known good state
    fn set_led_init_pattern(&mut self) -> Result<()>;

    /// Send a LED finalization pattern to the device. This should normally be used,
    /// to set the device to a known good state, on exit of the daemon
    fn set_led_off_pattern(&mut self) -> Result<()>;

    /// Returns true when the mouse supports a secondary sub-device like e.g. a keyboard panel
    fn has_secondary_device(&self) -> bool;
}

/// Misc Devices
pub trait MiscDeviceTrait: DeviceTrait {
    /// Returns true when the device supports an input sub-device like e.g. a dial or volume wheel on a headset
    fn has_input_device(&self) -> bool;

    /// Set the device specific brightness
    fn set_local_brightness(&mut self, brightness: i32) -> Result<()>;

    /// Get the device specific brightness
    fn get_local_brightness(&self) -> Result<i32>;

    /// Send RGBA LED map to the device
    fn send_led_map(&mut self, led_map: &[RGBA]) -> Result<()>;

    /// Send the LED init pattern to the device. This should be used to initialize
    /// all LEDs and set them to a known good state
    fn set_led_init_pattern(&mut self) -> Result<()>;

    /// Send a LED finalization pattern to the device. This should normally be used,
    /// to set the device to a known good state, on exit of the daemon
    fn set_led_off_pattern(&mut self) -> Result<()>;
}

/// Misc Serial Devices
pub trait MiscSerialDeviceTrait: SerialDeviceTrait {
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
    let config = crate::CONFIG.lock();

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

    let config = crate::CONFIG.lock();

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
/// from the .conf file as well), and then returns a tuple of all the supported devices
pub fn probe_devices() -> Result<(Vec<KeyboardDevice>, Vec<MouseDevice>, Vec<MiscDevice>)> {
    let mut keyboard_devices = vec![];
    let mut mouse_devices = vec![];
    let mut misc_devices = vec![];

    // bind all declared non-pnp devices from configuration file
    let declared_devices = get_non_pnp_devices()?;

    for device in declared_devices {
        if device.class == "serial" {
            info!(
                "Binding non-pnp serial LEDs device: {} ({})",
                device.name,
                device.device_file.display()
            );

            let serial_leds = custom_serial_leds::CustomSerialLeds::bind(device.device_file);

            // non pnp devices are currently always 'misc' devices
            misc_devices.push(Arc::new(RwLock::new(
                Box::new(serial_leds) as Box<dyn MiscDeviceTrait + Sync + Send>
            )));
        } else {
            error!("Unknown device class specified in the configuration file");
        }
    }

    let mut bound_devices = vec![];

    let hidapi = crate::HIDAPI.read();
    let api = hidapi.as_ref().unwrap();

    for device_info in api.device_list() {
        if !is_device_blacklisted(device_info.vendor_id(), device_info.product_id())? {
            if let Some(driver) = DRIVERS.lock().iter().find(|&d| {
                d.get_usb_vid() == device_info.vendor_id()
                    && d.get_usb_pid() == device_info.product_id()
            }) {
                debug!(
                    "Found supported device: 0x{:x}:0x{:x} - {} {}",
                    device_info.vendor_id(),
                    device_info.product_id(),
                    device_info
                        .manufacturer_string()
                        .unwrap_or("<unknown>")
                        .to_string(),
                    device_info
                        .product_string()
                        .unwrap_or("<unknown>")
                        .to_string()
                );

                let serial = device_info.serial_number().unwrap_or("");
                let path = device_info.path().to_string_lossy().to_string();

                if !bound_devices.contains(&(
                    device_info.vendor_id(),
                    device_info.product_id(),
                    serial,
                )) {
                    match driver.get_device_class() {
                        DeviceClass::Keyboard => {
                            info!(
                                "Found supported keyboard device: 0x{:x}:0x{:x} ({}) - {} {}",
                                device_info.vendor_id(),
                                device_info.product_id(),
                                path,
                                device_info
                                    .manufacturer_string()
                                    .unwrap_or("<unknown>")
                                    .to_string(),
                                device_info
                                    .product_string()
                                    .unwrap_or("<unknown>")
                                    .to_string()
                            );

                            let driver = driver.as_any().downcast_ref::<KeyboardDriver>().unwrap();
                            let driver_maturity_level = *crate::DRIVER_MATURITY_LEVEL.lock();

                            if driver.status <= driver_maturity_level {
                                if let Ok(device) = (*driver.bind_fn)(
                                    &api,
                                    driver.get_usb_vid(),
                                    driver.get_usb_pid(),
                                    serial,
                                ) {
                                    keyboard_devices.push(device);
                                    bound_devices.push((
                                        driver.get_usb_vid(),
                                        driver.get_usb_pid(),
                                        serial,
                                    ));
                                } else {
                                    error!("Failed to bind the device driver");
                                }
                            } else {
                                warn!("Not binding the device driver because it would require a lesser code maturity level");
                                warn!("To enable this device driver, please change the 'driver_maturity_level' setting in eruption.conf respectively");
                            }
                        }

                        DeviceClass::Mouse => {
                            info!(
                                "Found supported mouse device: 0x{:x}:0x{:x} ({}) - {} {}",
                                device_info.vendor_id(),
                                device_info.product_id(),
                                path,
                                device_info
                                    .manufacturer_string()
                                    .unwrap_or("<unknown>")
                                    .to_string(),
                                device_info
                                    .product_string()
                                    .unwrap_or("<unknown>")
                                    .to_string()
                            );

                            let hidapi = crate::HIDAPI.read();
                            let api = hidapi.as_ref().unwrap();

                            let driver = driver.as_any().downcast_ref::<MouseDriver>().unwrap();
                            let driver_maturity_level = *crate::DRIVER_MATURITY_LEVEL.lock();

                            if driver.status <= driver_maturity_level {
                                if let Ok(device) = (*driver.bind_fn)(
                                    &api,
                                    driver.get_usb_vid(),
                                    driver.get_usb_pid(),
                                    serial,
                                ) {
                                    mouse_devices.push(device);
                                    bound_devices.push((
                                        driver.get_usb_vid(),
                                        driver.get_usb_pid(),
                                        serial,
                                    ));
                                } else {
                                    error!("Failed to bind the device driver");
                                }
                            } else {
                                warn!("Not binding the device driver because it would require a lesser code maturity level");
                                warn!("To enable this device driver, please change the 'driver_maturity_level' setting in eruption.conf respectively");
                            }
                        }

                        DeviceClass::Misc => {
                            info!(
                                "Found supported misc device: 0x{:x}:0x{:x} ({}) - {} {}",
                                device_info.vendor_id(),
                                device_info.product_id(),
                                path,
                                device_info
                                    .manufacturer_string()
                                    .unwrap_or("<unknown>")
                                    .to_string(),
                                device_info
                                    .product_string()
                                    .unwrap_or("<unknown>")
                                    .to_string()
                            );

                            let driver = driver.as_any().downcast_ref::<MiscDriver>().unwrap();
                            let driver_maturity_level = *crate::DRIVER_MATURITY_LEVEL.lock();

                            if driver.status <= driver_maturity_level {
                                if let Ok(device) = (*driver.bind_fn)(
                                    &api,
                                    driver.get_usb_vid(),
                                    driver.get_usb_pid(),
                                    serial,
                                ) {
                                    misc_devices.push(device);
                                    bound_devices.push((
                                        driver.get_usb_vid(),
                                        driver.get_usb_pid(),
                                        serial,
                                    ));
                                } else {
                                    error!("Failed to bind the device driver");
                                }
                            } else {
                                warn!("Not binding the device driver because it would require a lesser code maturity level");
                                warn!("To enable this device driver, please change the 'driver_maturity_level' setting in eruption.conf respectively");
                            }
                        }

                        DeviceClass::Unknown => {
                            error!("Failed to bind the device driver, unsupported device class");
                        }
                    }
                }
            } else {
                // found an unsupported device

                debug!(
                    "Found unsupported device: 0x{:x}:0x{:x} - {} {}",
                    device_info.vendor_id(),
                    device_info.product_id(),
                    device_info
                        .manufacturer_string()
                        .unwrap_or("<unknown>")
                        .to_string(),
                    device_info
                        .product_string()
                        .unwrap_or("<unknown>")
                        .to_string()
                );

                let serial = device_info.serial_number().unwrap_or("");
                let path = device_info.path().to_string_lossy().to_string();

                if !bound_devices.contains(&(
                    device_info.vendor_id(),
                    device_info.product_id(),
                    serial,
                )) {
                    match get_usb_device_class(device_info.vendor_id(), device_info.product_id()) {
                        Ok(DeviceClass::Keyboard) => {
                            info!(
                                "Found unsupported keyboard device: 0x{:x}:0x{:x} ({}) - {} {}",
                                device_info.vendor_id(),
                                device_info.product_id(),
                                path,
                                device_info
                                    .manufacturer_string()
                                    .unwrap_or("<unknown>")
                                    .to_string(),
                                device_info
                                    .product_string()
                                    .unwrap_or("<unknown>")
                                    .to_string()
                            );

                            if let Ok(device) = generic_keyboard::bind_hiddev(
                                &api,
                                device_info.vendor_id(),
                                device_info.product_id(),
                                serial,
                            ) {
                                keyboard_devices.push(device);
                                bound_devices.push((
                                    device_info.vendor_id(),
                                    device_info.product_id(),
                                    serial,
                                ));
                            } else {
                                error!("Failed to bind the device driver");
                            }
                        }

                        Ok(DeviceClass::Mouse) => {
                            info!(
                                "Found unsupported mouse device: 0x{:x}:0x{:x} ({}) - {} {}",
                                device_info.vendor_id(),
                                device_info.product_id(),
                                path,
                                device_info
                                    .manufacturer_string()
                                    .unwrap_or("<unknown>")
                                    .to_string(),
                                device_info
                                    .product_string()
                                    .unwrap_or("<unknown>")
                                    .to_string()
                            );

                            if let Ok(device) = generic_mouse::bind_hiddev(
                                &api,
                                device_info.vendor_id(),
                                device_info.product_id(),
                                serial,
                            ) {
                                mouse_devices.push(device);
                                bound_devices.push((
                                    device_info.vendor_id(),
                                    device_info.product_id(),
                                    serial,
                                ));
                            } else {
                                error!("Failed to bind the device driver");
                            }
                        }

                        Ok(DeviceClass::Unknown) | Ok(DeviceClass::Misc) => { /* unknown device class, ignore the device */
                        }

                        Err(e) => {
                            error!("Failed to query device class: {}", e);
                        }
                    }
                }
            }
        } else {
            info!(
                "Skipping blacklisted device: 0x{:x}:0x{:x} - {} {}",
                device_info.vendor_id(),
                device_info.product_id(),
                device_info
                    .manufacturer_string()
                    .unwrap_or("<unknown>")
                    .to_string(),
                device_info
                    .product_string()
                    .unwrap_or("<unknown>")
                    .to_string()
            );
        }
    }

    Ok((keyboard_devices, mouse_devices, misc_devices))
}

/// Enumerates all HID devices on the system and then returns a tuple of all the supported devices
/// Already bound devices will be ignored
pub fn probe_devices_hotplug() -> Result<(Vec<KeyboardDevice>, Vec<MouseDevice>, Vec<MiscDevice>)> {
    // wait for devices to settle
    // thread::sleep(Duration::from_millis(3000));

    let mut keyboard_devices = vec![];
    let mut mouse_devices = vec![];
    let mut misc_devices = vec![];

    let mut bound_devices = vec![];

    let mut hidapi = crate::HIDAPI.write();
    let api = hidapi.as_mut().unwrap();

    api.refresh_devices()?;

    for device_info in api.device_list() {
        if !is_device_blacklisted(device_info.vendor_id(), device_info.product_id())? {
            if let Some(driver) = DRIVERS.lock().iter().find(|&d| {
                d.get_usb_vid() == device_info.vendor_id()
                    && d.get_usb_pid() == device_info.product_id()
            }) {
                debug!(
                    "Found supported device: 0x{:x}:0x{:x} - {} {}",
                    device_info.vendor_id(),
                    device_info.product_id(),
                    device_info
                        .manufacturer_string()
                        .unwrap_or("<unknown>")
                        .to_string(),
                    device_info
                        .product_string()
                        .unwrap_or("<unknown>")
                        .to_string()
                );

                let serial = device_info.serial_number().unwrap_or("");
                let path = device_info.path().to_string_lossy().to_string();

                if !bound_devices.contains(&(
                    device_info.vendor_id(),
                    device_info.product_id(),
                    serial,
                )) {
                    match driver.get_device_class() {
                        DeviceClass::Keyboard => {
                            info!(
                                "Found supported keyboard device: 0x{:x}:0x{:x} ({}) - {} {}",
                                device_info.vendor_id(),
                                device_info.product_id(),
                                path,
                                device_info
                                    .manufacturer_string()
                                    .unwrap_or("<unknown>")
                                    .to_string(),
                                device_info
                                    .product_string()
                                    .unwrap_or("<unknown>")
                                    .to_string()
                            );

                            let driver = driver.as_any().downcast_ref::<KeyboardDriver>().unwrap();
                            let driver_maturity_level = *crate::DRIVER_MATURITY_LEVEL.lock();

                            if driver.status <= driver_maturity_level {
                                if let Ok(device) = (*driver.bind_fn)(
                                    &api,
                                    driver.get_usb_vid(),
                                    driver.get_usb_pid(),
                                    serial,
                                ) {
                                    keyboard_devices.push(device);
                                    bound_devices.push((
                                        driver.get_usb_vid(),
                                        driver.get_usb_pid(),
                                        serial,
                                    ));
                                } else {
                                    error!("Failed to bind the device driver");
                                }
                            } else {
                                warn!("Not binding the device driver because it would require a lesser code maturity level");
                                warn!("To enable this device driver, please change the 'driver_maturity_level' setting in eruption.conf respectively");
                            }
                        }

                        DeviceClass::Mouse => {
                            info!(
                                "Found supported mouse device: 0x{:x}:0x{:x} ({}) - {} {}",
                                device_info.vendor_id(),
                                device_info.product_id(),
                                path,
                                device_info
                                    .manufacturer_string()
                                    .unwrap_or("<unknown>")
                                    .to_string(),
                                device_info
                                    .product_string()
                                    .unwrap_or("<unknown>")
                                    .to_string()
                            );

                            let driver = driver.as_any().downcast_ref::<MouseDriver>().unwrap();
                            let driver_maturity_level = *crate::DRIVER_MATURITY_LEVEL.lock();

                            if driver.status <= driver_maturity_level {
                                if let Ok(device) = (*driver.bind_fn)(
                                    &api,
                                    driver.get_usb_vid(),
                                    driver.get_usb_pid(),
                                    serial,
                                ) {
                                    mouse_devices.push(device);
                                    bound_devices.push((
                                        driver.get_usb_vid(),
                                        driver.get_usb_pid(),
                                        serial,
                                    ));
                                } else {
                                    error!("Failed to bind the device driver");
                                }
                            } else {
                                warn!("Not binding the device driver because it would require a lesser code maturity level");
                                warn!("To enable this device driver, please change the 'driver_maturity_level' setting in eruption.conf respectively");
                            }
                        }

                        DeviceClass::Misc => {
                            info!(
                                "Found supported misc device: 0x{:x}:0x{:x} ({}) - {} {}",
                                device_info.vendor_id(),
                                device_info.product_id(),
                                path,
                                device_info
                                    .manufacturer_string()
                                    .unwrap_or("<unknown>")
                                    .to_string(),
                                device_info
                                    .product_string()
                                    .unwrap_or("<unknown>")
                                    .to_string()
                            );

                            let driver = driver.as_any().downcast_ref::<MiscDriver>().unwrap();
                            let driver_maturity_level = *crate::DRIVER_MATURITY_LEVEL.lock();

                            if driver.status <= driver_maturity_level {
                                if let Ok(device) = (*driver.bind_fn)(
                                    &api,
                                    driver.get_usb_vid(),
                                    driver.get_usb_pid(),
                                    serial,
                                ) {
                                    misc_devices.push(device);
                                    bound_devices.push((
                                        driver.get_usb_vid(),
                                        driver.get_usb_pid(),
                                        serial,
                                    ));
                                } else {
                                    error!("Failed to bind the device driver");
                                }
                            } else {
                                warn!("Not binding the device driver because it would require a lesser code maturity level");
                                warn!("To enable this device driver, please change the 'driver_maturity_level' setting in eruption.conf respectively");
                            }
                        }

                        DeviceClass::Unknown => {
                            error!("Failed to bind the device driver, unsupported device class");
                        }
                    }
                }
            } else {
                // found an unsupported device

                debug!(
                    "Found unsupported device: 0x{:x}:0x{:x} - {} {}",
                    device_info.vendor_id(),
                    device_info.product_id(),
                    device_info
                        .manufacturer_string()
                        .unwrap_or("<unknown>")
                        .to_string(),
                    device_info
                        .product_string()
                        .unwrap_or("<unknown>")
                        .to_string()
                );

                let serial = device_info.serial_number().unwrap_or("");
                let path = device_info.path().to_string_lossy().to_string();

                if !bound_devices.contains(&(
                    device_info.vendor_id(),
                    device_info.product_id(),
                    serial,
                )) {
                    match get_usb_device_class(device_info.vendor_id(), device_info.product_id()) {
                        Ok(DeviceClass::Keyboard) => {
                            info!(
                                "Found unsupported keyboard device: 0x{:x}:0x{:x} ({}) - {} {}",
                                device_info.vendor_id(),
                                device_info.product_id(),
                                path,
                                device_info
                                    .manufacturer_string()
                                    .unwrap_or("<unknown>")
                                    .to_string(),
                                device_info
                                    .product_string()
                                    .unwrap_or("<unknown>")
                                    .to_string()
                            );

                            if let Ok(device) = generic_keyboard::bind_hiddev(
                                &api,
                                device_info.vendor_id(),
                                device_info.product_id(),
                                serial,
                            ) {
                                keyboard_devices.push(device);
                                bound_devices.push((
                                    device_info.vendor_id(),
                                    device_info.product_id(),
                                    serial,
                                ));
                            } else {
                                error!("Failed to bind the device driver");
                            }
                        }

                        Ok(DeviceClass::Mouse) => {
                            info!(
                                "Found unsupported mouse device: 0x{:x}:0x{:x} ({}) - {} {}",
                                device_info.vendor_id(),
                                device_info.product_id(),
                                path,
                                device_info
                                    .manufacturer_string()
                                    .unwrap_or("<unknown>")
                                    .to_string(),
                                device_info
                                    .product_string()
                                    .unwrap_or("<unknown>")
                                    .to_string()
                            );

                            if let Ok(device) = generic_mouse::bind_hiddev(
                                &api,
                                device_info.vendor_id(),
                                device_info.product_id(),
                                serial,
                            ) {
                                mouse_devices.push(device);
                                bound_devices.push((
                                    device_info.vendor_id(),
                                    device_info.product_id(),
                                    serial,
                                ));
                            } else {
                                error!("Failed to bind the device driver");
                            }
                        }

                        Ok(DeviceClass::Unknown) | Ok(DeviceClass::Misc) => { /* unknown device class, ignore the device */
                        }

                        Err(e) => {
                            error!("Failed to query device class: {}", e);
                        }
                    }
                }
            }
        } else {
            info!(
                "Skipping blacklisted device: 0x{:x}:0x{:x} - {} {}",
                device_info.vendor_id(),
                device_info.product_id(),
                device_info
                    .manufacturer_string()
                    .unwrap_or("<unknown>")
                    .to_string(),
                device_info
                    .product_string()
                    .unwrap_or("<unknown>")
                    .to_string()
            );
        }
    }

    Ok((keyboard_devices, mouse_devices, misc_devices))
}

/// Get the path of the USB device from udev
pub fn get_input_dev_from_udev(usb_vid: u16, usb_pid: u16) -> Result<String> {
    // retry up to n times, in case device enumeration fails
    let mut retry_counter = 3;

    loop {
        match Enumerator::new() {
            Ok(mut enumerator) => {
                // enumerator.match_is_initialized().unwrap();
                enumerator.match_subsystem("input").unwrap();

                match enumerator.scan_devices() {
                    Ok(devices) => {
                        for device in devices {
                            let found_dev = device.properties().any(|e| {
                                e.name() == "ID_VENDOR_ID"
                                    && ([usb_vid]
                                        .iter()
                                        .map(|v| format!("{:04x}", v))
                                        .any(|v| v == e.value().to_string_lossy()))
                            }) && device.properties().any(|e| {
                                e.name() == "ID_MODEL_ID"
                                    && ([usb_pid]
                                        .iter()
                                        .map(|v| format!("{:04x}", v))
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
                            error!("Requested device could not be found");

                            break Err(HwDeviceError::NoDevicesFound {}.into());
                        } else {
                            // wait for the device to be available
                            retry_counter -= 1;
                            thread::sleep(Duration::from_millis(500));
                        }
                    }

                    Err(_e) => {
                        if retry_counter <= 0 {
                            // give up the search
                            break Err(HwDeviceError::EnumerationError {}.into());
                        } else {
                            // wait for the enumerator to be available
                            retry_counter -= 1;
                            thread::sleep(Duration::from_millis(500));
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
                    thread::sleep(Duration::from_millis(500));
                }
            }
        }
    }
}

/// Get the path of the USB device from udev
/* pub fn get_input_sub_dev_from_udev(
    usb_vid: u16,
    usb_pid: u16,
    device_index: usize,
) -> Result<String> {
    // retry up to n times, in case device enumeration fails
    let mut retry_counter = 3;

    loop {
        match Enumerator::new() {
            Ok(mut enumerator) => {
                // enumerator.match_is_initialized();
                enumerator.match_subsystem("input").unwrap();

                match enumerator.scan_devices() {
                    Ok(devices) => {
                        for device in devices {
                            let found_dev = device.properties().any(|e| {
                                e.name() == "ID_VENDOR_ID"
                                    && ([usb_vid]
                                        .iter()
                                        .map(|v| format!("{:04x}", v))
                                        .any(|v| v == e.value().to_string_lossy()))
                            }) && device.properties().any(|e| {
                                e.name() == "ID_MODEL_ID"
                                    && ([usb_pid]
                                        .iter()
                                        .map(|v| format!("{:04x}", v))
                                        .any(|v| v == e.value().to_string_lossy()))
                            }) && device.properties().any(|e| {
                                e.name() == "ID_USB_INTERFACE_NUM"
                                    && ([device_index]
                                        .iter()
                                        .map(|v| format!("{:02}", v))
                                        .any(|v| v == e.value().to_string_lossy()))
                            }) && device.devnode().is_some();

                            if found_dev {
                                debug!(
                                    "Picking evdev sub-device: {}",
                                    device.devnode().unwrap().to_str().unwrap().to_string()
                                );

                                return Ok(device.devnode().unwrap().to_str().unwrap().to_string());
                            } else if device.devnode().is_some() {
                                debug!(
                                    "Ignoring evdev sub-device: {}",
                                    device.devnode().unwrap().to_str().unwrap().to_string()
                                );
                            } else {
                                debug!("Ignoring evdev sub-device");
                            }
                        }

                        if retry_counter <= 0 {
                            // give up the search
                            break Err(HwDeviceError::NoDevicesFound {}.into());
                        } else {
                            // wait for the device to be available
                            retry_counter -= 1;
                            thread::sleep(Duration::from_millis(500));
                        }
                    }

                    Err(_e) => {
                        if retry_counter <= 0 {
                            // give up the search
                            break Err(HwDeviceError::EnumerationError {}.into());
                        } else {
                            // wait for the enumerator to be available
                            retry_counter -= 1;
                            thread::sleep(Duration::from_millis(500));
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
                    thread::sleep(Duration::from_millis(500));
                }
            }
        }
    }
} */
// Get the path of the USB device from udev
// pub fn get_input_dev_from_udev_unsafe(usb_vid: u16, usb_pid: u16) -> Result<String> {
//     match Enumerator::new() {
//         Ok(mut enumerator) => {
//             enumerator.match_subsystem("input").unwrap();

//             match enumerator.scan_devices() {
//                 Ok(devices) => {
//                     for device in devices {
//                         let found_dev = device.properties().any(|e| {
//                             e.name() == "ID_VENDOR_ID"
//                                 && ([usb_vid]
//                                     .iter()
//                                     .map(|v| format!("{:04x}", v))
//                                     .any(|v| v == e.value().to_string_lossy()))
//                         }) && device.properties().any(|e| {
//                             e.name() == "ID_MODEL_ID"
//                                 && ([usb_pid]
//                                     .iter()
//                                     .map(|v| format!("{:04x}", v))
//                                     .any(|v| v == e.value().to_string_lossy()))
//                         }) && device.devnode().is_some();

//                         if found_dev {
//                             return Ok(device.devnode().unwrap().to_str().unwrap().to_string());
//                         }
//                     }

//                     Err(HwDeviceError::NoDevicesFound {}.into())
//                 }

//                 Err(_e) => Err(HwDeviceError::EnumerationError {}.into()),
//             }
//         }

//         Err(_e) => Err(HwDeviceError::UdevError {}.into()),
//     }
// }

/// Queries udev for the device class of an USB input device
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
                                    .map(|v| format!("{:04x}", v))
                                    .any(|v| v == e.value().to_string_lossy()))
                        }) && device.properties().any(|e| {
                            e.name() == "ID_MODEL_ID"
                                && ([usb_pid]
                                    .iter()
                                    .map(|v| format!("{:04x}", v))
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
