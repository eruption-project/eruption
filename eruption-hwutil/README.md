## eruption-hwutil - A CLI control utility for hardware supported by the Eruption Linux user-mode driver

This utility may be used to configure devices without the Eruption daemon required to be running

### eruption-hwutil

```shell
$ sudo eruption-hwutil

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


 Please stop the Eruption daemon prior to running this tool:
 $ sudo systemctl stop eruption.service && sudo systemctl mask eruption.service

 You can re-enable Eruption with this command afterwards:
 $ sudo systemctl unmask eruption.service && sudo systemctl start eruption.service
 
A CLI control utility for hardware supported by the Eruption Linux user-mode driver

Usage: eruption-hwutil [OPTIONS] <COMMAND>

Commands:
  list         List available devices, use this first to find out the index of the device to address
  status       Query device specific status like e.g.: Signal Strength/Battery Level
  blackout     Turn off all LEDs, but otherwise leave the device completely usable
  firmware     Firmware related subcommands (DANGEROUS, may brick the device)
  completions  Generate shell completions
  help         Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...       Verbose mode (-v, -vv, -vvv, etc.)
  -r, --repeat           Repeat output until ctrl+c is pressed
  -c, --config <CONFIG>  Sets the configuration file to use
  -h, --help             Print help information
  -V, --version          Print version information

```
