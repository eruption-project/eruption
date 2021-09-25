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
*/

use bincode;
use serde::Deserialize;
use serde::Serialize;

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ProxyCommand {
    NoOp,

    SendAudioBulk(Vec<u8>),

    GetMasterVolume,
    SendMasterVolume(i32),

    IsAudioMuted,
    SendAudioMuted(bool),
}

impl ProxyCommand {
    #[allow(dead_code)]
    pub fn from_buffer(buffer: &[u8]) -> Result<Self> {
        let decoded: Self = bincode::deserialize(&buffer)?;

        Ok(decoded)
    }

    #[allow(dead_code)]
    pub fn to_buffer(&self) -> Result<Vec<u8>> {
        let encoded: Vec<u8> = bincode::serialize(&self)?;

        Ok(encoded)
    }
}
