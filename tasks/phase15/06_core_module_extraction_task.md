# Task 06: Core Module Extraction

## Why

`crates/roomci-core/src/lib.rs` is above the 800-line maintainability target. The core runner is stable, but scenario execution, assertions, timeline construction, and device/domain behavior are coupled in one file. That weakens the release-candidate quality story because the size rule currently applies unevenly across the codebase.

## Acceptance Criteria

- Split `roomci-core` into focused modules such as runner, assertions, timeline, and domain-step handling.
- Keep public crate APIs stable unless a smaller internal API is clearly safer.
- Preserve all existing scenario behavior and report output.
- Add or keep focused tests for scenario ordering, failed assertions, retained MQTT behavior, Modbus, BMS, comfort, edge failover, and commissioning checklist flows.
- Keep the largest edited source file at or below the 800-line maintainability target, or document a justified exception in the task status.
- Keep coverage above 80%.

## Out of Scope

- Changing the scenario YAML format.
- Changing CLI behavior or report schemas.
- Adding new device protocol behavior during extraction.
