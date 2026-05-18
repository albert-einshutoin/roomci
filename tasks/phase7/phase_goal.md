# Phase 7 Goal — Production Readiness

## Goal

Raise roomci from a working MVP to a production-grade portfolio piece by closing documentation, test coverage, CLI ergonomics, and CI quality-gate gaps.

## In Scope

- Crate-level (`//!`) and public-API (`///`) documentation across all 9 crates.
- Workspace test coverage measured by `cargo tarpaulin`, with a floor of 80%.
- Error-path tests for every crate, including `roomci-fault`.
- CLI ergonomics: multi-scenario execution, `--verbose`, `--quiet`, `--dry-run`.
- CI quality gates: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo tarpaulin --fail-under 80`.
- README polish: badges, quick start, demo scenarios.
- Scenario JSON Schema enriched with structural constraints.
- Consolidate `docs/` as the canonical source and remove the `roomci-docs-latest/` duplicate.

## Exit Criteria

- `cargo doc --no-deps` completes with zero warnings.
- `cargo test --workspace` runs at least 55 tests, all passing.
- `cargo tarpaulin --workspace` reports coverage >= 80%.
- `cargo fmt --check` and `cargo clippy --workspace -- -D warnings` pass.
- `roomci run a.yaml b.yaml` executes multiple scenarios and aggregates exit codes.
- CI workflow enforces fmt/clippy/coverage gates on pull request and push to main.
- README shows badges, quick start, and a demo-scenario index.
- `roomci-docs-latest/` is removed; `docs/README.md` indexes the canonical docs.
