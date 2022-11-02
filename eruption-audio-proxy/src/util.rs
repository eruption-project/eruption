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

use std::path::Path;

use byteorder::{LittleEndian, WriteBytesExt};

pub type Result<T> = std::result::Result<T, eyre::Error>;

pub fn load_audio_file<P: AsRef<Path>>(file: P) -> Result<Vec<u8>> {
    let mut reader = hound::WavReader::open(file.as_ref())?;
    let samples = reader
        .samples::<i16>()
        .map(|sample| sample.unwrap())
        .collect::<Vec<i16>>();

    let mut buffer: Vec<u8> = vec![];
    for s in samples {
        buffer.write_i16::<LittleEndian>(s)?;
    }

    Ok(buffer)
}
