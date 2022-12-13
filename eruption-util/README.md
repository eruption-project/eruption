## eruption-util - A CLI developer support utility for the Eruption Linux user-mode driver

This is a utility used by the developers of Eruption to generate code and data tables for supported devices

### eruption-util

```shell
$ eruption-util

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

Copyright (c) 2019-2023, The Eruption Development Team

A CLI developer support utility for the Eruption Linux user-mode driver

Usage: eruption-util [OPTIONS] <COMMAND>

Commands:
  list                List available devices, use this first to find out the index of the device to use
  record-key-indices  Record key index information subcommands
  test-key-indices    Test key index information subcommands
  record-topology     Record key topology information subcommands
  test-topology       Test key topology maps subcommands
  help                Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...  Verbose mode (-v, -vv, -vvv, etc.)
  -h, --help        Print help information
  -V, --version     Print version information

```
