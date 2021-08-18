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

use super::KeyDef;
use super::Keyboard;
use crate::util::RGBA;

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug)]
pub struct NullKeyboard {}

impl NullKeyboard {
    pub fn new() -> Self {
        NullKeyboard {}
    }
}

impl Keyboard for NullKeyboard {
    fn get_device(&self) -> u64 {
        0
    }

    fn get_make_and_model(&self) -> (&'static str, &'static str) {
        ("Unknown", "Unknown")
    }

    fn draw_keyboard(
        &self,
        _da: &gtk::DrawingArea,
        _context: &cairo::Context,
    ) -> super::Result<()> {
        Ok(())
    }

    /// Paint a key on the keyboard widget
    fn paint_key(
        &self,
        _key: usize,
        _color: &RGBA,
        _cr: &cairo::Context,
        _layout: &pango::Layout,
    ) -> Result<()> {
        Ok(())
    }

    /// Returns a slice of `KeyDef`s representing the currently selected keyboard layout
    fn get_key_defs(&self, _layout: &str) -> &[KeyDef] {
        &KEY_DEFS
    }
}

// Key definitions for a generic keyboard with QWERTZ (de_DE) Layout
#[rustfmt::skip]
const KEY_DEFS: &[KeyDef] = &[
    KeyDef::dummy(0), // filler
];
