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

use super::Keyboard;
use super::{Caption, KeyDef};
use crate::constants;
use crate::util::RGBA;
use gdk::prelude::GdkContextExt;
use gdk_pixbuf::Pixbuf;
use gtk::prelude::WidgetExt;
use palette::{FromColor, Hsva, Lighten, LinSrgba};
use std::cell::RefCell;

const BORDER: (f64, f64) = (0.0, 0.0);

pub type Result<T> = std::result::Result<T, eyre::Error>;

thread_local! {
    // Pango font description, used to render the captions on the visual representation of keyboard
    static FONT_DESC: RefCell<pango::FontDescription> = RefCell::new(pango::FontDescription::from_string("sans-serif demibold 6"));
}

#[derive(Debug)]
pub struct RoccatVulcanProTKL {
    pub device: u64,
    pub pixbuf: Pixbuf,
}

impl RoccatVulcanProTKL {
    pub fn new(device: u64) -> Self {
        RoccatVulcanProTKL {
            device,
            pixbuf: Pixbuf::from_resource(
                "/org/eruption/eruption-gui-gtk3/img/roccat-vulcan-pro-tkl.png",
            )
            .unwrap(),
        }
    }
}

impl Keyboard for RoccatVulcanProTKL {
    fn get_device(&self) -> u64 {
        self.device
    }

    fn get_make_and_model(&self) -> (&'static str, &'static str) {
        ("ROCCAT", "Vulcan Pro TKL")
    }

    fn draw_keyboard(&self, da: &gtk::DrawingArea, context: &cairo::Context) -> super::Result<()> {
        let pixbuf = &self.pixbuf;

        let width = da.allocated_width() as f64;
        // let height = da.allocated_height() as f64;

        let scale_factor = (width / pixbuf.width() as f64) * 0.95;

        // paint the image
        context.scale(scale_factor, scale_factor);
        context.set_source_pixbuf(pixbuf, BORDER.0, BORDER.1);
        context.paint()?;

        let led_colors = crate::CANVAS.read();

        let layout = pangocairo::create_layout(context);
        FONT_DESC.with(|f| -> Result<()> {
            let desc = f.borrow();
            layout.set_font_description(Some(&desc));

            // paint all keys
            for i in 0..96 {
                self.paint_key(i + 1, &led_colors[index_to_canvas(i)], context, &layout)?;
            }

            Ok(())
        })?;

        // paint all other elements

        // paint the mute button
        const MUTE_BUTTON_INDEX: usize = 92;

        let color = (
            (led_colors[MUTE_BUTTON_INDEX].r as f64 / 255.0),
            (led_colors[MUTE_BUTTON_INDEX].g as f64 / 255.0),
            (led_colors[MUTE_BUTTON_INDEX].b as f64 / 255.0),
            0.0,
        );

        let black = (0.0, 0.0, 0.0, 0.0);

        rounded_rectangle(context, 537.0, 44.0, 20.0, 7.0, 2.0, &black, &color)?;

        Ok(())
    }

    /// Paint a key on the keyboard widget
    fn paint_key(
        &self,
        key: usize,
        color: &RGBA,
        cr: &cairo::Context,
        layout: &pango::Layout,
    ) -> Result<()> {
        let key_def = &self.get_key_defs("generic")[key];

        if !key_def.is_dummy {
            // compute scaling factor
            // let factor =
            //     ((100.0 - crate::STATE.read().current_brightness.unwrap_or(0) as f64) / 100.0) * 0.15;

            // post-process color
            let source_color = LinSrgba::new(
                color.r as f64 / 255.0,
                color.g as f64 / 255.0,
                color.b as f64 / 255.0,
                0.0,
            );

            // saturate and lighten color somewhat to use as the border color
            let border_color = Hsva::from_color(source_color);
            let border_color = LinSrgba::from_color(
                border_color // .saturate(0.75)
                    .lighten(0.4),
            )
            .into_components();

            // saturate and darken color somewhat to use as the key color
            let key_color = Hsva::from_color(source_color);
            let key_color = LinSrgba::from_color(
                key_color, // .saturate(0.75)
                          // .darken(0.15),
            )
            .into_components();

            rounded_rectangle(
                cr,
                key_def.x + BORDER.0 + 2.0,
                key_def.y + BORDER.1 + 2.0,
                key_def.width + 1.0 - 2.0,
                key_def.height + 1.0 - 2.0,
                500.0,
                &border_color,
                &key_color,
            )?;

            // draw caption
            cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
            cr.move_to(
                BORDER.0 + 7.0 + key_def.x + key_def.caption.x_offset + 2.5,
                BORDER.1
                    + 23.0
                    + ((key_def.y + key_def.caption.y_offset) - (key_def.height / 2.0))
                    + 2.0,
            );

            layout.set_text(key_def.caption.text);

            pangocairo::show_layout(cr, layout);
        }

        Ok(())
    }

