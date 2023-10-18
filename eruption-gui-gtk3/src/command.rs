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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Command {
    Unknown,
}

lazy_static! {
    pub static ref UNDO_STACK: Arc<RwLock<Vec<Command>>> = Arc::new(RwLock::new(vec![]));
    pub static ref REDO_STACK: Arc<RwLock<Vec<Command>>> = Arc::new(RwLock::new(vec![]));
}

pub fn push_command() {}

pub fn undo_command() {}
