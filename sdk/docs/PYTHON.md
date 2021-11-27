# Eruption Python 3 SDK

This is the documentation of the Python 3 SDK for Eruption

# Table of Contents

- [Eruption Python 3 SDK](#eruption-python-3-sdk)
- [Table of Contents](#table-of-contents)
- [Using the Python SDK](#using-the-python-sdk)
  - [Getting started](#getting-started)
    - [Installation Instructions](#installation-instructions)
  - [Example Code](#example-code)
    - [Imports](#imports)
    - [Establishing a Connection](#establishing-a-connection)
    - [The Canvas](#the-canvas)
    - [Terminating the Connection](#terminating-the-connection)
    - [Full Code Listing](#full-code-listing)

# Using the Python SDK

## Getting started

First you need to set up the required environment:

 * Create a virtual environment (venv)
 * Install the dependencies
   * Google Protocol Buffers
 * Run an example application to test the installation
   * `simple.py` located in `sdk/examples/python/`

### Installation Instructions

Create the virtual environment (venv)

```shell
python3 -m venv venv
source venv/bin/activate
```

Now install the dependencies

```shell
pip install protobuf
```

## Example Code

### Imports

```python
from eruption import SDK_VERSION, Connection, Canvas, Color
```

### Establishing a Connection

The following code will establish a connection to a running instance of Eruption via the
local transport (UNIX domain socket)

```python
    # connect to the Eruption daemon (via a local connection)
    try:
        print("Connecting to the Eruption daemon...")
        connection = Connection(type=Connection.LOCAL)

        connection.connect()
        print("Successfully connected to the Eruption daemon")

        status = connection.get_server_status()
        print(status)
```

### The Canvas

Using the canvas with the Color class

```python
        # create a new canvas
        canvas = Canvas()

        red = Color(255, 0, 0, 128)
        green = Color(0, 255, 0, 128)
        blue = Color(0, 0, 255, 128)
        final = Color(0, 0, 0, 0)

        canvas.fill(red)
        print("Submitting canvas...")
        connection.submit_canvas(canvas)

        time.sleep(1)

        canvas.fill(green)

        print("Submitting canvas...")
        connection.submit_canvas(canvas)

        time.sleep(1)

        canvas.fill(blue)
        print("Submitting canvas...")
        connection.submit_canvas(canvas)

        time.sleep(1)

        canvas.fill(final)
        print("Submitting canvas...")
        connection.submit_canvas(canvas)
```

### Terminating the Connection

```python
        connection.disconnect()
        print("Exiting now")

    except (Exception) as e:
        print(f"An error occurred: {type(e).__name__} {e}" )
```

### Full Code Listing

```python
import sys
import time

from eruption import SDK_VERSION, Connection, Canvas, Color

EXAMPLE_NAME = "Simple Python Example #1"

def main():
    """Main program entrypoint"""

    print(f"Welcome to the Eruption SDK!\nYou are running the \"{EXAMPLE_NAME}\" "
          f"from the Eruption SDK version {SDK_VERSION}\n")

    # connect to the Eruption daemon (via a local connection)
    try:
        print("Connecting to the Eruption daemon...")
        connection = Connection(type=Connection.LOCAL)

        connection.connect()
        print("Successfully connected to the Eruption daemon")

        status = connection.get_server_status()
        print(status)

        # create a new canvas
        canvas = Canvas()

        red = Color(255, 0, 0, 128)
        green = Color(0, 255, 0, 128)
        blue = Color(0, 0, 255, 128)
        final = Color(0, 0, 0, 0)

        canvas.fill(red)
        print("Submitting canvas...")
        connection.submit_canvas(canvas)

        time.sleep(1)

        canvas.fill(green)

        print("Submitting canvas...")
        connection.submit_canvas(canvas)

        time.sleep(1)

        canvas.fill(blue)
        print("Submitting canvas...")
        connection.submit_canvas(canvas)

        time.sleep(1)

        canvas.fill(final)
        print("Submitting canvas...")
        connection.submit_canvas(canvas)

        connection.disconnect()
        print("Exiting now")

    except (Exception) as e:
        print(f"An error occurred: {type(e).__name__} {e}" )


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        pass
```
