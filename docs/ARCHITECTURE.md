# Table of Contents

- [Table of Contents](#table-of-contents)
  -[Design Overview](#design-overview)
    -[Introduction](#introduction) 
    -[Systems Architecture](#systems-architecture)

## Design Overview

### Introduction

Eruption is a Linux daemon written in the Rust programming language. Eruption consists of a core daemon with an integrated
Lua interpreter, and additional plugin components. Its intended usage is to execute Lua scripts that may react to certain
events on the system like e.g. `Timer tick`, `Key pressed` or `Mouse moved` and subsequently control the connected LED
devices and/or transform the user input via the integrated programmable macro feature.
Eruption plugins may export additional functionality to the Lua scripting engine. Multiple Lua scripts may be run in
parallel, each one in its own VM thread. A Lua script shall compute some kind of effect resulting in a 'color map'.
Each Lua scripts 'submitted color map' will be combined with all other scripts 'submitted color maps' using a compositor
that performs an alpha blending step on each 'color map' before it finally gets sent to the connected LED devices.

### Systems Architecture

Eruption is split into multiple independent processes, `eruption` the core daemon that handles hardware access running
as `root`, and multiple session daemons, most notably `eruption-audio-proxy` that provides audio related functionality
to the core daemon, and `eruption-process-monitor` that is able to automatically switch profiles based on system
usage. Both of the session daemons run as the respective logged-in user.

The different processes communicate using multiple different IPC mechanisms like `D-Bus` and network sockets
(IP and UNIX domain sockets).
