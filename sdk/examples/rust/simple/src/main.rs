/*  SPDX-License-Identifier: GPL-3.0-or-later  */

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

    Copyright (c) 2019-2023, The Eruption Development Team
*/

use eruption_sdk::canvas::Canvas;
use eruption_sdk::color::Color;
use eruption_sdk::connection::{Connection, ConnectionType};
use std::thread;
use std::time::Duration;

const EXAMPLE_NAME: &str = "Simple Rust Example #1";

fn main() -> Result<(), eyre::Error> {
    println!(
        "Welcome to the Eruption SDK!\nYou are running the \"{}\" \
        from the Eruption SDK version {}\n",
        EXAMPLE_NAME,
        eruption_sdk::SDK_VERSION
    );

    println!("Connecting to the Eruption daemon...");
    let connection = Connection::new(ConnectionType::Local)?;

    connection.connect()?;
    println!("Successfully connected to the Eruption daemon");

    let status = connection.get_server_status()?;
    println!("{status:?}");

    // create a new canvas
    let mut canvas = Canvas::new();

    let red = Color::new(255, 0, 0, 128);
    let green = Color::new(0, 255, 0, 128);
    let blue = Color::new(0, 0, 255, 128);
    let final_val = Color::new(0, 0, 0, 0);

    canvas.fill(red);
    println!("Submitting canvas...");
    connection.submit_canvas(&canvas)?;

    thread::sleep(Duration::from_millis(1000));

    canvas.fill(green);
    println!("Submitting canvas...");
    connection.submit_canvas(&canvas)?;

    thread::sleep(Duration::from_millis(1000));

    canvas.fill(blue);
    println!("Submitting canvas...");
    connection.submit_canvas(&canvas)?;

    thread::sleep(Duration::from_millis(1000));

    canvas.fill(final_val);
    println!("Submitting canvas...");
    connection.submit_canvas(&canvas)?;

    println!("Exiting now");

    Ok(())
}
