# Phase 3 Test Plan

## Quality Gates

- Discovery payloads are snapshot-tested.
- Every supported device type has a mapping test.
- Payloads avoid unsupported or misleading Home Assistant fields.
- MQTT publish flow is integration-tested when broker is available.

## Required Test Cases

1. Smart lock maps to `lock`.
2. Light maps to `light`.
3. Climate maps to `climate`.
4. Cover maps to `cover`.
5. Occupancy and motion sensors map to `binary_sensor`.
6. Temperature and humidity sensors map to `sensor`.

## CI Expectations

- Snapshot tests run without Docker.
- Optional Home Assistant container validation may be separate from default CI.

## Done Means

Phase 3 is done when roomci can advertise its emulated devices through HA-style discovery payloads without changing the canonical core model.
