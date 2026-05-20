# Task 02 — MQTT 3.1.1 Conformance Subset

## Goal

Turn the current MQTT 3.1.1 serve ingress into a spec-backed conformance subset verified by standard clients, while keeping the subset intentionally narrow.

## Specification Source

- OASIS MQTT Specification: https://mqtt.org/mqtt-specification/

## Implementation Scope

- Define the supported MQTT 3.1.1 subset in the conformance registry:
  - `CONNECT`
  - `CONNACK`
  - QoS0 `PUBLISH`
  - UTF-8 topic names
  - JSON object payload contract validation
  - retained-state observation through HTTP reports
- Add standard MQTT client/library black-box coverage. Candidate tools:
  - `mosquitto_pub`
  - `paho-mqtt`
  - Rust MQTT client crate used only in tests
- Keep hand-written packet tests for malformed packets and exact rejection behavior.
- Add explicit unsupported behavior notes:
  - QoS1/QoS2
  - `SUBSCRIBE`
  - retained replay to MQTT subscribers
  - sessions
  - will messages
  - keepalive enforcement
  - auth/TLS
  - MQTT 5 properties

## Acceptance Criteria

- A standard external MQTT client can publish to `roomci serve --mqtt-port`.
- The published state appears in `/state`, `/timeline`, and `/reports/latest.json`.
- Unsupported protocol versions are rejected with documented `CONNACK` behavior.
- Existing packet-level tests still pass.
- Docker or CI can run the standard-client MQTT smoke test.
