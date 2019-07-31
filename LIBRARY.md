# Eruption Lua Support Library

_This document is a work-in-progress draft_

## Overview

Eruption provides a small, but hopefully useful library of functions that are intended to be used by Lua scripts. Functions can be provided either by the daemon proper, or by plugins. Plugin specific functions are only available if the respective plugin is loaded.

## Available Plugins

* Keyboard: Process keyboard events, like e.g. "Key pressed"
* System: Basic system information and status, like e.g. running processes
* Sensors: Query system sensor values, like e.g. CPU package temperature
* Audio: Audio related tasks, like playing sounds
* Introspection: Provides internal status information of the Eruption daemon
* Profiles: Switch profiles based on system state

## Available Functions

Eruption currently ships with the following library functions:

| Name      | Plugin         | Lib | Description                   |
| --------- | -------------- | ------ | ----------------------------- |
| `trace(message)`    | _core_  | Std  | Log message with severity: `trace` |
| `debug(message)`    | _core_  | Std  | Log message with severity: `debug` |
| `info(message)`    | _core_  | Std  | Log message with severity: `info` |
| `warn(message)`    | _core_  | Std  | Log message with severity: `warn` |
| `error(message)`    | _core_  | Std  | Log message with severity: `error` |
| `delay(message)`    | _core_  | Std  | Delay script execution for millis milliseconds |
| `abs(f) -> f`    | _core_  | Math  | Returns the absolute value of `f` |
| `sin(a) -> f`    | _core_  | Math  | Returns the sine of angle `a` |
| `pow(f, p) -> f`    | _core_  | Math  | Returns `f` to the power of `p` |
| `sqrt(f) -> f`    | _core_  | Math  | Returns the square root of `f` |
| `rand(l, h) -> f`    | _core_  | Math  | Returns a random number in the range `l..h` |
| `trunc(f) -> i`    | _core_  | Math  | Truncate the fractional part of `f` |
| `min(f1, f2) -> f`    | _core_  | Math  | Returns the smaller one of the two values |
| `max(f1, f2) -> f`    | _core_  | Math  | Returns the greater one of the two values |
| `clamp(f, l, h) -> f`    | _core_  | Math  | Clamp `f` to range `l..h` |
| `color_to_rgb(color) -> (b,b,b)` | _core_  | Color | Returns the red, green and blue component of `color` |  
| `rgb_to_color(r,g,b) -> color`    | _core_  | Color  | Returns a color, constructed fom r, g and b components |
| `linear_gradient(start_color, end_color, p) -> color`    | _core_  | Color  | Returns the interpolated color at position `p` located between `start_color`..`end_color`. The value of `p` should lie in the range of 0..1 |
| `get_num_keys() -> i`    | _core_  | Hw  | Returns the number of keys of the connected device (Approx. 144) |
| `get_key_color(key_index) -> color`    | _core_  | Hw  | Returns the current color of the key `key_index` |
| `set_key_color(key_index, color)`    | _core_  | Hw  | Sets the current color of the key `key_index` to `color` |
| `set_color_map([color_map])`    | _core_  | Hw  | Set all LEDs at once, to the colors specified in the array `color_map` |
| `get_current_load_avg_1() -> f`    | System  | Sys  | Returns the system load average of the last n minutes |
| `get_current_load_avg_5() -> f`    | System  | Sys  | Returns the system load average of the last n minutes |
| `get_current_load_avg_10() -> f`    | System  | Sys  | Returns the system load average of the last n minutes |
| `get_runnable_tasks() -> i`    | System  | Sys  | Returns the number of runnable tasks on the system |
| `get_total_tasks() -> i`    | System  | Sys  | Returns the total number of tasks on the system |
| `get_package_temp() -> f`    | Sensors  | Hw  | Returns the temperature of the CPU package |
| `get_package_max_temp() -> f`    | Sensors  | Hw  | Returns the max. temperature of the CPU package. (Approx. 80-100°C) |
| `get_mem_total_kb() -> i`    | Sensors  | Hw  | Returns the total installed memory size |
| `get_mem_used_kb() -> i`    | Sensors  | Hw  | Returns the amount of used memory |
| `get_swap_total_kb() -> i`    | Sensors  | Hw  | Returns the total size of the swap space |
| `get_swap_used_kb() -> i`    | Sensors  | Hw  | Returns the amount of used swap space |
| ...    | |   | ... |
_Non-exhaustive, more documentation coming soon_

## Available Callback Functions (Events)

Eruption currently calls the following event handler functions, if they are present in a Lua script:

| Name        | Plugin  | Parameters | Description                   |
| ----------- | ------- | ------     | ----------------------------- |
| `on_startup`  | _core_  | _n/a_    | Sent on daemon startup |
| `on_quit`     | _core_  | _n/a_    | Sent on daemon exit |
| `on_tick(delta)`     | _core_  | delta: Timer delta since last tick |  |
| `on_key_down(key_index)` | _core_  | key_index: Key index (column major order) |  |
Exhaustive listing of all currently available event callbacks

## Example Code

The following code will change a key's color to bright red after it was pressed.

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
