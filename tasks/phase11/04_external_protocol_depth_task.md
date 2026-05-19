# Task 04 — External Protocol Depth

## Goal

Move beyond one minimal MQTT ingress so `roomci` can support real pre-adoption evaluations where external systems connect through familiar protocol surfaces.

## Implementation Scope

- Test MQTT serve mode with a standard MQTT client/library.
- Add MQTT subscriber/retained replay if external observers need to receive state updates.
- Decide and document the next MQTT subset boundary:
  - QoS0 only
  - QoS1 subset
  - retained replay
  - reconnect/session assumptions
  - auth/TLS boundaries
- Add a second externally drivable protocol surface. Preferred candidates:
  - Modbus TCP endpoint for register read/write PoC
  - BMS webhook receiver/emitter for operations PoC
- Ensure external interactions update the same report/timeline model as scenario-mode steps.

## Acceptance Criteria

- At least one standard MQTT client/library can drive the MQTT PoC.
- At least one non-MQTT external endpoint can be driven by a separate client process.
- Reports clearly distinguish internal scenario steps from external observed interactions.
- Unsupported production protocol behavior is documented with examples.
