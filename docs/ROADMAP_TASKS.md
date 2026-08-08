# Roadmap Tasks (Execution Backlog)

This file breaks down M2 and M3 roadmap items into implementation-ready tasks.

## M2 Expansion

### M2-01 Hardware Batch: Priority Devices

- Scope: implement and validate one priority batch from HARDWARE_SUPPORT_MATRIX.md.
- Deliverables: driver improvements, regression tests, docs update in DEVICES.md.
- Suggested owner: hardware subsystem maintainer.
- Acceptance:
  - no critical regressions on existing supported devices
  - hotplug and resume path validated
  - release notes include support scope and limitations

### M2-02 SDK Examples and Migration Baseline

- Scope: extend SDK examples for connect/query/update/error handling flows.
- Deliverables: examples in sdk/examples and docs references.
- Suggested owner: SDK maintainer.
- Acceptance:
  - all documented examples compile/run in CI where applicable
  - migration guidance added for changed APIs

Status: completed (connect/query/update/error flow examples and docs baseline added)

### M2-03 Translation Expansion

- Scope: close high-priority locale gaps for release-critical UI strings.
- Deliverables: updated `*.ftl` files and parity check passing.
- Suggested owner: i18n coordinator.
- Acceptance:
  - key parity check passes across en-US/de-DE files
  - no missing keys in critical paths

### M2-04 CI Signal Quality

- Scope: reduce noise and improve actionable failure diagnostics.
- Deliverables: refined workflow steps, clearer logs, reduced flaky jobs.
- Suggested owner: CI/build maintainer.
- Acceptance:
  - failures identify root area (build, docs, i18n, tests)
  - CI runtime remains acceptable for PR workflow

Status: completed (preflight/docs+i18n, static analysis, and workspace build phases split in CI)

## M3 Desktop Ecosystem

### M3-01 KDE Plasma Widget Beta

- Scope: implement widget based on docs/howtos/KDE_PLASMA_WIDGET_PLAN.md.
- Deliverables: widget source, packaging instructions, validation notes.
- Suggested owner: desktop integration maintainer.
- Acceptance:
  - profile/slot display
  - slot switch action
  - brightness adjustment
  - reconnect/error handling

### M3-02 MATE Applet Beta

- Scope: implement applet based on docs/howtos/MATE_APPLET_PLAN.md.
- Deliverables: applet source, install instructions, validation notes.
- Suggested owner: desktop integration maintainer.
- Acceptance:
  - profile/slot display
  - slot switch action
  - brightness adjustment
  - reconnect/error handling

### M3-03 Packaging and Distribution

- Scope: package KDE and MATE integrations as optional artifacts.
- Deliverables: packaging manifests/scripts and installation docs.
- Suggested owner: packaging maintainer.
- Acceptance:
  - clean install/uninstall path
  - documented runtime dependencies
  - versioning strategy documented

Status: completed (local-install and documentation baseline delivered)

## Sequencing Recommendation

1. M2-03 i18n expansion baseline.
2. M2-01 hardware batch.
3. M2-02 SDK examples.
4. M2-04 CI signal refinement.
5. M3-01 KDE beta.
6. M3-02 MATE beta.
7. M3-03 packaging finalization.
