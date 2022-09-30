## eruption-keymap - A CLI keymap editor for Eruption

This utility may be used to define key mappings and associate Lua macros to key strokes

### eruption-keymap

```shell
$ eruption-keymap --help
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
