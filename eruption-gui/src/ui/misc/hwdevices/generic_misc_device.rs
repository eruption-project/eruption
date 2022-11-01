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

    Copyright (c) 2019-2022, The Eruption Development Team
*/

use super::MiscDevice;
use gdk::prelude::GdkContextExt;
use gdk_pixbuf::Pixbuf;
use gtk::prelude::WidgetExt;

const BORDER: (f64, f64) = (16.0, 16.0);

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug)]
pub struct GenericMiscDevice {
    pub device: u64,
    pub pixbuf: Pixbuf,
}

impl GenericMiscDevice {
    pub fn new(device: u64) -> Self {
        GenericMiscDevice {
            device,
            pixbuf: Pixbuf::from_resource("/org/eruption/eruption-gui/img/generic-misc.png")
                .unwrap(),
        }
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
        let pixbuf = &self.pixbuf;

        let width = da.allocated_width() as f64;
        // let height = da.allocated_height() as f64;

        let scale_factor = (width / pixbuf.width() as f64) * 0.95;

        // paint the image
        context.scale(scale_factor, scale_factor);
        context.set_source_pixbuf(pixbuf, BORDER.0, BORDER.1);
        context.paint()?;

        // let led_colors = crate::COLOR_MAP.lock();

        Ok(())
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