    /// Returns a slice of `KeyDef`s representing the currently selected keyboard layout
    fn get_key_defs(&self, layout: &str) -> &[KeyDef] {
        match layout.to_lowercase().as_str() {
            "generic" | "de_de" => KEY_DEFS_GENERIC_QWERTZ,
            "en_us" | "en_uk" => KEY_DEFS_GENERIC_QWERTY,

            _ => KEY_DEFS_GENERIC_QWERTZ,
        }
    }
}

fn rounded_rectangle(
    cr: &cairo::Context,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    radius: f64,
    color: &(f64, f64, f64, f64),
    color2: &(f64, f64, f64, f64),
) -> Result<()> {
    let aspect = 1.0; // aspect ratio
    let corner_radius = height / radius; // corner curvature radius

    let radius = corner_radius / aspect;
    let degrees = std::f64::consts::PI / 180.0;

    cr.new_sub_path();
    cr.arc(
        x + width - radius,
        y + radius,
        radius,
        -90.0 * degrees,
        0.0 * degrees,
    );
    cr.arc(
        x + width - radius,
        y + height - radius,
        radius,
        0.0 * degrees,
        90.0 * degrees,
    );
    cr.arc(
        x + radius,
        y + height - radius,
        radius,
        90.0 * degrees,
        180.0 * degrees,
    );
    cr.arc(
        x + radius,
        y + radius,
        radius,
        180.0 * degrees,
        270.0 * degrees,
    );
    cr.close_path();

    cr.set_source_rgba(color2.0, color2.1, color2.2, 1.0 /* - color2.3 */);
    cr.fill_preserve()?;

    cr.set_source_rgba(color.0, color.1, color.2, 1.0 /* - color.3 */);
    cr.set_line_width(1.85);
    cr.stroke()?;

    Ok(())
}

