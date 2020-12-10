# Network FX Protocol Specification

The Network FX protocol aims to be a simple and efficient protocol, used to assign colors to zones on the unified canvas. The Network FX server listens for commands on a TCP socket. Commands simply consist of 5 fields, each separated by a colon character (`:`)

The first part of a command specifies a comma separated list, either a single index into the led map (_"the canvas"_), or a zone of indices into the led map. Indices are numbered in column-major order, meaning that they are counted column-wise starting from top to bottom and from left to right.

The following four parts of a command specify the components of the desired color, including an alpha channel. The order of the components is: First red, then green, then blue and finally the alpha channel.

# Reference

## Command Syntax

**ZONE:RED:GREEN:BLUE:ALPHA**

**ZONE** can be one of: ALL, N-M or N, where N and M are integers

**RED, GREEN, BLUE, ALPHA**: Integers in the range [0..255]

## Examples

Paint the full canvas red: **ALL:255:0:0:255**

Set ESC key on the keyboard to white: **1:255:255:255:255**

Set F1-F3 keys on the keyboard to red: **12,18,24:255:0:0:255**

Set center of the keyboard to blue: **23-59:0:0:255:255**

Set "mouse zone" of the canvas to green: **144-180:0:255:00:255**

## Commands

The following commands are currently supported:

* STATUS: Returns server specific infos and status
* QUIT: Terminates the TCP connection to the server

## Error Codes

On each successful command execution, the Network FX server replies with: "OK".

The Network FX server replies with the following error codes in case that an internal error occurred:

| Server Reply | Caused by                                            |
| ------------ | ---------------------------------------------------- |
| ERROR: 100   | Ill-formed request                                   |
| ERROR: 110   | Color component value out of range                   |
| ERROR: 120   | Invalid index into the canvas (_index out of range_) |
