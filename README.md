# Eruption

A Linux user-mode driver for the ROCCAT Vulcan 100/12x series keyboards

*The project is still in a very early stage of development.
So expect some bugs to be present.*

## Overview

Eruption is a Linux daemon written in Rust, consisting of a core, an integrated
Lua interpreter and additional plugin components. Its intended usage is to
execute Lua scripts that may react to certain events on the system like e.g.
"Key pressed" and subsequently control the AIMO LEDs on the keyboard. Plugins
may export additional functionality to the Lua scripting engine.

# Features

* Integrated Lua interpreter
* AIMO LED Control via Lua scripts
* Event-based architecture
* Daemon plugins may export functions to Lua
* May be run as a Linux user process or as a system daemon

# Installation

> Please note that you need rust-nightly to successfully compile eruption!

### Arch Linux and derivatives like Manjaro

```
$ yay -Sy aur/eruption-roccat-vulcan-git
```

To activate eruption now, you may either reboot your system or manually start
the daemon with the command:

```
$ sudo systemctl start eruption.service
```

Note: You don't have to enable the eruption service, since it is started by an
udev rule as soon as a compatible keyboard device is plugged into your system.

*Support for other distributions is coming soon!*

### From Source

```
$ git clone https://gitlab.com/X3n0m0rph59/eruption-roccat-vulcan.git
$ cd eruption-roccat-vulcan
$ cargo build --all --release
```

# Configuration and Usage

To select one of the available Lua scripts, you may open this link while
eruption is running. This will open the eruption GUI in your browser:
[http://localhost:8059/](http://localhost:8059/)

## Support for audio playback and capture

If you want eruption to be able to play back sound effects, or use one of the
audio visualizer Lua scripts, then you have to perform a few additional steps.
The following steps will allow the eruption daemon to access the PulseAudio
server of the current user, for playback and for capturing of audio signals.

Create the PulseAudio config directory and edit the server configuration file
for your user account:

```
$ mkdir -p ~/.config/pulse/
$ cp /etc/pulse/default.pa ~/.config/pulse/default.pa
$ nano ~/.config/pulse/default.pa
```

then add the following line at the end of the file:

```
load-module module-native-protocol-unix auth-group=root socket=/tmp/pulse-server
```

Create the PulseAudio configuration directory and edit the client configuration
file in `/root/.config/pulse/client.conf` for the user that eruption runs as
(default: root)

```
$ sudo mkdir -p /root/.config/pulse/
$ EDITOR=nano sudoedit /root/.config/pulse/client.conf
```

And then add the following lines:

```
autospawn = no
default-server = unix:/tmp/pulse-server
enable-memfd = yes
```

Finally, restart PulseAudio and eruption for the changes to take effect:

```
$ systemctl --user restart pulseaudio.service
$ sudo systemctl restart eruption.service
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

| Name      | File             | Status | Description                                                                                                  |
| --------- | ---------------- | ------ | ------------------------------------------------------------------------------------------------------------ |
| Afterglow | `afterglow.lua`  | Ready  | Hit keys are lit for a certain amount of time, then they are faded out                                       |
| Afterhue  | `afterhue.lua`   | Ready  | Hit keys cycle through the HSL color-space, using a linearly decreasing hue angle                            |
| Batique   | `batique.lua`    | Ready  | Batique effect, based on the Open Simplex Noise function that serves as input to get a HSL color             |
| Billow    | `billow.lua`     | Ready  | Effect based on the Billow noise function that serves as input to produce a HSL color                        |
| Fractal Brownian Motion | `fbm.lua` | Ready | Effect based on the Fractal Brownian Motion noise function that serves as input to produce a HSL color |
| Perlin Noise | `perlin.lua` | Ready | Effect based on the Perlin Noise function that serves as input to produce a HSL color                          |
| Ridged Multifractal Noise | `rmf.lua` | Ready | Effect based on the Ridged Multifractal noise function that serves as input to produce a HSL color   |
| Voronoi | `voronoi.lua` | Ready | Effect based on the Voronoi noise function that serves as input to produce a HSL                                   |
| Heartbeat | `heartbeat.lua`  | Ready  | Heartbeat effect. The more the system is loaded the faster the heartbeat effect                              |
| Impact    | `impact.lua`     | Ready  | Hit keys and keys in their immediate vicinity stay lit for a certain amount of time, then they are faded out |
| Raindrops | `raindrops.lua`  | Ready  | Rain effect, randomly light up keys and fade them out again                                                  |

The following scripts are unfinished/still in development, and some of them have known bugs:

| Name        | File              | Progress         | Description                                                                                         |
| ----------- | ----------------- | ---------------- | --------------------------------------------------------------------------------------------------- |
| Fire        | `fire.lua`        | Approx. 45% done | Shows a bonfire effect on the keyboard                                                              |
| Fireworks   | `fireworks.lua`   | Approx. 45% done | Shows a fireworks effect on the keyboard                                                            |
| Water       | `water.lua`       | Approx. 95% done | Shows a water effect on the keyboard                                                                |
| Gaming      | `gaming.lua`      | Approx. 75% done | Highlight a fixed set of keys, like e.g. 'WASD'                                                     |
| Gradient    | `gradient.lua`    | Approx. 75% done | Display a color gradient                                                                            |
| Multi Gradient | `multigradient.lua` | Approx. 65% done | Display a color gradient, supports multiple gradient stops                                     |
| Rainbow     | `rainbow.lua`     | Approx. 65% done | Display a rainbow color gradient                                                                    |
| Shockwave   | `shockwave.lua`   | Approx. 75% done | Like Impact, but shows propagating waves when a key has been pressed                                |
| Sysmon      | `sysmon.lua`      | Approx. 10% done | System monitor, keyboard reflects system state                                                      |
| Temperature | `temperature.lua` | Approx. 85% done | Temperature monitor. The keyboard reflects the CPU temperature, from 'green = cold' to 'red = hot'  |
| Audio Visualizer #1 | `audioviz1.lua` | Approx 85% done | Shows the current loudness of the configured audio source as a color gradient                  |
| Audio Visualizer #2 | `audioviz2.lua` | Approx 65% done | Shows the current loudness of the configured audio source as HSL colors progressively          |
| Audio Visualizer #3 | `audioviz3.lua` | Approx 25% done | Shows spectrum visualization of the configured audio source                                    |
| Audio Visualizer #4 | `audioviz4.lua` | Approx 85% done | VU meter heartbeat effect                                                                      |
| Audio Visualizer #5 | `audioviz5.lua` | Approx 75% done | Like Batique, but with additional audio feedback                                               |

# Further Information

For a documentation of the supported Lua functions and libraries, please
refer to [LIBRARY.md](./LIBRARY.md)

# Contributing

Contributions are welcome!
Please see `src/scripts/examples/*.lua` directory for Lua scripting examples.
