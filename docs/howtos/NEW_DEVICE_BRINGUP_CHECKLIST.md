# New Device Bring-Up Checklist

Use this checklist when adding support for a new device family.

## Discovery and Scoping

- [ ] Confirm user demand and open a tracking issue.
- [ ] Capture USB identifiers, transport path, and firmware details.
- [ ] Define target support level: experimental, testing, or stable.

## Driver Implementation

- [ ] Implement probing and safe capability detection.
- [ ] Implement base control path (connect, read state, write effects/config).
- [ ] Add robust error handling for disconnect, resume, and permission failures.
- [ ] Add logging fields needed for debugging in production.

## Functional Validation

- [ ] Verify device discoverability at daemon startup.
- [ ] Verify hotplug add/remove behavior.
- [ ] Verify profile switching and global brightness behavior.
- [ ] Verify expected per-device features from capability matrix.
- [ ] Run long-duration stability check (no lockups, no input spam, no crash loop).

## Test Coverage

- [ ] Add or extend unit tests for parsing/protocol logic.
- [ ] Add regression tests for previously fixed bugs in similar devices.
- [ ] Validate no regressions in existing hardware classes.

## Documentation and Release Readiness

- [ ] Update DEVICES.md capability and status sections.
- [ ] Add caveats/known limitations for incomplete features.
- [ ] Add changelog entry with support level and version scope.
- [ ] Attach manual validation notes to PR.

## Promotion Criteria

To move from experimental to stable:

- [ ] No critical defects across at least one full release cycle.
- [ ] Feature behavior aligns with documented scope.
- [ ] Reconnect and resume paths are proven stable.