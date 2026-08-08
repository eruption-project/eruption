# Project Roadmap

Execution status for this roadmap is tracked in [docs/ROADMAP_STATUS.md](docs/ROADMAP_STATUS.md).

Current state:
* M1 Foundation: In progress
* M2 Expansion: Planned
* M3 Desktop Ecosystem: Planned

## Scope

This roadmap turns high-level ideas into an execution-ready plan.
It focuses on hardware support, SDK quality, localization, and desktop UX.

## Strategic Goals

* Increase hardware coverage with stable, tested support.
* Make the SDK easier and safer for third-party integrations.
* Improve localization quality and translation coverage.
* Refresh GUI quality in both Pyroclasm and GTK3+ tools.
* Expand desktop integration (KDE Plasma and MATE).

## Delivery Principles

* Keep backward compatibility by default for user-facing behavior and SDK APIs.
* Ship in small increments with clear release notes and migration hints.
* Require tests and docs for every feature accepted into a release milestone.
* Prioritize reliability and observability over feature count.

## Workstreams

### 1. Hardware Enablement

* Build and maintain a prioritized support matrix from user demand.
* Standardize bring-up checklist for new devices: protocol notes, quirks, tests, docs.
* Add automated regression tests for core device classes.
* Track most requested hardware in issues:
  <https://github.com/X3n0m0rph59/eruption/issues>

Definition of done:
* Device is discoverable, configurable, and stable across hotplug and resume paths.
* Documentation is updated in user and developer guides.

### 2. SDK Modernization

* Improve ergonomics of the Eruption SDK for third-party applications.
* Strengthen API contract documentation with clear examples and error semantics.
* Introduce compatibility policy (deprecation windows, semantic versioning expectations).
* Add integration tests that validate common application flows end-to-end.

Definition of done:
* New examples cover at least: connect, query, update profile/effects, error handling.
* SDK changes include changelog entries and migration notes where relevant.

### 3. i18n/l10n Expansion

* Increase translation coverage for high-traffic UI strings first.
* Add translation quality gates (missing keys, placeholder validation, stale entries).
* Establish a contributor flow for translators with review checklist.

Definition of done:
* No missing keys in supported locales for release-critical screens.
* Localization checks run in CI.

### 4. GUI Quality (Pyroclasm + GTK3+)

* Improve visual consistency, spacing, and readability across major screens.
* Reduce interaction friction in commonly used workflows.
* Standardize status and notification patterns.
* Keep changes incremental and consistent with existing visual identity.

Definition of done:
* Critical user journeys require fewer clicks and have clearer state feedback.
* UI updates include screenshots in PRs and basic manual test notes.

### 5. Desktop Integrations

* Add a KDE Plasma widget for key telemetry and quick actions.
* Add a MATE Desktop applet with lightweight status and profile control.
* Reuse backend APIs and transport layers already proven in existing tools.

Definition of done:
* Widget/applet supports startup, reconnect, and error states.
* Packaging and install instructions are documented.

## Milestones

### M1: Foundation

* Support matrix and bring-up checklist finalized.
* SDK docs baseline improved.
* Localization checks prepared.
* Initial GTK3+ style refresh and consistency pass.

### M2: Expansion

* New hardware support batch released with regression coverage.
* SDK examples and compatibility policy published.
* Translation expansion for key locales.
* Pyroclasm UX polish pass.

### M3: Desktop Ecosystem

* KDE Plasma widget beta.
* MATE applet beta.
* Final docs and release packaging pass.

## Operational Backlog (Continuous)

* Remove stale docs and align duplicate pages across top-level and docs/.
* Improve CI signal quality (faster feedback and clearer failure reasons).
* Keep dependencies and build tooling current within compatibility constraints.
* Regularly triage issue backlog and close stale items with reproducibility guidance.

## Note

This file defines strategic direction and delivery structure.
Not all roadmap items are expected to be completed in a single iteration.
