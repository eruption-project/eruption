# Eruption Lua Support Library

_This document is a work-in-progress draft_

## Overview

Eruption provides a small, but hopefully useful library of functions that are
intended to be used by Lua scripts. Functions can be provided either by the
daemon proper, or by plugins. Plugin specific functions are only available
if the respective plugin is loaded.

## Available Plugins

* Keyboard: Process keyboard events, like e.g. "Key pressed"
* System: Basic system information and status, like e.g. running processes
* Sensors: Query system sensor values, like e.g. CPU package temperature
* Audio: Audio related tasks, like playing sounds
* Introspection: Provides internal status information of the Eruption daemon
* Profiles: Switch profiles based on system state

## Available Functions

Eruption currently ships with the following library functions:

| Name      | Plugin         | Lib | Since | Description                   |
| --------- | -------------- | ------ | ---- | ----------------------------- |
| `trace(message)`    | _core_  | Std  | since before 0.0.9 | Log message with severity: `trace` |
| `debug(message)`    | _core_  | Std  | since before 0.0.9 | Log message with severity: `debug` |
| `info(message)`    | _core_  | Std  | since before 0.0.9 | Log message with severity: `info` |
| `warn(message)`    | _core_  | Std  | since before 0.0.9 | Log message with severity: `warn` |
| `error(message)`    | _core_  | Std  | since before 0.0.9 | Log message with severity: `error` |
| `delay(millis)`    | _core_  | Std  | since before 0.0.9 | Delay script execution for `millis` milliseconds |
| `abs(f) -> f`    | _core_  | Math  | since before 0.0.9 | Returns the absolute value of `f` |
| `sin(a) -> f`    | _core_  | Math  | since before 0.0.9 | Returns the sine of angle `a` |
| `pow(f, p) -> f`    | _core_  | Math  | since before 0.0.9 | Returns `f` to the power of `p` |
| `sqrt(f) -> f`    | _core_  | Math  | since before 0.0.9 | Returns the square root of `f` |
| `rand(l, h) -> f`    | _core_  | Math  | since before 0.0.9 | Returns a random number in the range `l..h` |
| `trunc(f) -> i`    | _core_  | Math  | since before 0.0.9 | Truncate the fractional part of `f` |
| `lerp(f0, f1, f) -> f`    | _core_  | Math  | since 0.0.9 | Linear interpolation of `f` to `f0`..`f1`, where `f` should lie in the range of -1.0..+1.0 |
| `min(f1, f2) -> f`    | _core_  | Math  | since before 0.0.9 | Returns the smaller one of the two values |
| `max(f1, f2) -> f`    | _core_  | Math  | since before 0.0.9 | Returns the greater one of the two values |
| `clamp(f, l, h) -> f`    | _core_  | Math  | since before 0.0.9 | Clamp `f` to range `l..h` |
| `color_to_rgb(color) -> (r, g, b)` | _core_  | Color | since before 0.0.9 | Returns the red, green and blue components of `color` |
| `color_to_hsl(color) -> (h, s, l)` | _core_  | Color | since 0.0.10 | Returns the hue, saturation and lightness components of `color` |
| `rgb_to_color(r, g, b) -> color`    | _core_  | Color  | since before 0.0.9 | Returns a color, constructed fom the r, g and b components |
| `hsl_to_color(h, s, l) -> color`    | _core_  | Color  | since 0.0.9 | Returns a color, constructed fom hue [0..360), saturation [0.0...1.0] and lightness [0.0..0.5] and (0.5..1.0] components |
| `color_to_rgba(color) -> (r, g, b, a)` | _core_  | Color | since 0.0.12 | Returns the red, green blue and alpha components of `color` |
| `color_to_hsla(color) -> (h, s, l, a)` | _core_  | Color | since 0.0.12 | Returns the hue, saturation, lightness and alpha components of `color` |
| `rgba_to_color(r, g, b, a) -> color`    | _core_  | Color  | since 0.0.12 | Returns a color, constructed fom the r, g, b and alpha components |
| `hsla_to_color(h, s, l, a) -> color`    | _core_  | Color  | since 0.0.12 | Returns a color, constructed fom hue [0..360), saturation [0.0...1.0], lightness [0.0..0.5], (0.5..1.0] an dalpha components |
| `linear_gradient(start_color, end_color, p) -> color`    | _core_  | Color  | since before 0.0.9 | Returns the interpolated color at position `p` located between `start_color`..`end_color`. The value of `p` should lie in the range of 0..1 |
| `noise(f1, f2, f3) -> f`    | _core_  | Noise | removed in 0.0.11 | Computes an Open Simplex Noise value |
| `perlin_noise(f1, f2, f3) -> f`    | _core_  | Noise | since 0.0.11 | Computes a Perlin noise value |
| `billow_noise(f1, f2, f3) -> f`    | _core_  | Noise | since 0.0.11 | Computes a Billow noise value |
| `voronoi_noise(f1, f2, f3) -> f`    | _core_  | Noise | since 0.0.11 | Computes a Voronoi noise value |
| `fractal_brownian_noise(f1, f2, f3) -> f`    | _core_  | Noise | since 0.0.11 | Computes a Fractal Brownian Motion noise value |
| `ridged_multifractal_noise(f1, f2, f3) -> f`    | _core_  | Noise | since 0.0.11 | Computes a Ridged Multifractal noise value |
| `open_simplex_noise(f1, f2, f3) -> f`    | _core_  | Noise | since 0.0.11 | Computes an Open Simplex Noise value |
| `get_num_keys() -> i`    | _core_  | Hw  | since before 0.0.9 | Returns the number of keys of the connected device (Approx. 144) |
| `get_key_color(key_index) -> color`    | _core_  | Hw  | since before 0.0.9 | Returns the current color of the key `key_index` |
| `set_key_color(key_index, color)`    | _core_  | Hw  | since before 0.0.9 | Sets the current color of the key `key_index` to `color` |
| `set_color_map([color_map])`    | _core_  | Hw  | since before 0.0.9 | Set all LEDs at once to the colors specified in the array `color_map`. This will directly access the keyboard. Please see also: submit_color_map() |
| `submit_color_map([color_map])`    | _core_  | Hw  | since 0.0.12 | Set all LEDs at once to the colors specified in the array `color_map`. Color maps of all scripts will be alpha blended together. and then sent to the keyboard once for each render frame. |
| `inject_key(ev_key, down)`    | _core_  | Hw  | since 0.1.1 | Inject a key event on the virtual keyboard |
| `get_current_load_avg_1() -> f`    | System  | Sys  | since before 0.0.9 | Returns the system load average of the last 1 minute |
| `get_current_load_avg_5() -> f`    | System  | Sys  | since before 0.0.9 | Returns the system load average of the last 5 minutes |
| `get_current_load_avg_10() -> f`    | System  | Sys  | since before 0.0.9 | Returns the system load average of the last 10 minutes |
| `get_runnable_tasks() -> i`    | System  | Sys  | since before 0.0.9 | Returns the number of runnable tasks on the system |
| `get_total_tasks() -> i`    | System  | Sys  | since before 0.0.9 | Returns the total number of tasks on the system |
| `get_package_temp() -> f`    | Sensors  | Hw  | since before 0.0.9 | Returns the temperature of the CPU package |
| `get_package_max_temp() -> f`    | Sensors  | Hw  | since before 0.0.9 | Returns the max. temperature of the CPU package. (Approx. 80-100°C) |
| `get_mem_total_kb() -> i`    | Sensors  | Hw  | since before 0.0.9 | Returns the total installed memory size |
| `get_mem_used_kb() -> i`    | Sensors  | Hw  | since before 0.0.9 | Returns the amount of used memory |
| `get_swap_total_kb() -> i`    | Sensors  | Hw  | since before 0.0.9 | Returns the total size of the swap space |
| `get_swap_used_kb() -> i`    | Sensors  | Hw  | since before 0.0.9 | Returns the amount of used swap space |
| `get_audio_loudness() -> i`    | Audio | dsp  | since 0.0.11 | Returns the current RMS loudness of the configured audio input |
| `get_audio_spectrum() -> [f]`    | Audio | dsp  | since 0.0.11 | Returns a vector of 1024 floats, containing results of a fourier transform (FFT) of the configured audio input |
| `get_audio_raw_data() -> [i]`    | Audio | dsp  | since 0.0.11 | Returns a buffer of 16-bit wide signed integer values, containing samples from the configured audio input |
_Non-exhaustive, more documentation coming soon_

