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

use crate::mapping::*;

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Parser)]
#[grammar = "parsers/grammar/source.pest"]
pub struct SourceParser;

pub fn parse(source: &str) -> Result<Source> {
    let mut result = None;

    let pairs = SourceParser::parse(Rule::Source, source)?.next().unwrap();

    for pair in pairs.into_inner() {
        match pair.as_rule() {
            Rule::EasyShiftKeyDown => {
                let key_index = pair.into_inner().as_str().parse::<usize>()?;
                result = Some(Source::new(Event::EasyShiftKeyDown(Key { key_index })));
            }

            Rule::EasyShiftKeyUp => {
                let key_index = pair.into_inner().as_str().parse::<usize>()?;
                result = Some(Source::new(Event::EasyShiftKeyUp(Key { key_index })));
            }

            Rule::EasyShiftMouseDown => {
                let key_index = pair.into_inner().as_str().parse::<usize>()?;
                result = Some(Source::new(Event::EasyShiftMouseDown(Key { key_index })));
            }

            Rule::EasyShiftMouseUp => {
                let key_index = pair.into_inner().as_str().parse::<usize>()?;
                result = Some(Source::new(Event::EasyShiftMouseUp(Key { key_index })));
            }

            Rule::EasyShiftMouseWheel => {
                let direction = pair.into_inner().as_str();

                match direction.to_ascii_lowercase().as_str() {
                    "up" => result = Some(Source::new(Event::EasyShiftMouseWheel(Direction::Up))),

                    "down" => {
                        result = Some(Source::new(Event::EasyShiftMouseWheel(Direction::Down)))
                    }

                    "left" => {
                        result = Some(Source::new(Event::EasyShiftMouseWheel(Direction::Left)))
                    }

                    "right" => {
                        result = Some(Source::new(Event::EasyShiftMouseWheel(Direction::Right)))
                    }

                    _ => { /* do nothing, will result in parse error below */ }
                }
            }

            Rule::HidKeyDown => {
                let key_index = pair.into_inner().as_str().parse::<usize>()?;
                result = Some(Source::new(Event::HidKeyDown(Key { key_index })));
            }

            Rule::HidKeyUp => {
                let key_index = pair.into_inner().as_str().parse::<usize>()?;
                result = Some(Source::new(Event::HidKeyUp(Key { key_index })));
            }

            Rule::HidMouseDown => {
                let key_index = pair.into_inner().as_str().parse::<usize>()?;
                result = Some(Source::new(Event::HidMouseDown(Key { key_index })));
            }

            Rule::HidMouseUp => {
                let key_index = pair.into_inner().as_str().parse::<usize>()?;
                result = Some(Source::new(Event::HidMouseUp(Key { key_index })));
            }

            Rule::SimpleKeyDown => {
                let key_index = pair.into_inner().as_str().parse::<usize>()?;
                result = Some(Source::new(Event::SimpleKeyDown(Key { key_index })));
            }

            Rule::SimpleKeyUp => {
                let key_index = pair.into_inner().as_str().parse::<usize>()?;
                result = Some(Source::new(Event::SimpleKeyUp(Key { key_index })));
            }

            Rule::SimpleMouseDown => {
                let key_index = pair.into_inner().as_str().parse::<usize>()?;
                result = Some(Source::new(Event::SimpleMouseDown(Key { key_index })));
            }

            Rule::SimpleMouseUp => {
                let key_index = pair.into_inner().as_str().parse::<usize>()?;
                result = Some(Source::new(Event::SimpleMouseUp(Key { key_index })));
            }

            Rule::SimpleMouseWheel => {
                let direction = pair.into_inner().as_str();

                match direction.to_ascii_lowercase().as_str() {
                    "up" => result = Some(Source::new(Event::SimpleMouseWheel(Direction::Up))),

                    "down" => result = Some(Source::new(Event::SimpleMouseWheel(Direction::Down))),

                    "left" => result = Some(Source::new(Event::SimpleMouseWheel(Direction::Left))),

                    "right" => {
                        result = Some(Source::new(Event::SimpleMouseWheel(Direction::Right)))
                    }

                    _ => { /* do nothing, will result in parse error below */ }
                }
            }

            Rule::Null => result = Some(Source::new(Event::Null)),

            _ => {
                /* do nothing */
                continue;
            }
        }
    }

    result.ok_or_else(|| eyre!("Parse error"))
}
