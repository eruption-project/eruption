## eruption-audio-proxy - Audio proxy daemon for the Eruption Linux user-mode driver

A daemon that reports the state of audio devices like the master volume and the muted state, and delivers an
audio stream to the `Eruption` daemon where it can be processed. E.g.: for consumption by audio visualizer plugins.
Additionally the `eruption-audio-proxy` can play back sound effects, triggered by `Eruption`.

### eruption-audio-proxy

```shell
$ eruption-audio-proxy --help

Audio proxy daemon for the Eruption Linux user-mode driver

Usage: eruption-audio-proxy [OPTIONS] <COMMAND>

Commands:
  daemon       Run in background
  completions  Generate shell completions
  help         Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...       Verbose mode (-v, -vv, -vvv, etc.)
  -c, --config <CONFIG>  Sets the configuration file to use
  -h, --help             Print help information
  -V, --version          Print version information
```
