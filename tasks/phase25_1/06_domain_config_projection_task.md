# Task 06: Domain Config Projection

## Status

`done`

## Problem

Runtime code should not repeatedly inspect loosely typed config maps for known
domain behavior. Domain config should be projected once into typed inputs, then
passed through execution.

## Scope

- Add typed projections for runtime-read config sections.
- Start with sections already used by core runtime behavior.
- Fail malformed config during validation with field-specific errors.
- Leave unknown extension config available at raw/report boundaries.

## Implementation Checklist

- [x] Inventory runtime reads of scenario config maps.
- [x] Add tests for malformed promoted config values.
- [x] Add tests that existing example config sections project successfully.
- [x] Introduce typed config structs for promoted sections such as WAN,
  comfort, commissioning, inputs, edge, Modbus, lighting, contacts, brokers, or
  adapter-facing config where currently read by runtime.
- [x] Wire runtime code to use typed config projections.
- [x] Remove repeated raw-map reads for promoted sections.
- [x] Document any remaining raw config access as extension data.

## Acceptance Criteria

- Runtime-read domain config is converted once and accessed through typed
  structs.
- Invalid promoted config values fail before execution.
- Unknown extension config does not block backward compatibility.
