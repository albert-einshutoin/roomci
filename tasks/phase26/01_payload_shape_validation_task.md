# Task 01: Payload Shape Validation

## Status

`todo`

## Problem

MQTT adapter contracts currently validate required payload field names, but do
not express field types, enum values, numeric ranges, or optional fields.

## Scope

- Extend adapter contract payload metadata with small, typed constraints.
- Keep existing contracts compatible.
- Validate invalid schema declarations before runtime.

## Acceptance Criteria

- Existing adapter contracts validate unchanged.
- Invalid field type/range/enum declarations fail with actionable errors.
- The feature remains contract-level validation, not a full JSON Schema engine.
