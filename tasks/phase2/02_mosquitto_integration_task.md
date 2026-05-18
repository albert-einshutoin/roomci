# Task 02 — Mosquitto Integration

## Objective

Connect roomci to a Mosquitto-compatible broker for local and CI testing.

## Implementation Scope

- Add MQTT client connection lifecycle.
- Subscribe to command topics for configured devices.
- Publish retained or current state where appropriate.
- Publish availability, telemetry, event, and fault messages.
- Add Docker Compose example with Mosquitto sidecar.

## Acceptance Criteria

- Compose setup starts roomci and Mosquitto.
- Publishing a command changes canonical state and emits state/event messages.
- Broker disconnect/reconnect behavior is handled deterministically enough for CI.
- Integration docs include local test commands.

## References

- `docs/05_docker_ci_design.md`
