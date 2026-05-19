# Task 02 — Demo Scenario Truthfulness

## Objective

Make the README demo list truthful by separating runnable scenarios from future placeholders.

## Acceptance Criteria

- Every scenario in README's passing demo section passes `roomci run`.
- Any intentionally failing scenario is listed separately and documents the expected non-zero exit.
- `examples/access_permission_drift.yaml` and `examples/commissioning_checklist.yaml` are either implemented to pass or moved into a clearly labeled future-milestone section.
- CI runs every scenario advertised as passing and verifies intentional failure-report scenarios separately.
- README explains the difference between `validate` support and executable behavior where future placeholders remain.

## Review Findings

- All 9 example scenarios pass `roomci validate`.
- `examples/access_permission_drift.yaml` fails `roomci run`.
- `examples/commissioning_checklist.yaml` fails `roomci run`.
- README currently lists both as demo scenarios without warning that they are future placeholders.
