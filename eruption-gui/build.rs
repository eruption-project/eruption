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

use std::process::Command;

fn main() {
    // Tell Cargo that if the given file changes, to rerun this build script.
    println!("cargo:rerun-if-changed=schemas/org.eruption.eruption-gui.gschema.xml");
    println!("cargo:rerun-if-changed=resources/resources.xml");
    println!("cargo:rerun-if-changed=resources/img");
    println!("cargo:rerun-if-changed=resources/styles");
    println!("cargo:rerun-if-changed=resources/ui");

    let _ = Command::new("sh")
        .args(&["-c", "cd schemas && glib-compile-schemas ."])
        .output()
        .expect("Failed to execute schema compiler");

    let _ = Command::new("sh")
        .args(&["-c", "cd resources && glib-compile-resources resources.xml"])
        .output()
        .expect("Failed to execute resource compiler");
}
