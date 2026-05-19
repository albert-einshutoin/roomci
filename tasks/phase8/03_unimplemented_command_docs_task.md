# Task 03 — Unimplemented Command Documentation

## Objective

Remove or clearly mark documentation for commands that do not exist in the CLI.

## Acceptance Criteria

- Search docs and README for `roomci serve` and `serve --config`.
- Either implement `roomci serve` with tests, or rewrite those references as roadmap/future examples.
- CLI reference in README lists only supported commands.
- Architecture and Docker/CI docs do not give copy-paste commands that fail.

## Review Findings

- `crates/roomci-cli/src/main.rs` implements `run` and `validate`.
- `docs/03_architecture.md`, `docs/17_docker_ci_design.md`, and `docs/18_mvp_roadmap.md` mention `roomci serve` or `serve --config`.
