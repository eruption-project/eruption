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
use crate::ui::misc::hwdevices::Rectangle;
use gdk_pixbuf::Pixbuf;
use gtk::prelude::WidgetExt;
use palette::{FromColor, Hsva, Shade, Srgba};

const BORDER: (f64, f64) = (16.0, 16.0);

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug)]
pub struct RoccatAimoPad {
    pub device: u64,
    pub pixbuf: Pixbuf,
}

impl RoccatAimoPad {
    pub fn new(device: u64) -> Self {
        RoccatAimoPad {
            device,
            pixbuf: Pixbuf::from_resource("/org/eruption/eruption-gui/img/generic-misc.png")
                .unwrap(),
        }
    }
}

impl MiscDevice for RoccatAimoPad {
    fn get_device(&self) -> u64 {
        self.device
    }

    fn get_make_and_model(&self) -> (&'static str, &'static str) {
        ("ROCCAT", "Aimo Pad Wide")
    }

    fn draw(&self, da: &gtk::DrawingArea, context: &cairo::Context) -> super::Result<()> {
        let width = da.allocated_width() as f64;
        let height = da.allocated_height() as f64;

        let scale_factor = 1.0;

        // let pixbuf = &self.pixbuf;

        // paint the schematic drawing
        // context.scale(scale_factor, scale_factor);
        // context.set_source_pixbuf(&pixbuf, 0.0, 0.0);
        // context.paint()?;

        let led_colors = crate::COLOR_MAP.lock();

        // paint all cells in the "mouse zone" of the canvas
        for i in [1, 5] {
            self.paint_cell(i, &led_colors[i], context, width, height, scale_factor)?;
        }

        Ok(())
    }

    fn paint_cell(
        &self,
        cell_index: usize,
        color: &crate::util::RGBA,
        cr: &cairo::Context,
        width: f64,
        height: f64,
        scale_factor: f64,
    ) -> Result<()> {
        let factor = ((100.0 - crate::STATE.read().current_brightness.unwrap_or_else(|| 0) as f64)
            / 100.0)
            * 0.15;

        match cell_index {
            1 => {
                let cell_def = Rectangle {
                    x: ((width + 170.0 + BORDER.0 * scale_factor) / 4.0) * scale_factor,
                    y: ((height / 2.0) + BORDER.1 * scale_factor) - (5.0 * scale_factor),
                    width: 65.0 * scale_factor,
                    height: 80.0 * scale_factor,
                };

                // post-process color
                let color = Srgba::new(
                    color.r as f64 / 255.0,
                    color.g as f64 / 255.0,
                    color.b as f64 / 255.0,
                    color.a as f64 / 255.0,
                );

                // saturate and lighten color somewhat
                let color = Hsva::from_color(color);
                let color = Srgba::from_color(
                    color
                        // .saturate(factor)
                        .lighten(factor),
                )
                .into_components();

                cr.set_source_rgba(color.0, color.1, color.2, 1.0 - color.3);
                cr.rectangle(cell_def.x, cell_def.y, cell_def.width, cell_def.height);
                cr.fill()?;
            }

            5 => {
                let cell_def = Rectangle {
                    x: ((width + 800.0 + BORDER.0 * scale_factor) / 4.0) * scale_factor,
                    y: ((height / 2.0) + BORDER.1 * scale_factor) - (5.0 * scale_factor),
                    width: 65.0 * scale_factor,
                    height: 80.0 * scale_factor,
                };

                // post-process color
                let color = Srgba::new(
                    color.r as f64 / 255.0,
                    color.g as f64 / 255.0,
                    color.b as f64 / 255.0,
                    color.a as f64 / 255.0,
                );

                // saturate and lighten color somewhat
                let color = Hsva::from_color(color);
                let color = Srgba::from_color(
                    color
                        // .saturate(factor)
                        .lighten(factor),
                )
                .into_components();

                cr.set_source_rgba(color.0, color.1, color.2, 1.0 - color.3);
                cr.rectangle(cell_def.x, cell_def.y, cell_def.width, cell_def.height);
                cr.fill()?;
            }

            _ => { /* do nothing  */ }
        }

        Ok(())
    }
}
