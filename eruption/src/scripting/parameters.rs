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

use serde::{
    de, ser::SerializeMap, ser::SerializeSeq, Deserialize, Deserializer, Serialize, Serializer,
};
use std::collections::btree_map::{self, Entry};
use std::collections::BTreeMap;
use std::fmt;
use std::marker::PhantomData;

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
            TypedValue::String(value) => f.write_str(value),
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

#[allow(dead_code)]
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

pub trait ToPlainParameter {
    fn to_plain_parameter(&self) -> PlainParameter;
}

impl ToPlainParameter for ProfileParameter {
    fn to_plain_parameter(&self) -> PlainParameter {
        PlainParameter {
            name: self.name.to_owned(),
            value: self.value.to_owned(),
        }
    }
}

impl ToPlainParameter for ManifestParameter {
    fn to_plain_parameter(&self) -> PlainParameter {
        PlainParameter {
            name: self.name.to_owned(),
            value: self.manifest.get_default(),
        }
    }
}

// Parameter containers

#[derive(Default, Clone, PartialEq)] // Serialize and Deserialize implemented below
pub struct ManifestConfiguration(BTreeMap<String, ManifestParameter>); // key is parameter name

#[derive(Default, Deserialize, Clone, PartialEq)] // Serialize implemented below
pub struct ProfileConfiguration(BTreeMap<String, ProfileScriptParameters>); // key is manifest name
#[derive(Default, Clone, PartialEq)] // Serialize and Deserialize implemented below
pub struct ProfileScriptParameters(BTreeMap<String, ProfileParameter>); // key is parameter name

#[allow(dead_code)]
impl ManifestConfiguration {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn set_parameter(&mut self, parameter: ManifestParameter) {
        self.0.insert(parameter.name.to_owned(), parameter);
    }

    pub fn get_parameter(&self, parameter_name: &str) -> Option<&ManifestParameter> {
        self.0.get(parameter_name)
    }

    pub fn iter(&self) -> btree_map::Values<String, ManifestParameter> {
        self.0.values()
    }
}

#[allow(dead_code)]
impl ProfileConfiguration {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn set_parameter(&mut self, script_name: &str, parameter: ProfileParameter) {
        let script_name = script_name.to_owned();
        match self.0.entry(script_name) {
            Entry::Occupied(mut o) => {
                o.get_mut().set_parameter(parameter);
            }
            Entry::Vacant(v) => {
                let mut parameters = ProfileScriptParameters::new();
                parameters.set_parameter(parameter);
                v.insert(parameters);
            }
        };
    }

    pub fn get_parameters_mut(&mut self, script_name: &str) -> &mut ProfileScriptParameters {
        if !self.0.contains_key(script_name) {
            let parameters = ProfileScriptParameters::new();
            self.0.insert(script_name.to_owned(), parameters);
        }
        self.0.get_mut(script_name).unwrap()
    }

    pub fn get_parameter(
        &self,
        script_name: &str,
        parameter_name: &str,
    ) -> Option<&ProfileParameter> {
        self.0.get(script_name)?.get_parameter(parameter_name)
    }

    pub fn get_parameter_mut(
        &mut self,
        script_name: &str,
        parameter_name: &str,
    ) -> Option<&mut ProfileParameter> {
        self.0
            .get_mut(script_name)?
            .get_parameter_mut(parameter_name)
    }

    pub fn get_parameters(&self, script_name: &str) -> Option<&ProfileScriptParameters> {
        self.0.get(script_name)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[allow(dead_code)]
impl ProfileScriptParameters {
    fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn set_parameter(&mut self, parameter: ProfileParameter) {
        self.0.insert(parameter.name.to_owned(), parameter);
    }

    pub fn get_parameter(&self, parameter_name: &str) -> Option<&ProfileParameter> {
        self.0.get(parameter_name)
    }

    pub fn get_parameter_mut(&mut self, parameter_name: &str) -> Option<&mut ProfileParameter> {
        self.0.get_mut(parameter_name)
    }

    pub fn iter(&self) -> btree_map::Values<String, ProfileParameter> {
        self.0.values()
    }
}

impl fmt::Debug for ManifestConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Debug for ProfileConfiguration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl fmt::Debug for ProfileScriptParameters {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl From<BTreeMap<String, ManifestParameter>> for ManifestConfiguration {
    fn from(map: BTreeMap<String, ManifestParameter>) -> Self {
        Self(map)
    }
}

impl From<BTreeMap<String, ProfileScriptParameters>> for ProfileConfiguration {
    fn from(map: BTreeMap<String, ProfileScriptParameters>) -> Self {
        Self(map)
    }
}

impl From<BTreeMap<String, ProfileParameter>> for ProfileScriptParameters {
    fn from(map: BTreeMap<String, ProfileParameter>) -> Self {
        Self(map)
    }
}

impl<const N: usize> From<[ManifestParameter; N]> for ManifestConfiguration {
    fn from(arr: [ManifestParameter; N]) -> Self {
        Self(arr.into_iter().map(|p| (p.name.to_owned(), p)).collect())
    }
}

impl<const N: usize> From<[(String, ProfileScriptParameters); N]> for ProfileConfiguration {
    fn from(arr: [(String, ProfileScriptParameters); N]) -> Self {
        Self(BTreeMap::from(arr))
    }
}

impl<const N: usize> From<[ProfileParameter; N]> for ProfileScriptParameters {
    fn from(arr: [ProfileParameter; N]) -> Self {
        Self(arr.into_iter().map(|p| (p.name.to_owned(), p)).collect())
    }
}

// Serializes as a list.  The key is the parameter name which is also present in the parameter struct
impl Serialize for ManifestConfiguration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;

        let mut sorted = BTreeMap::new();
        sorted.extend(&self.0);

        for param in sorted.values() {
            seq.serialize_element(param)?;
        }
        seq.end()
    }
}

/// Sorts by key (script name) before serializing
impl Serialize for ProfileConfiguration {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;

