# Project Roadmap

## Preliminary Roadmap

- Widen the scope of the project: Add support for more hardware devices
  Please find the list of most requested hardware here: <https://github.com/X3n0m0rph59/eruption/issues>

## Planned Features

_This is a non-exhaustive listing of planned features:_

- Improve the `Eruption SDK` that allows 3rd party applications to communicate with Eruption
- Improve i18n and l10n: Add more translations
- GUI support: Improve the GTK3+ based GUI
- GUI support: Improve the Pyroclasm UI
- Add a KDE Plasma widget
- Add a MATE Desktop Applet

## Bugs and known Problems

- Wayland support is still lacking: AmbientFx support is currently not available on Wayland-based compositors

## TODO

- Allocated Zones: Don't poll DBus, use a signal instead?
- Ambient FX switch not activated correctly
- Fix directory/file permissions in packages
- Update Python SDK (get_canvas)
- Add allocated zones support to scripts where applicable
- Add Lua event: function on_hotplug(new_device)