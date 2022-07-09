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

use evdev_rs::enums::{EventCode, EV_KEY};

pub mod corsair_strafe;
pub mod generic_keyboard;
pub mod roccat_magma;
pub mod roccat_vulcan_1xx;
pub mod roccat_vulcan_pro;
pub mod roccat_vulcan_pro_tkl;
pub mod roccat_vulcan_tkl;

/// Map an [EV_KEY] event to a key index
pub fn ev_key_to_index(ev_key: &EV_KEY, (usb_vid, usb_pid): (u16, u16)) -> Option<usize> {
    /*
        DeviceInfo { make: "ROCCAT", model: "Vulcan 100/12x",       usb_vid: 0x1e7d, usb_pid: 0x3098, },
        DeviceInfo { make: "ROCCAT", model: "Vulcan 100/12x",       usb_vid: 0x1e7d, usb_pid: 0x307a, },

        DeviceInfo { make: "ROCCAT", model: "Vulcan Pro",           usb_vid: 0x1e7d, usb_pid: 0x30f7, },

        DeviceInfo { make: "ROCCAT", model: "Vulcan TKL",           usb_vid: 0x1e7d, usb_pid: 0x2fee, },

        DeviceInfo { make: "ROCCAT", model: "Vulcan Pro TKL",       usb_vid: 0x1e7d, usb_pid: 0x311a, },

        DeviceInfo { make: "ROCCAT", model: "Magma",                usb_vid: 0x1e7d, usb_pid: 0x3124, },

        DeviceInfo { make: "Corsair", model: "Corsair STRAFE Gaming Keyboard", usb_vid: 0x1b1c, usb_pid: 0x1b15, },
    */

    let result = match (usb_vid, usb_pid) {
        (0x1e7d, 0x3098) | (0x1e7d, 0x307a) => {
            Some(roccat_vulcan_1xx::EV_TO_INDEX_ISO[*ev_key as usize] as usize)
        }

        (0x1e7d, 0x30f7) => Some(roccat_vulcan_pro::EV_TO_INDEX_ISO[*ev_key as usize] as usize),

        (0x1e7d, 0x2fee) => Some(roccat_vulcan_tkl::EV_TO_INDEX_ISO[*ev_key as usize] as usize),

        (0x1e7d, 0x311a) => Some(roccat_vulcan_pro_tkl::EV_TO_INDEX_ISO[*ev_key as usize] as usize),

        (0x1e7d, 0x3124) => Some(roccat_magma::EV_TO_INDEX_ISO[*ev_key as usize] as usize),

        (0x1e7d, 0x1b15) => Some(corsair_strafe::EV_TO_INDEX_ISO[*ev_key as usize] as usize),

        _ => None,
    };

    if let Some(result) = result {
        if result == 0xff {
            // filter out invalid results
            None
        } else {
            Some(result)
        }
    } else {
        None
    }
}

/// Map an index to a [EV_KEY] event
pub fn index_to_ev_key(index: usize, (usb_vid, usb_pid): (u16, u16)) -> Option<EV_KEY> {
    for event in EventCode::EV_KEY(EV_KEY::KEY_RESERVED).iter() {
        if let EventCode::EV_KEY(event) = event {
            if let Some(result) = ev_key_to_index(&event, (usb_vid, usb_pid)) {
                if result == index - 2 {
                    return Some(event);
                }
            }
        }
    }

    None
}
