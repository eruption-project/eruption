## eruption-netfx - A Network FX protocol client for the Eruption Linux user-mode driver

Network FX protocol client for the Eruption Linux user-mode driver, supporting ambient effects and much more

### Example usage

```shell
$ eruptionctl switch profile netfx.profile
Switching to profile: /var/lib/eruption/profiles/netfx.profile

$ eruption-netfx "ROCCAT Vulcan Pro TKL" ambient 20
```

```shell
$ eruptionctl switch profile netfx.profile
Switching to profile: /var/lib/eruption/profiles/netfx.profile

$ eruption-netfx command ALL:255:0:0:255
OK
```

### eruption-netfx

```shell
$ eruption-netfx

eruption-netfx 0.2.1

X3n0m0rph59 <x3n0m0rph59@gmail.com>

A Network FX protocol client for the Eruption Linux user-mode driver

USAGE:
    eruption-netfx [FLAGS] [ARGS] <SUBCOMMAND>

ARGS:
    <MODEL>       The keyboard model, e.g. "ROCCAT Vulcan Pro TKL" or "1e7d:311a"
    <HOSTNAME>    
    <PORT>        

FLAGS:
    -h, --help       Print help information
    -v, --verbose    Verbose mode (-v, -vv, -vvv, etc.)
    -V, --version    Print version information

SUBCOMMANDS:
    ambient        Make the LEDs of connected devices reflect what is shown on the screen
    animation      Load image files from a directory and display each one on the connected
                   devices
    command        Send Network FX raw protocol commands to the server
    completions    Generate shell completions
    help           Print this message or the help of the given subcommand(s)
    image          Load an image file and display it on the connected devices
    ping           Ping the server
```