Please Note:
* All color values should lie in the range [0..255] unless otherwise specified
* Some API calls currently involve bringing up of threads for each call,
	especially the `System` and `Sensor` ones, so better don't call them in a
	tight loop

## Available Callback Functions (Events)

Eruption currently calls the following event handler functions, if they are present in a Lua script:

| Name        | Plugin  | Parameters | Description                   |
| ----------- | ------- | ------     | ----------------------------- |
| `on_startup`  | _core_  | _n/a_    | Sent on startup, e.g. when a script is loaded |
| `on_quit`     | _core_  | _n/a_    | Sent on daemon exit |
| `on_tick(delta)`     | _core_  | delta: Timer delta since last tick |  |
| `on_key_down(key_index)` | _core_  | key_index: Key index (column major order) |  |
| `on_key_up(key_index)` | _core_  | key_index: Key index (column major order) |  |
Exhaustive listing of all currently available event callbacks

## Example Code

The following code will change a key's color to `bright red` after it has been
pressed.

#### Listing 01
```lua

-- global array that stores each key's current color
color_map = {}

function on_startup()
    -- turn off all key LEDs
    for i = 0, get_num_keys() do
        color_map[i] = rgba_to_color(0, 0, 0, 0)
    end

    -- update keyboard LED state
    submit_color_map(color_map)
end

function on_key_down(key_index)
    info("Pressed key: " .. key_index)

    -- set color of pressed key to red
    color_map[key_index] = rgb_to_color(255, 0, 0, 255)
    submit_color_map(color_map)
end
```

Please see the directories `src/scripts/` and `src/scripts/examples/` for further information.
