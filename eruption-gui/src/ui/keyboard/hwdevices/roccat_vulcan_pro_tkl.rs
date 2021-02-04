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

use super::Keyboard;
use super::{Caption, KeyDef};
use crate::util::RGBA;
use gdk::prelude::GdkContextExt;
use gdk_pixbuf::Pixbuf;
use palette::{Hsva, Srgba};
use std::cell::RefCell;

thread_local! {
    // Pango font description, used to render the captions on the visual representation of keyboard
    static FONT_DESC: RefCell<pango::FontDescription> = RefCell::new(pango::FontDescription::from_string("sans-serif demibold 6"));
}

#[derive(Debug)]
pub struct RoccatVulcanProTKL {}

impl RoccatVulcanProTKL {
    pub fn new() -> Self {
        RoccatVulcanProTKL {}
    }
}

impl Keyboard for RoccatVulcanProTKL {
    fn draw_keyboard(&self, _da: &gtk::DrawingArea, context: &cairo::Context) {
        // let da = da.as_ref();

        // let width = da.get_allocated_width();
        // let height = da.get_allocated_height();

        let scale_factor = 1.5;

        let pixbuf =
            Pixbuf::from_resource("/org/eruption/eruption-gui/img/keyboard-generic-tkl.png")
                .unwrap();

        // paint the schematic drawing
        context.scale(scale_factor, scale_factor);
        context.set_source_pixbuf(&pixbuf, 0.0, 0.0);
        context.paint();

        let led_colors = crate::dbus_client::get_led_colors().unwrap();

        let layout = pangocairo::create_layout(&context).unwrap();
        FONT_DESC.with(|f| {
            let desc = f.borrow();
            layout.set_font_description(Some(&desc));

            // paint all keys
            for i in 0..96 {
                self.paint_key(i + 1, &led_colors[i], &context, &layout);
            }
        });
    }

    /// Paint a key on the keyboard widget
    fn paint_key(&self, key: usize, color: &RGBA, cr: &cairo::Context, layout: &pango::Layout) {
        let key_def = &self.get_key_defs("generic")[key];

        // compute scaling factor
        // let factor =
        //     ((100.0 - crate::STATE.read().current_brightness.unwrap_or(0) as f64) / 100.0) * 0.15;

        // post-process color
        let color = Srgba::new(
            color.r as f64 / 255.0,
            color.g as f64 / 255.0,
            color.b as f64 / 255.0,
            color.a as f64 / 255.0,
        );

        // saturate and lighten color somewhat
        let color = Hsva::from(color);
        let color = Srgba::from(
            color
                // .saturate(factor)
                // .lighten(factor),
        )
        .into_components();

        cr.set_source_rgba(color.0, color.1, color.2, 1.0 - color.3);
        cr.rectangle(key_def.x, key_def.y, key_def.width, key_def.height);
        cr.fill();

        cr.set_source_rgba(1.0, 1.0, 1.0, 1.0);
        cr.move_to(
            7.0 + key_def.x + key_def.caption.x_offset,
            23.0 + ((key_def.y + key_def.caption.y_offset) - (key_def.height / 2.0)),
        );

        layout.set_text(&key_def.caption.text);

        pangocairo::show_layout(&cr, &layout);
    }

    /// Returns a slice of `KeyDef`s representing the currently selected keyboard layout
    fn get_key_defs(&self, layout: &str) -> &[KeyDef] {
        match layout.to_lowercase().as_str() {
            "generic" | "de_de" => &KEY_DEFS_GENERIC_QWERTZ,
            "en_us" | "en_uk" => &KEY_DEFS_GENERIC_QWERTY,

            _ => &KEY_DEFS_GENERIC_QWERTZ,
        }
    }
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
    KeyDef::new(117.0, 170.0, 32.0, 32.0, Caption::simple("Y"), 12), // Y
    KeyDef::new(107.0, 205.0, 32.0, 32.0, Caption::simple("ALT"), 13), // ALT
    KeyDef::new(78.0, 13.0, 32.0, 32.0, Caption::simple("F1"), 14), // F1
    KeyDef::new(83.0, 66.0, 32.0, 32.0, Caption::simple("2"), 15),  // 2
    KeyDef::new(100.0, 100.0, 32.0, 32.0, Caption::simple("W"), 16), // W
    KeyDef::new(107.0, 135.0, 32.0, 32.0, Caption::simple("S"), 17), // S

