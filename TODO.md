# Project Roadmap

## Preliminary Roadmap

- Widen the scope of the project: Add support for more hardware devices
  Please find the list of most requested hardware here: <https://github.com/X3n0m0rph59/eruption/issues>

## Planned Features

_This is a non-exhaustive listing of planned features:_

- GUI support: Improve the GTK3+ based GUI
- Improve the `Eruption SDK` that allows 3rd party applications to communicate with Eruption
- Improve i18n and l10n: Add more translations
- GUI support: Improve the Pyroclasm UI
- Add a KDE Plasma widget
- Add a MATE Desktop Applet

## Bugs and known Problems

- Wayland support is still lacking: AmbientFx support is currently not available on Wayland-based compositors

## TODO

- keyboard and misc gui pages: make battery/signal controls same as on mice page
- *dim profiles not dimmed
- Improve 2D-primitives drawing/rasterization API
- eruptionctl effects CLI
- Update manpages
- Allocated Zones: Don't poll zones via DBus, use a signal instead?
- Allocated Zones: Add allocated zones support to scripts where applicable
- Fix directory/file permissions in packaging
- Update Python SDK (get_canvas)
- Add Lua event: function on_hotplug(new_device) on_update_zones(...)
- Update all DBus interfaces in rust code
- Undo/redo in GUI?
