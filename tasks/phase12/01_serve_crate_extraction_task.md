# Task 01 — Extract `roomci-serve` Crate

## Goal

Move the Phase 10 HTTP runtime, MQTT wire decoder, and serve-state ownership out of `crates/roomci-cli/src/main.rs` into a new library crate `roomci-serve`, so that the CLI binary returns to a thin argument-parsing + dispatch layer and the serve runtime becomes unit-testable in isolation.

## Why This Matters

Today `crates/roomci-cli/src/main.rs` is over 1000 lines and mixes:

- Clap argument parsing
- HTTP request parsing, routing, and rendering (JSON/Markdown/JUnit)
- MQTT 3.1.1 packet decoding (CONNECT, PUBLISH, remaining-length varint)
- Serve-state lifecycle (`Mutex<ServeState>`, `RunResult` aggregation)
- Loopback safety enforcement

This violates the repo coding-style ceiling (200–400 lines typical, 800 max) and makes every subsequent Phase 12 task harder, because each of Tasks 02–05 needs to add tests against a structure that currently lives inside a binary `main.rs`.

## Implementation Scope

- Add a new workspace member `crates/roomci-serve` with `lib.rs`.
- Move into `roomci-serve`:
  - HTTP listener loop (currently in `serve_http` or equivalent function).
  - Request parsing, routing, and response rendering for `/health`, `/scenario`, `/state`, `/timeline`, `POST /fault`, `POST /finish`, `POST /run`, and the `/reports/latest.*` family.
  - JSON/Markdown/JUnit rendering helpers that exist only for serve responses (anything already in `roomci-report` stays put).
  - MQTT wire decoder (`MqttPacket`, `MqttPublish`, `parse_mqtt_publish`, `read_mqtt_remaining_length`, `handle_mqtt_client`).
  - `ServeState` struct and its `Mutex` wrapper.
  - Loopback-host validation (`is_loopback_host`) and the `--allow-non-loopback` escape hatch enforcement.
- Keep in `crates/roomci-cli/src/main.rs`:
  - Clap CLI definitions (`run`, `validate`, `serve` subcommands).
  - Argument parsing, default config loading, and verbose/quiet/dry-run flag handling.
  - The single call into `roomci_serve::run_serve(opts)` (or similarly named entry point).
- Add `roomci-serve = { path = "../roomci-serve" }` to `crates/roomci-cli/Cargo.toml`.
- Add a `//!` crate-level doc comment for `roomci-serve`.
- Move existing Phase 10 integration tests that exercise the serve runtime end-to-end to keep using the public `roomci-serve` API where it makes the test simpler. Keep `crates/roomci-cli/tests/cli.rs` integration tests that exercise the binary as a process unchanged.

## Acceptance Criteria

- `wc -l crates/roomci-cli/src/main.rs` reports under 500 lines.
- `cargo build --workspace` succeeds.
- `cargo test --workspace` passes with no behavior regressions.
- `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, and `cargo doc --no-deps --workspace` succeed.
- `cargo metadata` lists `roomci-serve` as a workspace member.
- `roomci-serve` has a `//!` crate-level doc and `///` doc on every public item.
- The Phase 10 integration tests (`serve_starts_http_runtime_and_exposes_reports`, `external_mqtt_publish_updates_retained_state_through_serve`, `external_http_controller_script_drives_serve_black_box`, etc.) still pass without changes to their behavior.

## Out of Scope

- Behavior changes to any route, MQTT handling, or state lifecycle. This task is a pure extraction; behavior fixes belong to Tasks 02–05.
- Renaming public types beyond what is required to move them across crates.
- Removing the hand-rolled HTTP/MQTT implementation. Library adoption is a Phase 13+ candidate.

## Evidence

- `git diff --stat` shows the line-count migration from `crates/roomci-cli` to `crates/roomci-serve`.
- `cargo test --workspace` log shows the existing serve/MQTT integration tests still passing.
