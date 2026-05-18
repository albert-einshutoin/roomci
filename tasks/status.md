# roomci Task Status

This file is the top-level progress board for roomci implementation.

## Status Values

| Status | Meaning |
|---|---|
| `todo` | Not started |
| `in_progress` | Actively being implemented |
| `blocked` | Waiting on a decision, dependency, or failed quality gate |
| `review` | Implementation exists and needs review or verification |
| `done` | Acceptance criteria and phase quality gates passed |

## Phase Status

| Phase | Status | Current Focus | Blocking Issue | Status File |
|---|---|---|---|---|
| Phase 0 | `in_progress` | Core engine, CLI, scenario runner, reports | None | `phase0/phase_status.md` |
| Phase 1 | `todo` | HTTP adapter | Phase 0 not complete | `phase1/phase_status.md` |
| Phase 2 | `todo` | MQTT adapter | Phase 1 not complete | `phase2/phase_status.md` |
| Phase 3 | `todo` | Home Assistant Discovery-like adapter | Phase 2 not complete | `phase3/phase_status.md` |
| Phase 4 | `todo` | AWS Shadow-like adapter | Phase 1/2 not complete | `phase4/phase_status.md` |
| Phase 5 | `todo` | Azure Device Twin-like adapter | Phase 1/2 not complete | `phase5/phase_status.md` |
| Phase 6 | `todo` | Hue-like scene adapter | Core scene model not started | `phase6/phase_status.md` |
| Phase 7 | `todo` | Matter-like profile adapter | Lower priority until adapter baseline exists | `phase7/phase_status.md` |

## Update Rules

- Update this file when a phase status changes.
- Update the matching `phase_status.md` when a task status changes.
- Do not mark a task `done` unless its acceptance criteria are satisfied.
- Do not mark a phase `done` unless its `phase_test.md` quality gates pass.
- Use `blocked` with a concrete blocker, not as a vague waiting state.

## Current Recommendation

Phase 0 is in progress. It is the dependency for every adapter phase because it owns the canonical state engine, scenario execution, assertion evaluation, and reports.
