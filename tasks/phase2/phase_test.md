# Phase 2 Test Plan

## Quality Gates

- MQTT integration tests run against Mosquitto in Docker or a controlled test broker.
- Topic payloads are JSON and schema-checked.
- Command handling is idempotent where request IDs are repeated.
- State updates are emitted only after core engine state transitions.

## Required Test Cases

1. Publish `unlock` to command topic and receive state update.
2. Device availability topic reflects offline fault activation and clearing.
3. Telemetry topic emits sensor state.
4. Fault topic or scenario fault produces event topic output.
5. Duplicate command request ID does not produce duplicate state mutation.

## CI Expectations

- MQTT tests may be marked integration and run in Docker-enabled CI.
- Unit tests for topic mapping must run without Docker.
- Broker lifecycle is isolated per test suite.

## Done Means

Phase 2 is done when IoT-style tests can validate roomci behavior without HTTP-specific assumptions.
