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

use std::path::PathBuf;


use csscolorparser::Color;
use serde::{Deserialize, Serialize};

pub type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum ColorSchemeError {
    #[error("File not found: {description}")]
    FileNotFound { description: String },

    #[error("Could not load state: {description}")]
    StateLoadError { description: String },

    #[error("Could not save state: {description}")]
    StateWriteError { description: String },

    #[error("Invalid index: {description}")]
    InvalidIndex { description: String },
}

pub trait ColorSchemeExt {
    fn num_colors(&self) -> usize;
    fn color_rgba_at(&self, index: usize) -> Result<Color>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColorScheme {
    pub colors: Vec<Color>,
}

impl ColorSchemeExt for ColorScheme {
    fn num_colors(&self) -> usize {
        self.colors.len()
    }

    fn color_rgba_at(&self, index: usize) -> Result<Color> {
        Ok(self
            .colors
            .get(index)
            .ok_or_else(|| ColorSchemeError::InvalidIndex {
                description: format!("{}", index),
            })?
            .to_owned())
    }
}

impl TryFrom<Vec<String>> for ColorScheme {
    type Error = eyre::Error;

    fn try_from(value: Vec<String>) -> std::result::Result<Self, Self::Error> {
        let mut colors = Vec::new();

        for color in value.chunks(4) {
            let r = color[0].parse()?;
            let g = color[1].parse()?;
            let b = color[2].parse()?;
            let a = color[3].parse()?;

            let color = Color::from_linear_rgba8(r, g, b, a);
            colors.push(color);
        }

        Ok(Self { colors })
    }
}

impl TryFrom<PywalColorScheme> for ColorScheme {
    type Error = eyre::Error;

    fn try_from(value: PywalColorScheme) -> std::result::Result<Self, Self::Error> {
        let count = value.num_colors();
        let mut colors = Vec::new();

        for index in 0..count {
            let color = value.color_rgba_at(index)?;

            colors.push(color);
        }

        Ok(Self { colors })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct PywalColorScheme {
    pub wallpaper: PathBuf,
    pub alpha: String,

    pub special: PaletteSpecial,
    pub colors: Palette16,
}

impl PywalColorScheme {
    /// Optimize the palette, remove outlier colors
    pub fn optimize(&mut self) {
        self.colors.color0 = self.colors.color1.clone();
        self.colors.color8 = self.colors.color9.clone();
    }
}

impl ColorSchemeExt for PywalColorScheme {
    fn num_colors(&self) -> usize {
        16
    }

    fn color_rgba_at(&self, index: usize) -> Result<Color> {
        match index {
            0 => Ok(csscolorparser::parse(&self.colors.color0)?),
            1 => Ok(csscolorparser::parse(&self.colors.color1)?),
            2 => Ok(csscolorparser::parse(&self.colors.color2)?),
            3 => Ok(csscolorparser::parse(&self.colors.color3)?),
            4 => Ok(csscolorparser::parse(&self.colors.color4)?),
            5 => Ok(csscolorparser::parse(&self.colors.color5)?),
            6 => Ok(csscolorparser::parse(&self.colors.color6)?),
            7 => Ok(csscolorparser::parse(&self.colors.color7)?),
            8 => Ok(csscolorparser::parse(&self.colors.color8)?),
            9 => Ok(csscolorparser::parse(&self.colors.color9)?),
            10 => Ok(csscolorparser::parse(&self.colors.color10)?),
            11 => Ok(csscolorparser::parse(&self.colors.color11)?),
            12 => Ok(csscolorparser::parse(&self.colors.color12)?),
            13 => Ok(csscolorparser::parse(&self.colors.color13)?),
            14 => Ok(csscolorparser::parse(&self.colors.color14)?),
            15 => Ok(csscolorparser::parse(&self.colors.color15)?),

            _ => Err(ColorSchemeError::InvalidIndex {
                description: format!("{}", index),
            }
            .into()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct PaletteSpecial {
    pub background: String,
    pub foreground: String,
    pub cursor: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub struct Palette16 {
    pub color0: String,
    pub color1: String,
    pub color2: String,
    pub color3: String,
    pub color4: String,
    pub color5: String,
    pub color6: String,
    pub color7: String,
    pub color8: String,
    pub color9: String,
    pub color10: String,
    pub color11: String,
    pub color12: String,
    pub color13: String,
    pub color14: String,
    pub color15: String,
}
