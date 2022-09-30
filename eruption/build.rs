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

use std::{error::Error, process::Command};

fn main() -> Result<(), Box<dyn Error + 'static>> {
    let output = Command::new("bash")
        .args(["-c", "../support/pkg/git-version.sh"])
        .output()
        .expect("Failed to execute command");

    let result = String::from_utf8_lossy(&output.stdout);

    println!("cargo:rustc-env=ERUPTION_GIT_PKG_VERSION={}", &*result);

    prost_build::compile_protos(
        &["../support/protobuf/audio-proxy.proto"],
        &["../support/protobuf/"],
    )?;

    prost_build::compile_protos(
        &["../support/protobuf/sdk-support.proto"],
        &["../support/protobuf/"],
    )?;

    Ok(())
}
