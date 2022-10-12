## eruption-keymap - A CLI keymap editor for Eruption

This utility may be used to define key mappings and associate Lua macros to key strokes

### eruption-keymap

```shell
$ eruption-keymap

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

A CLI keymap editor for Eruption

Usage: eruption-keymap [OPTIONS] <COMMAND>

Commands:
  list         List all available keymaps
  mapping      Add or remove a single mapping entry
  description  Show or set the description of the specified keymap
  show         Show some information about a keymap
  macros       Show a list of available macros in a Lua file
  events       Show a list of available Linux EVDEV events
  compile      Compile a keymap to Lua code and make it available to Eruption
  completions  Generate shell completions
  help         Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...  Verbose mode (-v, -vv, -vvv, etc.)
  -h, --help        Print help information
  -V, --version     Print version information

```