    // column 4
    KeyDef::new(151.0, 170.0, 32.0, 32.0, Caption::simple("X"), 18), // X
    KeyDef::dummy(19),                                               // filler
    KeyDef::dummy(20),                                               // filler
    KeyDef::new(112.0, 13.0, 32.0, 32.0, Caption::simple("F2"), 21), // F2
    KeyDef::new(117.0, 66.0, 32.0, 32.0, Caption::simple("3"), 22),  // 3
    KeyDef::new(134.0, 100.0, 32.0, 32.0, Caption::simple("E"), 23), // E
    KeyDef::new(141.0, 135.0, 32.0, 32.0, Caption::simple("D"), 24), // D

    // column 5
    KeyDef::new(184.0, 170.0, 32.0, 32.0, Caption::simple("C"), 25), // C
    KeyDef::new(146.0, 13.0, 32.0, 32.0, Caption::simple("F3"), 26), // F3
    KeyDef::new(151.0, 66.0, 32.0, 32.0, Caption::simple("4"), 27),  // 4
    KeyDef::new(168.0, 100.0, 32.0, 32.0, Caption::simple("R"), 28), // R
    KeyDef::new(175.0, 135.0, 32.0, 32.0, Caption::simple("F"), 29), // F

    // column 6
    KeyDef::new(218.0, 170.0, 32.0, 32.0, Caption::simple("V"), 30), // V
    KeyDef::new(180.0, 13.0, 32.0, 32.0, Caption::simple("F4"), 31), // F4
    KeyDef::new(185.0, 66.0, 32.0, 32.0, Caption::simple("5"), 32),  // 5
    KeyDef::new(202.0, 100.0, 32.0, 32.0, Caption::simple("T"), 33), // T
    KeyDef::new(209.0, 135.0, 32.0, 32.0, Caption::simple("G"), 34), // G

    // column 7
    KeyDef::new(252.0, 170.0, 32.0, 32.0, Caption::simple("B"), 35), // B
    KeyDef::new(141.0, 205.0, 148.0, 32.0, Caption::simple("SPACE BAR"), 36), // SPACE
    KeyDef::new(219.0, 66.0, 32.0, 32.0, Caption::simple("6"), 37), // 6
    KeyDef::new(236.0, 100.0, 32.0, 32.0, Caption::simple("Z"), 38), // Z
    KeyDef::new(243.0, 135.0, 32.0, 32.0, Caption::simple("H"), 39), // H

    // column 8
    KeyDef::new(286.0, 170.0, 32.0, 32.0, Caption::simple("N"), 40), // N
    KeyDef::new(225.0, 13.0, 32.0, 32.0, Caption::simple("F5"), 41), // F5
    KeyDef::new(253.0, 66.0, 32.0, 32.0, Caption::simple("7"), 42), // 7
    KeyDef::new(270.0, 100.0, 32.0, 32.0, Caption::simple("U"), 43), // U
    KeyDef::new(277.0, 135.0, 32.0, 32.0, Caption::simple("J"), 44), // J

    // column 9
    KeyDef::new(320.0, 170.0, 32.0, 32.0, Caption::simple("M"), 45), // M
    KeyDef::dummy(46),                                              // filler
    KeyDef::dummy(47),                                              // filler
    KeyDef::new(259.0, 13.0, 32.0, 32.0, Caption::simple("F6"), 48), // F6
    KeyDef::new(287.0, 66.0, 32.0, 32.0, Caption::simple("8"), 49), // 8
    KeyDef::new(304.0, 100.0, 32.0, 32.0, Caption::simple("I"), 50), // I
    KeyDef::new(311.0, 135.0, 32.0, 32.0, Caption::simple("K"), 51), // K

    // column 10
    KeyDef::new(354.0, 170.0, 32.0, 32.0, Caption::simple(","), 52), // ,
    KeyDef::dummy(53),                                              // filler
    KeyDef::new(293.0, 13.0, 32.0, 32.0, Caption::simple("F7"), 54), // F7
    KeyDef::new(321.0, 66.0, 32.0, 32.0, Caption::simple("9"), 55), // 9
    KeyDef::new(338.0, 100.0, 32.0, 32.0, Caption::simple("O"), 56), // O
    KeyDef::new(345.0, 135.0, 32.0, 32.0, Caption::simple("L"), 57), // L

