# Task 03 — CLI and CI Hardening

## Objective

Raise CLI ergonomics and CI quality gates to production-tool levels.

## Acceptance Criteria

- `roomci run` accepts multiple scenario paths and aggregates exit codes (non-zero if any scenario fails).
- `roomci run` supports `--verbose`, `--quiet`, and `--dry-run` flags.
- `README.md` has CI/coverage/license badges, a Quick Start, and a Demo Scenarios section.
- `schemas/scenario.schema.json` constrains version, scenario metadata, devices, faults, steps, and assertions.
- `.github/workflows/smart-home-ci.yml` runs `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo tarpaulin --workspace --fail-under 80`, and triggers on `push` to `main` as well as pull requests.
