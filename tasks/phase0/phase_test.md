# Phase 0 Test Plan

## Quality Gates

- Unit tests cover state transitions for lock, climate, sensor, and gateway devices.
- Unit tests cover fault precedence rules from `docs/07_fault_injection.md`.
- Parser tests cover valid and invalid scenario YAML.
- Golden tests cover JSON, Markdown, and JUnit output.
- CLI integration tests cover `run` and `validate`.

## Required Test Cases

1. Smart lock offline overrides an unlock command.
2. Gateway latency can cause scenario-level timeout behavior.
3. Sensor threshold assertion fails when temperature remains above target.
4. Event assertion passes only when matching event occurs within the expected window.
5. Probabilistic faults require a fixed seed.
6. Invalid device target fails validation before execution.

## CI Expectations

- Tests run without Docker.
- Scenario execution is deterministic.
- Golden report changes require explicit review.
- CLI exits `0` for passing scenarios and non-zero for failed scenario assertions or validation errors.

## Done Means

Phase 0 is done only when a backend engineer can add a YAML scenario and get a local report without writing Rust code.
