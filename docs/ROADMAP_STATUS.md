# Roadmap Status

This file tracks execution status for items defined in TODO.md.

Execution backlog details are tracked in [ROADMAP_TASKS.md](ROADMAP_TASKS.md).

## Snapshot (2026-08-08)

- M1 Foundation: In progress
- M2 Expansion: In progress
- M3 Desktop Ecosystem: In progress

## Completed In This Cycle

- [x] Build environment baseline stabilized for workspace checks.
- [x] Workspace compiles with cargo check --workspace in this container.
- [x] GTK3 UI style polish applied in eruption-gui-gtk3.
- [x] Pyroclasm egui compatibility fixes applied.
- [x] Documentation redirect files standardized.
- [x] Canonical docs location clarified in docs/README.md.
- [x] Hardware support planning matrix added.
- [x] New-device bring-up checklist added and linked from CONTRIBUTING.
- [x] SDK compatibility policy documented.
- [x] i18n quality gate added and wired into CI.
- [x] Existing Pyroclasm smoke test retained and validated.
- [x] KDE Plasma widget technical design notes drafted.
- [x] MATE applet technical design notes drafted.
- [x] M2/M3 roadmap split into execution-ready tasks.
- [x] KDE Plasma widget MVP scaffold implemented (template + backend + installer).
- [x] MATE companion indicator MVP scaffold implemented (indicator + installer).
- [x] Headless-friendly Pyroclasm smoke coverage added to CI.
- [x] i18n coverage status document added for current priority locale scope.
- [x] M2-01 hardware batch baseline completed with registry regression tests and docs updates.
- [x] Desktop integrations beta baseline hardened with reconnect/offline handling and validation notes.

## Milestone Checklist

### M1 Foundation

- [x] Hardware support matrix curated from issue demand.
- [x] New-device bring-up checklist committed and referenced by CONTRIBUTING docs.
- [x] SDK docs baseline refreshed with compatibility policy section.
- [x] i18n quality gate automation added in CI.
- [x] Initial GTK3 style refresh and consistency pass.
- [ ] Pyroclasm UX pass with screenshots and manual test notes.

### M2 Expansion

- [x] New hardware support batch shipped with regression tests.
- [x] SDK examples expanded for connect/query/update/error flows.
- [x] Translation expansion for top-priority locales completed.
- [x] CI quality metrics tightened (faster failure signal, clearer logs).

### M3 Desktop Ecosystem

- [x] KDE Plasma widget beta finalized and packaged.
- [x] MATE applet beta finalized and packaged.
- [x] Packaging and install docs finalized for desktop integrations.

## Immediate Next Actions

1. Convert roadmap workstreams into GitHub issues with owners and acceptance criteria.
2. Complete Pyroclasm UX pass with screenshots and manual test notes.
3. Create implementation branches for M3-01 (KDE) and M3-02 (MATE) beta finalization.
