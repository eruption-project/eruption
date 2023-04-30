# Project Roadmap

## Preliminary Roadmap

- Widen the scope of the project: Add support for more hardware devices
  Please find the list of most requested hardware here: <https://github.com/X3n0m0rph59/eruption/issues>

## Planned Features

_This is a non-exhaustive listing of planned features:_

- Improve the `Eruption SDK` that allows 3rd party applications to communicate with Eruption
- Improve i18n and l10n: Add more translations
- GUI support: Improve the Pyroclasm UI
- GUI support: Improve the GTK3+ based GUI
- Add a KDE Plasma widget
- Add a MATE Desktop Applet

## Bugs and known Problems

- Ratelimit all: WARN eruption::events: Not sending a message to a failed tx
- High CPU load while using the GTK3+ based GUI: Maybe use the Eruption SDK for communicating with the daemon instead of DBus?
- "Ambient effect disabled" error message
- Wayland support is still lacking: AmbientFx support is currently not available
