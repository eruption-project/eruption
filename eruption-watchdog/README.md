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
$ eruption-watchdog --help
A watchdog daemon for Eruption

Usage: eruption-watchdog [OPTIONS] <COMMAND>

Commands:
  daemon       Run watchdog daemon for Eruption
  completions  Generate shell completions
  help         Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...  Verbose mode (-v, -vv, -vvv, etc.)
  -h, --help        Print help information
  -V, --version     Print version information
```
