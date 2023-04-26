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

use env_logger::fmt::{Color, Style, StyledValue};
use log::Level;
use pretty_env_logger::env_logger;
use std::fmt;
use std::fmt::Display;
use std::sync::atomic::{AtomicUsize, Ordering};

type Result<T> = std::result::Result<T, eyre::Error>;

static MAX_MODULE_WIDTH: AtomicUsize = AtomicUsize::new(0);

pub fn initialize_logging(filters: &str) -> Result<()> {
    pretty_env_logger::formatted_builder()
        .format(|f, record| {
            use std::io::Write;

            let target = record.target();
            let line = record.line().unwrap_or(0);
            let max_width = max_target_width(target);

            let mut style = f.style();
            let level = colored_level(&mut style, record.level());

            let mut style = f.style();
            let target = style.set_bold(true).value(Padded {
                value: format!("{}:{}", target, line),
                width: max_width,
            });

            writeln!(f, " {} {} > {}", level, target, record.args(),)
        })
        .parse_filters(filters)
        .try_init()?;

    Ok(())
}

fn max_target_width(target: &str) -> usize {
    let max_width = MAX_MODULE_WIDTH.load(Ordering::Relaxed);
    if max_width < target.len() {
        MAX_MODULE_WIDTH.store(target.len(), Ordering::Relaxed);
        target.len()
    } else {
        max_width
    }
}

fn colored_level(style: &mut Style, level: Level) -> StyledValue<'_, &'static str> {
    match level {
        Level::Trace => style.set_color(Color::Magenta).value("TRACE"),
        Level::Debug => style.set_color(Color::Blue).value("DEBUG"),
        Level::Info => style.set_color(Color::Green).value("INFO "),
        Level::Warn => style.set_color(Color::Yellow).value("WARN "),
        Level::Error => style.set_color(Color::Red).value("ERROR"),
    }
}

struct Padded<T> {
    value: T,
    width: usize,
}

impl<T: Display> Display for Padded<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{: <width$}", self.value, width = self.width)
    }
}
