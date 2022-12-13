## eruption-hotplug-helper - A utility used to notify Eruption about device hotplug events

A utility used by systemd/udev to notify Eruption about device hotplug events

### Example usage

```shell
$ sudo eruption-hotplug-helper hotplug
 INFO  eruption_hotplug_helper > A hotplug event has been triggered, notifying the Eruption daemon...
 INFO  eruption_hotplug_helper > Waiting for the devices to settle...
 INFO  eruption_hotplug_helper > Done, all devices have settled
 INFO  eruption_hotplug_helper > Connecting to the Eruption daemon...
 INFO  eruption_hotplug_helper > Notifying the Eruption daemon about the hotplug event...
 INFO  eruption_hotplug_helper > Disconnected from the Eruption daemon
```

### eruption-hotplug-helper

```shell
$ eruption-hotplug-helper --help

Eruption is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

Eruption is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with Eruption.  If not, see <http://www.gnu.org/licenses/>.

Copyright (c) 2019-2023, The Eruption Development Team

A utility used to notify Eruption about device hotplug events

Usage: eruption-hotplug-helper [OPTIONS] <COMMAND>

Commands:
  hotplug  Trigger a hotplug event
  help     Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...  Verbose mode (-v, -vv, -vvv, etc.)
  -h, --help        Print help information
  -V, --version     Print version information

```
