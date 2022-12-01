## eruption-process-monitor - A daemon to monitor and introspect system processes and events

A daemon that monitors the system for certain events, and subsequently triggers certain
actions like e.g. switching slots and profiles

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

Copyright (c) 2019-2023, The Eruption Development Team

A daemon to monitor and introspect system processes and events

Usage: eruption-process-monitor [OPTIONS] <COMMAND>

Commands:
  rules   Rules related sub-commands (supports offline manipulation of rules)
  daemon  Run in background and monitor running processes
  help    Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...       Verbose mode (-v, -vv, -vvv, etc.)
  -c, --config <CONFIG>  Sets the configuration file to use
  -h, --help             Print help information
  -V, --version          Print version information

```
