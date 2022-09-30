## eruption-hwutil - A CLI control utility for hardware supported by the Eruption Linux user-mode driver

This utility may be used to configure devices without the Eruption daemon required to be running

### eruption-hwutil

```shell
$ sudo eruption-hwutil --help
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
