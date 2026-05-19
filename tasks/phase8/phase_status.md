# Phase 8 Status — Public Release and Interview Polish

## Phase Status

`done`

## Task Board

| Task | Status | Owner | Evidence | Notes |
|---|---|---|---|---|
| `01_license_public_metadata_task.md` | `done` | Codex | Added `LICENSE`; README MIT badge links to an existing file; `Cargo.toml` remains `license = "MIT"` | `LICENSE-NOTE.txt` now points to `LICENSE` and keeps provenance caution |
| `02_demo_scenario_truthfulness_task.md` | `done` | Codex | `cargo run -p roomci-cli -- run` passing demo set → 6 passed; DALI failure-report demo exits 1 as expected; future placeholders moved out of passing demos | README now separates passing, failure-report, and future milestone scenarios |
| `03_unimplemented_command_docs_task.md` | `done` | Codex | `rg "roomci serve|serve --config"` shows only future/planned references; CLI reference lists only `run` and `validate` | `serve` is documented as post-v0.1, not current CLI |
| `04_readme_claims_refresh_task.md` | `done` | Codex | `cargo test --workspace --all-targets` → 66 tests pass; `cargo tarpaulin --workspace --engine llvm --fail-under 80 --skip-clean` → 86.15% | README badge and Quality Gates updated |
| `05_public_positioning_review_task.md` | `done` | Codex | README, `docs/01_notahotel_research_synthesis.md`, and `docs/19_interview_positioning.md` updated to frame claims as public research/external interpretation | Avoids implying access to private/internal NOT A HOTEL systems |
| `06_release_readiness_gate_task.md` | `done` | Codex | fmt/clippy/test/doc/tarpaulin passed; release Quick Start commands passed; Docker build/run/validate passed; Compose config passed | Final release gate completed locally |

## Blockers

- None.

## Quality Gate Status

| Gate | Status | Evidence |
|---|---|---|
| Public license consistency | `done` | `LICENSE` exists; README badge and Cargo metadata agree on MIT |
| Passing demo truthfulness | `done` | `cargo run -p roomci-cli -- run examples/local_first_cloud_outage.yaml examples/edge_server_failover.yaml examples/modbus_floor_heating.yaml examples/bms_sauna_emergency_alert.yaml examples/starlink_failover.yaml examples/comfort_auto_mode.yaml` → 6 passed |
| Intentional failure-report demo | `done` | `cargo run -p roomci-cli -- run examples/dali_scene_partial_failure.yaml ...; test "$?" -eq 1` passed and emitted reports |
| Documentation command accuracy | `done` | `serve` references are marked planned/future; current CLI docs list only `run` and `validate` |
| README freshness | `done` | README now states 66 tests and 86.15% line coverage |
| Public positioning safety | `done` | Public-facing NOT A HOTEL wording now says public materials / external interpretation |
| Release-readiness command set | `done` | `cargo fmt --all --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace --all-targets`; `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps`; `cargo tarpaulin --workspace --engine llvm --fail-under 80 --skip-clean`; all-example validate; passing demo run; Docker build/run; Compose config |
