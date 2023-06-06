# Project Roadmap

## Roadmap

- Widen the scope of the project: Add support for more hardware devices
  Please find the list of most requested hardware here: <https://github.com/X3n0m0rph59/eruption/issues>
- Update the `develop` branch, merge `unified-canvas` as soon as it is ready

### TODO before merging the `unified-canvas` branch into the `develop` branch

- Wherever possible, convert handling of mem-copies and per-pixel routines from Lua code to Rust, push the inner loop down into Rust code and use high-level graphics primitives instead. Having the inner loop in Lua code becomes infeasible with higher canvas resolutions.
  Convert all existing effects scripts to make use of the hwaccel API (GPU acceleration) or at least use the
  2D-rasterization library (rasterops plugin)

### Status of the Eruption porting effort to Microsoft Windows

- WIP support code for Microsoft Windows has been merged recently to the `unified-canvas` branch
- Cross-compilation on Linux host (in a Fedora 38 `podman` container) for Windows x86_64 is up and running
- Successful compilation of selected binaries has been achieved
- Deployment to Windows via an NSIS-based installer binary is working
- Eruption daemon is able to drive LED lighting on Windows; most devices that are supported under Linux work as well
- Eruption GTK+3 GUI is able to start but is currently unable to connect to the Eruption daemon
- There is no support for handling of input: No macros, no remapping, no event handling; only lighting effects

## Planned Features

_This is a non-exhaustive listing of planned features:_

- GUI support: Improve the GTK+3 based GUI
- Improve i18n and l10n: Add more translations
- Improve the `Eruption SDK` that allows 3rd party applications to communicate with Eruption
- Add a KDE Plasma widget
- Add a MATE Desktop Applet
- GUI support: Improve the Pyroclasm UI

## Bugs and known Problems

- Wayland support is still lacking: The Ambient-Effect support is currently not available on Wayland-based compositors,
  since screenshot APIs are not fully settled on the Wayland side

## TODO

- Update all manpages
- Update all DBus interfaces in rust code
- keyboard and misc gui pages: make battery/signal-strength controls same as on mice page
- Improve 2D-primitives drawing/rasterization API
- Improve hardware acceleration with Vulkan/WebGPU (GPGPU)
- eruptionctl: Add new `zones` subcommand to define zone allocations
- eruptionctl: implement effects CLI
- Allocated Zones: Don't poll zones via DBus, use a signal instead?
- Allocated Zones: Add allocated zones support to scripts where applicable
- Add Lua event: function on_hotplug(new_device) on_update_zones(...)
- Fix directory/file permissions in packaging (for the `eruption` user)
- Update Python SDK (get_canvas)
- Implement Undo/redo ops in the GUI?
- Simplify handling of internal data structures for representation of devices; maybe use a slotmap of Box<dyn Device> instead of integer-indices
- Improve wording of identifiers: e.g: LED_MAP -> CANVAS
- System Plugin: Add time of day API
- Improve driver code: provide default implementations and routines
- Add LED count, description, position, and names to each driver
- Add type of device to capabilities: HID or RAWUSB
- Device pixel format: RGB BGR LRGB in capabilities
- Device endpoint in capabilities
- Add shutdown routines to all drivers, to put devices into a known good state on exit of Eruption
