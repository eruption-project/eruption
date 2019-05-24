# Eruption Lua Support Library

_This document is a work-in-progress draft_

## Overview

Eruption provides a small, but hopefully useful library of functions that are intended to be used by Lua scripts. Functions can be provided either by the daemon proper, or by plugins. Plugin specific functions are only available if the respective plugin is loaded.

## Available Plugins

* DBUS: Provide support for a DBUS API
* Keyboard: Process keyboard events, like e.g. "Key pressed"
* System: Basic system information and status, like e.g. running processes
* Sensors: Query system sensor values, like e.g. CPU package temperature
* Audio: Audio related tasks, like playing sounds

## Available Functions

Eruption currently ships with the following library functions:

| Name      | Plugin         | Lib | Description                   |
| --------- | -------------- | ------ | ----------------------------- |
| `min(f1, f2) -> f`    | _core_  | Math  | Returns the smaller of the two values |
| `max(f1, f2) -> f`    | _core_  | Math  | Returns the greater of the two values |
| `clamp(val, l, h) -> f`    | _core_  | Math  | Clamp val to range l..h |
| ...    | |   | ... |
_Non-exhaustive, more documentation coming soon_

## Available Callback Functions (Events)

Eruption currently calls the following event handler functions, if they are present in a Lua script:

| Name        | Plugin  | Parameters | Description                   |
| ----------- | ------- | ------     | ----------------------------- |
| `on_startup`  | _core_  | _n/a_ |  |
| `on_quit`     | _core_  | _n/a_ |  |
| `on_tick(delta)`     | _core_  | delta: Timer delta since last tick |  |
| `on_key_down(key_index)` | _core_  | key_index: Key index (column major order) |  |
Exhaustive listing of all currently available event callbacks

## Example Code

The following code will light up a key, and turn its color to bright red after it was pressed.

#### Listing 01
```lua

-- global array that stores each key's current color
color_map = {}

function on_startup()
    -- turn off all key LEDs
    for i = 0, get_num_keys() do
        color_map[i] = rgb_to_color(0, 0, 0)
    end

    -- update keyboard LED state
    set_color_map(color_map)
end

function on_key_down(key_index)
    info("Pressed key: " .. key_index)

    -- set color of pressed key to red
    color_map[key_index] = rgb_to_color(255, 0, 0)
    set_color_map(color_map)
end
```

Please see the directories `src/scripts/` and `src/scripts/examples/` for further information.
