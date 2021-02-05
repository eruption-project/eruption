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
mod roccat_vulcan_pro_tkl;
mod roccat_vulcan_tkl;

pub type KeyboardDevice = Box<dyn Keyboard + Sync + Send>;

// pub type Result<T> = std::result::Result<T, eyre::Error>;

// #[derive(Debug, thiserror::Error)]
// pub enum HwDevicesError {
//     #[error("The device is not supported")]
//     UnsupportedDevice,
// }

pub trait Keyboard {
    fn get_num_keys(&self) -> usize;

    fn get_num_rows(&self) -> usize;
    fn get_num_cols(&self) -> usize;

    fn get_rows_topology(&self) -> &'static [u8];
}

pub fn get_keyboard_device(vid: u16, pid: u16) -> KeyboardDevice {
    match (vid, pid) {
        // ROCCAT Vulcan 1xx series
        (0x1e7d, 0x3098) | (0x1e7d, 0x307a) => {
            return Box::new(roccat_vulcan_1xx::RoccatVulcan1xx::new())
        }

        // // ROCCAT Vulcan Pro series
        // (0x1e7d, 0x30f7) => return Box::new(roccat_vulcan_pro::RoccatVulcanPro::new()),

        // ROCCAT Vulcan Pro TKL series
        (0x1e7d, 0x311a) => return Box::new(roccat_vulcan_pro_tkl::RoccatVulcanProTKL::new()),

        // ROCCAT Vulcan TKL series
        (0x1e7d, 0x2fee) => return Box::new(roccat_vulcan_tkl::RoccatVulcanTKL::new()),

        _ => return Box::new(generic_keyboard::GenericKeyboard::new()),
    }
}
