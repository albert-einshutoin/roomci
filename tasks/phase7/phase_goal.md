# Phase 7 Goal — Matter-like Profile Adapter

## Goal

Add Matter-like profile import/export as a vocabulary bridge while avoiding full Matter protocol implementation.

## In Scope

- Matter-like cluster to roomci capability mapping.
- Profile import/export for supported capabilities.
- Validation that profile metadata matches canonical device definitions.

## Non-goals

- Matter commissioning.
- Fabrics, certificates, secure channels, or CHIP stack.
- Certified Matter compatibility claim.

## Deliverables

- Profile mapper for OnOff, LevelControl, DoorLock, Thermostat, TemperatureMeasurement, OccupancySensing, and WindowCovering.
- Import/export CLI or validation command.
- Tests that prove canonical model remains source of truth.

## Exit Criteria

- A Matter-like profile can be generated from a roomci device.
- A supported Matter-like profile can be imported into canonical device metadata.
- Unsupported clusters fail validation with clear errors.
