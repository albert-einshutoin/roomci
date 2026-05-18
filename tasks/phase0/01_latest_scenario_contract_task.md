# Task 01 — Latest Scenario Contract

## Objective

Support the scenario shape from `docs/15_scenario_spec.md`.

## Implementation Scope

- Make `scenario.clock` optional.
- Add scenario tags.
- Parse top-level `devices`, `mqtt`, `faults`, `steps`, `assertions`, and `report`.
- Support top-level faults with `at`, `target`, `type`, and optional duration.
- Preserve stable validation errors for malformed scenarios.

## Acceptance Criteria

- `roomci validate examples/local_first_cloud_outage.yaml` exits `0`.
- Unknown or malformed latest scenario steps fail validation with useful errors.
- Existing tests are updated to reflect the new product direction.
