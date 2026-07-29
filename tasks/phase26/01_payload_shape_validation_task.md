# Task 01: Payload Shape Validation

## Status

`done`

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

## Evidence

- `MqttPayloadFieldType` and `MqttPayloadFieldConstraint` provide the closed
  `string` / `integer` / `number` / `boolean` / `object` / `array` vocabulary.
- `validate_mqtt_contract_publish` enforces declared type, enum, and numeric
  bounds for both scenario and serve-mode publishes.
- Unknown constraint keys, invalid required/optional classification, unsafe
  field names, and ambiguous large integer/float comparisons fail closed.
- Existing adapter contracts validate without migration.
- `make verify` passes, including 87.61% workspace coverage, Docker/Compose,
  protocol smoke, SDK smoke, evidence checks, and editor asset checks.
