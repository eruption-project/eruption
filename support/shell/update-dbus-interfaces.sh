#!/bin/bash
#  SPDX-License-Identifier: GPL-3.0-or-later
#
#  This file is part of Eruption.
#
#  Eruption is free software: you can redistribute it and/or modify
#  it under the terms of the GNU General Public License as published by
#  the Free Software Foundation, either version 3 of the License, or
#  (at your option) any later version.
#
#  Eruption is distributed in the hope that it will be useful,
#  but WITHOUT ANY WARRANTY; without even the implied warranty of
#  MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
#  GNU General Public License for more details.
#
#  You should have received a copy of the GNU General Public License
#  along with Eruption.  If not, see <http://www.gnu.org/licenses/>.
#
#  Copyright (c) 2019-2023, The Eruption Development Team

function gen_xml_interface_file {
	busctl introspect $1 $2 --xml-interface > "support/dbus/interfaces/$3"
}

function gen_rust_interface_code_system_bus {
	dbus-codegen-rust --system-bus --prop-newtype --destination $1 --path $2 > "support/dbus/interfaces/rust/$3"
}

function gen_rust_interface_code_session_bus {
	dbus-codegen-rust --prop-newtype --destination $1 --path $2 > "support/dbus/interfaces/rust/$3"
}

export LANG=C

# generate XML files
mkdir -p "support/dbus/interfaces/"

gen_xml_interface_file "org.eruption" "/org/eruption/config" 	"org.eruption.Config.xml"
gen_xml_interface_file "org.eruption" "/org/eruption/devices" 	"org.eruption.Devices.xml"
gen_xml_interface_file "org.eruption" "/org/eruption/profile" 	"org.eruption.Profile.xml"
gen_xml_interface_file "org.eruption" "/org/eruption/slot" 		"org.eruption.Slot.xml"
gen_xml_interface_file "org.eruption" "/org/eruption/status" 	"org.eruption.Status.xml"


# generate Rust interface glue code
mkdir -p "support/dbus/interfaces/rust/org.eruption/"

gen_rust_interface_code_system_bus "org.eruption" "/org/eruption/config"	"org.eruption/config.rs"
gen_rust_interface_code_system_bus "org.eruption" "/org/eruption/devices"	"org.eruption/devices.rs"
gen_rust_interface_code_system_bus "org.eruption" "/org/eruption/profile"	"org.eruption/profile.rs"
gen_rust_interface_code_system_bus "org.eruption" "/org/eruption/slot"		"org.eruption/slot.rs"
gen_rust_interface_code_system_bus "org.eruption" "/org/eruption/status"	"org.eruption/status.rs"

mkdir -p "support/dbus/interfaces/rust/org.eruption.fx_proxy/"

gen_rust_interface_code_session_bus "org.eruption.fx_proxy" "/org/eruption/fx_proxy/effects" "org.eruption.fx_proxy/effects.rs"

mkdir -p "support/dbus/interfaces/rust/org.eruption.process_monitor/"

gen_rust_interface_code_session_bus "org.eruption.process_monitor" "/org/eruption/process_monitor/rules" "org.eruption.process_monitor/rules.rs"

exit 0
