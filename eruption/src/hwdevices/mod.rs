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

use failure::Fail;
use log::*;
use parking_lot::RwLock;
use std::sync::Arc;

mod roccat_kone_pure_ultra;
// mod roccat_nyth;
mod roccat_vulcan;

use roccat_kone_pure_ultra::RoccatKonePureUltra;
// use roccat_nyth::RoccatNyth;
use roccat_vulcan::{KeyboardHidEventCode, RoccatVulcan1xx};

pub use roccat_vulcan::hid_code_to_key_index; // TODO: Fix this

pub type KeyboardDevice = Arc<RwLock<Box<dyn KeyboardDeviceTrait + Sync + Send>>>;
pub type MouseDevice = Arc<RwLock<Box<dyn MouseDeviceTrait + Sync + Send>>>;

pub type Result<T> = std::result::Result<T, HwDeviceError>;

// List of supported devices
// Supported keyboards
pub const VENDOR_IDS: [u16; 1] = [0x1e7d]; // ROCCAT
pub const PRODUCT_IDS: [u16; 2] = [
    0x3098, 0x307a, // ROCCAT Vulcan 100/12x
];

// Supported mice
pub const VENDOR_IDS_MICE: [u16; 1] = [0x1e7d]; // ROCCAT
pub const PRODUCT_IDS_MICE: [u16; 3] = [
    0x2dd2, // ROCCAT Kone Pure Ultra
    0x2e7c, 0x2e7d, // ROCCAT Nyth
];

pub const NUM_KEYS: usize = roccat_vulcan::NUM_KEYS; // TODO: Fix this

#[derive(Debug, Fail)]
pub enum HwDeviceError {
    #[fail(display = "Could not enumerate devices")]
    EnumerationError {},

    #[fail(display = "Could not open the device file")]
    DeviceOpenError {},

    // #[fail(display = "Invalid init sequence")]
    // InitSequenceError {},

    // #[fail(display = "Invalid operation")]
    // InvalidOperation {},

    // #[fail(display = "Unsupported operation")]
    // UnsupportedOperationError {},
    #[fail(display = "Device not bound")]
    DeviceNotBound {},

    #[fail(display = "Device not opened")]
    DeviceNotOpened {},

    #[fail(display = "Device not initialized")]
    DeviceNotInitialized {},

    #[fail(display = "Invalid status code")]
    InvalidStatusCode {},

    #[fail(display = "Invalid result")]
    InvalidResult {},

    #[fail(display = "Write error")]
    WriteError {},

    //#[fail(display = "Could not close the device")]
    //CloseError {},
    #[fail(display = "Invalid value: {}", description)]
    ValueError { description: String },
    // #[fail(display = "Unknown error: {}", description)]
    // UnknownError { description: String },
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

    // Audio related
    MuteDown,
    MuteUp,
    VolumeDown,
    VolumeUp,
}

/// A Keyboard HID event
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MouseHidEvent {
    Unknown,

    // Button events
    DpiChange(u8),
}

/// Status LEDs
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

impl LedKind {
    /// Instantiate a LedKind using an integer constant
    pub fn from_id(id: u8) -> Result<Self> {
        match id {
            0 => Ok(Self::Unknown),
            1 => Ok(Self::AudioMute),
            2 => Ok(Self::Fx),
            3 => Ok(Self::Volume),
            4 => Ok(Self::NumLock),
            5 => Ok(Self::CapsLock),
            6 => Ok(Self::ScrollLock),
            7 => Ok(Self::GameMode),

            _ => Err(HwDeviceError::ValueError {
                description: "Invalid LED identifier".to_owned(),
            }),
        }
    }
}

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

/// Information about a generic device
pub trait DeviceInfoTrait {
    type NativeDeviceInfo;

    /// Get device capabilities
    fn get_device_capabilities(&self) -> DeviceCapabilities;

    /// Get device specific information
    fn get_device_info(&self) -> Result<Self::NativeDeviceInfo>;
}

/// Generic device trait
pub trait DeviceTrait {
    /// Returns the USB path/ID of the device
    fn get_usb_path(&self) -> String;

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
}

