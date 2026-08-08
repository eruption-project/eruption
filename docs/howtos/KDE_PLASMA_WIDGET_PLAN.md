# KDE Plasma Widget Plan

This document defines the technical baseline for a KDE Plasma widget integration.

## Scope

Provide lightweight desktop access to Eruption state and quick actions:

- active profile
- active slot
- global brightness
- quick profile/slot switch actions

## Architecture

- Frontend: Plasma widget (QML)
- Transport: D-Bus session/system API already used by Eruption tools
- State flow: poll on startup, then periodic refresh, plus manual refresh action

## Failure Handling

- Backend unavailable: show warning state and reconnect action
- Permission error: show explicit message and troubleshooting hint
- Timeout: keep last known state with stale marker

## Minimum Viable Feature Set (Beta)

- Show active profile and slot
- Change slot (F1-F4 equivalent)
- Set global brightness slider
- Open full UI action (launch `pyroclasm` if installed)

## Packaging Notes

- Target packaging as optional artifact, independent of daemon packages
- Keep runtime deps minimal (Plasma + Qt stack only)
- Version widget alongside core release but allow independent updates

## Validation Checklist

- Startup with daemon running and daemon stopped
- Reconnect after daemon restart
- State updates reflected within refresh interval
- No crash loop on malformed D-Bus responses

## Current Implementation Baseline

- Widget template UI: support/templates/kde-plasma-widget/contents/ui/main.qml
- Local backend bridge: support/integrations/kde-plasma/eruption-plasma-backend.sh
- Local installer helper: support/integrations/kde-plasma/install-local.sh
