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

#![allow(dead_code)]

// use log::trace;
use parking_lot::Mutex;
use serialport::SerialPort;
use std::{sync::Arc, time::Duration};

use super::{HwDeviceError, RGBA};

#[allow(unused)]
use crate::{constants, eprintln_v, println_v};

const BAUD_RATE: u32 = 460800;
const NUM_LEDS: usize = 80;

pub type Result<T> = super::Result<T>;

#[derive(Clone)]
pub struct CustomSerialLeds {
    pub serial_port: Arc<Mutex<Option<Box<dyn SerialPort>>>>,
}

impl CustomSerialLeds {
    pub fn bind(device_file: &str) -> Result<Self> {
        match serialport::new(device_file, BAUD_RATE)
            .timeout(Duration::from_millis(1000))
            // .data_bits(DataBits::Eight)
            // .stop_bits(StopBits::One)
            // .parity(Parity::Even)
            .open()
        {
            Ok(port) => Ok(Self {
                serial_port: Arc::new(Mutex::new(Some(port))),
            }),

            Err(_e) => Err(HwDeviceError::DeviceNotOpened {}.into()),
        }
    }

    pub fn send_init_sequence(&mut self) -> Result<()> {
        // some devices need many iterations to sync, so we need to try multiple times
        for _ in 0..8 {
            let led_map = [RGBA {
                r: 255,
                g: 255,
                b: 255,
                a: 255,
            }; NUM_LEDS];

            self.send_led_map(&led_map)?;
        }

        Ok(())
    }

    pub fn send_led_off_sequence(&mut self) -> Result<()> {
        // some devices need many iterations to sync, so we need to try multiple times
        for _ in 0..8 {
            let led_map = [RGBA {
                r: 0,
                g: 0,
                b: 0,
                a: 0,
            }; NUM_LEDS];

            self.send_led_map(&led_map)?;
        }

        Ok(())
    }

    pub fn send_led_map(&mut self, led_map: &[RGBA]) -> Result<()> {
        println_v!(1, "Setting LEDs from supplied map...");

        if let Some(ref mut port) = *self.serial_port.lock() {
            const HEADER_OFFSET: usize = 6;

            let mut buffer: [u8; HEADER_OFFSET + (NUM_LEDS * 3)] =
                [0x00; HEADER_OFFSET + (NUM_LEDS * 3)];

            buffer[0..HEADER_OFFSET].clone_from_slice(&[
                b'A',
                b'd',
                b'a',
                0x00,
                NUM_LEDS as u8,
                NUM_LEDS as u8 ^ 0x55,
            ]);

            let mut cntr = 0;
            for e in led_map[0..NUM_LEDS].iter() {
                buffer[HEADER_OFFSET + cntr] = e.r;
                buffer[HEADER_OFFSET + cntr + 1] = e.g;
                buffer[HEADER_OFFSET + cntr + 2] = e.b;

                cntr += 3;
            }

            port.write_all(&buffer)?;

            Ok(())
        } else {
            Err(HwDeviceError::DeviceNotBound {}.into())
        }
    }
}
