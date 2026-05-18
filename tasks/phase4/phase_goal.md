# Phase 4 Goal — AWS Shadow-like Adapter

## Goal

Provide a local AWS IoT Device Shadow-inspired surface for application tests that use desired/reported/delta state semantics.

## In Scope

- Shadow document per thing/device.
- REST-like get, update, and delete routes.
- MQTT-like shadow topic mapping when MQTT adapter exists.
- Desired to command mapping.
- Reported state and delta generation.

## Non-goals

- AWS IAM, policies, certificates, TLS mutual auth, or IoT Core clone behavior.
- Claiming full AWS IoT compatibility.

## Deliverables

- `GET /aws/things/{thingName}/shadow`.
- `POST /aws/things/{thingName}/shadow`.
- `DELETE /aws/things/{thingName}/shadow`.
- Accepted/rejected/delta response behavior.
- Shadow tests for lock and climate examples.

## Exit Criteria

- Desired `lock_state=unlocked` maps to canonical unlock command.
- Reported state is updated only through core state transitions.
- Delta is generated when desired and reported differ.
