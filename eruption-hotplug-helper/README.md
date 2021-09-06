## eruption-hotplug-helper - A utility used to notify Eruption about device hotplug events

A utility used by systemd/udev to notify Eruption about device hotplug events

### Example usage

```shell
$ sudo eruption-hotplug-helper hotplug
 INFO  eruption_hotplug_helper > A hotplug event has been triggered, notifying the Eruption daemon...
 INFO  eruption_hotplug_helper > Waiting for the devices to settle...
 INFO  eruption_hotplug_helper > Done, all devices have settled
 INFO  eruption_hotplug_helper > Now restarting the eruption.service...
 INFO  eruption_hotplug_helper > Waiting for Eruption to be fully operational...
 INFO  eruption_hotplug_helper > Notification sent successfully, exiting now
```

### eruption-hotplug-helper

```shell
$ eruption-hotplug-helper
eruption-hotplug-helper 0.0.6

X3n0m0rph59 <x3n0m0rph59@gmail.com>

A utility used to notify Eruption about device hotplug events

USAGE:
    eruption-hotplug-helper [FLAGS] <SUBCOMMAND>

FLAGS:
    -h, --help       Print help information
    -v, --verbose    Verbose mode (-v, -vv, -vvv, etc.)
    -V, --version    Print version information

SUBCOMMANDS:
    completions    Generate shell completions
    help           Print this message or the help of the given subcommand(s)
    hotplug        Trigger a hotplug event
```
