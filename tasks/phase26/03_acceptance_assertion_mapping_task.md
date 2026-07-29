# Task 03: Acceptance Assertion Mapping

## Status

`done`

## Problem

Adapter contracts collect acceptance criteria, but they are not explicitly
linked to scenario assertions or report evidence.

## Scope

- Add optional acceptance criterion ids.
- Allow criteria to reference scenario assertion names or evidence artifacts.
- Render the mapping in validation or debug output.

## Delivered

- Preserved legacy scalar criteria and added optional stable mapping ids.
- Added scoped `scenario` + named `assertion` references and artifact
  capability references.
- Added fail-closed cross-validation through `adapter validate --scenario`.
- Preserved runtime diagnostic names while exporting stable `reference_id`
  values to JSON, Markdown, and JUnit evidence.
- Added a complete mapping example and regression coverage for invalid
  references, artifact declarations, and unsafe or duplicate assertion names.

## Acceptance Criteria

- Existing contracts validate unchanged.
- New examples can map at least one criterion to a scenario assertion.
- Invalid references fail validation clearly.
