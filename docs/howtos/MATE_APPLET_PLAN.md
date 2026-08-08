# MATE Applet Plan

This document defines the baseline plan for an Eruption applet in MATE.

## Scope

Provide lightweight panel integration with minimal dependencies:

- active profile/slot indicator
- quick slot switching
- brightness adjustment
- launch full UI action

## Architecture

- Frontend: MATE panel applet (GTK-based)
- Backend transport: D-Bus API shared with existing Eruption tools
- Update strategy: startup read, timer refresh, manual refresh

## Failure Handling

- D-Bus unavailable: show disconnected status and retry action
- Runtime error: keep applet responsive and expose error tooltip
- Permission issues: actionable hint in applet menu

## Minimum Viable Feature Set (Beta)

- Status indicator (profile + slot)
- Slot switching menu
- Brightness control dialog/popover
- Open Pyroclasm action

## Packaging Notes

- Package separately from core daemon
- Keep dependencies focused on MATE panel + GTK stack
- Include service and desktop integration notes in docs

## Validation Checklist

- Applet loads at login and survives panel restart
- D-Bus reconnect works after daemon restart
- Slot/brightness actions reflect in running daemon
- Degraded mode remains usable when daemon is offline
