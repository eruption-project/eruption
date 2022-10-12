## eruption-netfx - A Network FX protocol client for the Eruption Linux user-mode driver

Network FX protocol client for the Eruption Linux user-mode driver, supporting ambient effects and much more

### Example usage

```shell
$ eruptionctl switch profile netfx.profile
Switching to profile: /var/lib/eruption/profiles/netfx.profile

$ eruption-netfx "ROCCAT Vulcan Pro TKL" ambient 20
```

```shell
$ eruptionctl switch profile netfx.profile
Switching to profile: /var/lib/eruption/profiles/netfx.profile

$ eruption-netfx command ALL:255:0:0:255
OK
```

### eruption-netfx

```shell
$ eruption-netfx

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

A Network FX protocol client for the Eruption Linux user-mode driver

Usage: eruption-netfx [OPTIONS] [MODEL] [HOSTNAME] [PORT] <COMMAND>

Commands:
  ping         Ping the server
  command      Send Network FX raw protocol commands to the server
  image        Load an image file and display it on the connected devices
  animation    Load image files from a directory and display each one on the connected devices
  ambient      Make the LEDs of connected devices reflect what is shown on the screen
  completions  Generate shell completions
  help         Print this message or the help of the given subcommand(s)

Arguments:
  [MODEL]     The keyboard model, e.g. "ROCCAT Vulcan Pro TKL" or "1e7d:311a"
  [HOSTNAME]  
  [PORT]      

Options:
  -v, --verbose...  Verbose mode (-v, -vv, -vvv, etc.)
  -h, --help        Print help information
  -V, --version     Print version information

```