    // column 11
    KeyDef::new(389.0, 170.0, 32.0, 32.0, Caption::simple("."), 58), // .
    KeyDef::new(292.0, 205.0, 50.0, 32.0, Caption::simple("ALT GR"), 59), // ALT GR
    KeyDef::new(327.0, 13.0, 32.0, 32.0, Caption::simple("F8"), 60), // F8
    KeyDef::new(355.0, 66.0, 32.0, 32.0, Caption::simple("0"), 61), // 0
    KeyDef::new(372.0, 100.0, 32.0, 32.0, Caption::simple("P"), 62), // P
    KeyDef::new(379.0, 135.0, 32.0, 32.0, Caption::simple("Ö"), 63), // Ö

    // column 12
    KeyDef::new(423.0, 170.0, 32.0, 32.0, Caption::simple("-"), 64), // -
    KeyDef::new(346.0, 205.0, 50.0, 32.0, Caption::simple("FN"), 65), // FN
    KeyDef::new(371.0, 13.0, 32.0, 32.0, Caption::simple("F9"), 66), // F9
    KeyDef::new(389.0, 66.0, 32.0, 32.0, Caption::simple("ß"), 67), // ß
    KeyDef::new(406.0, 100.0, 32.0, 32.0, Caption::simple("Ü"), 68), // Ü
    KeyDef::new(413.0, 135.0, 32.0, 32.0, Caption::simple("Ä"), 69), // Ä

    //
    KeyDef::dummy(70),                                               // filler

    // column 13
    KeyDef::new(399.0, 205.0, 50.0, 32.0, Caption::simple("MENU"), 71), // MENU
    KeyDef::new(405.0, 13.0, 32.0, 32.0, Caption::simple("F10"), 72), // F10
    KeyDef::new(424.0, 66.0, 32.0, 32.0, Caption::simple("´"), 73), // ´
    KeyDef::new(441.0, 100.0, 32.0, 32.0, Caption::simple("+"), 74), // +
    KeyDef::new(448.0, 135.0, 26.0, 32.0, Caption::simple("#"),75), // #
    KeyDef::new(457.0, 170.0, 51.0, 32.0, Caption::simple("SHIFT"), 76), // SHIFT

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
    KeyDef::new(117.0, 170.0, 32.0, 32.0, Caption::simple("Y"), 12), // Y
    KeyDef::new(107.0, 205.0, 32.0, 32.0, Caption::simple("ALT"), 13), // ALT
    KeyDef::new(78.0, 13.0, 32.0, 32.0, Caption::simple("F1"), 14), // F1
    KeyDef::new(83.0, 66.0, 32.0, 32.0, Caption::simple("2"), 15),  // 2
    KeyDef::new(100.0, 100.0, 32.0, 32.0, Caption::simple("W"), 16), // W
    KeyDef::new(107.0, 135.0, 32.0, 32.0, Caption::simple("S"), 17), // S

    // column 4
    KeyDef::new(151.0, 170.0, 32.0, 32.0, Caption::simple("X"), 18), // X
    KeyDef::dummy(19),                                               // filler
    KeyDef::dummy(20),                                               // filler
    KeyDef::new(112.0, 13.0, 32.0, 32.0, Caption::simple("F2"), 21), // F2
    KeyDef::new(117.0, 66.0, 32.0, 32.0, Caption::simple("3"), 22),  // 3
    KeyDef::new(134.0, 100.0, 32.0, 32.0, Caption::simple("E"), 23), // E
    KeyDef::new(141.0, 135.0, 32.0, 32.0, Caption::simple("D"), 24), // D

    // column 5
    KeyDef::new(184.0, 170.0, 32.0, 32.0, Caption::simple("C"), 25), // C
    KeyDef::new(146.0, 13.0, 32.0, 32.0, Caption::simple("F3"), 26), // F3
    KeyDef::new(151.0, 66.0, 32.0, 32.0, Caption::simple("4"), 27),  // 4
    KeyDef::new(168.0, 100.0, 32.0, 32.0, Caption::simple("R"), 28), // R
    KeyDef::new(175.0, 135.0, 32.0, 32.0, Caption::simple("F"), 29), // F

    // column 6
    KeyDef::new(218.0, 170.0, 32.0, 32.0, Caption::simple("V"), 30), // V
    KeyDef::new(180.0, 13.0, 32.0, 32.0, Caption::simple("F4"), 31), // F4
    KeyDef::new(185.0, 66.0, 32.0, 32.0, Caption::simple("5"), 32),  // 5
    KeyDef::new(202.0, 100.0, 32.0, 32.0, Caption::simple("T"), 33), // T
    KeyDef::new(209.0, 135.0, 32.0, 32.0, Caption::simple("G"), 34), // G

