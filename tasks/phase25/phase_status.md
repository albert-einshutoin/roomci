# Phase 25 Status: Public Surface Evidence Freshness

## Status

`done`

## Task Board

| Task | Status | Owner | Evidence | Notes |
|---|---|---|---|---|
| `01_readme_interview_demo_link_task.md` | `done` | Codex | `test -f docs/INTERVIEW_DEMO.md`; `rg "INTERVIEW_DEMO" README.md README.ja.md docs` | Added the linked walkthrough instead of removing the README evaluator path |
| `02_readme_quality_measurement_refresh_task.md` | `done` | Codex | `cargo test --workspace --all-targets` | README and README.ja now report the current passing test count |

## Quality Gates

- `cargo test --workspace --all-targets` passed: 129 tests.
- `cargo run -p roomci-cli -- validate examples/local_first_cloud_outage.yaml examples/generic_mqtt_retained_state.yaml examples/dali_scene_partial_failure.yaml examples/matter_gateway_profile.yaml` passed.
- `cargo run -p roomci-cli -- serve --config examples/local_first_cloud_outage.yaml --check` passed.

## Notes

`make verify` was not rerun in this phase because the change is documentation
and task-board only. The full release checklist still requires it before a
formal release or company evaluation build.
