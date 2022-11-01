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

#![allow(unused)]

macro_rules! error {
    ($($arg:tt)+) => ($crate::messages::message!($crate::messages::Level::Error, $($arg)+))
}

macro_rules! warning {
    ($($arg:tt)+) => ($crate::messages::message!($crate::messages::Level::Warning, $($arg)+))
}

macro_rules! debug {
    ($($arg:tt)+) => ($crate::messages::message!($crate::messages::Level::Debug, $($arg)+))
}

macro_rules! info {
    ($($arg:tt)+) => ($crate::messages::message!($crate::messages::Level::Info, $($arg)+))
}

macro_rules! trace {
    ($($arg:tt)+) => ($crate::messages::message!($crate::messages::Level::Trace, $($arg)+))
}

macro_rules! message {
    ($lvl:expr, $($arg:tt)+) => {
        match $lvl {
            $crate::messages::Level::Error => eprint!("{}", "ERROR: ".red()),
            $crate::messages::Level::Warning => eprint!("{}", "WARN:  ".yellow()),
            $crate::messages::Level::Info => eprint!("{}", "INFO:  ".white().bold()),
            $crate::messages::Level::Debug => eprint!("{}", "DEBUG: ".blue()),
            $crate::messages::Level::Trace => eprint!("{}", "TRACE: ".blue()),
        }

        eprintln!($($arg)+)
    };
}

pub(crate) use debug;
pub(crate) use error;
pub(crate) use info;
pub(crate) use message;
pub(crate) use trace;
pub(crate) use warning;

#[repr(usize)]
#[derive(Copy, Eq, Debug, Hash, PartialEq, Clone)]
pub enum Level {
    Error = 1,
    Warning,
    Info,
    Debug,
    Trace,
}
