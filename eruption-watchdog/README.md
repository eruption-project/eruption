## eruption-watchdog - A watchdog daemon for Eruption

The watchdog daemon polls the state of the `eruption` process at regular intervals, and kills the `eruption` process
in case it should hang. The watchdog daemon may be especially useful during the development of Eruption when dealing
with unstable drivers.

> NOTE:
> Since version `0.2.0`, Eruption supports using systemd as a software watchdog.
> Running the `eruption-watchdog` daemon is therefore not necessary when the `eruption` process is managed through
> systemd!

### Example usage

```shell
$ sudo eruption-watchdog
```

### eruption-watchdog

```shell
$ eruption-watchdog 

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

A watchdog daemon for Eruption

Usage: eruption-watchdog [OPTIONS] <COMMAND>

Commands:
  daemon  Run watchdog daemon for Eruption
  help    Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...  Verbose mode (-v, -vv, -vvv, etc.)
  -h, --help        Print help information
  -V, --version     Print version information

```