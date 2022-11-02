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

use std::boxed::Box;

pub type Result<T> = std::result::Result<T, eyre::Error>;

pub trait Assistant {
    fn title(&self) -> &'static str;
    fn description(&self) -> &'static str;

    fn previous_step(&mut self) -> Result<()>;
    fn next_step(&mut self) -> Result<()>;

    fn render_page(&mut self) -> Result<()>;
}

pub fn register_assistants() -> Vec<Box<dyn Assistant + 'static>> {
    let mut result = Vec::new();

    // let assistant: Box<dyn Assistant + 'static> =
    //     Box::new(simple_mapping::SimpleMappingAssistant::new());
    // result.push(assistant);

    result
}
