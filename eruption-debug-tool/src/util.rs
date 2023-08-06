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

use colored::*;
use eyre::eyre;
use lazy_static::lazy_static;
use parking_lot::Mutex;
use serde::{Deserialize, Serialize};
use std::ffi::OsStr;
use std::sync::Arc;
use std::{collections::HashMap, fmt, fs};
use std::{num::ParseIntError, path::Path};

use crate::constants;

type Result<T> = std::result::Result<T, eyre::Error>;

lazy_static! {
    static ref CRC8: Arc<Mutex<crc8::Crc8>> = Arc::new(Mutex::new(crc8::Crc8::create_msb(0x01)));
}

pub struct HexSlice<'a>(pub &'a [u8]);

impl<'a> HexSlice<'a> {
    pub fn new<T>(data: &'a T) -> HexSlice<'a>
    where
        T: ?Sized + AsRef<[u8]> + 'a,
    {
        HexSlice(data.as_ref())
    }
}

impl fmt::Display for HexSlice<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "0x{byte:02x}, ")?;
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeviceState {
    pub serial: String,
    pub device_name: String,
    pub data: HashMap<u8, Vec<u8>>,
}

impl DeviceState {
    pub fn new(serial: &str, device_name: &str) -> Self {
        DeviceState {
            serial: serial.to_string(),
            device_name: device_name.to_string(),
            data: HashMap::new(),
        }
    }
}

pub fn is_eruption_daemon_running() -> bool {
    let result = fs::read_to_string(constants::PID_FILE);

    // .map_err(|e| {
    //     eprintln!(
    //         "Could not determine whether the Eruption daemon is running: {}",
    //         e
    //     )
    // });

    result.is_ok()
}

pub fn load_data_from_file<P: AsRef<Path>>(path: &P) -> Result<Vec<DeviceState>> {
    let data = match fs::read_to_string(path.as_ref()) {
        Ok(data) => data,
        Err(_e) => "[]".to_string(),
    };

    let result: Vec<DeviceState> = serde_json::from_str(&data)?;

    Ok(result)
}

pub fn save_data_to_file<P: AsRef<Path>>(path: &P, data: &[DeviceState]) -> Result<()> {
    let data = serde_json::to_string(&data)?;
    fs::write(path.as_ref(), data)?;

    Ok(())
}

pub fn print_diff(current_state: &DeviceState, data: &[DeviceState]) {
    for state in data.iter().rev() {
        if current_state.serial == state.serial && current_state.device_name == state.device_name {
            for ds in state.data.iter() {
                if !current_state.data.contains_key(ds.0) {
                    // report is missing from current_state
                    println!(
                        "{}: {}\n",
                        format!("0x{:02x}", ds.0).to_string().red(),
                        format!("{}", HexSlice(ds.1)).red()
                    );
                }
            }

            for ds in current_state.data.iter() {
                if !state.data.contains_key(ds.0) {
                    // report is completely new
                    println!(
                        "{}: {}\n",
                        format!("0x{:02x}", ds.0).to_string().green(),
                        format!("{}", HexSlice(ds.1)).green()
                    );
                } else {
                    // not new, check for differences...
                    let mut diff = vec![];

                    let stored_data = state.data.get(ds.0).unwrap();
                    for (index, current_val) in ds.1.iter().enumerate() {
                        if let Some(stored_val) = stored_data.get(index) {
                            if current_val != stored_val {
                                diff.push(index);
                            }
                        } else {
                            diff.push(index);
                        }
                    }

                    // print differences
                    if !diff.is_empty() {
                        println!("Changed bytes: {diff:?}");

                        print!("{}: [", format!("0x{:02x}", ds.0).bold().on_green());
                        for (index, current_val) in ds.1.iter().enumerate() {
                            if diff.iter().any(|e| *e == index) {
                                print!(
                                    "{}=>{}, ",
                                    format!("0x{:02x}", stored_data[index]).bold().on_red(),
                                    format!("0x{current_val:02x}").bold().on_green()
                                );
                            } else {
                                print!("0x{current_val:02x}, ");
                            }
                        }
                        println!("]\n");
                    }
                }
            }

            break;
        }
    }
}

pub fn parse_report_id(src: &str) -> std::result::Result<u8, ParseIntError> {
    if let Some(stripped) = src.strip_prefix("0x") {
        u8::from_str_radix(stripped, 16)
    } else {
        src.parse()
    }
}

pub fn parse_hex_vec(src: &str) -> Result<Vec<u8>> {
    let mut result = vec![];

    let src = src.trim_matches('[').trim_end_matches(']');

    for e in src.split(',') {
        let e = e.trim();

        if !e.is_empty() && !e.starts_with(']') {
            let val = if let Some(stripped) = e.strip_prefix("0x") {
                u8::from_str_radix(stripped, 16)
            } else {
                e.parse()
            }?;

            result.push(val);
        }
    }

    Ok(result)
}

pub fn crc8_slow_with_poly(data: &[u8], init: u8, poly: u8) -> u8 {
    // TODO: avoid rebuilding of lookup table on each call

    crc8::Crc8::create_msb(poly).calc(data, data.len() as i32, init)
}

pub fn find_crc8_from_params(sum: u8, buf: &[u8], p: &[(u8, u8)]) -> Vec<(u8, u8)> {
    let mut result = Vec::new();

    for (init, poly) in p {
        let crc8 = crc8_slow_with_poly(buf, *init, *poly);

        if crc8 == sum {
            result.push((*init, *poly));
        }
    }

    result
}

/// For some devices, such as the Vulcan 1xx, after sending the report to update the LEDs, the device's evdev LED interface
/// goes crazy and starts spewing out KEY_UNKNOWN events.  This is ignored by X and Wayland, but is interpreted as real key
/// stroke inputs on virtual consoles.  As best as I can tell, this behavior is a bug somewhere in udev/evdev/hidraw.  As a
/// workaround, toggling the "inhibited" attribute back and forth as a privileged user silences these events for as long as
/// the device is plugged in.  Not all Roccat devices require this workaround, headphones don't, but I don't know which all
/// do and which don't.  Note that this workaround can also be applied manually by writing to the "inhibited" file found at
/// path "/sys/class/input/eventX/inhibited", where the X in "eventX" is the udev number associated with the LED interface.
pub fn udev_inhibited_workaround(vendor_id: u16, product_id: u16, interface_num: i32) -> Result<()> {
    let interface_num_str = format!("{interface_num:02}");
    let interface_num_osstr = OsStr::new(&interface_num_str);

    let mut enumerator = udev::Enumerator::new()?;
    enumerator.match_subsystem("input")?;
    enumerator.match_property("ID_VENDOR_ID", format!("{vendor_id:04x}"))?;
    enumerator.match_property("ID_MODEL_ID", format!("{product_id:04x}"))?;
    enumerator.match_property("ID_USB_INTERFACE_NUM", &interface_num_str)?;
    enumerator.match_attribute("inhibited", "0")?;

    enumerator
        .scan_devices()?
        .find(|dev| {
            // For some reason, the above match_property() still brings back devices with different interface_nums, so filter again.
            dev.property_value("ID_USB_INTERFACE_NUM")
                .map_or(false, |value| value == interface_num_osstr)
        })
        .map_or_else(
            || Err(eyre!("Udev device not found.")),
            |mut dev|
            {
                // Toggling the value on and off is enough to quiet spurious events.
                dev.set_attribute_value("inhibited", "1")?;
                dev.set_attribute_value("inhibited", "0")?;
                Ok(())
            })
}
