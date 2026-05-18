# Phase 5 Test Plan

## Quality Gates

- Twin documents are schema-tested.
- Desired and reported patches are validated separately.
- Cloud-to-device command simulation uses core command validation.
- Failure behavior is consistent with active faults.

## Required Test Cases

1. Desired AC setpoint patch maps to `set_temperature`.
2. Reported temperature patch updates sensor telemetry.
3. Cloud-to-device unlock message maps to lock command.
4. Invalid desired path returns stable validation error.
5. Gateway latency fault affects command timing semantics.

## CI Expectations

- Unit tests run without cloud services.
- Adapter integration tests reuse Phase 1/2 server harness where appropriate.

## Done Means

Phase 5 is done when Azure-style backend code can be tested against roomci's local twin abstraction.
