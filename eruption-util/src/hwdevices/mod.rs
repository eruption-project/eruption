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

mod roccat_vulcan_1xx;
mod roccat_vulcan_pro;
mod roccat_vulcan_pro_tkl;
mod roccat_vulcan_tkl;

use evdev_rs::enums::EV_KEY;
use hidapi::{HidApi, HidDevice};
use log::{debug, error, trace};
use std::{thread, time::Duration};
use thiserror::Error;
use udev::Enumerator;

pub type HwDevice = dyn DeviceTrait + Sync + Send;

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[allow(dead_code)]
#[derive(Error, Debug)]
enum HwDeviceError {
    #[error("The device is not bound")]
    DeviceNotBound,

    #[error("The device is not opened")]
    DeviceNotOpened,

    #[error("Invalid result")]
    InvalidResult {},

    #[error("Write error")]
    WriteError {},

    #[error("Invalid status code")]
    InvalidStatusCode {},

    #[error("LED map error")]
    LedMapError {},

    #[error("The device is not supported")]
    DeviceNotSupported,

    #[error("No compatible devices found")]
    NoDevicesFound {},

    #[error("An error occurred during device enumeration")]
    EnumerationError {},

    #[error("Could not enumerate udev devices")]
    UdevError {},

    #[error("Could not open the device file")]
    DeviceOpenError {},

    #[error("Could not map an evdev event code to a key or button")]
    MappingError {},
}

#[derive(Debug, thiserror::Error)]
pub enum EvdevError {
    #[error("Could not peek evdev event")]
    EvdevEventError {},

    #[error("Could not get the name of the evdev device from udev")]
    UdevError {},

    #[error("Could not open the evdev device")]
    EvdevError {},

    #[error("Could not create a libevdev device handle")]
    EvdevHandleError {},
}

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
    // // Brightness related
    // BrightnessUp,
    // BrightnessDown,
    // SetBrightness(u8),

    // // Audio related
    // MuteDown,
    // MuteUp,
    // VolumeDown,
    // VolumeUp,
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

pub trait DeviceTrait {
    fn send_init_sequence(&self) -> Result<()>;

    fn write_data_raw(&self, buf: &[u8]) -> Result<()>;
    fn read_data_raw(&self, size: usize) -> Result<Vec<u8>>;

    fn get_next_event_timeout(&self, millis: i32) -> Result<KeyboardHidEvent>;

    fn ev_key_to_key_index(&self, key: EV_KEY) -> u8;
    fn hid_event_code_to_key_index(&self, code: &KeyboardHidEventCode) -> u8;
    fn hid_event_code_to_report(&self, code: &KeyboardHidEventCode) -> u8;

    fn get_rows_topology(&self) -> Vec<u8>;
    fn get_cols_topology(&self) -> Vec<u8>;
    fn get_neighbor_topology(&self) -> Vec<u8>;

    fn send_led_map(&self, led_map: &[RGBA]) -> Result<()>;
}

pub fn bind_device(
    hiddev: HidDevice,
    hidapi: &HidApi,
    vendor_id: u16,
    product_id: u16,
) -> Result<Box<HwDevice>> {
    hiddev.set_blocking_mode(true)?;

    match (vendor_id, product_id) {
        // Keyboard devices

        // ROCCAT Vulcan 1xx series
        (0x1e7d, 0x3098) | (0x1e7d, 0x307a) => {
            let leddev = hidapi
                .device_list()
                .find(|dev| {
                    dev.product_id() == product_id
                        && dev.vendor_id() == vendor_id
                        && dev.interface_number() == roccat_vulcan_1xx::LED_INTERFACE
                })
                .expect("Could not bind LED sub-device")
                .open_device(&hidapi)
                .expect("Could not open LED sub-device");

            Ok(Box::new(roccat_vulcan_1xx::RoccatVulcan1xx::bind(
                hiddev, leddev,
            )))
        }

        // ROCCAT Vulcan Pro series
        (0x1e7d, 0x30f7) => {
            let leddev = hidapi
                .device_list()
                .find(|dev| {
                    dev.product_id() == product_id
                        && dev.vendor_id() == vendor_id
                        && dev.interface_number() == roccat_vulcan_pro::LED_INTERFACE
                })
                .expect("Could not bind LED sub-device")
                .open_device(&hidapi)
                .expect("Could not open LED sub-device");

            Ok(Box::new(roccat_vulcan_pro::RoccatVulcanPro::bind(
                hiddev, leddev,
            )))
        }

        // ROCCAT Vulcan TKL series
        (0x1e7d, 0x2fee) => {
            let leddev = hidapi
                .device_list()
                .find(|dev| {
                    dev.product_id() == product_id
                        && dev.vendor_id() == vendor_id
                        && dev.interface_number() == roccat_vulcan_tkl::LED_INTERFACE
                })
                .expect("Could not bind LED sub-device")
                .open_device(&hidapi)
                .expect("Could not open LED sub-device");

            Ok(Box::new(roccat_vulcan_tkl::RoccatVulcanTKL::bind(
                hiddev, leddev,
            )))
        }

        // ROCCAT Vulcan Pro TKL series
        (0x1e7d, 0x311a) => {
            let leddev = hidapi
                .device_list()
                .find(|dev| {
                    dev.product_id() == product_id
                        && dev.vendor_id() == vendor_id
                        && dev.interface_number() == roccat_vulcan_pro_tkl::LED_INTERFACE
                })
                .expect("Could not bind LED sub-device")
                .open_device(&hidapi)
                .expect("Could not open LED sub-device");

            Ok(Box::new(roccat_vulcan_pro_tkl::RoccatVulcanProTKL::bind(
                hiddev, leddev,
            )))
        }

        _ => Err(HwDeviceError::DeviceNotSupported.into()),
    }
}

#[derive(Clone, Copy)]
pub enum DeviceClass {
    Unknown,
    Keyboard,
    Mouse,
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
#[allow(dead_code)]
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
#[allow(dead_code)]
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
