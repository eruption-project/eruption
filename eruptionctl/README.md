## eruptionctl - A CLI control utility for the Eruption Linux user-mode driver

This is the command line interface to the Eruption daemon

### Example usage

```shell
$ eruptionctl switch profile swirl-perlin-blue-red-dim.profile 
```

```shell
$ eruptionctl switch slot 4
```

```shell
$ eruptionctl config brightness 100
```

```shell
$ eruptionctl status profile
Current profile: /var/lib/eruption/profiles/swirl-perlin-blue-red-dim.profile
```

```shell
$ eruptionctl devices debounce 1
Selected device: ROCCAT Kone Pure Ultra (1)
Debounce: true
```

```shell
$ eruptionctl devices brightness 1 20
Selected device: ROCCAT Kone Pure Ultra (1)
```

### eruptionctl

```shell
$ eruptionctl --help
A CLI control utility for the Eruption Linux user-mode driver

Usage: eruptionctl [OPTIONS] <COMMAND>

Commands:
  config         Configuration related sub-commands
  color-schemes  Define, import or delete a named color scheme
  devices        Get or set some device specific configuration parameters
  status         Shows the currently active profile or slot
  switch         Switch to a different profile or slot
  profiles       Profile related sub-commands
  names          Naming related commands such as renaming of profile slots
  scripts        Script related sub-commands
  param          Get or set script parameters on the currently active profile
  completions    Generate shell completions
  help           Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...       Verbose mode (-v, -vv, -vvv, etc.)
  -r, --repeat           Repeat output until ctrl+c is pressed
  -c, --config <CONFIG>  Sets the configuration file to use
  -h, --help             Print help information
  -V, --version          Print version information
```
