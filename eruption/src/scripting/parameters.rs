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

use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(tag = "type", content = "value", rename_all = "lowercase")]
pub enum TypedValue {
    Int(i64),
    Float(f64),
    Bool(bool),
    String(String),
    Color(u32),
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct UntypedParameter {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct PlainParameter {
    pub name: String,
    pub value: TypedValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ManifestValue {
    Int {
        default: i64,
        min: Option<i64>,
        max: Option<i64>,
    },
    Float {
        default: f64,
        min: Option<f64>,
        max: Option<f64>,
    },
    Bool {
        default: bool,
    },
    String {
        default: String,
    },
    Color {
        default: u32,
        min: Option<u32>,
        max: Option<u32>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub struct ManifestParameter {
    pub name: String,
    pub description: String,
    #[serde(flatten)]
    pub manifest: ManifestValue,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub struct ProfileParameter {
    pub name: String,
    #[serde(flatten)]
    pub value: TypedValue,
    #[serde(skip)]
    pub manifest: Option<ManifestValue>,
}

impl fmt::Display for TypedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self {
            TypedValue::Int(value) => write!(f, "{}", value),
            TypedValue::Float(value) => write!(f, "{}", value),
            TypedValue::Bool(value) => write!(f, "{}", value),
            TypedValue::String(value) => write!(f, "{}", value),
            TypedValue::Color(value) => write!(f, "#{:06x}", value),
        }
    }
}

impl ManifestValue {
    pub fn get_default(&self) -> TypedValue {
        match &self {
            Self::Int { default, .. } => TypedValue::Int(default.to_owned()),
            Self::Float { default, .. } => TypedValue::Float(default.to_owned()),
            Self::Bool { default, .. } => TypedValue::Bool(default.to_owned()),
            Self::String { default, .. } => TypedValue::String(default.to_owned()),
            Self::Color { default, .. } => TypedValue::Color(default.to_owned()),
        }
    }
}

impl ManifestParameter {
    pub fn get_default(&self) -> TypedValue {
        self.manifest.get_default()
    }
}

impl ProfileParameter {
    pub fn get_default(&self) -> Option<TypedValue> {
        Some(self.manifest.as_ref()?.get_default())
    }
}

pub trait ToParameterValue {
    fn to_parameter_value(&self) -> PlainParameter;
}

impl ToParameterValue for ProfileParameter {
    fn to_parameter_value(&self) -> PlainParameter {
        PlainParameter {
            name: self.name.to_owned(),
            value: self.value.to_owned(),
        }
    }
}

impl ToParameterValue for ManifestParameter {
    fn to_parameter_value(&self) -> PlainParameter {
        PlainParameter {
            name: self.name.to_owned(),
            value: self.manifest.get_default(),
        }
    }
}
