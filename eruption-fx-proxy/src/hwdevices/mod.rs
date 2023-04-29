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

use dyn_clonable::clonable;

mod corsair_strafe;
mod generic_keyboard;
mod roccat_magma;
mod roccat_vulcan_1xx;
mod roccat_vulcan_pro;
mod roccat_vulcan_pro_tkl;
mod roccat_vulcan_tkl;
mod wooting_two_he_arm;

pub type KeyboardDevice = Box<dyn Keyboard + Sync + Send>;

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum HwDevicesError {
    // #[error("The device is not supported")]
    // UnsupportedDevice,
}

#[clonable]
pub trait Keyboard: Clone {
    fn get_num_keys(&self) -> usize;

    fn get_num_rows(&self) -> usize;
    fn get_num_cols(&self) -> usize;

    fn get_rows_topology(&self) -> &'static [u8];
}

pub fn get_keyboard_device(vid: u16, pid: u16) -> Result<KeyboardDevice> {
    match (vid, pid) {
        // Wooting

        // Wooting Two HE (ARM)
        (0x31e3, 0x1230) => Ok(Box::new(wooting_two_he_arm::WootingTwoHeArm::new())),

        // Roccat

        // ROCCAT Vulcan 1xx series
        (0x1e7d, 0x3098) | (0x1e7d, 0x307a) => {
            Ok(Box::new(roccat_vulcan_1xx::RoccatVulcan1xx::new()))
        }

        // ROCCAT Vulcan Pro series
        (0x1e7d, 0x30f7) => Ok(Box::new(roccat_vulcan_pro::RoccatVulcanPro::new())),

        // ROCCAT Vulcan Pro TKL series
        (0x1e7d, 0x311a) => Ok(Box::new(roccat_vulcan_pro_tkl::RoccatVulcanProTKL::new())),

        // ROCCAT Vulcan TKL series
        (0x1e7d, 0x2fee) => Ok(Box::new(roccat_vulcan_tkl::RoccatVulcanTKL::new())),

        // ROCCAT Magma
        (0x1e7d, 0x3124) => Ok(Box::new(roccat_magma::RoccatMagma::new())),

        // Corsair

        // Corsair STRAFE Gaming Keyboard
        (0x1b1c, 0x1b15) => Ok(Box::new(corsair_strafe::CorsairStrafe::new())),

        _ => {
            tracing::warn!("Unknown keyboard model specified, assuming generic model");

            Ok(Box::new(generic_keyboard::GenericKeyboard::new()))
        }
    }
}
