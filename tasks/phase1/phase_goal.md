# Phase 1 Goal — Local MQTT Retained-state Model

## Goal

Implement MQTT as a first-class primitive for local-first smart-home testing.

## In Scope

- Local broker model in scenario mode.
- Retained state messages.
- QoS1 duplicate delivery simulation.
- Reconnect state synchronization.
- Cloud broker mock as separate availability domain.

## Exit Criteria

- Local retained state works when cloud broker is offline.
- Duplicate delivery does not double-apply idempotent commands.
- MQTT state assertions are stable and deterministic.
