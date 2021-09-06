## eruption-process-monitor - A daemon to monitor and introspect system processes and events

A daemon that monitors the system for certain events, and subsequently triggers certain
actions like e.g. switching slots and profiles

### Example usage

```shell
$ eruption-process-monitor rules list
  0: On window focused: Name: '.*YouTube.*Chrome' => Switch to profile: /var/lib/eruption/profiles/spectrum-analyzer-swirl.profile (enabled: true, internal: false)
  1: On window focused: Instance: 'Steam' => Switch to profile: /var/lib/eruption/profiles/gaming.profile (enabled: true, internal: false)
  2: On window focused: Instance: 'vlc' => Switch to profile: /var/lib/eruption/profiles/spectrum-analyzer-swirl.profile (enabled: true, internal: false)
  3: On window focused: Name: 'Skype' => Switch to profile: /var/lib/eruption/profiles/vu-meter.profile (enabled: false, internal: false)
  4: On window focused: Instance: 'totem' => Switch to profile: /var/lib/eruption/profiles/spectrum-analyzer-swirl.profile (enabled: true, internal: false)
  5: On window focused: Instance: '.*' => Switch to profile: /var/lib/eruption/profiles/swirl-perlin-blue-red-dim.profile (enabled: true, internal: true)
```

```shell
$ eruption-process-monitor rules add window-instance '.*vlc.*' /var/lib/eruption/profiles/spectrum-analyzer-swirl.profile
```

```shell
$ eruption-process-monitor rules remove 5
```

### eruption-process-monitor

```shell
$ eruption-process-monitor
eruption-process-monitor 0.0.9

X3n0m0rph59 <x3n0m0rph59@gmail.com>

A daemon to monitor and introspect system processes and events

USAGE:
    eruption-process-monitor [FLAGS] [OPTIONS] [ARGS] <SUBCOMMAND>

ARGS:
    <HOSTNAME>    
    <PORT>        

FLAGS:
    -h, --help       Print help information
    -v, --verbose    Verbose mode (-v, -vv, -vvv, etc.)
    -V, --version    Print version information

OPTIONS:
    -c, --config <CONFIG>    Sets the configuration file to use

SUBCOMMANDS:
    completions    Generate shell completions
    daemon         Run in background and monitor running processes
    help           Print this message or the help of the given subcommand(s)
    rules          Rules related sub-commands
```
