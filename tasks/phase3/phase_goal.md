# Phase 3 Goal — Home Assistant Discovery-like Adapter

## Goal

Emit Home Assistant MQTT Discovery-like payloads so roomci devices can be inspected through a familiar smart-home model.

## In Scope

- Mapping canonical device types to HA-like components.
- Discovery config topics and payloads.
- State, command, and availability topic references.
- Docker Compose validation with a Home Assistant container when feasible.

## Non-goals

- Home Assistant replacement.
- Certified compatibility claim.
- Supporting every Home Assistant platform.

## Deliverables

- HA-like discovery payload generator.
- Discovery publish flow in MQTT serve mode.
- Fixture tests for lock, light, climate, cover, sensor, and binary_sensor mappings.

## Exit Criteria

- Canonical roomci devices produce valid discovery-like payloads.
- Payloads refer to working Phase 2 MQTT topics.
- Compatibility language remains `Home Assistant Discovery-like`.
