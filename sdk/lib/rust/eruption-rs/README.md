# eruption-sdk

![Stars](https://img.shields.io/crates/v/eruption-sdk?style=flat-square)
![Stars](https://img.shields.io/crates/d/eruption-sdk?style=flat-square)
![Stars](https://img.shields.io/github/stars/X3n0m0rph59/eruption?style=flat-square)
![Stars](https://img.shields.io/crates/l/eruption-sdk?style=flat-square)

This crate provides an interface to the [Eruption Realtime RGB LED Driver](https://github.com/X3n0m0rph59/eruption) for Linux

## Table of Contents

- [eruption-sdk](#eruption-sdk)
  - [Table of Contents](#table-of-contents)
  - [License](#license)
  - [Usage](#usage)
  - [MSRV](#msrv)
  - [Example Code](#example-code)
  - [Support](#support)

## License

The Eruption SDK is licensed under the GNU LGPL-3.0 license

## Usage

Please add this to your `Cargo.toml`:

```toml
[dependencies]
eruption-sdk = "0.0.5"
```

## MSRV

Minimum Supported Rust Version: `Rust 1.64`

## Example Code

```rust
use eruption_sdk::canvas::Canvas;
use eruption_sdk::color::Color;
use eruption_sdk::connection::{Connection, ConnectionType};
use std::thread;
use std::time::Duration;

const EXAMPLE_NAME: &str = "Simple Rust Example #1";

fn main() -> Result<(), Box<dyn std::error::Error>> {
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
    println!("{:?}", status);

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
```

## Support

Support for the Eruption SDK is available on [GitHub](https://github.com/X3n0m0rph59/eruption/issues)
