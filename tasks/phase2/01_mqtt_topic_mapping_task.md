# Task 01 — MQTT Topic Mapping

## Objective

Define and implement canonical MQTT topic and payload mapping.

## Implementation Scope

- Add `roomci-mqtt` crate.
- Implement topic builders and parsers for:
  - `roomci/{room_id}/{device_id}/command`
  - `roomci/{room_id}/{device_id}/state`
  - `roomci/{room_id}/{device_id}/telemetry`
  - `roomci/{room_id}/{device_id}/availability`
  - `roomci/{room_id}/{device_id}/event`
  - `roomci/{room_id}/{device_id}/fault`
- Add payload structs and validation.

## Acceptance Criteria

- Topic parser rejects malformed room/device/topic forms.
- Payload schema covers action, request ID, state, availability, event, and fault.
- Mapping tests are independent of any broker.

## References

- `docs/04_protocol_adapters.md`
