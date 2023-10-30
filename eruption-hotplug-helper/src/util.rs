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

use std::path::PathBuf;

use eruption_sdk::hardware::HotplugInfo;
use eyre::Error;

pub(crate) fn get_hotplug_info(devpath: Option<&PathBuf>) -> Result<HotplugInfo, Error> {
    // let mut found = false;

    // TODO: Fully implement this
    let result = HotplugInfo {
        devpath: devpath.cloned(),
        usb_vid: 0x0000,
        usb_pid: 0x0000,
    };

    /* if let Some(devpath) = devpath {
        let mut enumerator = udev::Enumerator::new()?;

        for device in enumerator.scan_devices().unwrap() {
            tracing::warn!("{:?}", device.devpath());

            if device.devpath() == PathBuf::from("/").join(devpath) {
                tracing::warn!("Found info for device: {:?}", device.devnode());

                found = true;

                result.devpath = Some(Path::new("/sys/").join(devpath).to_owned());

                result.usb_vid = device
                    .property_value("ID_VENDOR_ID")
                    .unwrap_or(OsStr::new("0x0000"))
                    .to_string_lossy()
                    .parse::<u16>()?;

                result.usb_pid = device
                    .property_value("ID_MODEL_ID")
                    .unwrap_or(OsStr::new("0x0000"))
                    .to_string_lossy()
                    .parse::<u16>()?;

                break;
            }
        }
    }

    if !found {
        tracing::error!("Could not find the attributes of the plugged device");
    } */

    Ok(result)
}
