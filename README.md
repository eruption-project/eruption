# Eruption

A Linux user-mode driver for the ROCCAT Vulcan 100/12x series keyboards

## Overview

Eruption is a Linux daemon written in Rust, consisting of a core, an integrated Lua interpreter and additional plugin components. Its intended usage is to execute Lua scripts that may react to certain events on the system like e.g. "Key pressed" and subsequently control the AIMO LEDs on the keyboard. Plugins may export additional functionality to the Lua scripting engine.

# Features

* Integrated Lua interpreter
* AIMO LED Control via Lua scripts
* Event-based architecture
* Daemon plugins may export functions to Lua
* May be run as a Linux user process or as a system daemon

# Available Plugins

* Keyboard: Process keyboard events, like e.g. "Key pressed"
* System: Basic system information and status, like e.g. running processes
* Sensors: Query system sensor values, like e.g. CPU package temperature

# Installation

### Arch Linux and derivatives like Manjaro

```
$ yay -Sy aur/eruption-roccat-vulcan
$ sudo systemctl enable --now eruption.service
```

### From Source

```
$ git clone https://gitlab.com/X3n0m0rph59/eruption-roccat-vulcan.git
$ cd eruption-roccat-vulcan
$ cargo build --all --release
```

# Contributing

Contributions are welcome! Please see src/scripts/examples/*.lua directory for Lua scripting examples.

# Project Status

The project is still in a very early stage of development. So expect some bugs to be present.
