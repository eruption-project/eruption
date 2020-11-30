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

use crate::preferences;
use gtk::prelude::*;

type Result<T> = std::result::Result<T, eyre::Error>;

/// Initialize page "Profiles"
pub fn initialize_settings_page(builder: &gtk::Builder) -> Result<()> {
    let host_name: gtk::Entry = builder.get_object("host_name").unwrap();
    let port_number: gtk::SpinButton = builder.get_object("port_number").unwrap();

    host_name.connect_changed(move |entry| {
        preferences::set_host_name(&entry.get_text())
            .unwrap_or_else(|e| log::error!("Could not save a settings value: {}", e));
    });

    port_number.connect_changed(move |entry| {
        preferences::set_port_number(entry.get_value() as u16)
            .unwrap_or_else(|e| log::error!("Could not save a settings value: {}", e));
    });

    host_name.set_text(&preferences::get_host_name()?);
    port_number.set_value(preferences::get_port_number()? as f64);

    Ok(())
}