        let mut sorted = BTreeMap::new();
        sorted.extend(&self.0);

        for entry in sorted.iter() {
            map.serialize_entry(entry.0, entry.1)?;
        }
        map.end()
    }
}

// Serializes as a list.  The key is the parameter name which is also present in the parameter struct
impl Serialize for ProfileScriptParameters {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;

        let mut sorted = BTreeMap::new();
        sorted.extend(&self.0);

        for param in sorted.values() {
            seq.serialize_element(param)?;
        }
        seq.end()
    }
}

impl<'de> Deserialize<'de> for ManifestConfiguration {
    fn deserialize<D>(deserializer: D) -> Result<ManifestConfiguration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let visitor = MapAsListVisitor::<'de, ManifestConfiguration, ManifestParameter>::new();
        deserializer.deserialize_seq(visitor)
    }
}

impl<'de> Deserialize<'de> for ProfileScriptParameters {
    fn deserialize<D>(deserializer: D) -> Result<ProfileScriptParameters, D::Error>
    where
        D: Deserializer<'de>,
    {
        let visitor = MapAsListVisitor::<'de, ProfileScriptParameters, ProfileParameter>::new();
        deserializer.deserialize_seq(visitor)
    }
}

trait GetStringKey {
    fn get_key(&self) -> String;
}

impl GetStringKey for ManifestParameter {
    fn get_key(&self) -> String {
        self.name.to_owned()
    }
}

impl GetStringKey for ProfileParameter {
    fn get_key(&self) -> String {
        self.name.to_owned()
    }
}

struct MapAsListVisitor<'de, Parent, Child>
where
    Parent: From<BTreeMap<String, Child>>,
    Child: GetStringKey + Deserialize<'de>,
{
    parent: PhantomData<Parent>,
    child: PhantomData<&'de Child>,
}

impl<'de, Parent, Child> MapAsListVisitor<'de, Parent, Child>
where
    Parent: From<BTreeMap<String, Child>>,
    Child: GetStringKey + Deserialize<'de>,
{
    fn new() -> MapAsListVisitor<'de, Parent, Child> {
        MapAsListVisitor {
            parent: PhantomData,
            child: PhantomData,
        }
    }
}

impl<'de, Parent, Child> de::Visitor<'de> for MapAsListVisitor<'de, Parent, Child>
where
    Parent: From<BTreeMap<String, Child>>,
    Child: GetStringKey + Deserialize<'de>,
{
    type Value = Parent;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("sequence of parameters")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
    where
        A: de::SeqAccess<'de>,
    {
        let mut map = BTreeMap::new();

        while let Some(param) = seq.next_element::<Child>()? {
            map.insert(param.get_key(), param);
        }

        Ok(Parent::from(map))
    }
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use std::error::Error;

    use super::{
        ManifestConfiguration, ManifestParameter, ManifestValue, ProfileConfiguration,
        ProfileParameter, TypedValue,
    };

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct ProfileTestContainer {
        config: ProfileConfiguration,
    }

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct ManifestTestContainer {
        config: ManifestConfiguration,
    }

    #[test]
    fn profile_configuration_serialization_and_deserialization() -> Result<(), Box<dyn Error>> {
        let test_container = ProfileTestContainer {
            config: [
                (
                    "First Manifest".to_string(),
                    [
                        ProfileParameter {
                            name: "parameter_one".to_string(),
                            value: TypedValue::Bool(true),
                            manifest: None,
                        },
                        ProfileParameter {
                            name: "parameter_two".to_string(),
                            value: TypedValue::Float(1.23),
                            manifest: None,
                        },
                    ]
                    .into(),
                ),
                (
                    "Second Manifest".to_string(),
                    [ProfileParameter {
                        name: "abc".to_string(),
                        value: TypedValue::Int(64),
                        manifest: None,
                    }]
                    .into(),
                ),
            ]
            .into(),
        };

        let toml = toml::ser::to_string(&test_container)?;
        assert_eq!(
            toml.trim(),
            r#"
[[config."First Manifest"]]
name = "parameter_one"
type = "bool"
value = true

[[config."First Manifest"]]
name = "parameter_two"
type = "float"
value = 1.23

[[config."Second Manifest"]]
name = "abc"
type = "int"
value = 64
        "#
            .trim()
        );

        let de_test_container = toml::de::from_str::<ProfileTestContainer>(&toml)?;
        assert_eq!(test_container, de_test_container);

        Ok(())
    }

    #[test]
    fn manifest_configuration_serialization_and_deserialization() -> Result<(), Box<dyn Error>> {
        let test_container = ManifestTestContainer {
            config: [
                ManifestParameter {
                    name: "parameter_one".to_string(),
                    description: "a boolean parameter".to_string(),
                    manifest: ManifestValue::Bool { default: true },
                },
                ManifestParameter {
                    name: "parameter_two".to_string(),
                    description: "a floatation parameter".to_string(),
                    manifest: ManifestValue::Float {
                        default: 1.23,
                        min: Some(0.0),
                        max: None,
                    },
                },
            ]
            .into(),
        };

        let toml = toml::ser::to_string(&test_container)?;
        assert_eq!(
            toml.trim(),
            r#"
[[config]]
name = "parameter_one"
description = "a boolean parameter"
type = "bool"
default = true

[[config]]
name = "parameter_two"
description = "a floatation parameter"
type = "float"
default = 1.23
min = 0.0
        "#
            .trim()
        );

        let de_test_container = toml::de::from_str::<ManifestTestContainer>(&toml)?;
        assert_eq!(test_container, de_test_container);

        Ok(())
    }
}
