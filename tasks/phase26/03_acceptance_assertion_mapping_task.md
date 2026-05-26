# Task 03: Acceptance Assertion Mapping

## Status

`todo`

## Problem

Adapter contracts collect acceptance criteria, but they are not explicitly
linked to scenario assertions or report evidence.

## Scope

- Add optional acceptance criterion ids.
- Allow criteria to reference scenario assertion names or evidence artifacts.
- Render the mapping in validation or debug output.

## Acceptance Criteria

- Existing contracts validate unchanged.
- New examples can map at least one criterion to a scenario assertion.
- Invalid references fail validation clearly.
