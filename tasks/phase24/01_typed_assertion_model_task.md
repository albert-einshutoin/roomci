# Task 01: Typed Assertion Model

## Status

`todo`

## Problem

Core assertion dispatch currently depends on target strings and condition
strings. This keeps YAML flexible, but loses type information before execution
and makes new assertions easy to add incorrectly.

## Scope

- Introduce a typed assertion representation for known assertions.
- Keep a compatibility path for existing `target` + `condition` scenarios.
- Move validation errors earlier in the scenario-loading path where possible.

## Acceptance Criteria

- Current examples pass unchanged.
- Unknown typed assertions fail validation clearly.
- Core assertion evaluation uses exhaustive matching for promoted assertion
  kinds.

