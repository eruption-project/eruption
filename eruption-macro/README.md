## eruption-macro - A CLI macro utility for Eruption

This utility may be used to record macros for the Eruption daemon

### eruption-macro

```shell
$ eruption-macro

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

A CLI macro utility for Eruption

Usage: eruption-macro [OPTIONS] <COMMAND>

Commands:
  list         Show a list of available macros in a Lua file
  record       Record a key sequence and save it as a macro
  create       Create a new macro from a description
  remove       Remove an existing macro
  enable       Enable an existing macro
  disable      Disable an existing macro
  description  Show or set the description of a specified macro
  compile      Compile macros to Lua code and make them available to Eruption
  completions  Generate shell completions
  help         Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...  Verbose mode (-v, -vv, -vvv, etc.)
  -h, --help        Print help information
  -V, --version     Print version information

```