/// Device like e.g. a supported mouse
pub trait MouseDeviceTrait: DeviceTrait {
    /// Get the next HID event from the control device (blocking)
    fn get_next_event(&self) -> Result<MouseHidEvent>;

    /// Get the next HID event from the control device (with timeout)
    fn get_next_event_timeout(&self, millis: i32) -> Result<MouseHidEvent>;

    /// Send RGBA LED map to the device
    fn send_led_map(&mut self, led_map: &[RGBA]) -> Result<()>;

    /// Send the LED init pattern to the device. This should be used to initialize
    /// all LEDs and set them to a known good state
    fn set_led_init_pattern(&mut self) -> Result<()>;

    /// Send a LED finalization pattern to the device. This should normally be used,
    /// to set the device to a known good state, on exit of the daemon
    fn set_led_off_pattern(&mut self) -> Result<()>;
}

/// Enumerates all HID devices on the system and returns the first supported device that was found
pub fn enumerate_devices(api: &hidapi::HidApi) -> Result<(KeyboardDevice, Option<MouseDevice>)> {
    trace!("Enumerating all available HID devices on the system...");

    let mut found_led_dev = false;
    let mut found_ctrl_dev = false;

    let mut ctrl_device = None;
    let mut led_device = None;

    let mut found_mouse_dev = false;
    let mut mouse_device = None;

    for device in api.device_list() {
        debug!(
            "Device: {} {}, interface: {}",
            device.manufacturer_string().unwrap(),
            device.product_string().unwrap(),
            device.interface_number()
        );

        if VENDOR_IDS.iter().any(|p| *p == device.vendor_id())
            && PRODUCT_IDS.iter().any(|p| *p == device.product_id())
        {
            if device.interface_number() == roccat_vulcan::CTRL_INTERFACE {
                let path = device.path();

                let product_string = device.product_string().clone().unwrap_or_else(|| {
                    error!("Could not query device information");
                    "<unknown>"
                });

                info!("Found Control interface: {:?}: {}", path, product_string);

                found_ctrl_dev = true;
                ctrl_device = Some(device);
            } else if device.interface_number() == roccat_vulcan::LED_INTERFACE {
                let path = device.path();

                let product_string = device.product_string().clone().unwrap_or_else(|| {
                    error!("Could not query device information");
                    "<unknown>"
                });

                info!("Found LED interface: {:?}: {}", path, product_string);

                found_led_dev = true;
                led_device = Some(device);
            }
        } else if !found_mouse_dev
            && VENDOR_IDS_MICE.iter().any(|p| *p == device.vendor_id())
            && PRODUCT_IDS_MICE.iter().any(|p| *p == device.product_id())
            // TODO: Fix this
            && device.interface_number() == roccat_kone_pure_ultra::KEYBOARD_SUB_DEVICE as i32
        {
            info!(
                "Found Mouse device: {:?}: {}",
                device.path(),
                device.product_string().unwrap_or_else(|| "<unknown>"),
            );

            found_mouse_dev = true;
            mouse_device = Some(device);
        }
    }

    if !found_ctrl_dev || !found_led_dev {
        warn!("At least one required device could not be detected");

        Err(HwDeviceError::EnumerationError {})
    } else {
        // We only support the ROCCAT Vulcan 100/12x series, for now
        let keyboard_device = Arc::new(RwLock::new(Box::from(RoccatVulcan1xx::bind(
            &ctrl_device.unwrap(),
            &led_device.unwrap(),
        ))
            as Box<dyn KeyboardDeviceTrait + Send + Sync + 'static>));

        // We only support the ROCCAT Kone Pure Ultra, for now
        let mouse_device = if found_mouse_dev {
            Some(Arc::new(RwLock::new(
                Box::from(RoccatKonePureUltra::bind(&mouse_device.unwrap()))
                    as Box<dyn MouseDeviceTrait + Send + Sync + 'static>,
            )))
        } else {
            None
        };

        Ok((keyboard_device, mouse_device))
    }
}
