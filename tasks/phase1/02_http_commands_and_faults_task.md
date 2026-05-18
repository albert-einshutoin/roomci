# Task 02 — HTTP Commands, Faults, and Scenario Run

## Objective

Allow test clients to mutate roomci through canonical commands and fault injection.

## Implementation Scope

- Implement:
  - `POST /rooms/{room_id}/devices/{device_id}/commands/{command}`
  - `POST /faults`
  - `POST /scenarios/run`
- Validate command compatibility with device capabilities.
- Record command, state, fault, and assertion events in the timeline.
- Return request IDs when provided by clients.

## Acceptance Criteria

- Online smart lock accepts `unlock`.
- Offline smart lock command produces deterministic failed command behavior.
- Fault payloads validate target and type.
- Scenario run endpoint returns report summary and timeline.

## References

- `docs/03_device_model.md`
- `docs/07_fault_injection.md`
