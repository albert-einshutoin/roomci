# Task 12 — MQTT Retained State Run-boundary Preservation

## Goal

Ensure externally published MQTT retained state survives `POST /run` the same way BMS/contact observations and injected faults do, so external MQTT controllers can use `roomci serve` as a stable pre-adoption emulator instead of a transient demo surface.

## Problem

`apply_external_mqtt_publish` currently writes matched MQTT payloads directly into `latest_report.final_state` and `latest_report.retained_messages`. A successful `POST /run` replaces `latest_report` with a fresh scenario report, while the external-observation overlay only drains timeline events and BMS observations into that new report.

This means an external MQTT publish recorded before a run can lose its retained state after the run boundary, even though the timeline event survives. That weakens the Phase 10/11 promise that external clients can drive the emulator and collect stable reports.

## Implementation Scope

- Add an overlay or equivalent preservation path for externally published MQTT retained messages.
- Preserve both:
  - the device `final_state` updated by the external MQTT publish
  - the retained MQTT topic payload exposed through `/state.retained_messages` and report JSON
- Merge the preserved MQTT state into rendered report views before `/run`.
- Drain the preserved MQTT state into the fresh `RunReport` on successful `/run`.
- Keep rejected MQTT publishes as timeline-only observations.
- Document the boundary clearly: this is retained-state preservation, not full MQTT broker session persistence.

## Acceptance Criteria

- A regression test covers: external MQTT publish -> `POST /run` -> retained topic still appears in `/state` or latest report output.
- Existing external MQTT publish tests still pass.
- Existing BMS/contact run-boundary preservation tests still pass.
- `cargo test -p roomci-serve --lib` passes.
- Documentation or Phase 11 status no longer implies MQTT external state survives unless this task is complete.
