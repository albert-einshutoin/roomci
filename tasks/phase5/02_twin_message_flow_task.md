# Task 02 — Twin Message Flow

## Objective

Expose twin patches and cloud-to-device messages through local adapter behavior.

## Implementation Scope

- Implement desired property patch handling.
- Implement reported property patch handling.
- Implement cloud-to-device message simulation.
- Add tests for command mapping and telemetry updates.

## Acceptance Criteria

- Desired patch can drive climate setpoint changes.
- Reported patch can update sensor values.
- Cloud-to-device message can target lock, light, climate, or cover commands.
- Faults produce deterministic failure outputs.

## References

- `docs/07_fault_injection.md`
