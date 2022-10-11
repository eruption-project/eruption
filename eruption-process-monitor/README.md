## eruption-process-monitor - A daemon to monitor and introspect system processes and events

A daemon that monitors the system for certain events, and subsequently triggers certain
actions like e.g. switching slots and profiles

### Example usage

```shell
$ eruption-process-monitor rules list
  0: On window focused: Name: '.*YouTube.*Chrome' => Switch to profile: /var/lib/eruption/profiles/spectrum-analyzer-swirl.profile (enabled: true, internal: false)
  1: On window focused: Instance: 'Steam' => Switch to profile: /var/lib/eruption/profiles/gaming.profile (enabled: true, internal: false)
  2: On window focused: Instance: 'vlc' => Switch to profile: /var/lib/eruption/profiles/spectrum-analyzer-swirl.profile (enabled: true, internal: false)
  3: On window focused: Name: 'Skype' => Switch to profile: /var/lib/eruption/profiles/vu-meter.profile (enabled: false, internal: false)
  4: On window focused: Instance: 'totem' => Switch to profile: /var/lib/eruption/profiles/spectrum-analyzer-swirl.profile (enabled: true, internal: false)
  5: On window focused: Instance: '.*' => Switch to profile: /var/lib/eruption/profiles/swirl-perlin-blue-red-dim.profile (enabled: true, internal: true)
```

```shell
$ eruption-process-monitor rules add window-instance '.*vlc.*' /var/lib/eruption/profiles/spectrum-analyzer-swirl.profile
```

```shell
$ eruption-process-monitor rules remove 5
```

### eruption-process-monitor

```shell
$ eruption-process-monitor

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

A daemon to monitor and introspect system processes and events

Usage: eruption-process-monitor [OPTIONS] <COMMAND>

Commands:
  daemon       Run in background and monitor running processes
  rules        Rules related sub-commands
  completions  Generate shell completions
  help         Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...       Verbose mode (-v, -vv, -vvv, etc.)
  -c, --config <CONFIG>  Sets the configuration file to use
  -h, --help             Print help information
  -V, --version          Print version information

```
