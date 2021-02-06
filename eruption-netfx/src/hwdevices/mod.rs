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

mod generic_keyboard;
mod roccat_vulcan_1xx;
mod roccat_vulcan_pro;
mod roccat_vulcan_pro_tkl;
mod roccat_vulcan_tkl;

pub type KeyboardDevice = Box<dyn Keyboard + Sync + Send>;

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum HwDevicesError {
    // #[error("The device is not supported")]
    // UnsupportedDevice,
    #[error("Invalid hex number format")]
    InvalidHexFormat,
}

pub trait Keyboard {
    fn get_num_keys(&self) -> usize;

    fn get_num_rows(&self) -> usize;
    fn get_num_cols(&self) -> usize;

    fn get_rows_topology(&self) -> &'static [u8];
}

pub fn get_keyboard_device(model: &Option<String>) -> Result<KeyboardDevice> {
    match model {
        Some(model) => {
            if model.contains(":") {
                let spl: Vec<_> = model.split(":").collect();

                let vid = u16::from_str_radix(spl[0], 16)
                    .map_err(|_op| HwDevicesError::InvalidHexFormat {})?;

                let pid = u16::from_str_radix(spl[1], 16)
                    .map_err(|_op| HwDevicesError::InvalidHexFormat {})?;

                match (vid, pid) {
                    // ROCCAT Vulcan 1xx series
                    (0x1e7d, 0x3098) | (0x1e7d, 0x307a) => {
                        Ok(Box::new(roccat_vulcan_1xx::RoccatVulcan1xx::new()))
                    }

                    // ROCCAT Vulcan Pro series
                    (0x1e7d, 0x30f7) => Ok(Box::new(roccat_vulcan_pro::RoccatVulcanPro::new())),

                    // ROCCAT Vulcan Pro TKL series
                    (0x1e7d, 0x311a) => {
                        Ok(Box::new(roccat_vulcan_pro_tkl::RoccatVulcanProTKL::new()))
                    }

                    // ROCCAT Vulcan TKL series
                    (0x1e7d, 0x2fee) => Ok(Box::new(roccat_vulcan_tkl::RoccatVulcanTKL::new())),

                    _ => {
                        log::warn!("Unknown keyboard model specified, assuming generic model");

                        Ok(Box::new(generic_keyboard::GenericKeyboard::new()))
                    }
                }
            } else {
                match model.as_str() {
                    // ROCCAT Vulcan 1xx series
                    "ROCCAT Vulcan 1xx" | "ROCCAT Vulcan 100" | "ROCCAT Vulcan 110"
                    | "ROCCAT Vulcan 120" | "ROCCAT Vulcan 121" | "ROCCAT Vulcan 122" => {
                        Ok(Box::new(roccat_vulcan_1xx::RoccatVulcan1xx::new()))
                    }

                    // // ROCCAT Vulcan Pro series
                    // (0x1e7d, 0x30f7) => return Box::new(roccat_vulcan_pro::RoccatVulcanPro::new()),

                    // ROCCAT Vulcan Pro TKL series
                    "ROCCAT Vulcan Pro TKL" => {
                        Ok(Box::new(roccat_vulcan_pro_tkl::RoccatVulcanProTKL::new()))
                    }

                    // ROCCAT Vulcan TKL series
                    "ROCCAT Vulcan TKL" => Ok(Box::new(roccat_vulcan_tkl::RoccatVulcanTKL::new())),

                    _ => {
                        log::warn!("Unknown keyboard model specified, assuming generic model");

                        Ok(Box::new(generic_keyboard::GenericKeyboard::new()))
                    }
                }
            }
        }

        None => {
            log::warn!("No keyboard model specified, assuming generic model");

            Ok(Box::new(generic_keyboard::GenericKeyboard::new()))
        }
    }
}
