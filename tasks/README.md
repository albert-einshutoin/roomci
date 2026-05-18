# roomci Task Plan

This directory breaks the roomci roadmap into implementation phases.

Each phase contains:

- `phase_status.md`: task progress, quality gate status, blockers, and next action.
- `phase_goal.md`: scope, non-goals, deliverables, and exit criteria.
- `phase_test.md`: quality gates and test strategy for the phase.
- `01_*_task.md`, `02_*_task.md`: concrete implementation task files.

Use `status.md` as the top-level progress board.

## Phase Map

| Phase | Focus | Primary Outcome |
|---|---|---|
| Phase 0 | Core engine, CLI, scenario runner, reports | Deterministic scenario execution without network adapters |
| Phase 1 | HTTP adapter | Backend/app tests can control roomci through REST APIs |
| Phase 2 | MQTT adapter | IoT-style command/state/event topics work through Mosquitto-compatible flow |
| Phase 3 | Home Assistant Discovery-like adapter | Canonical devices can be advertised through HA-style MQTT discovery payloads |
| Phase 4 | AWS Shadow-like adapter | Desired/reported/delta shadow behavior is available locally |
| Phase 5 | Azure Device Twin-like adapter | Desired/reported property flow and cloud-to-device messages are available locally |
| Phase 6 | Hue-like lighting and scene adapter | Room scenes and partial scene failures can be tested |
| Phase 7 | Matter-like profile adapter | Canonical devices can import/export Matter-like profile metadata |

## Quality Policy

Every phase must be independently shippable behind documented scope boundaries.

Before a phase is considered complete:

- All phase tasks have acceptance criteria checked off.
- `phase_test.md` quality gates pass.
- User-facing examples or docs are updated when behavior changes.
- Deterministic behavior is preserved for CI scenarios.
- Compatibility wording remains `*-like` unless actual vendor certification exists.

## Progress Management

Status is tracked in two layers:

- `tasks/status.md`: phase-level status and current blocking issues.
- `tasks/phase*/phase_status.md`: task-level status, quality gate status, evidence, blockers, and next action.

Valid status values are:

- `todo`
- `in_progress`
- `blocked`
- `review`
- `done`

When implementation starts, update the target task to `in_progress`. When code exists but quality gates are still running, use `review`. Only use `done` after the task acceptance criteria and relevant phase quality gates pass.
