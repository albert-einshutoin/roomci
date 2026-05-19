# Task 05 — External Controller E2E

## Objective

Prove that `roomci` can be used as a black-box dependency by a separate controller process in local or CI workflows.

## Acceptance Criteria

- Add a sample external controller that connects to `roomci` through network endpoints.
- Add a Docker Compose service that starts `roomci` and the sample controller.
- The controller drives at least one scenario to completion without using internal Rust APIs.
- The E2E produces JSON, Markdown, and JUnit reports.
- The Compose run exits non-zero when a required assertion fails.
- `make verify` or a dedicated CI target runs this E2E.

## Notes

- The sample controller can be minimal; its job is to prove integration shape, not to become a production app.
- Keep generated reports under the existing `reports/` convention.
