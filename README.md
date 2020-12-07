![Clippy check](https://github.com/X3n0m0rph59/eruption/workflows/Clippy%20check/badge.svg)

# Table of Contents

- <a href="#eruption">Eruption</a>
- <a href="#devices">Supported Devices</a>
- <a href="#issues">Remarks and known Issues</a>
- <a href="#important">Important Information</a>
- <a href="#overview">Design Overview</a>
- <a href="#installation">Installation</a>
- <a href="#after_setup">After Setup</a>
- <a href="#audio">Support for Audio Playback and Capture </a>
- <a href="#process-monitor">The `eruption-process-monitor` Daemon</a>
- <a href="#info">Further Reading</a>
- <a href="#contributing">Contributing</a>

## Eruption <a name="eruption"></a>

A Linux user-mode input and LED driver for keyboards, mice and other devices

[![Eruption Video](https://img.youtube.com/vi/ig_71zg14nQ/0.jpg)](https://www.youtube.com/watch?v=ig_71zg14nQ)

## Supported Devices <a name="devices"></a>

### Keyboard devices

* ROCCAT Vulcan 100/12x series keyboard

### Mouse devices

* ROCCAT Kone Aimo (experimental)
* ROCCAT Kone Pure Ultra
* ROCCAT Kova AIMO (experimental)

### Remarks and known Issues <a name="issues"></a>

### ROCCAT Vulcan 100/12x series keyboard:

- Mute button will stay lit even if audio is muted

- Keyboard may get into an inconsistent state when Eruption terminates while `Game Mode` is enabled. The state may be fixed manually or by a reboot/device hotplug

- The default `MODIFIER` key is the **`FN`** key. Use it to switch slots (with `F1-F4`) or access macros (`M1-M6`).
- Use the `FN` key too to access special keys/media functions (`F5`-`F12`)
- Easy Shift+ may be activated by pressing `FN`+`Scroll Lock/GameMode`.
- You may want to set a different profile for each slot (`F1`-`F4`).
- Maybe you want to use the GNOME Shell extension [Eruption Profile Switcher](https://extensions.gnome.org/extension/2621/eruption-profile-switcher/)
or visit the [Github page](https://github.com/X3n0m0rph59/eruption-profile-switcher)



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
$ paru -Sy aur/eruption-git
```

#### Fedora based

```sh
$ sudo dnf copr enable x3n0m0rph59/eruption
$ sudo dnf install eruption-git
```

#### Ubuntu

```sh
sudo add-apt-repository ppa:x3n0m0rph59/eruption
sudo apt update
sudo apt install eruption-git
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
$ git clone https://github.com/X3n0m0rph59/eruption.git
$ cd eruption
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

## The `eruption-process-monitor` Daemon <a name="process-monitor"></a>

As of Eruption `0.1.19`, automatic switching of profiles and slots is now supported via the `eruption-process-monitor` daemon. It gathers data via multiple sensor plugins and matches this data against a rule engine. It currently supports executing actions on process execution, as well as on X11 "window focus changed" events.

### Examples

To enable the daemon please run the command:

`systemctl --user enable --now eruption-process-monitor.service`

To list all rules, run the command:

`eruption-process-monitor rules list`

Switch to `spectrum-analyzer-swirl.profile` when a YouTube tab is active in Google Chrome:

`eruption-process-monitor rules add window-name '.*YouTube.*Google Chrome' spectrum-analyzer-swirl.profile`

Switch to `profile3.profile` when a YouTube tab is active in Mozilla Firefox:

`eruption-process-monitor rules add window-name '.*YouTube.*Mozilla Firefox' profile3.profile`


To list all supported sensors and actions please run the command:

`eruption-process-monitor rules add help`

### Removing a rule

```Bash
$ eruption-process-monitor rules list
  0: On window focused: Name: '.*YouTube.*Mozilla Firefox' => Switch to profile: spectrum-analyzer-swirl.profile (enabled: false, internal: false)
  1: On window focused: Name: 'Skype' => Switch to profile: vu-meter.profile (enabled: false, internal: false)
  2: On window focused: Name: 'Left 4 Dead 2.*' => Switch to profile: gaming.profile (enabled: true, internal: false)
  3: On window focused: Name: '.*YouTube.*Google Chrome' => Switch to profile: spectrum-analyzer-swirl.profile (enabled: true, internal: false)
  4: On window focused: Instance: '.*' => Switch to profile: profile1.profile (enabled: true, internal: true)
```

To remove a rule, please run the following command:

`eruption-process-monitor rules remove 1`

This will remove the rule for the window named `Skype` from the ruleset.

## Further Reading <a name="info"></a>

Please see [DOCUMENTATION.md](./DOCUMENTATION.md) for a more thorough explanation of what Eruption is, and how to use and customize it properly.

For further information about the supported Lua functions and libraries, please refer to the developer documentation [LIBRARY.md](./LIBRARY.md).

For a detailed documentation on how to write your own macros, please refer to [MACROS.md](./MACROS.md)

## Contributing <a name="contributing"></a>

Contributions are welcome!
Please see `src/scripts/examples/*.lua` directory for Lua scripting examples.