// Key definitions for a generic keyboard with QWERTZ (de_DE) Layout
#[rustfmt::skip]
const KEY_DEFS_GENERIC_QWERTZ: &[KeyDef] = &[
    KeyDef::dummy(0), // filler

    // column 1
    KeyDef::new(15.0, 170.0, 66.0, 32.0, Caption::simple("SHIFT"), 1), // SHIFT
    KeyDef::new(15.0, 205.0, 50.0, 32.0, Caption::simple("CTRL"), 2),  // CTRL
    KeyDef::new(15.0, 13.0, 32.0, 32.0, Caption::simple("ESC"), 3),    // ESC
    KeyDef::new(15.0, 66.0, 32.0, 32.0, Caption::simple("^"), 4),      // GRAVE_ACCENT
    KeyDef::new(15.0, 100.0, 48.0, 32.0, Caption::simple("TAB"), 5),   // TAB
    KeyDef::new(15.0, 135.0, 56.0, 32.0, Caption::simple("CAPS LCK"), 6), // CAPS_LOCK

    // column 2
    KeyDef::new(84.0, 170.0, 32.0, 32.0, Caption::simple("<"), 7), // <
    KeyDef::new(67.0, 205.0, 38.0, 32.0, Caption::simple("WIN"), 8), // SUPER
    KeyDef::new(49.0, 66.0, 32.0, 32.0, Caption::simple("1"), 9),  // 1
    KeyDef::new(66.0, 100.0, 32.0, 32.0, Caption::simple("Q"), 10), // Q
    KeyDef::new(74.0, 135.0, 32.0, 32.0, Caption::simple("A"), 11), // A

    // column 3
    KeyDef::new(118.0, 170.0, 32.0, 32.0, Caption::simple("Y"), 12), // Y
    KeyDef::new(107.0, 205.0, 32.0, 32.0, Caption::simple("ALT"), 13), // ALT
    KeyDef::new(78.0, 13.0, 32.0, 32.0, Caption::simple("F1"), 14), // F1
    KeyDef::new(83.0, 66.0, 32.0, 32.0, Caption::simple("2"), 15),  // 2
    KeyDef::new(100.0, 100.0, 32.0, 32.0, Caption::simple("W"), 16), // W
    KeyDef::new(108.0, 135.0, 32.0, 32.0, Caption::simple("S"), 17), // S

    // column 4
    KeyDef::new(152.0, 170.0, 32.0, 32.0, Caption::simple("X"), 18), // X
    KeyDef::dummy(19),                                               // filler
    KeyDef::dummy(20),                                               // filler
    KeyDef::new(112.0, 13.0, 32.0, 32.0, Caption::simple("F2"), 21), // F2
    KeyDef::new(117.0, 66.0, 32.0, 32.0, Caption::simple("3"), 22),  // 3
    KeyDef::new(134.0, 100.0, 32.0, 32.0, Caption::simple("E"), 23), // E
    KeyDef::new(142.0, 135.0, 32.0, 32.0, Caption::simple("D"), 24), // D

    // column 5
    KeyDef::new(186.0, 170.0, 32.0, 32.0, Caption::simple("C"), 25), // C
    KeyDef::new(146.0, 13.0, 32.0, 32.0, Caption::simple("F3"), 26), // F3
    KeyDef::new(151.0, 66.0, 32.0, 32.0, Caption::simple("4"), 27),  // 4
    KeyDef::new(168.0, 100.0, 32.0, 32.0, Caption::simple("R"), 28), // R
    KeyDef::new(176.0, 135.0, 32.0, 32.0, Caption::simple("F"), 29), // F

    // column 6
    KeyDef::new(220.0, 170.0, 32.0, 32.0, Caption::simple("V"), 30), // V
    KeyDef::new(180.0, 13.0, 32.0, 32.0, Caption::simple("F4"), 31), // F4
    KeyDef::new(185.0, 66.0, 32.0, 32.0, Caption::simple("5"), 32),  // 5
    KeyDef::new(202.0, 100.0, 32.0, 32.0, Caption::simple("T"), 33), // T
    KeyDef::new(210.0, 135.0, 32.0, 32.0, Caption::simple("G"), 34), // G

    // column 7
    KeyDef::new(254.0, 170.0, 32.0, 32.0, Caption::simple("B"), 35), // B
    KeyDef::new(141.0, 205.0, 148.0, 32.0, Caption::simple("SPACE BAR"), 36), // SPACE
    KeyDef::new(219.0, 66.0, 32.0, 32.0, Caption::simple("6"), 37), // 6
    KeyDef::new(236.0, 100.0, 32.0, 32.0, Caption::simple("Z"), 38), // Z
    KeyDef::new(244.0, 135.0, 32.0, 32.0, Caption::simple("H"), 39), // H

    // column 8
    KeyDef::new(288.0, 170.0, 32.0, 32.0, Caption::simple("N"), 40), // N
    KeyDef::new(225.0, 13.0, 32.0, 32.0, Caption::simple("F5"), 41), // F5
    KeyDef::new(253.0, 66.0, 32.0, 32.0, Caption::simple("7"), 42), // 7
    KeyDef::new(270.0, 100.0, 32.0, 32.0, Caption::simple("U"), 43), // U
    KeyDef::new(278.0, 135.0, 32.0, 32.0, Caption::simple("J"), 44), // J

    // column 9
    KeyDef::new(322.0, 170.0, 32.0, 32.0, Caption::simple("M"), 45), // M
    KeyDef::dummy(46),                                              // filler
    KeyDef::dummy(47),                                              // filler
    KeyDef::new(259.0, 13.0, 32.0, 32.0, Caption::simple("F6"), 48), // F6
    KeyDef::new(287.0, 66.0, 32.0, 32.0, Caption::simple("8"), 49), // 8
    KeyDef::new(304.0, 100.0, 32.0, 32.0, Caption::simple("I"), 50), // I
    KeyDef::new(312.0, 135.0, 32.0, 32.0, Caption::simple("K"), 51), // K

    // column 10
    KeyDef::new(356.0, 170.0, 32.0, 32.0, Caption::simple(","), 52), // ,
    KeyDef::dummy(53),                                              // filler
    KeyDef::new(293.0, 13.0, 32.0, 32.0, Caption::simple("F7"), 54), // F7
    KeyDef::new(321.0, 66.0, 32.0, 32.0, Caption::simple("9"), 55), // 9
    KeyDef::new(338.0, 100.0, 32.0, 32.0, Caption::simple("O"), 56), // O
    KeyDef::new(346.0, 135.0, 32.0, 32.0, Caption::simple("L"), 57), // L

    // column 11
    KeyDef::new(390.0, 170.0, 32.0, 32.0, Caption::simple("."), 58), // .
    KeyDef::new(292.0, 205.0, 50.0, 32.0, Caption::simple("ALT GR"), 59), // ALT GR
    KeyDef::new(327.0, 13.0, 32.0, 32.0, Caption::simple("F8"), 60), // F8
    KeyDef::new(355.0, 66.0, 32.0, 32.0, Caption::simple("0"), 61), // 0
    KeyDef::new(372.0, 100.0, 32.0, 32.0, Caption::simple("P"), 62), // P
    KeyDef::new(380.0, 135.0, 32.0, 32.0, Caption::simple("Ö"), 63), // Ö

    // column 12
    KeyDef::new(424.0, 170.0, 32.0, 32.0, Caption::simple("-"), 64), // -
    KeyDef::new(346.0, 205.0, 50.0, 32.0, Caption::simple("FN"), 65), // FN
    KeyDef::new(371.0, 13.0, 32.0, 32.0, Caption::simple("F9"), 66), // F9
    KeyDef::new(389.0, 66.0, 32.0, 32.0, Caption::simple("ß"), 67), // ß
    KeyDef::new(406.0, 100.0, 32.0, 32.0, Caption::simple("Ü"), 68), // Ü
    KeyDef::new(414.0, 135.0, 32.0, 32.0, Caption::simple("Ä"), 69), // Ä

    //
    KeyDef::dummy(70),                                               // filler

    // column 13
    KeyDef::new(399.0, 205.0, 50.0, 32.0, Caption::simple("MENU"), 71), // MENU
    KeyDef::new(405.0, 13.0, 32.0, 32.0, Caption::simple("F10"), 72), // F10
    KeyDef::new(424.0, 66.0, 32.0, 32.0, Caption::simple("´"), 73), // ´
    KeyDef::new(441.0, 100.0, 32.0, 32.0, Caption::simple("+"), 74), // +
    KeyDef::new(448.0, 135.0, 26.0, 32.0, Caption::simple("#"),75), // #
    KeyDef::new(459.0, 170.0, 49.0, 32.0, Caption::simple("SHIFT"), 76), // SHIFT

    // column 14
    KeyDef::new(452.0, 205.0, 56.0, 32.0, Caption::simple("CTRL"), 77), // CTRL
    KeyDef::new(439.0, 13.0, 32.0, 32.0, Caption::simple("F11"), 78), // F11
    //
    KeyDef::dummy(79),                                               // filler
    KeyDef::new(473.0, 13.0, 32.0, 32.0, Caption::simple("F12"), 80), // F12
    KeyDef::new(459.0, 66.0, 50.0, 32.0, Caption::simple("BKSPC"), 81), // BACKSPACE

    //
    KeyDef::dummy(82),                                               // filler

    // column 15
    KeyDef::new(
        476.0,
        100.0,
        32.0,
        68.0,
        Caption::new("RETRN", -5.0, 24.0),
        83,
    ), // RETURN

    KeyDef::dummy(84),                                               // filler
    KeyDef::new(515.0, 66.0, 32.0, 32.0, Caption::simple("INS"), 85), // INSERT
    KeyDef::new(515.0, 100.0, 32.0, 32.0, Caption::simple("DEL"), 86), // DELETE
    KeyDef::new(515.0, 205.0, 32.0, 32.0, Caption::simple("←"), 87), // LEFT

    KeyDef::dummy(88),                                               // filler

    // column 16
    KeyDef::new(
        549.0,
        66.0,
        32.0,
        32.0,
        Caption::new("HOME", -4.0, 0.0),
        89,
    ), // HOME
    KeyDef::new(549.0, 100.0, 32.0, 32.0, Caption::simple("END"),90), // END
    KeyDef::new(549.0, 170.0, 32.0, 32.0, Caption::simple("↑"), 91), // UP
    KeyDef::new(549.0, 205.0, 32.0, 32.0, Caption::simple("↓"), 92), // DOWN

    KeyDef::dummy(93),                                               // filler

    // column 17
    KeyDef::new(
        583.0,
        66.0,
        32.0,
        32.0,
        Caption::new("PGUP", -4.0, 0.0),
        94,
    ), // PAGE UP
    KeyDef::new(
        583.0,
        100.0,
        32.0,
        32.0,
        Caption::new("PGDN", -4.0, 0.0),
        95,
    ), // PAGE DOWN
    KeyDef::new(583.0, 205.0, 32.0, 32.0, Caption::simple("→"), 96), // RIGHT
];

