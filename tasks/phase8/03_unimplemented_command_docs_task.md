# Task 03 — Unimplemented Command Documentation

## Objective

Ensure service-mode documentation matches the current CLI.

## Acceptance Criteria

- Search docs and README for `roomci serve` and `serve --config`.
- Document `roomci serve --config <scenario> --check` as the supported service-mode entrypoint.
- CLI reference in README lists `run`, `validate`, and `serve --check`.
- Architecture and Docker/CI docs do not give copy-paste commands that fail.

## Review Findings

- `crates/roomci-cli/src/main.rs` implements `run`, `validate`, and `serve --check`.
- `docs/03_architecture.md`, `docs/17_docker_ci_design.md`, and `docs/18_mvp_roadmap.md` use `roomci serve --check`.
