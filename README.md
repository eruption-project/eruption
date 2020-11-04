![Clippy check](https://github.com/X3n0m0rph59/eruption-roccat-vulcan/workflows/Clippy%20check/badge.svg)

# Table of Contents

- <a href="#eruption">Eruption</a>
- <a href="#devices">Supported Devices</a>
- <a href="#issues">Known Issues</a>
- <a href="#important">Important Information</a>
- <a href="#overview">Design Overview</a>
- <a href="#installation">Installation</a>
- <a href="#after_setup">After Setup</a>
- <a href="#audio">Support for Audio Playback and Capture </a>
- <a href="#info">Further Reading</a>
- <a href="#contributing">Contributing</a>

## Eruption <a name="eruption"></a>

A Linux user-mode driver for the ROCCAT Vulcan 100/12x series keyboards.
Support for other hardware devices is planned and will be included in future releases.
Please see [TODO.md](./TODO.md) and [CHANGES.md](./CHANGES.md) for further information.

[![Eruption Video](https://img.youtube.com/vi/ig_71zg14nQ/0.jpg)](https://www.youtube.com/watch?v=ig_71zg14nQ)

### __TL;DR__ what you absolutely need to know

- The default `MODIFIER` key is the **`FN`** key. Use it to switch slots (with `F1-F4`) or access macros (`M1-M6`).
- Use the `FN` key too to access special keys/media functions (`F5`-`F12`)
- Easy Shift+ may be activated by pressing `FN`+`Scroll Lock/GameMode`.
- You may want to set a different profile for each slot (`F1`-`F4`).
- Maybe you want to use the GNOME Shell extension [Eruption Profile Switcher](https://extensions.gnome.org/extension/2621/eruption-profile-switcher/)
or visit the [Github page](https://github.com/X3n0m0rph59/eruption-profile-switcher)

## Supported Devices <a name="devices"></a>

### Keyboard devices

* ROCCAT Vulcan 100/12x series keyboard

### Mouse devices

* ROCCAT Kone Pure Ultra
* ROCCAT Kone Aimo (experimental)

## Known Issues <a name="issues"></a>

- Mute button will stay lit even if audio is muted

- Keyboard may get into an inconsistent state when Eruption terminates while `Game Mode` is enabled. The state may be fixed manually or by a reboot/device hotplug


## Important Information <a name="important"></a>

This project is still in an early stage of development, and thus may contain
some, possibly serious bugs.

If you ever need to forcefully disable the Eruption daemon you may do so by adding
the following text snippet to the bootloader's (e.g. GRUB) kernel command line:

```sh
  systemd.mask=eruption.service
```
Or with systemctl to mask the service:
```sh
$ sudo systemctl mask eruption.service
```
You can always re-enable the Eruption service with the command:
```sh
$ sudo systemctl unmask eruption.service
```

## Design Overview <a name="overview"></a>

Eruption is a Linux daemon written in Rust, consisting of a core, an integrated
Lua interpreter and additional plugin components. Its intended usage is to
execute Lua scripts that may react to certain events on the system like e.g.
"Key pressed" and subsequently control the AIMO LEDs on the keyboard. Plugins
may export additional functionality to the Lua scripting engine.
Multiple Lua scripts may be run in parallel. Each Lua scripts "submitted color
map" will be combined with all other scripts "submitted color maps" using a
compositor that does an alpha blending step on each color map,
prior to sending the resulting final color map to the keyboard.

## Installation <a name="installation"></a>

#### Arch Linux and derivatives like ArcoLinux or Manjaro

```sh
$ yay -Sy aur/eruption-roccat-vulcan-git
```

#### Fedora based

```sh
$ sudo dnf copr enable x3n0m0rph59/eruption-roccat-vulcan
$ sudo dnf install eruption-roccat-vulcan-git
```

#### Ubuntu

```sh
sudo add-apt-repository ppa:x3n0m0rph59/eruption-roccat-vulcan
sudo apt update
sudo apt install eruption-roccat-vulcan-git
```

To activate Eruption now, you may either reboot your system or manually start
the daemon with the command:

```sh
$ sudo systemctl start eruption.service
```

> Note: You don't have to enable the eruption service, since it is started by an
`udev` rule as soon as a compatible keyboard device is plugged into your system.

#### From Source

```sh
$ git clone https://github.com/X3n0m0rph59/eruption-roccat-vulcan.git
$ cd eruption-roccat-vulcan
$ cargo build --all --release
```

## After Setup <a name="after_setup"></a>

> You may want to try the
[Eruption Profile Switcher](https://extensions.gnome.org/extension/2621/eruption-profile-switcher/)
GNOME Shell extension, for easy switching of profiles on the fly.


### Support for Audio Playback and Capture <a name="audio"></a>

If you want Eruption to be able to play back sound effects, or use one of the
audio visualizer Lua scripts, then you have to perform a few additional steps.
The following steps will allow the Eruption daemon to access the PulseAudio
server of the current user, for playback and for capturing of audio signals.

Create the PulseAudio config directory and edit the server configuration file
for your user account:

```sh
$ mkdir -p ~/.config/pulse/
$ cp /etc/pulse/default.pa ~/.config/pulse/default.pa
$ nano ~/.config/pulse/default.pa
```

then add the following line at the end of the file:

```
load-module module-native-protocol-unix auth-group=root socket=/tmp/pulse-server
```

Create the PulseAudio configuration directory and edit the client configuration
file in `/root/.config/pulse/client.conf` for the user that Eruption runs as
(default: root)

```sh
$ sudo mkdir -p /root/.config/pulse/
$ EDITOR=nano sudoedit /root/.config/pulse/client.conf
```

and then add the following lines:

```ini
autospawn = no
default-server = unix:/tmp/pulse-server
enable-memfd = yes
```

Finally, restart PulseAudio and Eruption for the changes to take effect:

```sh
$ systemctl --user restart pulseaudio.service
$ sudo systemctl restart eruption.service
```

## Further Reading <a name="info"></a>

Please see [DOCUMENTATION.md](./DOCUMENTATION.md) for a more thorough explanation of what Eruption is, and how to use and customize it properly.

For further information about the supported Lua functions and libraries, please refer to the developer documentation [LIBRARY.md](./LIBRARY.md).

For a detailed documentation on how to write your own macros, please refer to [MACROS.md](./MACROS.md)

## Contributing <a name="contributing"></a>

Contributions are welcome!
Please see `src/scripts/examples/*.lua` directory for Lua scripting examples.