// Key definitions for a generic keyboard with QWERTY (en_US) Layout
// TODO: Implement this
#[rustfmt::skip]
const KEY_DEFS_GENERIC_QWERTY: &[KeyDef] = &[
    KeyDef::dummy(0), // filler

    // column 1
    KeyDef::new(15.0, 170.0, 66.0, 32.0, Caption::simple("SHIFT"), 1), // SHIFT
    KeyDef::new(15.0, 205.0, 50.0, 32.0, Caption::simple("CTRL"), 2),  // CTRL
    KeyDef::new(15.0, 13.0, 32.0, 32.0, Caption::simple("ESC"), 3),    // ESC
    KeyDef::new(15.0, 66.0, 32.0, 32.0, Caption::simple("^"), 4),      // GRAVE_ACCENT
    KeyDef::new(15.0, 100.0, 48.0, 32.0, Caption::simple("TAB"), 5),   // TAB
    KeyDef::new(15.0, 135.0, 56.0, 32.0, Caption::simple("CAPS LCK"), 6), // CAPS_LOCK

    // column 2
    KeyDef::new(84.0, 170.0, 32.0, 32.0, Caption::simple("<"), 7), // <
    KeyDef::new(67.0, 205.0, 38.0, 32.0, Caption::simple("WIN"), 8), // SUPER
    KeyDef::new(49.0, 66.0, 32.0, 32.0, Caption::simple("1"), 9),  // 1
    KeyDef::new(66.0, 100.0, 32.0, 32.0, Caption::simple("Q"), 10), // Q
    KeyDef::new(74.0, 135.0, 32.0, 32.0, Caption::simple("A"), 11), // A

    // column 3
    KeyDef::new(118.0, 170.0, 32.0, 32.0, Caption::simple("Y"), 12), // Y
    KeyDef::new(107.0, 205.0, 32.0, 32.0, Caption::simple("ALT"), 13), // ALT
    KeyDef::new(78.0, 13.0, 32.0, 32.0, Caption::simple("F1"), 14), // F1
    KeyDef::new(83.0, 66.0, 32.0, 32.0, Caption::simple("2"), 15),  // 2
    KeyDef::new(100.0, 100.0, 32.0, 32.0, Caption::simple("W"), 16), // W
    KeyDef::new(108.0, 135.0, 32.0, 32.0, Caption::simple("S"), 17), // S

    // column 4
    KeyDef::new(152.0, 170.0, 32.0, 32.0, Caption::simple("X"), 18), // X
    KeyDef::dummy(19),                                               // filler
    KeyDef::dummy(20),                                               // filler
    KeyDef::new(112.0, 13.0, 32.0, 32.0, Caption::simple("F2"), 21), // F2
    KeyDef::new(117.0, 66.0, 32.0, 32.0, Caption::simple("3"), 22),  // 3
    KeyDef::new(134.0, 100.0, 32.0, 32.0, Caption::simple("E"), 23), // E
    KeyDef::new(142.0, 135.0, 32.0, 32.0, Caption::simple("D"), 24), // D

    // column 5
    KeyDef::new(186.0, 170.0, 32.0, 32.0, Caption::simple("C"), 25), // C
    KeyDef::new(146.0, 13.0, 32.0, 32.0, Caption::simple("F3"), 26), // F3
    KeyDef::new(151.0, 66.0, 32.0, 32.0, Caption::simple("4"), 27),  // 4
    KeyDef::new(168.0, 100.0, 32.0, 32.0, Caption::simple("R"), 28), // R
    KeyDef::new(176.0, 135.0, 32.0, 32.0, Caption::simple("F"), 29), // F

    // column 6
    KeyDef::new(220.0, 170.0, 32.0, 32.0, Caption::simple("V"), 30), // V
    KeyDef::new(180.0, 13.0, 32.0, 32.0, Caption::simple("F4"), 31), // F4
    KeyDef::new(185.0, 66.0, 32.0, 32.0, Caption::simple("5"), 32),  // 5
    KeyDef::new(202.0, 100.0, 32.0, 32.0, Caption::simple("T"), 33), // T
    KeyDef::new(210.0, 135.0, 32.0, 32.0, Caption::simple("G"), 34), // G

    // column 7
    KeyDef::new(254.0, 170.0, 32.0, 32.0, Caption::simple("B"), 35), // B
    KeyDef::new(141.0, 205.0, 148.0, 32.0, Caption::simple("SPACE BAR"), 36), // SPACE
    KeyDef::new(219.0, 66.0, 32.0, 32.0, Caption::simple("6"), 37), // 6
    KeyDef::new(236.0, 100.0, 32.0, 32.0, Caption::simple("Z"), 38), // Z
    KeyDef::new(244.0, 135.0, 32.0, 32.0, Caption::simple("H"), 39), // H

    // column 8
    KeyDef::new(288.0, 170.0, 32.0, 32.0, Caption::simple("N"), 40), // N
    KeyDef::new(225.0, 13.0, 32.0, 32.0, Caption::simple("F5"), 41), // F5
    KeyDef::new(253.0, 66.0, 32.0, 32.0, Caption::simple("7"), 42), // 7
    KeyDef::new(270.0, 100.0, 32.0, 32.0, Caption::simple("U"), 43), // U
    KeyDef::new(278.0, 135.0, 32.0, 32.0, Caption::simple("J"), 44), // J

    // column 9
    KeyDef::new(322.0, 170.0, 32.0, 32.0, Caption::simple("M"), 45), // M
    KeyDef::dummy(46),                                              // filler
    KeyDef::dummy(47),                                              // filler
    KeyDef::new(259.0, 13.0, 32.0, 32.0, Caption::simple("F6"), 48), // F6
    KeyDef::new(287.0, 66.0, 32.0, 32.0, Caption::simple("8"), 49), // 8
    KeyDef::new(304.0, 100.0, 32.0, 32.0, Caption::simple("I"), 50), // I
    KeyDef::new(312.0, 135.0, 32.0, 32.0, Caption::simple("K"), 51), // K

    // column 10
    KeyDef::new(356.0, 170.0, 32.0, 32.0, Caption::simple(","), 52), // ,
    KeyDef::dummy(53),                                              // filler
    KeyDef::new(293.0, 13.0, 32.0, 32.0, Caption::simple("F7"), 54), // F7
    KeyDef::new(321.0, 66.0, 32.0, 32.0, Caption::simple("9"), 55), // 9
    KeyDef::new(338.0, 100.0, 32.0, 32.0, Caption::simple("O"), 56), // O
    KeyDef::new(346.0, 135.0, 32.0, 32.0, Caption::simple("L"), 57), // L

    // column 11
    KeyDef::new(390.0, 170.0, 32.0, 32.0, Caption::simple("."), 58), // .
    KeyDef::new(292.0, 205.0, 50.0, 32.0, Caption::simple("ALT GR"), 59), // ALT GR
    KeyDef::new(327.0, 13.0, 32.0, 32.0, Caption::simple("F8"), 60), // F8
    KeyDef::new(355.0, 66.0, 32.0, 32.0, Caption::simple("0"), 61), // 0
    KeyDef::new(372.0, 100.0, 32.0, 32.0, Caption::simple("P"), 62), // P
    KeyDef::new(380.0, 135.0, 32.0, 32.0, Caption::simple("Ö"), 63), // Ö

    // column 12
    KeyDef::new(424.0, 170.0, 32.0, 32.0, Caption::simple("-"), 64), // -
    KeyDef::new(346.0, 205.0, 50.0, 32.0, Caption::simple("FN"), 65), // FN
    KeyDef::new(371.0, 13.0, 32.0, 32.0, Caption::simple("F9"), 66), // F9
    KeyDef::new(389.0, 66.0, 32.0, 32.0, Caption::simple("ß"), 67), // ß
    KeyDef::new(406.0, 100.0, 32.0, 32.0, Caption::simple("Ü"), 68), // Ü
    KeyDef::new(414.0, 135.0, 32.0, 32.0, Caption::simple("Ä"), 69), // Ä

    //
    KeyDef::dummy(70),                                               // filler

    // column 13
    KeyDef::new(399.0, 205.0, 50.0, 32.0, Caption::simple("MENU"), 71), // MENU
    KeyDef::new(405.0, 13.0, 32.0, 32.0, Caption::simple("F10"), 72), // F10
    KeyDef::new(424.0, 66.0, 32.0, 32.0, Caption::simple("´"), 73), // ´
    KeyDef::new(441.0, 100.0, 32.0, 32.0, Caption::simple("+"), 74), // +
    KeyDef::new(448.0, 135.0, 26.0, 32.0, Caption::simple("#"),75), // #
    KeyDef::new(459.0, 170.0, 49.0, 32.0, Caption::simple("SHIFT"), 76), // SHIFT

    // column 14
    KeyDef::new(452.0, 205.0, 56.0, 32.0, Caption::simple("CTRL"), 77), // CTRL
    KeyDef::new(439.0, 13.0, 32.0, 32.0, Caption::simple("F11"), 78), // F11
    //
    KeyDef::dummy(79),                                               // filler
    KeyDef::new(473.0, 13.0, 32.0, 32.0, Caption::simple("F12"), 80), // F12
    KeyDef::new(459.0, 66.0, 50.0, 32.0, Caption::simple("BKSPC"), 81), // BACKSPACE

    //
    KeyDef::dummy(82),                                               // filler

    // column 15
    KeyDef::new(
        476.0,
        100.0,
        32.0,
        68.0,
        Caption::new("RETRN", -5.0, 24.0),
        83,
    ), // RETURN

    KeyDef::dummy(84),                                               // filler
    KeyDef::new(515.0, 66.0, 32.0, 32.0, Caption::simple("INS"), 85), // INSERT
    KeyDef::new(515.0, 100.0, 32.0, 32.0, Caption::simple("DEL"), 86), // DELETE
    KeyDef::new(515.0, 205.0, 32.0, 32.0, Caption::simple("←"), 87), // LEFT

    KeyDef::dummy(88),                                               // filler

    // column 16
    KeyDef::new(
        549.0,
        66.0,
        32.0,
        32.0,
        Caption::new("HOME", -4.0, 0.0),
        89,
    ), // HOME
    KeyDef::new(549.0, 100.0, 32.0, 32.0, Caption::simple("END"),90), // END
    KeyDef::new(549.0, 170.0, 32.0, 32.0, Caption::simple("↑"), 91), // UP
    KeyDef::new(549.0, 205.0, 32.0, 32.0, Caption::simple("↓"), 92), // DOWN

    KeyDef::dummy(93),                                               // filler

    // column 17
    KeyDef::new(583.0, 66.0, 32.0, 32.0, Caption::simple("PGUP"), 94), // PAGE UP
    KeyDef::new(583.0, 100.0, 32.0, 32.0, Caption::simple("PGDN"), 95), // PAGE DOWN
    KeyDef::new(583.0, 205.0, 32.0, 32.0, Caption::simple("→"), 96), // RIGHT
];

