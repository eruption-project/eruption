# Desktop Integrations: Install, Uninstall, and Packaging Notes

This document covers local installation and packaging guidance for desktop integrations.

## Scope

- KDE Plasma widget integration
- MATE companion indicator integration

## Runtime Dependencies

### Common

- `eruptionctl` in `PATH`
- running Eruption daemon

### KDE Plasma

- Plasma desktop with support for Plasma applets
- executable engine support for QML data source

### MATE

- Python 3
- `PyGObject` (`gi` bindings)
- GTK3 runtime

## KDE Plasma: Local Install

From repository root:

```bash
bash support/integrations/kde-plasma/install-local.sh
```

Installed artifacts:

- `~/.local/share/plasma/plasmoids/org.eruption.widget`
- `~/.local/bin/eruption-plasma-backend`

### KDE Local Uninstall

```bash
bash support/integrations/kde-plasma/uninstall-local.sh
```

## MATE: Local Install

From repository root:

```bash
bash support/integrations/mate/install-local.sh
```

Installed artifacts:

- `~/.local/bin/eruption-mate-indicator`
- `~/.config/autostart/eruption-mate-indicator.desktop`

### MATE Local Uninstall

```bash
bash support/integrations/mate/uninstall-local.sh
```

## Troubleshooting

### `eruptionctl` not found

Ensure `eruptionctl` is installed and in `PATH`:

```bash
command -v eruptionctl
```

### Actions do not update state

Check daemon and D-Bus availability:

```bash
eruptionctl status profile
eruptionctl status slot
eruptionctl config brightness
```

### No desktop UI opened from integration

The helpers first try `pyroclasm`, then `eruption-gui-gtk3`.
Install at least one of them or adjust launcher command in helper scripts.

## Packaging Guidance

- Keep integrations optional and independent from daemon package.
- Provide package-specific dependencies explicitly.
- Use post-install notes to mention user-session restart requirements.
- Keep integration versions aligned with Eruption release, while allowing patch-level independent updates.

## Release Checklist (Desktop Integrations)

- [ ] install/uninstall paths verified
- [ ] runtime dependencies validated in clean environment
- [ ] reconnect behavior validated after daemon restart
- [ ] docs links and local scripts smoke-tested

## Validation Notes

Operational validation notes for beta baseline are tracked in:

- `docs/howtos/DESKTOP_INTEGRATIONS_VALIDATION.md`
