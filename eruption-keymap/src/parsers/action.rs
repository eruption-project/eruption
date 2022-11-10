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

use eyre::eyre;
use pest::Parser;
use pest_derive::Parser;

use crate::{mapping::*, util};

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Parser)]
#[grammar = "parsers/grammar/action.pest"]
pub struct ActionParser;

pub fn parse(action: &str) -> Result<Action> {
    let mut result = None;

    let pairs = ActionParser::parse(Rule::Action, action)?.next().unwrap();

    for pair in pairs.into_inner() {
        match pair.as_rule() {
            Rule::Event => {
                let text = pair.into_inner().as_str();

                if let Some(code) = util::evdev_event_code_from_string(text) {
                    let event = code as u32;
                    result = Some(Action::InjectKey(EvdevEvent { event }));
                } else {
                    let event = text.parse::<u32>()?;
                    result = Some(Action::InjectKey(EvdevEvent { event }));
                }
            }

            Rule::Call => {
                let function_name = pair.into_inner().as_str().to_string();
                result = Some(Action::Call(Macro { function_name }));
            }

            Rule::Null => result = Some(Action::Null),

            _ => {
                /* do nothing */
                continue;
            }
        }
    }

    result.ok_or_else(|| eyre!("Parse error in action expression"))
}
