# Changelog

Table of new and noteworthy changes:

| Since  | Description                                                                                                             |
| ------ | ----------------------------------------------------------------------------------------------------------------------- |
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
