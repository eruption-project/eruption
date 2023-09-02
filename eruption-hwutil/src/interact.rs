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

use console::Term;
use lazy_static::lazy_static;
use std::{
    sync::atomic::{AtomicBool, Ordering},
    thread,
    time::Duration,
};

lazy_static! {
    /// Global flag to toggle user-interactive mode
    pub static ref INTERACTIVE: AtomicBool = AtomicBool::new(false);
}

pub fn is_interactive() -> bool {
    INTERACTIVE.load(Ordering::SeqCst)
}

pub fn prompt(prompt: &str) {
    if is_interactive() {
        println!("{prompt}");
        let _ = Term::stdout().read_key();
    }
}

pub fn prompt_or_wait(prompt: &str, duration: Duration) {
    if is_interactive() {
        println!("{prompt}");
        let _ = Term::stdout().read_key();
    } else {
        thread::sleep(duration);
    }
}
