# Task 04: Typed Condition Parser

## Status

`done`

## Problem

Assertion conditions that are meaningful to runtime behavior should not be
parsed repeatedly from free-form strings. They should be validated once and then
executed as typed condition variants.

## Scope

- Parse promoted assertion conditions into typed variants.
- Keep raw condition strings available only for compatibility or report output.
- Reject malformed promoted conditions at validation time.
- Preserve current assertion behavior for valid examples.

## Implementation Checklist

- [x] Add tests for accepted promoted conditions such as availability, active
  state, false/unaffected state, numeric ranges, and expected value checks.
- [x] Add tests for malformed promoted conditions with actionable errors.
- [x] Introduce a typed `Condition` model or extend the existing assertion model.
- [x] Convert raw target/condition pairs into typed assertion conditions.
- [x] Update runtime assertion evaluation to consume typed conditions.
- [x] Keep report output stable for users.

## Acceptance Criteria

- Runtime assertion evaluation does not depend on ad hoc parsing of promoted
  condition strings.
- Malformed promoted conditions fail during scenario validation.
- Existing valid scenarios produce the same pass/fail outcomes.
