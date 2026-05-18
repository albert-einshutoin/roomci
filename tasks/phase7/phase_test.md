# Phase 7 Test Plan

## Quality Gates

- Cluster mappings are table-driven and unit-tested.
- Import/export round trips are snapshot-tested.
- Unsupported profile fields fail safely.
- Docs avoid implying Matter certification.

## Required Test Cases

1. OnOff maps to `on_off`.
2. LevelControl maps to `brightness`.
3. DoorLock maps to `lock`.
4. Thermostat maps to `thermostat`.
5. TemperatureMeasurement maps to `temperature_measurement`.
6. OccupancySensing maps to `occupancy`.
7. WindowCovering maps to `cover_position`.

## CI Expectations

- Tests run without Matter SDK or device stack.
- Compatibility wording is reviewed in docs.

## Done Means

Phase 7 is done when Matter-like vocabulary can help profile interoperability without changing roomci into a Matter emulator.
