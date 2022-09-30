## eruption-debug-tool - A CLI utility to debug USB HID devices

A helper utility that can be used to debug USB HID devices

### Example usage

```shell
 $ sudo eruption-debug-tool list

 Please stop the Eruption daemon prior to running this tool:
 $ sudo systemctl stop eruption.service && sudo systemctl mask eruption.service

 You can re-enable Eruption with this command afterwards:
 $ sudo systemctl unmask eruption.service && sudo systemctl start eruption.service


Please find the device you want to debug below and use its respective
index number (column 1) as the device index for the other sub-commands of this tool

Index: 00: ID: 1e7d:2dd2 ROCCAT/ROCCAT Kone Pure Ultra Subdev: 0
Index: 01: ID: 1e7d:2dd2 ROCCAT/ROCCAT Kone Pure Ultra Subdev: 1
Index: 02: ID: 1e7d:2dd2 ROCCAT/ROCCAT Kone Pure Ultra Subdev: 2
Index: 03: ID: 1e7d:2dd2 ROCCAT/ROCCAT Kone Pure Ultra Subdev: 3
Index: 04: ID: 1e7d:311a ROCCAT/ROCCAT Vulcan Pro TKL Subdev: 0
Index: 05: ID: 1e7d:311a ROCCAT/ROCCAT Vulcan Pro TKL Subdev: 1
Index: 06: ID: 1e7d:311a ROCCAT/ROCCAT Vulcan Pro TKL Subdev: 2
Index: 07: ID: 1e7d:311a ROCCAT/ROCCAT Vulcan Pro TKL Subdev: 3
Index: 08: ID: 1e7d:3a37 Turtle Beach/Elo 7.1 Air Subdev: 0
Index: 09: ID: 1e7d:3a37 Turtle Beach/Elo 7.1 Air Subdev: 5

Enumeration of HID devices completed

Special devices

Index: 255: Serial Port 1 (/dev/ttyACM0)
Index: 254: Serial Port 2 (/dev/ttyACM1)
Index: 253: Serial Port 3 (/dev/ttyACM2)
Index: 252: Serial Port 4 (/dev/ttyACM3)
```

```shell
$ sudo eruption-debug-tool state-diff 0

 Please stop the Eruption daemon prior to running this tool:
 $ sudo systemctl stop eruption.service && sudo systemctl mask eruption.service

 You can re-enable Eruption with this command afterwards:
 $ sudo systemctl unmask eruption.service && sudo systemctl start eruption.service

Index: 00: ID: 1e7d:2dd2 ROCCAT/ROCCAT Kone Pure Ultra Subdev: 0
Reading data from device...
The following USB HID report IDs have changed bytes:

Saving state data...
Done
```

```shell
$  sudo eruption-debug-tool state-diff 0

 Please stop the Eruption daemon prior to running this tool:
 $ sudo systemctl stop eruption.service && sudo systemctl mask eruption.service

 You can re-enable Eruption with this command afterwards:
 $ sudo systemctl unmask eruption.service && sudo systemctl start eruption.service

Index: 00: ID: 1e7d:2dd2 ROCCAT/ROCCAT Kone Pure Ultra Subdev: 0
Reading data from device...
The following USB HID report IDs have changed bytes:

Changed bytes: [80]
0x04: [0x04, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x03, 0x00, 0xef, 0xe7, 0x01, 0x00, 0x00, 0x00, 0x90, 0x93, 0x00, 0x00, 0x94, 0x93, 0x00, 0x00, 0xa4, 0x93, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x09, 0x01, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00, 0x20, 0x00, 0x00, 0x00, 0x23, 0x00, 0x00, 0x00, 0x6a, 0x88, 0x78, 0x00, 0x00=>0x01, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0xff, 0x00, 0xff, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0xff, 0x00, 0x00, 0x00, 0x05, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x34, 0x00, ]

Changed bytes: [75]
0x06: [0x06, 0x3f, 0x00, 0x06, 0x06, 0x1f, 0x01, 0x08, 0x00, 0x10, 0x00, 0x18, 0x00, 0x20, 0x00, 0x40, 0x00, 0x08, 0x00, 0x10, 0x00, 0x18, 0x00, 0x20, 0x00, 0x40, 0x00, 0x00, 0x00, 0x03, 0x09, 0x06, 0xff, 0x0f, 0x00, 0x00, 0x14, 0xff, 0xff, 0x00, 0x00, 0x14, 0xff, 0x00, 0x48, 0xff, 0x14, 0xff, 0x00, 0x48, 0xff, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0xf7, 0x09, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00=>0x88, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, ]

Saving state data...
Done
```

```shell
$  sudo eruption-debug-tool run-tests 0

 Please stop the Eruption daemon prior to running this tool:
 $ sudo systemctl stop eruption.service && sudo systemctl mask eruption.service

 You can re-enable Eruption with this command afterwards:
 $ sudo systemctl unmask eruption.service && sudo systemctl start eruption.service

Index: 00: ID: 1e7d:2dd2 ROCCAT/ROCCAT Kone Pure Ultra Subdev: 0
Bound driver: ROCCAT Kone Pure Ultra
Sending device init sequence...
Step 2
Sending control device feature report
  |0e060101 00ff|                       ......           00000000
                                                         00000006
Waiting for control device to respond...
  |04010000|                            ....             00000000
                                                         00000004
Step 3
Sending control device feature report
  |0d0b0000 00000000 000000|            ...........      00000000
                                                         0000000b
Waiting for control device to respond...
  |04030000|                            ....             00000000
                                                         00000004
  |04010000|                            ....             00000000
                                                         00000004
Setting LEDs from supplied map...
  |0d0bff00 00000000 000000|            ...........      00000000
                                                         0000000b
Setting LEDs from supplied map...
  |0d0b0000 ff000000 000000|            ...........      00000000
                                                         0000000b

```

### eruption-debug-tool

```shell
$ eruption-debug-tool --help
A CLI utility to debug USB HID devices

Usage: eruption-debug-tool [OPTIONS] <COMMAND>

Commands:
  list         List available devices, use this first to find out the index of the device to use
  report       Generate a report for the specified device
  trace        Dump a trace of events originating from the specified device (May hang the device)
  state-diff   Read out the device state and show differences to previous state (May hang the device)
  read         Read a single USB HID feature report from device
  write        Send a single USB HID feature report to device (dangerous)
  read-raw     Read data from device
  write-raw    Send data to device (dangerous)
  run-tests    Send a device specific init sequence and try to set colors
  utils        Special utility functions, like searching for CRC polynoms and parameters
  completions  Generate shell completions
  help         Print this message or the help of the given subcommand(s)

Options:
  -v, --verbose...  Verbose mode (-v, -vv, -vvv, etc.)
  -h, --help        Print help information
  -V, --version     Print version information
```
