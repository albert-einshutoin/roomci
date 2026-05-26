# Task 02: Identifier and Topic Newtypes

## Status

`done`

## Problem

Runtime logic still risks mixing unrelated identifiers because device IDs,
scene IDs, fixture IDs, contacts, alerts, brokers, edges, and MQTT topics can
all appear as plain strings.

## Scope

- Add small newtypes for promoted identifiers and MQTT topic values.
- Parse and validate these newtypes at the validated scenario boundary.
- Use the newtypes inside runtime/domain code where identity semantics matter.
- Preserve string serialization at CLI/report boundaries.

## Implementation Checklist

- [x] Add tests for empty identifiers being rejected.
- [x] Add tests for invalid MQTT topics and topic templates being rejected.
- [x] Introduce identifier newtypes for promoted domain IDs.
- [x] Introduce MQTT topic/topic-template newtypes with focused validation.
- [x] Replace runtime string parameters with newtypes where promoted semantics
  already exist.
- [x] Add `Display`/serialization conversions only at external boundaries.
- [x] Confirm reports remain byte-shape compatible where public behavior depends
  on strings.

## Acceptance Criteria

- Internally promoted IDs are not freely interchangeable strings.
- Invalid IDs/topics fail during validation, not during late runtime matching.
- Existing examples and reports remain compatible.
