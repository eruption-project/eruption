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

use colored::*;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, fmt, fs};
use std::{num::ParseIntError, path::Path};

type Result<T> = std::result::Result<T, eyre::Error>;

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
            write!(f, "0x{:02x}, ", byte)?;
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

pub fn load_data_from_file<P: AsRef<Path>>(path: &P) -> Result<Vec<DeviceState>> {
    let data = match fs::read_to_string(&path.as_ref()) {
        Ok(data) => data,
        Err(_e) => "[]".to_string(),
    };

    let result: Vec<DeviceState> = serde_json::from_str(&data)?;

    Ok(result)
}

pub fn save_data_to_file<P: AsRef<Path>>(path: &P, data: &[DeviceState]) -> Result<()> {
    let data = serde_json::to_string(&data)?;
    fs::write(&path.as_ref(), &data)?;

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
                        println!("Changed bytes: {:?}", diff);

                        print!("{}: [", format!("0x{:02x}", ds.0).bold().on_green());
                        for (index, current_val) in ds.1.iter().enumerate() {
                            if diff.iter().any(|e| *e == index) {
                                print!(
                                    "{}=>{}, ",
                                    format!("0x{:02x}", stored_data[index]).bold().on_red(),
                                    format!("0x{:02x}", current_val).bold().on_green()
                                );
                            } else {
                                print!("0x{:02x}, ", current_val);
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
        u8::from_str_radix(src, 10)
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
                u8::from_str_radix(e, 10)
            }?;

            result.push(val);
        }
    }

    Ok(result)
}
