# Task 05 — Demo Targets

## Objective

Make demos explicit for both hospitality-focused and generic MQTT-facing audiences.

## Acceptance Criteria

- Add `make demo-hospitality`.
- Add `make demo-generic-mqtt`.
- `make demo` can remain the full curated demo.
- `make verify` includes the new generic MQTT examples.
- README documents the two demo paths.

## Demo Intent

- `demo-hospitality`: local-first outage, edge failover, BMS alert, commissioning checklist, access drift.
- `demo-generic-mqtt`: retained state, duplicate delivery, reconnect or offline behavior.
