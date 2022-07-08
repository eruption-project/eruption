/*
    This file is part of Eruption.

    Eruption is free software: &you can redistribute it and/or modify
    it under the terms of &the GNU General Public License as published by
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

use color_eyre::owo_colors::OwoColorize;
use eyre::eyre;
use pest::Parser;
use pest_derive::Parser;

use crate::{mapping::*, util};

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum SourceError {
    #[error("Invalid 'source' parameter")]
    InvalidParameter {},

    #[error("Invalid device")]
    InvalidDevice {},
    // #[error("Operation not supported")]
    // OpNotSupported {},
}

#[derive(Parser)]
#[grammar = "parsers/grammar/source.pest"]
pub struct SourceParser;

pub fn parse(source: &str, device: usize) -> Result<Source> {
    let mut result = None;

    let device_info =
        util::get_device_info_from_index(device as u64).ok_or(SourceError::InvalidDevice {})?;
    let (usb_vid, usb_pid) = (device_info.usb_vid, device_info.usb_pid);

    println!(
        "Selected device: {} {}",
        device_info.make.bold(),
        device_info.model.bold()
    );

    let pairs = SourceParser::parse(Rule::Source, source)?.next().unwrap();

    for pair in pairs.into_inner() {
        match pair.as_rule() {
            Rule::EasyShiftKeyDown => {
                let symbol = pair.into_inner().as_str();
                let key_index = util::symbol_to_key_index(symbol, (usb_vid, usb_pid))
                    .ok_or(SourceError::InvalidParameter {})?;

                result = Some(Source::new(Event::EasyShiftKeyDown(Key::new(
                    key_index,
                    (usb_vid, usb_pid),
                ))));
            }

            Rule::EasyShiftKeyUp => {
                let symbol = pair.into_inner().as_str();
                let key_index = util::symbol_to_key_index(symbol, (usb_vid, usb_pid))
                    .ok_or(SourceError::InvalidParameter {})?;

                result = Some(Source::new(Event::EasyShiftKeyUp(Key::new(
                    key_index,
                    (usb_vid, usb_pid),
                ))));
            }

            Rule::EasyShiftMouseDown => {
                let key_index = pair.into_inner().as_str().parse::<usize>()?;

                result = Some(Source::new(Event::EasyShiftMouseDown(Key::new(
                    key_index,
                    (usb_vid, usb_pid),
                ))));
            }

            Rule::EasyShiftMouseUp => {
                let key_index = pair.into_inner().as_str().parse::<usize>()?;

                result = Some(Source::new(Event::EasyShiftMouseUp(Key::new(
                    key_index,
                    (usb_vid, usb_pid),
                ))));
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

            Rule::EasyShiftMouseDpi => {
                let direction = pair.into_inner().as_str();

                match direction.to_ascii_lowercase().as_str() {
                    "up" => result = Some(Source::new(Event::EasyShiftMouseDpi(Direction::Up))),

                    "down" => result = Some(Source::new(Event::EasyShiftMouseDpi(Direction::Down))),

                    _ => { /* do nothing, will result in parse error below */ }
                }
            }

            Rule::HidKeyDown => {
                let key_index = pair.into_inner().as_str().parse::<usize>()?;
                result = Some(Source::new(Event::HidKeyDown(Key::new(
                    key_index,
                    (usb_vid, usb_pid),
                ))));
            }

            Rule::HidKeyUp => {
                let key_index = pair.into_inner().as_str().parse::<usize>()?;
                result = Some(Source::new(Event::HidKeyUp(Key::new(
                    key_index,
                    (usb_vid, usb_pid),
                ))));
            }

            Rule::HidMouseDown => {
                let key_index = pair.into_inner().as_str().parse::<usize>()?;
                result = Some(Source::new(Event::HidMouseDown(Key::new(
                    key_index,
                    (usb_vid, usb_pid),
                ))));
            }

            Rule::HidMouseUp => {
                let key_index = pair.into_inner().as_str().parse::<usize>()?;
                result = Some(Source::new(Event::HidMouseUp(Key::new(
                    key_index,
                    (usb_vid, usb_pid),
                ))));
            }

            Rule::SimpleKeyDown => {
                let symbol = pair.into_inner().as_str();
                let key_index = util::symbol_to_key_index(symbol, (usb_vid, usb_pid))
                    .ok_or(SourceError::InvalidParameter {})?;

                result = Some(Source::new(Event::SimpleKeyDown(Key::new(
                    key_index,
                    (usb_vid, usb_pid),
                ))));
            }

            Rule::SimpleKeyUp => {
                let symbol = pair.into_inner().as_str();
                let key_index = util::symbol_to_key_index(symbol, (usb_vid, usb_pid))
                    .ok_or(SourceError::InvalidParameter {})?;

                result = Some(Source::new(Event::SimpleKeyUp(Key::new(
                    key_index,
                    (usb_vid, usb_pid),
                ))));
            }

            Rule::SimpleMouseDown => {
                let key_index = pair.into_inner().as_str().parse::<usize>()?;
                result = Some(Source::new(Event::SimpleMouseDown(Key::new(
                    key_index,
                    (usb_vid, usb_pid),
                ))));
            }

            Rule::SimpleMouseUp => {
                let key_index = pair.into_inner().as_str().parse::<usize>()?;
                result = Some(Source::new(Event::SimpleMouseUp(Key::new(
                    key_index,
                    (usb_vid, usb_pid),
                ))));
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

            Rule::SimpleMouseDpi => {
                let direction = pair.into_inner().as_str();

                match direction.to_ascii_lowercase().as_str() {
                    "up" => result = Some(Source::new(Event::SimpleMouseDpi(Direction::Up))),

                    "down" => result = Some(Source::new(Event::SimpleMouseDpi(Direction::Down))),

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

    result.ok_or_else(|| eyre!("Parse error in source expression"))
}
