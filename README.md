# Eruption

A Linux user-mode driver for the ROCCAT Vulcan 100/12x series keyboards

*The project is still in a very early stage of development. So expect some bugs to be present.*

## Overview

Eruption is a Linux daemon written in Rust, consisting of a core, an integrated Lua interpreter and additional plugin components. Its intended usage is to execute Lua scripts that may react to certain events on the system like e.g. "Key pressed" and subsequently control the AIMO LEDs on the keyboard. Plugins may export additional functionality to the Lua scripting engine.

# Features

* Integrated Lua interpreter
* AIMO LED Control via Lua scripts
* Event-based architecture
* Daemon plugins may export functions to Lua
* May be run as a Linux user process or as a system daemon

# Installation

### Arch Linux and derivatives like Manjaro

```
$ yay -Sy aur/eruption-roccat-vulcan
```

To activate eruption now, you may either reboot your system or manually start the daemon with the command:

```
$ sudo systemctl start eruption.service
```

Note: You don't have to enable the eruption service, since it is started by an udev rule as soon as a compatible keyboard device is plugged into your system.

*Support for other distributions is coming soon!*

### From Source

```
$ git clone https://gitlab.com/X3n0m0rph59/eruption-roccat-vulcan.git
$ cd eruption-roccat-vulcan
$ cargo build --all --release
```

# Available Plugins

* Keyboard: Process keyboard events, like e.g. "Key pressed"
* System: Basic system information and status, like e.g. running processes
* Sensors: Query system sensor values, like e.g. CPU package temperature
* Audio: Audio related tasks, like playing sounds
* Introspection: Provides internal status information of the Eruption daemon
* Profiles: Switch profiles based on system state

# Available Effects

Eruption currently ships with the following effect scripts:

| Name      | File             | Status | Description                                                                                                 |
| --------- | ---------------- | ------ | ----------------------------------------------------------------------------------------------------------- |
| Afterglow | `afterglow.lua`  | Ready  | Hit keys are lit for a certain amount of time, then they are faded out                                      |
| Batique   | `batique.lua`    | Ready  | Batique effect, based on the OpenSimplexNoise function that serves as input to get a HSL color-space color  |
| Heartbeat | `heartbeat.lua`  | Ready  | Heartbeat effect. The more the system is loaded the faster the heartbeat effect                             |
| Impact    | `impact.lua`     | Ready  | Hit keys and keys in their immediate vicinity are lit for a certain amount of time, then they are faded out |
| Raindrops | `raindrops.lua`  | Ready  | Rain effect, randomly light up keys and fade them out again                                                 |

The following scripts are unfinished/still in development, and some of them have known bugs:

| Name        | File              | Progress         | Description                                                                                         |
| ----------- | ----------------- | ---------------- | --------------------------------------------------------------------------------------------------- |
| Fire        | `fire.lua`        | Approx. 25% done | Shows a bonfire effect on the Keyboard                                                              |
| Gaming      | `gaming.lua`      | Approx. 95% done | Highlight a fixed set of keys, like e.g. 'WASD'                                                     |
| Gradient    | `gradient.lua`    | Approx. 75% done | Display a color gradient                                                                            |
| Rainbow     | `rainbow.lua`     | Approx. 65% done | Display a color gradient, supports multiple gradient stops                                          |
| Shockwave   | `shockwave.lua`   | Approx. 55% done | Like impact, but shows propagating waves when a key has been pressed                                |
| Sysmon      | `sysmon.lua`      | Approx. 10% done | System monitor, keyboard reflects system state                                                      |
| Temperature | `temperature.lua` | Approx. 85% done | Temperature monitor. The keyboard reflects the CPU temperature, from 'green = cold' to 'red = hot'  |

# Further Information

For a documentation of the supported Lua functions and libraries, please refer to [LIBRARY.md](./LIBRARY.md)

# Contributing

Contributions are welcome!
Please see `src/scripts/examples/*.lua` directory for Lua scripting examples.
