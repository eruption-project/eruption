# Hardware Support Matrix

This matrix is used to prioritize hardware enablement work from user demand and support risk.
It complements the capability details in DEVICES.md.

## Priority Model

- Priority P1: High demand, medium implementation risk, high ecosystem value.
- Priority P2: Medium demand or high risk, still important for broad coverage.
- Priority P3: Niche or low demand, parked unless contributors drive implementation.

## Candidate Backlog

| Priority | Device Family | Current State | Primary Gaps | Exit Criteria |
| --- | --- | --- | --- | --- |
| P1 | ROCCAT Vulcan II Max | Work in progress | Driver maturity, test coverage, hotplug validation | Stable hotplug, profile switching, release notes and docs |
| P1 | ROCCAT Pyro | Work in progress | Protocol completeness, key mapping edge cases | Full key map validation and runtime stability |
| P1 | ROCCAT Kone XP | Work in progress | Device parameters and profile behavior | Device config parity with supported ROCCAT mice |
| P2 | ROCCAT Kone Pro | Work in progress | DPI and polling controls, robustness | No critical regressions in daemon restart and reconnect |
| P2 | ROCCAT Kone Aimo Remastered | Experimental | Feature completion and long-run reliability | Capability matrix updated and stress-tested |
| P2 | ROCCAT Vulcan TKL / Pro | Experimental / untested | Validation depth and test automation | Regression tests and documented known limitations |
| P3 | Corsair Strafe (non-RGB) | Partial | Input-path consistency and maintenance confidence | Clear support scope and CI coverage for basic flows |

## Triage Workflow

1. Aggregate demand from GitHub issues and discussions.
2. Classify requests by priority and implementation risk.
3. Assign owner and target milestone in roadmap status.
4. Track progress with bring-up checklist and regression tests.

## Notes

- Canonical feature coverage remains in DEVICES.md.
- This file is intentionally operational and release-planning oriented.