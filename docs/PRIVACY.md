# Eruption - Privacy Considerations

Eruption does not store any sensitive data by default and does not 'phone home' in any way. Only under the following
circumstances will Eruption store certain usage information:

If you remove the comment sign and therefore enable the `stats.lua` script in a `.profile` file, Eruption will store a
counter (a histogram) of how many times each key has been pressed, in the `/var/lib/eruption/` directory.
This is currently used by the `heatmap.profile` and `heatmap-errors.profile` to color each key depending on usage
frequency. This feature is disabled in the default installation.

### Other privacy sensitive behavior:

#### `eruption-process-monitor` Daemon

The `eruption-process-monitor` daemon listens on a Linux Netlink socket and processes Linux kernel events related to
process creation. Additionally it has multiple ways to query the properties of the currently active Window on X11 and
Wayland. Introspection of process memory is currently not implemented. You can disable processing of Linux kernel
process events and window notifications at any time by running the following command:

```shell
$ systemctl --user disable --now eruption-process-monitor.service
```

This will disable the automatic profile switching mechanism since the `eruption-process-monitor` daemon can't instruct
the `eruption` daemon to switch profiles anymore.

The ambient effect transfers the contents of the screen encoded in the `Network FX` protocol.
You can check whether the ambient effect may be running by searching for a running `eruption-netfx .* ambient` process.

The sensors built into the `eruption-process-monitor` daemon can be configured via certain build flags

#### `eruption-audio-proxy` Daemon

The `eruption-audio-proxy` daemon records audio on the selected device of the `PipeWire` or `PulseAudio` server.
You can disable processing of audio data at any time by running the following command:

```shell
$ systemctl --user disable --now eruption-audio-proxy.service
```

This will disable the support for audio visualizations, mute status indication and sound effects.
