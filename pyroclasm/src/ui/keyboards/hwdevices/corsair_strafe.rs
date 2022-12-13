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

// pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug)]
pub struct CorsairStrafe {
    pub device: u64,
}

impl CorsairStrafe {
    pub fn new(device: u64, _ui: &mut egui::Ui, _ctx: &egui::Context) -> Self {
        CorsairStrafe { device }
    }
}

impl Keyboard for CorsairStrafe {
    fn get_device(&self) -> u64 {
        self.device
    }

    fn get_make_and_model(&self) -> (&'static str, &'static str) {
        ("Corsair", "STRAFE Gaming Keyboard")
    }

    fn draw_keyboard(&self, _ui: &mut egui::Ui, _ctx: &egui::Context) -> super::Result<()> {
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

// Key definitions for a generic keyboard with QWERTZ (de_DE) Layout
#[rustfmt::skip]
const KEY_DEFS_GENERIC_QWERTZ: &[KeyDef] = &[
    KeyDef::dummy(0), // filler

    // column 1
    KeyDef::new(24.0, 23.0, 32.0, 32.0, Caption::simple("ESC"), 1), // ESC
    KeyDef::new(24.0, 76.0, 32.0, 32.0, Caption::simple("^"), 2),   // GRAVE_ACCENT
    KeyDef::new(24.0, 110.0, 48.0, 32.0, Caption::simple("TAB"), 3), // TAB
    KeyDef::new(24.0, 145.0, 56.0, 32.0, Caption::simple("CAPS LCK"), 4), // CAPS_LOCK
    KeyDef::new(24.0, 180.0, 66.0, 32.0, Caption::simple("SHIFT"), 5), // SHIFT
    KeyDef::new(24.0, 215.0, 50.0, 32.0, Caption::simple("CTRL"), 6), // CTRL

    // column 2
    KeyDef::new(58.0, 76.0, 32.0, 32.0, Caption::simple("1"), 7), // 1
    KeyDef::new(74.0, 110.0, 32.0, 32.0, Caption::simple("Q"), 8), // Q
    KeyDef::new(82.0, 145.0, 32.0, 32.0, Caption::simple("A"), 9), // A
    KeyDef::new(92.0, 180.0, 32.0, 32.0, Caption::simple("<"), 10), // <
    KeyDef::new(76.0, 215.0, 38.0, 32.0, Caption::simple("WIN"), 11), // SUPER

    // column 3
    KeyDef::new(87.0, 23.0, 32.0, 32.0, Caption::simple("F1"), 12), // F1
    KeyDef::new(92.0, 76.0, 32.0, 32.0, Caption::simple("2"), 13),  // 2
    KeyDef::new(108.0, 110.0, 32.0, 32.0, Caption::simple("W"), 14), // W
    KeyDef::new(116.0, 145.0, 32.0, 32.0, Caption::simple("S"), 15), // S
    KeyDef::new(126.0, 180.0, 32.0, 32.0, Caption::simple("Y"), 16), // Y
    KeyDef::new(116.0, 215.0, 32.0, 32.0, Caption::simple("ALT"), 17), // ALT

    // column 4
    KeyDef::new(121.0, 23.0, 32.0, 32.0, Caption::simple("F2"), 18), // F2
    KeyDef::new(126.0, 76.0, 32.0, 32.0, Caption::simple("3"), 19),  // 3
    KeyDef::new(142.0, 110.0, 32.0, 32.0, Caption::simple("E"), 20), // E
    KeyDef::new(150.0, 145.0, 32.0, 32.0, Caption::simple("D"), 21), // D
    KeyDef::new(160.0, 180.0, 32.0, 32.0, Caption::simple("X"), 22), // X
    KeyDef::dummy(23),                                               // filler

    // column 5
    KeyDef::new(155.0, 23.0, 32.0, 32.0, Caption::simple("F3"), 24), // F3
    KeyDef::new(160.0, 76.0, 32.0, 32.0, Caption::simple("4"), 25),  // 4
    KeyDef::new(176.0, 110.0, 32.0, 32.0, Caption::simple("R"), 26), // R
    KeyDef::new(184.0, 145.0, 32.0, 32.0, Caption::simple("F"), 27), // F
    KeyDef::new(194.0, 180.0, 32.0, 32.0, Caption::simple("C"), 28), // C

    // column 6
    KeyDef::new(189.0, 23.0, 32.0, 32.0, Caption::simple("F4"), 29), // F4
    KeyDef::new(194.0, 76.0, 32.0, 32.0, Caption::simple("5"), 30),  // 5
    KeyDef::new(210.0, 110.0, 32.0, 32.0, Caption::simple("T"), 31), // T
    KeyDef::new(218.0, 145.0, 32.0, 32.0, Caption::simple("G"), 32), // G
    KeyDef::new(228.0, 180.0, 32.0, 32.0, Caption::simple("V"), 33), // V

    // column 7
    KeyDef::new(228.0, 76.0, 32.0, 32.0, Caption::simple("6"), 34), // 6
    KeyDef::new(244.0, 110.0, 32.0, 32.0, Caption::simple("Z"), 35), // Z
    KeyDef::new(252.0, 145.0, 32.0, 32.0, Caption::simple("H"), 36), // H
    KeyDef::new(262.0, 180.0, 32.0, 32.0, Caption::simple("B"), 37), // B
    KeyDef::new(150.0, 215.0, 156.0, 32.0, Caption::simple("SPACE BAR"), 38), // SPACE

    // filler
    KeyDef::dummy(39), // filler
    KeyDef::dummy(40), // filler
    KeyDef::dummy(41), // filler
    KeyDef::dummy(42), // filler
    KeyDef::dummy(43), // filler
    KeyDef::dummy(44), // filler
    KeyDef::dummy(45), // filler
    KeyDef::dummy(46), // filler
    KeyDef::dummy(47), // filler
    KeyDef::dummy(48), // filler
    //
    KeyDef::new(233.0, 23.0, 32.0, 32.0, Caption::simple("F5"), 49), // F5

    // column 8
    KeyDef::new(262.0, 76.0, 32.0, 32.0, Caption::simple("7"), 50), // 7
    KeyDef::new(278.0, 110.0, 32.0, 32.0, Caption::simple("U"), 51), // U
    KeyDef::new(286.0, 145.0, 32.0, 32.0, Caption::simple("J"), 52), // J
    KeyDef::new(296.0, 180.0, 32.0, 32.0, Caption::simple("N"), 53), // N
    KeyDef::new(267.0, 23.0, 32.0, 32.0, Caption::simple("F6"), 54), // F6

    // column 9
    KeyDef::new(296.0, 76.0, 32.0, 32.0, Caption::simple("8"), 55), // 8
    KeyDef::new(312.0, 110.0, 32.0, 32.0, Caption::simple("I"), 56), // I
    KeyDef::new(320.0, 145.0, 32.0, 32.0, Caption::simple("K"), 57), // K
    KeyDef::new(330.0, 180.0, 32.0, 32.0, Caption::simple("M"), 58), // M
    KeyDef::dummy(59),                                              // filler
    KeyDef::new(301.0, 23.0, 32.0, 32.0, Caption::simple("F7"), 60), // F7

    // column 10
    KeyDef::new(330.0, 76.0, 32.0, 32.0, Caption::simple("9"), 61), // 9
    KeyDef::new(346.0, 110.0, 32.0, 32.0, Caption::simple("O"), 62), // O
    KeyDef::new(354.0, 145.0, 32.0, 32.0, Caption::simple("L"), 63), // L
    KeyDef::new(364.0, 180.0, 32.0, 32.0, Caption::simple(","), 64), // ,
    KeyDef::dummy(65),                                              // filler
    KeyDef::new(335.0, 23.0, 32.0, 32.0, Caption::simple("F8"), 66), // F8

    // column 11
    KeyDef::new(364.0, 76.0, 32.0, 32.0, Caption::simple("0"), 67), // 0
    KeyDef::new(380.0, 110.0, 32.0, 32.0, Caption::simple("P"), 68), // P
    KeyDef::new(388.0, 145.0, 32.0, 32.0, Caption::simple("Ö"), 69), // Ö
    KeyDef::new(398.0, 180.0, 32.0, 32.0, Caption::simple("."), 70), // .
    KeyDef::new(308.0, 215.0, 50.0, 32.0, Caption::simple("ALT GR"), 71), // ALT GR

    //
    KeyDef::dummy(72), // filler

    // column 12
    KeyDef::new(398.0, 76.0, 32.0, 32.0, Caption::simple("ß"), 73), // ß
    KeyDef::new(414.0, 110.0, 32.0, 32.0, Caption::simple("Ü"), 74), // Ü
    KeyDef::new(422.0, 145.0, 32.0, 32.0, Caption::simple("Ä"), 75), // Ä
    KeyDef::new(432.0, 180.0, 32.0, 32.0, Caption::simple("-"), 76), // -
    KeyDef::new(360.0, 215.0, 50.0, 32.0, Caption::simple("FN"), 77), // FN

    //
    KeyDef::dummy(78),                                               // filler
    KeyDef::new(379.0, 23.0, 32.0, 32.0, Caption::simple("F9"), 79), // F9

    // column 13
    KeyDef::new(432.0, 76.0, 32.0, 32.0, Caption::simple("´"), 80), // ´
    KeyDef::new(448.0, 110.0, 32.0, 32.0, Caption::simple("+"), 81), // +
    KeyDef::dummy(82),                                               // filler
    KeyDef::new(466.0, 180.0, 48.0, 32.0, Caption::simple("SHIFT"), 83), // SHIFT
    KeyDef::new(412.0, 215.0, 50.0, 32.0, Caption::simple("MENU"), 84), // MENU
    KeyDef::new(413.0, 23.0, 32.0, 32.0, Caption::simple("F10"), 85), // F10
    KeyDef::new(447.0, 23.0, 32.0, 32.0, Caption::simple("F11"), 86), // F11

    // column 14
    KeyDef::new(481.0, 23.0, 32.0, 32.0, Caption::simple("F12"), 87), // F12
    KeyDef::new(466.0, 76.0, 48.0, 32.0, Caption::simple("BKSPC"), 88), // BACKSPACE
    KeyDef::new(
        482.0,
        110.0,
        32.0,
        68.0,
        Caption::new("RETRN", -5.0, 24.0),
        89,
    ), // RETURN
    KeyDef::new(464.0, 215.0, 50.0, 32.0, Caption::simple("CTRL"), 90), // CTRL
    KeyDef::dummy(91),                                                // filler
    KeyDef::dummy(92),                                                // filler

    // filler
    KeyDef::dummy(93),                                               // filler
    KeyDef::dummy(94),                                               // filler
    KeyDef::dummy(95),                                               // filler
    KeyDef::dummy(96),                                               // filler
    KeyDef::new(456.0, 145.0, 24.0, 32.0, Caption::simple("#"), 97), // #
    KeyDef::dummy(98),                                               // filler
    KeyDef::dummy(99),                                               // filler

    // column 15
    KeyDef::new(525.0, 23.0, 32.0, 32.0, Caption::simple("PRT"), 100), // PRINT SCREEN
    KeyDef::new(525.0, 76.0, 32.0, 32.0, Caption::simple("INS"), 101), // INSERT
    KeyDef::new(525.0, 110.0, 32.0, 32.0, Caption::simple("DEL"), 102), // DELETE
    KeyDef::new(525.0, 215.0, 32.0, 32.0, Caption::simple("←"), 103), // LEFT

    // column 15
    KeyDef::new(559.0, 23.0, 32.0, 32.0, Caption::simple("SCRL"), 104), // SCROLL LOCK
    KeyDef::new(
        559.0,
        76.0,
        32.0,
        32.0,
        Caption::new("HOME", -4.0, 0.0),
        105,
    ), // HOME
    KeyDef::new(559.0, 110.0, 32.0, 32.0, Caption::simple("END"), 106), // END
    KeyDef::new(559.0, 180.0, 32.0, 32.0, Caption::simple("↑"), 107), // UP
    KeyDef::new(559.0, 215.0, 32.0, 32.0, Caption::simple("↓"), 108), // DOWN

    // column 16
    KeyDef::new(
        593.0,
        23.0,
        32.0,
        32.0,
        Caption::new("PAUSE", -4.0, 0.0),
        109,
    ), // PAUSE
    KeyDef::new(593.0, 76.0, 32.0, 32.0, Caption::simple("PGUP"), 110), // PAGE UP
    KeyDef::new(
        593.0,
        110.0,
        32.0,
        32.0,
        Caption::new("PGDN", 0.0, 0.0),
        111,
    ), // PAGE DOWN
    KeyDef::new(593.0, 215.0, 32.0, 32.0, Caption::simple("→"), 112), // RIGHT
    KeyDef::dummy(113),                                                 // filler

    // column 17
    KeyDef::new(635.0, 76.0, 32.0, 32.0, Caption::simple("NUM"), 114), // NUM LOCK
    KeyDef::new(635.0, 110.0, 32.0, 32.0, Caption::simple("7"), 115),  // 7
    KeyDef::new(635.0, 145.0, 32.0, 32.0, Caption::simple("4"), 116),  // 4
    KeyDef::new(635.0, 180.0, 32.0, 32.0, Caption::simple("1"), 117),  // 1
    KeyDef::new(635.0, 215.0, 66.0, 32.0, Caption::simple("0"), 118),  // 0
    KeyDef::dummy(119),                                                // filler

    // column 18
    KeyDef::new(669.0, 76.0, 32.0, 32.0, Caption::simple("/"), 120), // /
    KeyDef::new(669.0, 110.0, 32.0, 32.0, Caption::simple("8"), 121), // 8
    KeyDef::new(669.0, 145.0, 32.0, 32.0, Caption::simple("5"), 122), // 5
    KeyDef::new(669.0, 180.0, 32.0, 32.0, Caption::simple("2"), 123), // 2
    KeyDef::dummy(124),                                              // filler

    // column 19
    KeyDef::new(703.0, 76.0, 32.0, 32.0, Caption::simple("*"), 125), // *
    KeyDef::new(703.0, 110.0, 32.0, 32.0, Caption::simple("9"), 126), // 9
    KeyDef::new(703.0, 145.0, 32.0, 32.0, Caption::simple("6"), 127), // 6
    KeyDef::new(703.0, 180.0, 32.0, 32.0, Caption::simple("3"), 128), // 3
    KeyDef::new(703.0, 215.0, 32.0, 32.0, Caption::simple(","), 129), // ,

    // column 20
    KeyDef::new(737.0, 76.0, 32.0, 32.0, Caption::simple("-"), 132), // -
    KeyDef::new(737.0, 110.0, 32.0, 67.0, Caption::new("+", 0.0, 24.0), 131), // +
    KeyDef::new(
        737.0,
        180.0,
        32.0,
        67.0,
        Caption::new("ENTER", -4.0, 24.0),
        132,
    ), // ENTER

    // filler
    KeyDef::dummy(133), // filler
    KeyDef::dummy(134), // filler
    KeyDef::dummy(135), // filler
    KeyDef::dummy(136), // filler
    KeyDef::dummy(137), // filler
    KeyDef::dummy(138), // filler
    KeyDef::dummy(139), // filler
    KeyDef::dummy(140), // filler
    KeyDef::dummy(141), // filler
    KeyDef::dummy(142), // filler
    KeyDef::dummy(143), // filler
    KeyDef::dummy(144), // filler
];

// Key definitions for a generic keyboard with QWERTY (en_US) Layout
const KEY_DEFS_GENERIC_QWERTY: &[KeyDef] = &[
    KeyDef::dummy(0), // filler
    // column 1
    KeyDef::new(24.0, 23.0, 32.0, 32.0, Caption::simple("ESC"), 1), // ESC
    KeyDef::new(24.0, 76.0, 32.0, 32.0, Caption::simple("^"), 2),   // GRAVE_ACCENT
    KeyDef::new(24.0, 110.0, 48.0, 32.0, Caption::simple("TAB"), 3), // TAB
    KeyDef::new(24.0, 145.0, 56.0, 32.0, Caption::simple("CAPS LCK"), 4), // CAPS_LOCK
    KeyDef::new(24.0, 180.0, 66.0, 32.0, Caption::simple("SHIFT"), 5), // SHIFT
    KeyDef::new(24.0, 215.0, 50.0, 32.0, Caption::simple("CTRL"), 6), // CTRL
    // column 2
    KeyDef::new(58.0, 76.0, 32.0, 32.0, Caption::simple("1"), 7), // 1
    KeyDef::new(74.0, 110.0, 32.0, 32.0, Caption::simple("Q"), 8), // Q
    KeyDef::new(82.0, 145.0, 32.0, 32.0, Caption::simple("A"), 9), // A
    KeyDef::new(92.0, 180.0, 32.0, 32.0, Caption::simple("<"), 10), // <
    KeyDef::new(76.0, 215.0, 38.0, 32.0, Caption::simple("WIN"), 11), // SUPER
    // column 3
    KeyDef::new(87.0, 23.0, 32.0, 32.0, Caption::simple("F1"), 12), // F1
    KeyDef::new(92.0, 76.0, 32.0, 32.0, Caption::simple("2"), 13),  // 2
    KeyDef::new(108.0, 110.0, 32.0, 32.0, Caption::simple("W"), 14), // W
    KeyDef::new(116.0, 145.0, 32.0, 32.0, Caption::simple("S"), 15), // S
    KeyDef::new(126.0, 180.0, 32.0, 32.0, Caption::simple("Y"), 16), // Y
    KeyDef::new(116.0, 215.0, 32.0, 32.0, Caption::simple("ALT"), 17), // ALT
    // column 4
    KeyDef::new(121.0, 23.0, 32.0, 32.0, Caption::simple("F2"), 18), // F2
    KeyDef::new(126.0, 76.0, 32.0, 32.0, Caption::simple("3"), 19),  // 3
    KeyDef::new(142.0, 110.0, 32.0, 32.0, Caption::simple("E"), 20), // E
    KeyDef::new(150.0, 145.0, 32.0, 32.0, Caption::simple("D"), 21), // D
    KeyDef::new(160.0, 180.0, 32.0, 32.0, Caption::simple("X"), 22), // X
    KeyDef::dummy(23),                                               // filler
    // column 5
    KeyDef::new(155.0, 23.0, 32.0, 32.0, Caption::simple("F3"), 24), // F3
    KeyDef::new(160.0, 76.0, 32.0, 32.0, Caption::simple("4"), 25),  // 4
    KeyDef::new(176.0, 110.0, 32.0, 32.0, Caption::simple("R"), 26), // R
    KeyDef::new(184.0, 145.0, 32.0, 32.0, Caption::simple("F"), 27), // F
    KeyDef::new(194.0, 180.0, 32.0, 32.0, Caption::simple("C"), 28), // C
    // column 6
    KeyDef::new(189.0, 23.0, 32.0, 32.0, Caption::simple("F4"), 29), // F4
    KeyDef::new(194.0, 76.0, 32.0, 32.0, Caption::simple("5"), 30),  // 5
    KeyDef::new(210.0, 110.0, 32.0, 32.0, Caption::simple("T"), 31), // T
    KeyDef::new(218.0, 145.0, 32.0, 32.0, Caption::simple("G"), 32), // G
    KeyDef::new(228.0, 180.0, 32.0, 32.0, Caption::simple("V"), 33), // V
    // column 7
    KeyDef::new(228.0, 76.0, 32.0, 32.0, Caption::simple("6"), 34), // 6
    KeyDef::new(244.0, 110.0, 32.0, 32.0, Caption::simple("Z"), 35), // Z
    KeyDef::new(252.0, 145.0, 32.0, 32.0, Caption::simple("H"), 36), // H
    KeyDef::new(262.0, 180.0, 32.0, 32.0, Caption::simple("B"), 37), // B
    KeyDef::new(150.0, 215.0, 156.0, 32.0, Caption::simple("SPACE BAR"), 38), // SPACE
    // filler
    KeyDef::dummy(39), // filler
    KeyDef::dummy(40), // filler
    KeyDef::dummy(41), // filler
    KeyDef::dummy(42), // filler
    KeyDef::dummy(43), // filler
    KeyDef::dummy(44), // filler
    KeyDef::dummy(45), // filler
    KeyDef::dummy(46), // filler
    KeyDef::dummy(47), // filler
    KeyDef::dummy(48), // filler
    //
    KeyDef::new(233.0, 23.0, 32.0, 32.0, Caption::simple("F5"), 49), // F5
    // column 8
    KeyDef::new(262.0, 76.0, 32.0, 32.0, Caption::simple("7"), 50), // 7
    KeyDef::new(278.0, 110.0, 32.0, 32.0, Caption::simple("U"), 51), // U
    KeyDef::new(286.0, 145.0, 32.0, 32.0, Caption::simple("J"), 52), // J
    KeyDef::new(296.0, 180.0, 32.0, 32.0, Caption::simple("N"), 53), // N
    KeyDef::new(267.0, 23.0, 32.0, 32.0, Caption::simple("F6"), 54), // F6
    // column 9
    KeyDef::new(296.0, 76.0, 32.0, 32.0, Caption::simple("8"), 55), // 8
    KeyDef::new(312.0, 110.0, 32.0, 32.0, Caption::simple("I"), 56), // I
    KeyDef::new(320.0, 145.0, 32.0, 32.0, Caption::simple("K"), 57), // K
    KeyDef::new(330.0, 180.0, 32.0, 32.0, Caption::simple("M"), 58), // M
    KeyDef::dummy(59),                                              // filler
    KeyDef::new(301.0, 23.0, 32.0, 32.0, Caption::simple("F7"), 60), // F7
    // column 10
    KeyDef::new(330.0, 76.0, 32.0, 32.0, Caption::simple("9"), 61), // 9
    KeyDef::new(346.0, 110.0, 32.0, 32.0, Caption::simple("O"), 62), // O
    KeyDef::new(354.0, 145.0, 32.0, 32.0, Caption::simple("L"), 63), // L
    KeyDef::new(364.0, 180.0, 32.0, 32.0, Caption::simple(","), 64), // ,
    KeyDef::dummy(65),                                              // filler
    KeyDef::new(335.0, 23.0, 32.0, 32.0, Caption::simple("F8"), 66), // F8
    // column 11
    KeyDef::new(364.0, 76.0, 32.0, 32.0, Caption::simple("0"), 67), // 0
    KeyDef::new(380.0, 110.0, 32.0, 32.0, Caption::simple("P"), 68), // P
    KeyDef::new(388.0, 145.0, 32.0, 32.0, Caption::simple("Ö"), 69), // Ö
    KeyDef::new(398.0, 180.0, 32.0, 32.0, Caption::simple("."), 70), // .
    KeyDef::new(308.0, 215.0, 50.0, 32.0, Caption::simple("ALT GR"), 71), // ALT GR
    //
    KeyDef::dummy(72), // filler
    // column 12
    KeyDef::new(398.0, 76.0, 32.0, 32.0, Caption::simple("ß"), 73), // ß
    KeyDef::new(414.0, 110.0, 32.0, 32.0, Caption::simple("Ü"), 74), // Ü
    KeyDef::new(422.0, 145.0, 32.0, 32.0, Caption::simple("Ä"), 75), // Ä
    KeyDef::new(432.0, 180.0, 32.0, 32.0, Caption::simple("-"), 76), // -
    KeyDef::new(360.0, 215.0, 50.0, 32.0, Caption::simple("FN"), 77), // FN
    //
    KeyDef::dummy(78),                                               // filler
    KeyDef::new(379.0, 23.0, 32.0, 32.0, Caption::simple("F9"), 79), // F9
    // column 13
    KeyDef::new(432.0, 76.0, 32.0, 32.0, Caption::simple("´"), 80), // ´
    KeyDef::new(448.0, 110.0, 32.0, 32.0, Caption::simple("+"), 81), // +
    KeyDef::dummy(82),                                               // filler
    KeyDef::new(466.0, 180.0, 48.0, 32.0, Caption::simple("SHIFT"), 83), // SHIFT
    KeyDef::new(412.0, 215.0, 50.0, 32.0, Caption::simple("MENU"), 84), // MENU
    KeyDef::new(413.0, 23.0, 32.0, 32.0, Caption::simple("F10"), 85), // F10
    KeyDef::new(447.0, 23.0, 32.0, 32.0, Caption::simple("F11"), 86), // F11
    // column 14
    KeyDef::new(481.0, 23.0, 32.0, 32.0, Caption::simple("F12"), 87), // F12
    KeyDef::new(466.0, 76.0, 48.0, 32.0, Caption::simple("BKSPC"), 88), // BACKSPACE
    KeyDef::new(
        482.0,
        110.0,
        32.0,
        68.0,
        Caption::new("RETRN", -5.0, 24.0),
        89,
    ), // RETURN
    KeyDef::new(464.0, 215.0, 50.0, 32.0, Caption::simple("CTRL"), 90), // CTRL
    KeyDef::dummy(91),                                                // filler
    KeyDef::dummy(92),                                                // filler
    // filler
    KeyDef::dummy(93),                                               // filler
    KeyDef::dummy(94),                                               // filler
    KeyDef::dummy(95),                                               // filler
    KeyDef::dummy(96),                                               // filler
    KeyDef::new(456.0, 145.0, 24.0, 32.0, Caption::simple("#"), 97), // #
    KeyDef::dummy(98),                                               // filler
    KeyDef::dummy(99),                                               // filler
    // column 15
    KeyDef::new(525.0, 23.0, 32.0, 32.0, Caption::simple("PRT"), 100), // PRINT SCREEN
    KeyDef::new(525.0, 76.0, 32.0, 32.0, Caption::simple("INS"), 101), // INSERT
    KeyDef::new(525.0, 110.0, 32.0, 32.0, Caption::simple("DEL"), 102), // DELETE
    KeyDef::new(525.0, 215.0, 32.0, 32.0, Caption::simple("←"), 103), // LEFT
    // column 15
    KeyDef::new(559.0, 23.0, 32.0, 32.0, Caption::simple("SCRL"), 104), // SCROLL LOCK
    KeyDef::new(
        559.0,
        76.0,
        32.0,
        32.0,
        Caption::new("HOME", -4.0, 0.0),
        105,
    ), // HOME
    KeyDef::new(559.0, 110.0, 32.0, 32.0, Caption::simple("END"), 106), // END
    KeyDef::new(559.0, 180.0, 32.0, 32.0, Caption::simple("↑"), 107), // UP
    KeyDef::new(559.0, 215.0, 32.0, 32.0, Caption::simple("↓"), 108), // DOWN
    // column 16
    KeyDef::new(
        593.0,
        23.0,
        32.0,
        32.0,
        Caption::new("PAUSE", -4.0, 0.0),
        109,
    ), // PAUSE
    KeyDef::new(593.0, 76.0, 32.0, 32.0, Caption::simple("PGUP"), 110), // PAGE UP
    KeyDef::new(
        593.0,
        110.0,
        32.0,
        32.0,
        Caption::new("PGDN", 0.0, 0.0),
        111,
    ), // PAGE DOWN
    KeyDef::new(593.0, 215.0, 32.0, 32.0, Caption::simple("→"), 112), // RIGHT
    KeyDef::dummy(113),                                                 // filler
    // column 17
    KeyDef::new(635.0, 76.0, 32.0, 32.0, Caption::simple("NUM"), 114), // NUM LOCK
    KeyDef::new(635.0, 110.0, 32.0, 32.0, Caption::simple("7"), 115),  // 7
    KeyDef::new(635.0, 145.0, 32.0, 32.0, Caption::simple("4"), 116),  // 4
    KeyDef::new(635.0, 180.0, 32.0, 32.0, Caption::simple("1"), 117),  // 1
    KeyDef::new(635.0, 215.0, 66.0, 32.0, Caption::simple("0"), 118),  // 0
    KeyDef::dummy(119),                                                // filler
    // column 18
    KeyDef::new(669.0, 76.0, 32.0, 32.0, Caption::simple("/"), 120), // /
    KeyDef::new(669.0, 110.0, 32.0, 32.0, Caption::simple("8"), 121), // 8
    KeyDef::new(669.0, 145.0, 32.0, 32.0, Caption::simple("5"), 122), // 5
    KeyDef::new(669.0, 180.0, 32.0, 32.0, Caption::simple("2"), 123), // 2
    KeyDef::dummy(124),                                              // filler
    // column 19
    KeyDef::new(703.0, 76.0, 32.0, 32.0, Caption::simple("*"), 125), // *
    KeyDef::new(703.0, 110.0, 32.0, 32.0, Caption::simple("9"), 126), // 9
    KeyDef::new(703.0, 145.0, 32.0, 32.0, Caption::simple("6"), 127), // 6
    KeyDef::new(703.0, 180.0, 32.0, 32.0, Caption::simple("3"), 128), // 3
    KeyDef::new(703.0, 215.0, 32.0, 32.0, Caption::simple(","), 129), // ,
    // column 20
    KeyDef::new(737.0, 76.0, 32.0, 32.0, Caption::simple("-"), 132), // -
    KeyDef::new(737.0, 110.0, 32.0, 67.0, Caption::new("+", 0.0, 24.0), 131), // +
    KeyDef::new(
        737.0,
        180.0,
        32.0,
        67.0,
        Caption::new("ENTER", -4.0, 24.0),
        132,
    ), // ENTER
    // filler
    KeyDef::dummy(133), // filler
    KeyDef::dummy(134), // filler
    KeyDef::dummy(135), // filler
    KeyDef::dummy(136), // filler
    KeyDef::dummy(137), // filler
    KeyDef::dummy(138), // filler
    KeyDef::dummy(139), // filler
    KeyDef::dummy(140), // filler
    KeyDef::dummy(141), // filler
    KeyDef::dummy(142), // filler
    KeyDef::dummy(143), // filler
    KeyDef::dummy(144), // filler
];
