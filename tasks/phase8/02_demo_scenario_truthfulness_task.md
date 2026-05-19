# Task 02 — Demo Scenario Truthfulness

## Objective

Make the README demo list truthful by separating passing scenarios from intentional failure-report scenarios.

## Acceptance Criteria

- Every scenario in README's passing demo section passes `roomci run`.
- Any intentionally failing scenario is listed separately and documents the expected non-zero exit.
- `examples/access_permission_drift.yaml` and `examples/commissioning_checklist.yaml` pass with `roomci run`.
- CI runs every scenario advertised as passing and verifies intentional failure-report scenarios separately.
- README explains that `dali_scene_partial_failure.yaml` is expected to exit non-zero.

## Review Findings

- All 9 example scenarios pass `roomci validate`.
- `examples/access_permission_drift.yaml` and `examples/commissioning_checklist.yaml` now pass `roomci run`.
- `examples/dali_scene_partial_failure.yaml` remains an intentional failure-report scenario.
