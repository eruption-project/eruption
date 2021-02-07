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

use crate::{dbus_client, util::RGBA};

mod roccat_vulcan_1xx;
mod roccat_vulcan_pro;
mod roccat_vulcan_pro_tkl;
mod roccat_vulcan_tkl;

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum HwDevicesError {
    #[error("The device is not supported")]
    UnsupportedDevice,
}

pub fn get_keyboard_device() -> Result<Box<dyn Keyboard>> {
    match dbus_client::get_managed_devices()?[0] {
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

        _ => Err(HwDevicesError::UnsupportedDevice {}.into()),
    }
}

pub trait Keyboard {
    fn get_make_and_model(&self) -> (&'static str, &'static str);

    /// Draw an animated keyboard with live action colors
    fn draw_keyboard(&self, _da: &gtk::DrawingArea, context: &cairo::Context);

    fn paint_key(&self, key: usize, color: &RGBA, cr: &cairo::Context, layout: &pango::Layout);
    fn get_key_defs(&self, layout: &str) -> &[KeyDef];
}

#[derive(Debug, PartialEq)]
pub struct KeyDef<'a> {
    is_dummy: bool,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    caption: Caption<'a>,
    // index: usize,
}

impl<'a> KeyDef<'a> {
    const fn new(
        x: f64,
        y: f64,
        width: f64,
        height: f64,
        caption: Caption<'a>,
        _index: usize,
    ) -> Self {
        Self {
            is_dummy: false,
            x,
            y,
            width,
            height,
            caption,
            // index, // currently only included for documentation purposes
        }
    }

    const fn dummy(_index: usize) -> Self {
        Self {
            is_dummy: true,
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            caption: Caption::simple(""),
            // index, // currently only included for documentation purposes
        }
    }
}

#[derive(Debug, PartialEq)]
struct Caption<'a> {
    text: &'a str,
    x_offset: f64,
    y_offset: f64,
}

impl<'a> Caption<'a> {
    const fn new(text: &'a str, x_offset: f64, y_offset: f64) -> Self {
        Self {
            text,
            x_offset,
            y_offset,
        }
    }

    const fn simple(text: &'a str) -> Self {
        Self {
            text,
            x_offset: 0.0,
            y_offset: 0.0,
        }
    }
}
