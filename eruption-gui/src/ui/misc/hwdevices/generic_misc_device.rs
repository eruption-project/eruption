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

use super::MiscDevice;
use crate::ui::misc::MiscError;
use gdk::prelude::GdkContextExt;
use gdk_pixbuf::Pixbuf;
use gtk::prelude::WidgetExt;

const BORDER: (f64, f64) = (16.0, 16.0);

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug)]
pub struct GenericMiscDevice {
    pub device: u64,
}

impl GenericMiscDevice {
    pub fn new(device: u64) -> Self {
        GenericMiscDevice { device }
    }
}

impl MiscDevice for GenericMiscDevice {
    fn get_device(&self) -> u64 {
        self.device
    }

    fn get_make_and_model(&self) -> (&'static str, &'static str) {
        ("Unknown", "Generic Misc Device")
    }

    fn draw(&self, da: &gtk::DrawingArea, context: &cairo::Context) -> super::Result<()> {
        let pixbuf =
            Pixbuf::from_resource("/org/eruption/eruption-gui/img/generic-misc.png").unwrap();

        let width = da.allocated_width() as f64;
        // let height = da.allocated_height() as f64;

        let scale_factor = (width / pixbuf.width() as f64) * 0.95;

        // paint the image
        context.scale(scale_factor, scale_factor);
        context.set_source_pixbuf(&pixbuf, BORDER.0, BORDER.1);
        context.paint()?;

        match crate::dbus_client::get_led_colors() {
            Ok(_led_colors) => Ok(()),

            Err(_e) => Err(MiscError::CommunicationError {}.into()),
        }
    }

    fn paint_cell(
        &self,
        _cell_index: usize,
        _color: &crate::util::RGBA,
        _cr: &cairo::Context,
        _width: f64,
        _height: f64,
        _scale_factor: f64,
    ) -> Result<()> {
        Ok(())
    }
}
