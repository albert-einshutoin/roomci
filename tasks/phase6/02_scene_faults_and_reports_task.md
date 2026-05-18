# Task 02 — Scene Faults and Reports

## Objective

Make partial scene failures visible and testable.

## Implementation Scope

- Implement `partial_scene_failure` fault behavior.
- Add scene consistency assertion evaluation.
- Add Markdown and JSON report sections for scene members.
- Add `welcome_scene_partial_failure` example.

## Acceptance Criteria

- Fault can target one or more scene members.
- Report identifies failed scene members and guest impact.
- JUnit maps scene consistency failure to a failed test case.

## References

- `docs/07_fault_injection.md`
- `docs/10_notahotel_positioning.md`
