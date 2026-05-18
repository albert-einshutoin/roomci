# Phase 2 Goal — MQTT Adapter

## Goal

Expose command, state, telemetry, availability, event, and fault flows through MQTT-compatible topics.

## In Scope

- Mosquitto sidecar compatibility as the recommended v0.1 path.
- MQTT client adapter that subscribes to command topics and publishes state/event topics.
- Topic convention from `docs/04_protocol_adapters.md`.
- Docker Compose example with roomci, Mosquitto, and a sample client.

## Non-goals

- Full embedded broker unless it is cheaper than sidecar support.
- MQTT auth, TLS, ACLs, or advanced broker administration.
- Full MQTT 5 feature coverage.

## Deliverables

- `roomci serve --mqtt <broker-url>` or equivalent config.
- State and availability topics for configured devices.
- Command topic handling routed through the core engine.
- MQTT integration tests with Mosquitto.

## Exit Criteria

- A test can publish an unlock command and observe state/event output.
- Faults affect MQTT command results consistently with HTTP and core runner.
- Docker Compose can run the MQTT flow locally.