    // column 7
    KeyDef::new(252.0, 170.0, 32.0, 32.0, Caption::simple("B"), 35), // B
    KeyDef::new(141.0, 205.0, 148.0, 32.0, Caption::simple("SPACE BAR"), 36), // SPACE
    KeyDef::new(219.0, 66.0, 32.0, 32.0, Caption::simple("6"), 37), // 6
    KeyDef::new(236.0, 100.0, 32.0, 32.0, Caption::simple("Z"), 38), // Z
    KeyDef::new(243.0, 135.0, 32.0, 32.0, Caption::simple("H"), 39), // H

    // column 8
    KeyDef::new(286.0, 170.0, 32.0, 32.0, Caption::simple("N"), 40), // N
    KeyDef::new(225.0, 13.0, 32.0, 32.0, Caption::simple("F5"), 41), // F5
    KeyDef::new(253.0, 66.0, 32.0, 32.0, Caption::simple("7"), 42), // 7
    KeyDef::new(270.0, 100.0, 32.0, 32.0, Caption::simple("U"), 43), // U
    KeyDef::new(277.0, 135.0, 32.0, 32.0, Caption::simple("J"), 44), // J

    // column 9
    KeyDef::new(320.0, 170.0, 32.0, 32.0, Caption::simple("M"), 45), // M
    KeyDef::dummy(46),                                              // filler
    KeyDef::dummy(47),                                              // filler
    KeyDef::new(259.0, 13.0, 32.0, 32.0, Caption::simple("F6"), 48), // F6
    KeyDef::new(287.0, 66.0, 32.0, 32.0, Caption::simple("8"), 49), // 8
    KeyDef::new(304.0, 100.0, 32.0, 32.0, Caption::simple("I"), 50), // I
    KeyDef::new(311.0, 135.0, 32.0, 32.0, Caption::simple("K"), 51), // K

    // column 10
    KeyDef::new(354.0, 170.0, 32.0, 32.0, Caption::simple(","), 52), // ,
    KeyDef::dummy(53),                                              // filler
    KeyDef::new(293.0, 13.0, 32.0, 32.0, Caption::simple("F7"), 54), // F7
    KeyDef::new(321.0, 66.0, 32.0, 32.0, Caption::simple("9"), 55), // 9
    KeyDef::new(338.0, 100.0, 32.0, 32.0, Caption::simple("O"), 56), // O
    KeyDef::new(345.0, 135.0, 32.0, 32.0, Caption::simple("L"), 57), // L

    // column 11
    KeyDef::new(389.0, 170.0, 32.0, 32.0, Caption::simple("."), 58), // .
    KeyDef::new(292.0, 205.0, 50.0, 32.0, Caption::simple("ALT GR"), 59), // ALT GR
    KeyDef::new(327.0, 13.0, 32.0, 32.0, Caption::simple("F8"), 60), // F8
    KeyDef::new(355.0, 66.0, 32.0, 32.0, Caption::simple("0"), 61), // 0
    KeyDef::new(372.0, 100.0, 32.0, 32.0, Caption::simple("P"), 62), // P
    KeyDef::new(379.0, 135.0, 32.0, 32.0, Caption::simple("Ö"), 63), // Ö

    // column 12
    KeyDef::new(423.0, 170.0, 32.0, 32.0, Caption::simple("-"), 64), // -
    KeyDef::new(346.0, 205.0, 50.0, 32.0, Caption::simple("FN"), 65), // FN
    KeyDef::new(371.0, 13.0, 32.0, 32.0, Caption::simple("F9"), 66), // F9
    KeyDef::new(389.0, 66.0, 32.0, 32.0, Caption::simple("ß"), 67), // ß
    KeyDef::new(406.0, 100.0, 32.0, 32.0, Caption::simple("Ü"), 68), // Ü
    KeyDef::new(413.0, 135.0, 32.0, 32.0, Caption::simple("Ä"), 69), // Ä

    //
    KeyDef::dummy(70),                                               // filler

    // column 13
    KeyDef::new(399.0, 205.0, 50.0, 32.0, Caption::simple("MENU"), 71), // MENU
    KeyDef::new(405.0, 13.0, 32.0, 32.0, Caption::simple("F10"), 72), // F10
    KeyDef::new(424.0, 66.0, 32.0, 32.0, Caption::simple("´"), 73), // ´
    KeyDef::new(441.0, 100.0, 32.0, 32.0, Caption::simple("+"), 74), // +
    KeyDef::new(448.0, 135.0, 26.0, 32.0, Caption::simple("#"),75), // #
    KeyDef::new(457.0, 170.0, 51.0, 32.0, Caption::simple("SHIFT"), 76), // SHIFT

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
