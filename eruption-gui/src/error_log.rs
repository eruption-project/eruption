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

use std::sync::Arc;

use lazy_static::lazy_static;
use parking_lot::RwLock;

lazy_static! {
    /// A log of errors
    pub static ref ERRORS: Arc<RwLock<Vec<LoggedError>>> = Arc::new(RwLock::new(Vec::new()));
}

/// The type of an error in the log
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ErrorType {
    Fatal,
}

#[derive(Debug)]
pub struct LoggedError {
    pub error_type: ErrorType,
    pub message: String,
    pub code: i32,
}

pub fn fatal_error(message: &str, code: i32) {
    ERRORS.write().push(LoggedError {
        error_type: ErrorType::Fatal,
        message: message.to_owned(),
        code,
    });
}
