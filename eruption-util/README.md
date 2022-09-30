## eruption-util - A CLI developer support utility for the Eruption Linux user-mode driver

This is a utility used by the developers of Eruption to generate code and data tables for supported devices

### eruption-util

```shell
$ eruption-util --help
A CLI developer support utility for the Eruption Linux user-mode driver

Usage: eruption-util [OPTIONS] <COMMAND>

Commands:
  list                List available devices, use this first to find out the index of the device to use
  record-key-indices  Record key index information subcommands
  test-key-indices    Test key index information subcommands
  record-topology     Record key topology information subcommands
  test-topology       Test key topology maps subcommands
  completions         Generate shell completions
  help                Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...  Verbose mode (-v, -vv, -vvv, etc.)
  -h, --help        Print help information
  -V, --version     Print version information
```
