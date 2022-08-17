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
$ eruptionctl

eruptionctl 0.0.21

X3n0m0rph59 <x3n0m0rph59@gmail.com>

A CLI control utility for the Eruption Linux user-mode driver

USAGE:
    eruptionctl [FLAGS] [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Print help information
    -v, --verbose    Verbose mode (-v, -vv, -vvv, etc.)
    -V, --version    Print version information

OPTIONS:
    -c, --config <CONFIG>    Sets the configuration file to use

SUBCOMMANDS:
    color-schemes  Define, import or delete a named color scheme
    completions    Generate shell completions
    config         Configuration related sub-commands
    devices        Get or set some device specific configuration parameters
    help           Print this message or the help of the given subcommand(s)
    names          Naming related commands such as renaming of profile slots
    param          Get or set script parameters on the currently active profile
    profiles       Profile related sub-commands
    scripts        Script related sub-commands
    status         Shows the currently active profile or slot
    switch         Switch to a different profile or slot
```
