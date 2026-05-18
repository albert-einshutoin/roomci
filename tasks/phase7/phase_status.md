# Phase 7 Status — Production Readiness

## Phase Status

`in_progress`

## Task Board

| Task | Status | Owner | Evidence | Notes |
|---|---|---|---|---|
| `01_docs_coverage_task.md` | `done` | Codex | `cargo doc --no-deps --workspace` (0 warnings); `cargo tarpaulin --workspace --skip-clean --engine llvm` → 84.23% (1095/1300 lines); `ls docs/README.md`; `roomci-docs-latest/` removed | Crate/API docs added to all 9 crates, tarpaulin baseline above 80%, docs consolidated |
| `02_test_error_paths_task.md` | `done` | Codex | `cargo test --workspace` → 61 tests pass (was 38); `cargo tarpaulin --workspace --skip-clean --engine llvm` → 85.46% (1111/1300 lines, +1.23%); CLI integration tests cover missing-file, missing-args, unknown subcommand | `roomci-fault` 0→3 tests; error-path tests added to mqtt/edge/device-model/ops/scenario; CLI integration 2→5 tests |
| `03_cli_ci_hardening_task.md` | `todo` | Codex | _pending_ | Multi-scenario CLI, fmt/clippy/tarpaulin CI gates |

## Blockers

- None.

## Quality Gate Status

| Gate | Status | Evidence |
|---|---|---|
| Crate and public-API documentation | `done` | `cargo doc --no-deps --workspace` produces 0 warnings; every crate has `//!` module docs and `///` docs on the primary public API |
| Workspace test count and coverage | `done` | `cargo test --workspace` → 61 tests pass (≥55 target); `cargo tarpaulin --workspace --skip-clean --engine llvm` → 85.46% (1111/1300 lines, ≥80% target) |
| CLI ergonomics regression tests | `todo` | `cargo run -p roomci-cli -- run examples/local_first_cloud_outage.yaml examples/modbus_floor_heating.yaml`; `cargo test -p roomci-cli` |
| CI quality-gate workflow parity | `todo` | `cargo fmt --check`; `cargo clippy --workspace -- -D warnings`; `.github/workflows/smart-home-ci.yml` job parity |
| README, schema, and docs index freshness | `todo` | `grep -n 'Quick start' README.md`; `python -c "import json,sys; json.load(open('schemas/scenario.schema.json'))"`; `ls docs/README.md` |
