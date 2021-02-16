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
*/

use evdev_rs::enums::EV_KEY;
use hidapi::HidApi;
use lazy_static::lazy_static;
use log::*;
use parking_lot::{Mutex, RwLock};
use std::time::Duration;
use std::{any::Any, sync::Arc, thread};
use udev::Enumerator;

mod generic_keyboard;
mod generic_mouse;
mod roccat_kone_aimo;
mod roccat_kone_pure_ultra;
mod roccat_kova_aimo;
mod roccat_nyth;
mod roccat_vulcan_1xx;
mod roccat_vulcan_pro;
mod roccat_vulcan_pro_tkl;
mod roccat_vulcan_tkl;

pub type KeyboardDevice = Arc<RwLock<Box<dyn KeyboardDeviceTrait + Sync + Send>>>;
pub type MouseDevice = Arc<RwLock<Box<dyn MouseDeviceTrait + Sync + Send>>>;

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[rustfmt::skip]
lazy_static! {
    // List of supported devices
    pub static ref DRIVERS: Arc<Mutex<[Box<(dyn DriverMetadata + Sync + Send + 'static)>; 10]>> = Arc::new(Mutex::new([
        // Supported keyboards

        // ROCCAT

        // Vulcan 100/12x/Pro (TKL) series
        KeyboardDriver::register("ROCCAT", "Vulcan 100/12x", 0x1e7d, 0x3098, &roccat_vulcan_1xx::bind_hiddev),
        KeyboardDriver::register("ROCCAT", "Vulcan 100/12x", 0x1e7d, 0x307a, &roccat_vulcan_1xx::bind_hiddev),

        KeyboardDriver::register("ROCCAT", "Vulcan Pro",     0x1e7d, 0x30f7, &roccat_vulcan_pro::bind_hiddev),

        KeyboardDriver::register("ROCCAT", "Vulcan TKL",     0x1e7d, 0x2fee, &roccat_vulcan_tkl::bind_hiddev),

        KeyboardDriver::register("ROCCAT", "Vulcan Pro TKL", 0x1e7d, 0x311a, &roccat_vulcan_pro_tkl::bind_hiddev),


        // Supported mice

        // ROCCAT
        MouseDriver::register("ROCCAT", "Kone Aimo",         0x1e7d, 0x2e27, &roccat_kone_aimo::bind_hiddev),

        MouseDriver::register("ROCCAT", "Kone Pure Ultra",   0x1e7d, 0x2dd2, &roccat_kone_pure_ultra::bind_hiddev),

        MouseDriver::register("ROCCAT", "Kova AIMO",         0x1e7d, 0x2cf1, &roccat_kova_aimo::bind_hiddev),

        MouseDriver::register("ROCCAT", "Nyth",              0x1e7d, 0x2e7c, &roccat_nyth::bind_hiddev),
        MouseDriver::register("ROCCAT", "Nyth",              0x1e7d, 0x2e7d, &roccat_nyth::bind_hiddev),
    ]));
}

#[derive(Debug, thiserror::Error)]
pub enum HwDeviceError {
    #[error("No compatible devices found")]
    NoDevicesFound {},

    #[error("An error occurred during device enumeration")]
    EnumerationError {},

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

pub struct KeyboardDriver<'a> {
    pub device_make: &'a str,
    pub device_name: &'a str,

    pub device_class: DeviceClass,

    pub usb_vid: u16,
    pub usb_pid: u16,

    pub bind_fn: &'a (dyn Fn(&HidApi, u16, u16, &str) -> Result<KeyboardDevice> + Sync + Send),
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
    ) -> Box<(dyn DriverMetadata + Sync + Send + 'static)> {
        Box::new(KeyboardDriver {
            device_make,
            device_name,
            device_class: DeviceClass::Keyboard,
            usb_vid,
            usb_pid,
            bind_fn,
        })
    }
}

impl<'a> DriverMetadata for KeyboardDriver<'a>
where
    'a: 'static,
{
    fn get_usb_vid(&self) -> u16 {
        self.usb_vid
    }

    fn get_usb_pid(&self) -> u16 {
        self.usb_pid
    }

    fn get_device_class(&self) -> DeviceClass {
        self.device_class
    }

    fn as_any(&self) -> &(dyn Any) {
        self
    }
}

pub struct MouseDriver<'a> {
    pub device_make: &'a str,
    pub device_name: &'a str,

    pub device_class: DeviceClass,

    pub usb_vid: u16,
    pub usb_pid: u16,

    pub bind_fn: &'a (dyn Fn(&HidApi, u16, u16, &str) -> Result<MouseDevice> + Sync + Send),
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
    ) -> Box<(dyn DriverMetadata + Sync + Send + 'static)> {
        Box::new(MouseDriver {
            device_make,
            device_name,
            device_class: DeviceClass::Mouse,
            usb_vid,
            usb_pid,
            bind_fn,
        })
    }
}

impl<'a> DriverMetadata for MouseDriver<'a>
where
    'a: 'static,
{
    fn get_usb_vid(&self) -> u16 {
        self.usb_vid
    }

    fn get_usb_pid(&self) -> u16 {
        self.usb_pid
    }

    fn get_device_class(&self) -> DeviceClass {
        self.device_class
    }

    fn as_any(&self) -> &(dyn Any) {
        self
    }
}

#[derive(Clone, Copy)]
pub enum DeviceClass {
    Unknown,
    Keyboard,
    Mouse,
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

/// Information about a generic device
pub trait DeviceInfoTrait {
    /// Get device capabilities
    fn get_device_capabilities(&self) -> DeviceCapabilities;

    /// Get device specific information
    fn get_device_info(&self) -> Result<DeviceInfo>;

