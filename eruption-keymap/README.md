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

eruption-keymap 0.0.3
X3n0m0rph59 <x3n0m0rph59@gmail.com>
A CLI keymap editor for Eruption

USAGE:
    eruption-keymap [OPTIONS] <SUBCOMMAND>

OPTIONS:
    -h, --help       Print help information
    -v, --verbose    Verbose mode (-v, -vv, -vvv, etc.)
    -V, --version    Print version information

SUBCOMMANDS:
    compile        Compile a keymap to Lua code and make it available to Eruption
    completions    Generate shell completions
    description    Show or set the description of the specified keymap
    events         Show a list of available Linux EVDEV events
    help           Print this message or the help of the given subcommand(s)
    list           List all available keymaps
    macros         Show a list of available macros
    mapping        Add or remove a single mapping entry
    show           Show some information about a keymap
```
