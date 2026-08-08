# Desktop Integrations Validation Notes

This document captures validation notes for the desktop integration beta baseline.

## Scope

- KDE Plasma widget backend bridge
- MATE indicator helper
- local install/uninstall scripts

## Performed Checks

- Shell syntax checks:
  - `bash -n support/integrations/kde-plasma/eruption-plasma-backend.sh`
  - `bash -n support/integrations/kde-plasma/install-local.sh`
  - `bash -n support/integrations/kde-plasma/uninstall-local.sh`
  - `bash -n support/integrations/mate/install-local.sh`
  - `bash -n support/integrations/mate/uninstall-local.sh`
- Python syntax check:
  - `python3 -m py_compile support/integrations/mate/eruption-mate-indicator.py`
- Basic execution safety checks by inspection:
  - backend input validation for slot and brightness
  - disconnected/offline state rendering paths in KDE QML and MATE status text
  - reconnect path via periodic refresh and explicit refresh action

## Beta Baseline Coverage

- Profile/slot display: implemented in KDE widget and MATE indicator status line/tooltip.
- Slot switch action: implemented for slots 1-4.
- Brightness adjustment: implemented with constrained numeric input and predefined controls.
- Reconnect/error handling: implemented with explicit offline status, retry/refresh actions, and non-crashing behavior.

## Remaining Manual Validation (Desktop Session)

- KDE Plasma session:
  - verify applet add/remove flow from widget picker
  - verify reconnect after daemon restart
- MATE session:
  - verify autostart on login
  - verify tray/panel behavior after panel restart

## Notes

- These checks are designed to be run in headless CI/container environments for baseline safety.
- Full UX verification still requires real desktop sessions (Plasma and MATE).
