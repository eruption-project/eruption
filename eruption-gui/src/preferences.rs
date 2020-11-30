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

use gio::SettingsExt;
use std::path::PathBuf;

type Result<T> = std::result::Result<T, eyre::Error>;

#[derive(Debug, thiserror::Error)]
pub enum PreferencesError {
    #[error("Could not store preferences")]
    SetPreferencesError,
    // #[error("Unknown error: {description}")]
    // UnknownError { description: String },
}

fn get_settings() -> Result<gio::Settings> {
    let default_source = gio::SettingsSchemaSource::get_default().unwrap();

    #[cfg(debug_assertions)]
    let file_name = "eruption-gui/schemas/";

    #[cfg(not(debug_assertions))]
    let file_name = "/usr/share/eruption-gui/schemas/";

    let schema_source = gio::SettingsSchemaSource::from_directory(
        &PathBuf::from(&file_name),
        Some(&default_source),
        false,
    )?;

    let schema = schema_source
        .lookup("org.eruption.eruption-gui", true)
        .unwrap();

    let backend = gio::SettingsBackend::get_default().unwrap();
    let result = gio::Settings::new_full(&schema, Some(&backend), None);

    Ok(result)
}

pub fn get_host_name() -> Result<String> {
    let result = get_settings()?
        .get_string("netfx-host-name")
        .unwrap_or("localhost".into())
        .to_string();

    Ok(result)
}

pub fn get_port_number() -> Result<u16> {
    let result = get_settings()?.get_int("netfx-port-number") as u16;

    Ok(result)
}

pub fn set_host_name(host_name: &str) -> Result<()> {
    get_settings()?
        .set_string("netfx-host-name", &host_name)
        .map_err(|_e| PreferencesError::SetPreferencesError {}.into())
}

pub fn set_port_number(port: u16) -> Result<()> {
    get_settings()?
        .set_int("netfx-port-number", port as i32)
        .map_err(|_e| PreferencesError::SetPreferencesError {}.into())
}
