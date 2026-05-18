# Phase 0 Goal — Core Engine and CLI

## Goal

Build the minimum executable roomci foundation: parse YAML scenarios, run deterministic device/fault timelines, evaluate assertions, and emit reports.

Phase 0 must prove roomci can validate the two existing hospitality scenarios without HTTP, MQTT, or cloud adapter surfaces.

## In Scope

- Rust workspace and crate skeleton matching `docs/02_architecture.md`.
- CLI commands: `run` and `validate`.
- Canonical room, device, capability, state, command, event, fault, and assertion models.
- YAML scenario parser for `docs/examples/checkin_lock_offline.yaml` and `docs/examples/ac_preheat_failed.yaml`.
- Simulated clock and relative time expressions.
- Deterministic fault application.
- JSON, Markdown, and JUnit report generation.

## Non-goals

- Long-running server mode.
- HTTP or MQTT API.
- Vendor-shaped compatibility endpoints.
- Real device bridge or production gateway behavior.

## Deliverables

- `roomci run <scenario.yaml>` executes a scenario to completion.
- `roomci validate <scenario.yaml>` validates schema and semantic constraints.
- Reports include scenario result, timeline, assertions, and guest impact.
- Golden fixtures exist for the two example scenarios.

## Exit Criteria

- Existing example scenarios can be parsed and executed.
- Failed assertions produce non-zero CLI exit in `run`.
- JUnit report represents failed assertions as failed tests.
- Markdown report is readable by product/operations stakeholders.
- No network dependency is required for tests.
