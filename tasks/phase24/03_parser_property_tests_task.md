# Task 03: Parser Property Tests

## Status

`done`

## Problem

MQTT packets, Modbus frames, and YAML scenarios are parser-like inputs but are
currently covered mostly by example tests.

## Scope

- Add property or fuzz-style regression tests for malformed MQTT and Modbus
  inputs.
- Add schema/parser regression cases for surprising but syntactically valid YAML.
- Keep generated cases bounded so CI remains fast.

## Acceptance Criteria

- Malformed protocol inputs never panic.
- Invalid frames return documented errors or protocol responses.
- Tests run in the normal workspace test suite.

## Evidence

- `cargo test -p roomci-serve bounded`