#[inline]
pub fn index_to_canvas(index: usize) -> usize {
    let index = ROWS_TOPOLOGY
        .iter()
        .position(|e| *e as usize == index)
        .unwrap_or(0);

    let x = index % NUM_COLS;
    let y = index / NUM_COLS;

    let scale_x = 1; // constants::CANVAS_WIDTH / NUM_COLS;
    let scale_y = 1; // constants::CANVAS_HEIGHT / NUM_ROWS;

    let result = (constants::CANVAS_WIDTH * y * scale_y) + (x * scale_x);

    result.clamp(0, constants::CANVAS_SIZE - 1)
}

// pub const NUM_KEYS: usize = 82;

// pub const NUM_ROWS: usize = 6;
pub const NUM_COLS: usize = 17;

// pub const LED_INDICES: usize = 96;

#[rustfmt::skip]
pub const ROWS_TOPOLOGY: [u8; 102] = [
    // ISO model
    0x02, 0x0d, 0x14, 0x19, 0x1e, 0x28, 0x2f, 0x35, 0x3b, 0x41, 0x47, 0x4d, 0x4f, 0x5c, 0xff, 0xff, 0xff,
    0x03, 0x08, 0x0e, 0x15, 0x1a, 0x1f, 0x24, 0x29, 0x30, 0x36, 0x3c, 0x42, 0x48, 0x50, 0x54, 0x58, 0x5d,
    0x04, 0x09, 0x0f, 0x16, 0x1b, 0x20, 0x25, 0x2a, 0x31, 0x37, 0x3d, 0x43, 0x49, 0x52, 0x55, 0x59, 0x5e,
    0x05, 0x0a, 0x10, 0x17, 0x1c, 0x21, 0x26, 0x2b, 0x32, 0x38, 0x3e, 0x44, 0x4a, 0xff, 0xff, 0xff, 0xff,
    0x00, 0x06, 0x0b, 0x11, 0x18, 0x1d, 0x22, 0x27, 0x2c, 0x33, 0x39, 0x3f, 0x4b, 0xff, 0x5a, 0xff, 0xff,
    0x01, 0x07, 0x0c, 0x23, 0x3a, 0x40, 0x46, 0x4c, 0x56, 0x5b, 0x5f, 0x40, 0xff, 0xff, 0xff, 0xff, 0xff,

    // ANSI model
    // TODO: Implement this
];

