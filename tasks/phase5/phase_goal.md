# Phase 5 Goal — Azure Device Twin-like Adapter

## Goal

Provide Azure IoT Hub Device Twin-inspired desired/reported property behavior for local backend tests.

## In Scope

- Twin document model.
- Desired property patch.
- Reported property patch.
- Cloud-to-device message simulation.
- Mapping to canonical commands, state, and telemetry.

## Non-goals

- Azure auth, IoT Hub service SDK emulation, SAS tokens, TLS, or production cloud behavior.
- Full Azure IoT Hub clone claims.

## Deliverables

- Twin-like API or adapter module.
- Desired/reported patch handling.
- Cloud-to-device command simulation.
- Tests for AC setpoint and sensor reported values.

## Exit Criteria

- Desired AC setpoint maps to canonical climate command.
- Reported sensor values update canonical telemetry/state.
- Cloud-to-device message can drive a canonical command.
