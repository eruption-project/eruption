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

mod roccat_vulcan;
pub use roccat_vulcan::*;

pub type HwDevice = Arc<RwLock<dyn Device + Sync + Send>>;

pub type Result<T> = std::result::Result<T, HwDeviceError>;

// Suported devices
// pub const VENDOR_STR: &str = "ROCCAT";
pub const VENDOR_ID: u16 = 0x1e7d; // ROCCAT
pub const PRODUCT_ID: [u16; 2] = [0x3098, 0x307a]; // Vulcan 100/12x series keyboards

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

/// A HID event
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum HidEvent {
    Unknown,

    // Keyboard events
    KeyDown { code: HidEventCode },
    KeyUp { code: HidEventCode },

    // Audio related
    MuteDown,
    MuteUp,
    VolumeDown,
    VolumeUp,
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

/// Represents a device like e.g. a supported keyboard
pub trait Device {
    /// Returns the USB path/ID of the device
    fn get_usb_path(&self) -> String;

    /// Opens the sub-devices, should called after `bind()`ing a driver
    fn open(&mut self, api: &hidapi::HidApi) -> Result<()>;

    /// Close the device files
    fn close_all(&mut self) -> Result<()>;

    /// Send a device specific initialization sequence to set the device
    /// to a known good state. Should be called after `open()`ing the device
    fn send_init_sequence(&mut self) -> Result<()>;

    /// Set the state of a device status LED, like e.g. Num Lock, etc...
    fn set_status_led(&self, led_kind: LedKind, on: bool) -> Result<()>;

    /// Send raw data to the control device
    fn write_data_raw(&self, buf: &[u8]) -> Result<()>;

    /// Read raw data from the control device
    fn read_data_raw(&self, size: usize) -> Result<Vec<u8>>;

    /// Get device specific information
    fn get_device_info(&self) -> Result<DeviceInfo>;

    /// Get the next HID event from the control device (blocking)
    fn get_next_event(&self) -> Result<HidEvent>;

    /// Get the next HID event from the control device (with timeout)
    fn get_next_event_timeout(&self, millis: i32) -> Result<HidEvent>;

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
pub fn enumerate_devices(api: &hidapi::HidApi) -> Result<impl Device + Sync + Send> {
    trace!("Enumerating all available HID devices on the system...");

    let mut found_led_dev = false;
    let mut found_ctrl_dev = false;

    let mut ctrl_device = None;
    let mut led_device = None;

    for device in api.device_list() {
        if device.vendor_id() == VENDOR_ID && PRODUCT_ID.iter().any(|p| *p == device.product_id()) {
            if device.interface_number() == CTRL_INTERFACE {
                let product_string = device.product_string().clone().unwrap_or_else(|| {
                    error!("Could not query device information");
                    "<unknown>"
                });
                let path = device.path();

                found_ctrl_dev = true;
                ctrl_device = Some(device);

                info!("Found Control interface: {:?}: {}", path, product_string);
            } else if device.interface_number() == LED_INTERFACE {
                let product_string = device.product_string().clone().unwrap_or_else(|| {
                    error!("Could not query device information");
                    "<unknown>"
                });
                let path = device.path();

                found_led_dev = true;
                led_device = Some(device);

                info!("Found LED interface: {:?}: {}", path, product_string);
            }
        }
        // } else if !found_mouse_dev {
        //     if let Ok(_result) = util::is_mouse_device(device.vendor_id(), device.product_id())
        //     {
        //         found_mouse_dev = true;
        //         mouse_device = Some(device.clone());

        //         info!(
        //             "Found Mouse device: {:?}: {}",
        //             device.path(),
        //             device.product_string().unwrap_or_else(|| "<unknown>")
        //         );
        //     }
        // }
    }

    if !found_ctrl_dev || !found_led_dev {
        warn!("At least one required device could not be detected");

        Err(HwDeviceError::EnumerationError {})
    } else {
        // We only support the ROCCAT Vulcan 100/12x series, for now
        let device = RoccatVulcan1xx::bind(&ctrl_device.unwrap(), &led_device.unwrap());

        Ok(device)
    }
}
