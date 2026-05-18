# Phase 0 Status — Core Engine and CLI

## Phase Status

`in_progress`

## Task Board

| Task | Status | Owner | Evidence | Notes |
|---|---|---|---|---|
| `01_workspace_and_cli_task.md` | `done` | Codex | `cargo run -p roomci-cli -- --help`; `cargo run -p roomci-cli -- validate ...` | Workspace and CLI skeleton implemented |
| `02_core_runner_and_reports_task.md` | `review` | Codex | `cargo test`; CLI `run` generated JSON/Markdown/JUnit reports | Runner and reports implemented; dedicated golden fixture files still pending |

## Quality Gate Status

| Gate | Status | Evidence |
|---|---|---|
| Unit tests for state transitions | `done` | `cargo test` passed; `roomci-device-model` state transition tests |
| Unit tests for fault precedence | `review` | `cargo test` passed; offline fault covered by `checkin_lock_offline` runner test |
| Parser tests for scenario YAML | `done` | `cargo test` passed; relative time and invalid target tests |
| Golden tests for JSON/Markdown/JUnit | `review` | Report rendering tests pass; dedicated golden fixture files not yet added |
| CLI integration tests for `run` and `validate` | `done` | `cargo test` passed; `crates/roomci-cli/tests/cli.rs` |

## Blockers

- None.

## Next Action

Add dedicated golden fixture files for report outputs, then mark Phase 0 complete if no behavior changes are required.
