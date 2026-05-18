# 08. Adapter Implementation Plan

## Phase 0 — Core

- Define device model.
- Define capability model.
- Define scenario parser.
- Define event timeline.
- Implement assertion engine.
- Implement JSON/Markdown/JUnit report.

## Phase 1 — HTTP Adapter

Routes:

```text
GET    /healthz
GET    /rooms
GET    /rooms/{room_id}
GET    /rooms/{room_id}/devices
GET    /rooms/{room_id}/devices/{device_id}/state
POST   /rooms/{room_id}/devices/{device_id}/commands/{command}
POST   /faults
GET    /timeline
POST   /scenarios/run
```

## Phase 2 — MQTT Adapter

- Support state topic.
- Support command topic.
- Support telemetry topic.
- Support availability topic.
- Support event topic.

Decision:
- Either embed a simple MQTT broker.
- Or document Mosquitto sidecar as the first implementation.

Recommended v0.1:
- Start with Mosquitto sidecar compatibility if full broker implementation is too large.
- Implement MQTT client adapter that subscribes/publishes to Mosquitto.

## Phase 3 — Home Assistant Discovery-like Adapter

- Emit config payloads.
- Map canonical devices to Home Assistant components.
- Validate with a Home Assistant container in Docker Compose.

Device mapping:

```text
smart_lock -> lock
light -> light
climate -> climate
cover -> cover
occupancy_sensor -> binary_sensor
motion_sensor -> binary_sensor
temperature_sensor -> sensor
humidity_sensor -> sensor
```

## Phase 4 — AWS Shadow-like Adapter

- Implement local shadow document.
- Support desired/reported/delta.
- Support shadow update and get.
- Support accepted/rejected topics.
- Do not implement AWS IAM, policies, certificates, or TLS mutual auth in v0.1.

## Phase 5 — Azure Device Twin-like Adapter

- Implement twin document.
- Support desired property patch.
- Support reported property patch.
- Support cloud-to-device message simulation.

## Phase 6 — Hue-like Lighting / Scene Adapter

- Room/zone/group model.
- Scene activation.
- Partial scene failure.
- Scene consistency assertion.

## Phase 7 — Matter-like Profile Adapter

- Implement profile import/export only.
- Do not implement commissioning, fabric, certificates, or CHIP stack.

## Test Strategy

### Unit tests

- State transitions.
- Fault application.
- Assertion evaluation.

### Integration tests

- HTTP command -> state change.
- MQTT command -> state change.
- Scenario -> JUnit report.

### Compatibility tests

- Home Assistant discovery payload format.
- Shadow-like desired/reported/delta behavior.

### Golden tests

- Keep expected Markdown/JSON/JUnit outputs for scenarios.
