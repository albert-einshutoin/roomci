# Phase 12 Status — Serve Runtime Hardening & Release Plumbing

## Phase Status

`todo`

## Task Board

| Task | Status | Owner | Evidence | Notes |
|---|---|---|---|---|
| `01_serve_crate_extraction_task.md` | `done` | Codex | `crates/roomci-serve`; `wc -l crates/roomci-cli/src/main.rs` -> 299; `cargo test --workspace --all-targets`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo doc --workspace --no-deps` | HTTP routing, MQTT wire decoder, serve-state, loopback policy, and serve unit tests moved into `roomci-serve` |
| `02_http_concurrency_timeouts_task.md` | `done` | Codex | `concurrent_health_requests_do_not_serialize`; `slow_http_client_does_not_block_fast_client`; `slow_http_client_is_closed_by_read_timeout`; `docs/HTTP_SERVE_BEHAVIOR.md`; `cargo test -p roomci-serve --all-targets` | Per-connection worker threads, 2s read/write timeout, and 32 in-flight connection cap implemented |
| `03_mutex_safety_and_run_lock_task.md` | `done` | Codex | `poisoned_mutex_returns_500_response`; `second_run_while_first_in_flight_returns_409`; `run_clears_in_progress_flag_after_success`; `cargo test -p roomci-serve --all-targets`; `docs/HTTP_SERVE_BEHAVIOR.md` | Mutex poison maps to HTTP 500; `/run` does not hold serve-state lock during scenario execution; concurrent `/run` returns 409 |
| `04_health_semantics_task.md` | `todo` | Unassigned | None yet | `/health` reflects `latest_report.result` and serve lifecycle instead of a constant `"ok"` |
| `05_mqtt_connect_validation_task.md` | `todo` | Unassigned | None yet | Validate MQTT protocol name (`MQTT`) and protocol level (`4`) and return the documented CONNACK return code on mismatch |
| `06_release_metadata_and_changelog_task.md` | `done` | Codex | `Cargo.toml`; crate manifests; `CHANGELOG.md`; README static badges; `cargo metadata`; `cargo package --list --allow-dirty` for all current crates including `roomci-serve`; badge `curl -I` checks | Current crates are metadata-ready and README badges resolve |

## Blockers

- No current Phase 12 task is blocked.

## Quality Gate Status

| Gate | Status | Evidence |
|---|---|---|
| `roomci-serve` crate exists and `roomci-cli/src/main.rs` < 500 lines | `done` | `cargo metadata`; `wc -l crates/roomci-cli/src/main.rs` -> 299 |
| Concurrent HTTP clients served independently | `done` | `concurrent_health_requests_do_not_serialize` |
| HTTP read timeout prevents slow-client stall | `done` | `slow_http_client_does_not_block_fast_client`; `slow_http_client_is_closed_by_read_timeout` |
| Poisoned mutex returns 500 instead of panicking | `done` | `poisoned_mutex_returns_500_response` |
| `/run` does not block other routes for its full duration | `done` | `/run` snapshots scenario before `run_scenario`; `second_run_while_first_in_flight_returns_409`; `run_clears_in_progress_flag_after_success` |
| `/health` reflects `latest_report.result` | `todo` | None yet |
| MQTT `CONNECT` validates protocol name and level | `todo` | None yet |
| Workspace metadata is publish-ready | `done` | `cargo metadata --format-version 1 --no-deps`; `cargo package --list --allow-dirty` for all current crates including `roomci-serve` |
| `CHANGELOG.md` exists at repo root | `done` | `grep -c '^## \[' CHANGELOG.md` -> 14 |
| README badges resolve (or are removed) | `done` | Static `img.shields.io` badge URLs return HTTP 200 |
| `cargo tarpaulin --workspace --fail-under 80` still passes | `todo` | None yet |

## Current Recommendation

Start with Task 01 (crate extraction). Every other Phase 12 task is easier to land — and easier to test in isolation — once HTTP routing, MQTT wire decoding, and serve-state live in `roomci-serve` instead of `roomci-cli`. Tasks 02–05 can then proceed in parallel because they touch different concerns inside the new crate. Task 06 is independent of the runtime work and can be picked up at any time.
