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

use std::{path::PathBuf, process::Command};

#[test]
fn test_eruption_debug_tool_version() {
    let command = PathBuf::from(&env!("CARGO_BIN_EXE_eruption-debug-tool"));

    let output = Command::new(&command)
        .args(&["-V"])
        .output()
        .expect("Failed to execute the test");

    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        format!("eruption-debug-tool {}\n", env!("CARGO_PKG_VERSION"))
    );
}