// #[rustfmt::skip]
// pub const COLS_TOPOLOGY: [u8; 108] = [
//     // ISO model
//     0x02, 0x03, 0x04, 0x05, 0x00, 0x01,
//     0x08, 0x09, 0x0a, 0x06, 0x07, 0xff,
//     0x0d, 0x0e, 0x0f, 0x10, 0x0b, 0x0c,
//     0x14, 0x15, 0x16, 0x17, 0x11, 0xff,
//     0x19, 0x1a, 0x1b, 0x1c, 0x18, 0xff,
//     0x1e, 0x1f, 0x20, 0x21, 0x1d, 0xff,
//     0xff, 0x24, 0x25, 0x26, 0x22, 0x23,
//     0x28, 0x29, 0x2a, 0x2b, 0x27, 0xff,
//     0x2f, 0x30, 0x31, 0x32, 0x2c, 0xff,
//     0x35, 0x36, 0x37, 0x38, 0x33, 0xff,
//     0x3b, 0x3c, 0x3d, 0x3e, 0x39, 0x3a,
//     0x41, 0x42, 0x43, 0x44, 0x3f, 0x40,
//     0x47, 0x48, 0x49, 0x4a, 0x4b, 0x46,
//     0x4d, 0x50, 0x52, 0xff, 0x4c, 0xff,
//     0x4f, 0x54, 0x55, 0xff, 0xff, 0x56,
//     0x5c, 0x58, 0x59, 0xff, 0x5a, 0x5b,
//     0xff, 0x5d, 0x5e, 0xff, 0x40, 0x5f,
//     0xff, 0xff, 0xff, 0xff, 0xff, 0xff,

//     // ANSI model
//     // TODO: Implement this
// ];
