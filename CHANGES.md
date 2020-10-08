# Changelog

Table of new and noteworthy changes:

| Since  | Description                                                                                                             |
| ------ | ----------------------------------------------------------------------------------------------------------------------- |
| 0.1.17 | __New Release__                                                                                                         |
| 0.1.16 | __New Release__                                                                                                         |
| 0.1.16 | Released a new version of the `Eruption Profile Switcher` GNOME Shell extension; please be sure to update!              |
| 0.1.16 | Add support for ROCCAT Kone Pure Ultra LED                                                                              |
| 0.1.16 | Add a new companion tool `eruption-debug-tool` that may be used to debug USB HID devices                                |
| 0.1.16 | Revert to previous version of the Lua script `shockwave.lua` but with an updated neighbor selection algorithm           |
| 0.1.16 | Added a new Lua script `halo.lua` that shows a rainbow colored animated halo when a key has been pressed                |
| 0.1.15 | __New Release__                                                                                                         |
| 0.1.15 | Released a new version of the `Eruption Profile Switcher` GNOME Shell extension; please be sure to update!              |
| 0.1.15 | Improved robustness of device initialization code                                                                       |
| 0.1.15 | Stopped original key events from leaking through on macro invocations                                                   |
| 0.1.15 | Repaired broken key repetition functionality (on Linux virtual terminals)                                               |
| 0.1.15 | Allow Lua VMs to load additional Lua extension modules at runtime                                                       |
| 0.1.15 | Added support for the new `Network FX` protocol - please see [NETFX.md](./NETFX.md) for further information             |
| 0.1.15 | Added a new Lua script `netfx.lua` implementing the Network FX server                                                   |
| 0.1.15 | Added a new Profile `netfx.profile` that makes use of `netfx.lua`                                                       |
| 0.1.15 | Added a new companion tool `eruption-netfx`, that implements the `Network FX` reference client                          |
| 0.1.15 | Lowered CPU load and power consumption in the spectrum analyzer code                                                    |
| 0.1.14 | __New Release__                                                                                                         |
| 0.1.14 | Improved the spectrum analyzer                                                                                          |
| 0.1.13 | __New Release__                                                                                                         |
| 0.1.13 | Fixed a bug with suspend/resume. Eruption will now be restarted after system wakes up from suspend                      |
| 0.1.13 | Fixed multiple bugs in color handling code that artificially limited the usable color-space                             |
| 0.1.13 | Added the "Hamming" window function to the spectrum analyzer                                                            |
| 0.1.13 | Added a few new profiles based on new Lua scripts: "Color swirls - {Perlin, Turbulence, Voronoi}" and "Flight - Perlin" |
| 0.1.13 | Reduced CPU usage further by approximately 1-2%, when under load (4 Lua VMs @24 fps)                                    |
| 0.1.13 | Reduced CPU usage further, to now be around 0.5% - 1.3%, when idle (no frame generation updates)                        |
| 0.1.12 | Switched from `lua 5.4` to `luajit` (still using `mlua`), to mitigate SIGBUS issues and to improve performance          |
| 0.1.13 | Improve the `eruptionctl` CLI utility                                                                                   |
| 0.1.13 | Fix multiple bugs in the sensors.rs module that surfaced in sysmon.lua                                                  |
| 0.1.13 | Crash the daemon with abort() on a critical error, instead of just deadlocking                                          |
| 0.1.13 | Improved the main loop, use async constructs                                                                            |
| 0.1.12 | __New Release__                                                                                                         |
| 0.1.12 | Switched from `rlua` to `mlua` (now using Lua version 5.4)                                                              |
| 0.1.12 | Beginnings of the CLI tool `eruptionctl`                                                                                |
| 0.1.12 | Added Lua effect-script: wave.lua                                                                                       |
| 0.1.12 | AFK effect: Assign a .profile to show, when the user is AFK (Away From Keyboard)                                        |
| 0.1.12 | __Start of this changelog__                                                                                             |