    /// Get device firmware revision suitable for display to the user
    fn get_firmware_revision(&self) -> String;
}

/// Generic device trait
pub trait DeviceTrait: DeviceInfoTrait {
    /// Returns the USB path/ID of the device
    fn get_usb_path(&self) -> String;

    /// Returns the USB vendor ID of the device
    fn get_usb_vid(&self) -> u16;

    /// Returns the USB product ID of the device
    fn get_usb_pid(&self) -> u16;

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

/// Enumerates all HID devices on the system and returns supported devices
pub fn probe_hid_devices(api: &hidapi::HidApi) -> Result<(Vec<KeyboardDevice>, Vec<MouseDevice>)> {
    let mut keyboard_devices = vec![];
    let mut mouse_devices = vec![];

    let mut bound_devices = vec![];

    for device_info in api.device_list() {
        if let Some(driver) = DRIVERS.lock().iter().find(|&d| {
            d.get_usb_vid() == device_info.vendor_id()
                && d.get_usb_pid() == device_info.product_id()
        }) {
            debug!(
                "Found supported HID device: 0x{:x}:0x{:x} - {} {}",
                device_info.vendor_id(),
                device_info.product_id(),
                device_info
                    .manufacturer_string()
                    .unwrap_or_else(|| "<unknown>")
                    .to_string(),
                device_info
                    .product_string()
                    .unwrap_or_else(|| "<unknown>")
                    .to_string()
            );

            let serial = device_info.serial_number().unwrap_or_else(|| "");
            let path = device_info.path().to_string_lossy().to_string();

            if !bound_devices.contains(&(device_info.vendor_id(), device_info.product_id(), serial))
            {
                match driver.get_device_class() {
                    DeviceClass::Keyboard => {
                        info!(
                            "Found supported keyboard device: 0x{:x}:0x{:x} ({}) - {} {}",
                            device_info.vendor_id(),
                            device_info.product_id(),
                            path,
                            device_info
                                .manufacturer_string()
                                .unwrap_or_else(|| "<unknown>")
                                .to_string(),
                            device_info
                                .product_string()
                                .unwrap_or_else(|| "<unknown>")
                                .to_string()
                        );

                        let driver = driver.as_any().downcast_ref::<KeyboardDriver>().unwrap();

                        if let Ok(device) = (*driver.bind_fn)(
                            &api,
                            driver.get_usb_vid(),
                            driver.get_usb_pid(),
                            &serial,
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
                    }

                    DeviceClass::Mouse => {
                        info!(
                            "Found supported mouse device: 0x{:x}:0x{:x} ({}) - {} {}",
                            device_info.vendor_id(),
                            device_info.product_id(),
                            path,
                            device_info
                                .manufacturer_string()
                                .unwrap_or_else(|| "<unknown>")
                                .to_string(),
                            device_info
                                .product_string()
                                .unwrap_or_else(|| "<unknown>")
                                .to_string()
                        );

                        let driver = driver.as_any().downcast_ref::<MouseDriver>().unwrap();

                        if let Ok(device) = (*driver.bind_fn)(
                            &api,
                            driver.get_usb_vid(),
                            driver.get_usb_pid(),
                            &serial,
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
                    }

                    DeviceClass::Unknown => {
                        error!("Failed to bind the device driver, unsupported device class");
                    }
                }
            }
        } else {
            // found an unsupported device

            debug!(
                "Found unsupported HID device: 0x{:x}:0x{:x} - {} {}",
                device_info.vendor_id(),
                device_info.product_id(),
                device_info
                    .manufacturer_string()
                    .unwrap_or_else(|| "<unknown>")
                    .to_string(),
                device_info
                    .product_string()
                    .unwrap_or_else(|| "<unknown>")
                    .to_string()
            );

            let serial = device_info.serial_number().unwrap_or_else(|| "");
            let path = device_info.path().to_string_lossy().to_string();

            if !bound_devices.contains(&(device_info.vendor_id(), device_info.product_id(), serial))
            {
                match get_usb_device_class(device_info.vendor_id(), device_info.product_id()) {
                    Ok(DeviceClass::Keyboard) => {
                        info!(
                            "Found unsupported keyboard device: 0x{:x}:0x{:x} ({}) - {} {}",
                            device_info.vendor_id(),
                            device_info.product_id(),
                            path,
                            device_info
                                .manufacturer_string()
                                .unwrap_or_else(|| "<unknown>")
                                .to_string(),
                            device_info
                                .product_string()
                                .unwrap_or_else(|| "<unknown>")
                                .to_string()
                        );

                        if let Ok(device) = generic_keyboard::bind_hiddev(
                            &api,
                            device_info.vendor_id(),
                            device_info.product_id(),
                            &serial,
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
                                .unwrap_or_else(|| "<unknown>")
                                .to_string(),
                            device_info
                                .product_string()
                                .unwrap_or_else(|| "<unknown>")
                                .to_string()
                        );

                        if let Ok(device) = generic_mouse::bind_hiddev(
                            &api,
                            device_info.vendor_id(),
                            device_info.product_id(),
                            &serial,
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

                    Ok(DeviceClass::Unknown) => { /* unknown device class, ignore the device */ }

                    Err(e) => {
                        error!("Failed to query HID device class: {}", e);
                    }
                }
            }
        }
    }

    Ok((keyboard_devices, mouse_devices))
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
                                } else {
                                    if let Some(devname) =
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
pub fn get_input_sub_dev_from_udev(
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
                            } else {
                                if device.devnode().is_some() {
                                    debug!(
                                        "Ignoring evdev sub-device: {}",
                                        device.devnode().unwrap().to_str().unwrap().to_string()
                                    );
                                } else {
                                    debug!("Ignoring evdev sub-device");
                                }
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
}

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
