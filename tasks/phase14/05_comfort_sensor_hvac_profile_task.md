# Task 05 — Comfort Sensor / HVAC Profile

## Goal

Make comfort automation coverage reflect hospitality-specific quality concerns: occupied-zone comfort, ceiling-zone drift, humidity, user override, and room-specific target tuning.

## Implementation Scope

- Extend comfort docs and/or scenarios around:
  - occupied-zone temperature/humidity sensor
  - ceiling-zone temperature/humidity sensor
  - discomfort index target
  - HVAC auto-mode decision
  - user override behavior
  - room-specific threshold tuning
- Keep real sensor hardware, firmware, PCB, and cloud API calls out of scope.

## Acceptance Criteria

- Comfort profile shows why this is more than generic thermostat testing.
- Scenario/report evidence explains comfort impact in field-readable language.
- Hardware details such as ESP32, DHT20, I2C, PCB, and enclosure design are treated as input assumptions or future profiles, not core emulation.
