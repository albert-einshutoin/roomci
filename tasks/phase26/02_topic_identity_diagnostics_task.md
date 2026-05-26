# Task 02: Topic Identity Diagnostics

## Status

`todo`

## Problem

Topic template and `device_id_from_topic` errors are currently functional but
minimal. Evaluators need clearer feedback when mapping private MQTT topics.

## Scope

- Improve diagnostics for missing `{device_id}` placeholders.
- Detect mismatched placeholders between command and state topics.
- Keep `placeholder:{device_id}` as the only runtime strategy until a concrete
  need is proven.

## Acceptance Criteria

- Invalid topic templates fail validation with the contract name and field.
- Existing examples still validate.
- Runtime matching behavior is unchanged.
