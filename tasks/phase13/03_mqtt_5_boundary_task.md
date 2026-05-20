# Task 03 — MQTT 5 Boundary

## Goal

Make MQTT 5 support status explicit: currently unsupported and rejected, not silently accepted or partially claimed.

## Specification Source

- OASIS Standard MQTT v5.0: https://docs.oasis-open.org/mqtt/mqtt/v5.0/mqtt-v5.0.html

## Implementation Scope

- Add MQTT 5 to the conformance registry as `unsupported` or `future_profile`.
- Keep the existing protocol-level `5` rejection behavior tested.
- Document what would be required before MQTT 5 could move out of unsupported status:
  - property parsing
  - reason codes
  - session expiry
  - user properties
  - topic aliases
  - request/response fields
  - updated client interoperability tests
- Ensure README and MQTT docs do not imply MQTT 5 compatibility.

## Acceptance Criteria

- MQTT 5 appears in the registry with official spec link.
- MQTT 5 `CONNECT` rejection remains covered by tests.
- Docs clearly say MQTT 5 is not implemented.
