# Phase 12 Status — Serve Runtime Hardening & Release Plumbing

## Phase Status

`todo`

## Task Board

| Task | Status | Owner | Evidence | Notes |
|---|---|---|---|---|
| `01_serve_crate_extraction_task.md` | `todo` | Unassigned | None yet | Move HTTP routing, MQTT wire decoder, and serve-state into a new `roomci-serve` crate; restore `crates/roomci-cli/src/main.rs` to under 500 lines |
| `02_http_concurrency_timeouts_task.md` | `todo` | Unassigned | None yet | Per-connection handler with read/write timeout; concurrent client regression test |
| `03_mutex_safety_and_run_lock_task.md` | `todo` | Unassigned | None yet | Replace `.expect("serve state mutex poisoned")` with 500 response; run scenarios without holding the mutex across the whole `/run` request |
| `04_health_semantics_task.md` | `todo` | Unassigned | None yet | `/health` reflects `latest_report.result` and serve lifecycle instead of a constant `"ok"` |
| `05_mqtt_connect_validation_task.md` | `todo` | Unassigned | None yet | Validate MQTT protocol name (`MQTT`) and protocol level (`4`) and return the documented CONNACK return code on mismatch |
| `06_release_metadata_and_changelog_task.md` | `review` | Codex | `Cargo.toml`; crate manifests; `CHANGELOG.md`; README static badges; `cargo metadata`; `cargo package --list --allow-dirty` for all current crates; badge `curl -I` checks | Current crates are metadata-ready and README badges resolve; `roomci-serve` package check remains pending until Task 01 creates that crate |

## Blockers

- No current Phase 12 task is blocked.
- `roomci-serve` package verification in Task 06 remains pending until Task 01 creates the crate.

## Quality Gate Status

| Gate | Status | Evidence |
|---|---|---|
| `roomci-serve` crate exists and `roomci-cli/src/main.rs` < 500 lines | `todo` | None yet |
| Concurrent HTTP clients served independently | `todo` | None yet |
| HTTP read timeout prevents slow-client stall | `todo` | None yet |
| Poisoned mutex returns 500 instead of panicking | `todo` | None yet |
| `/run` does not block other routes for its full duration | `todo` | None yet |
| `/health` reflects `latest_report.result` | `todo` | None yet |
| MQTT `CONNECT` validates protocol name and level | `todo` | None yet |
| Workspace metadata is publish-ready | `done` | `cargo metadata --format-version 1 --no-deps`; `cargo package --list --allow-dirty` for all current crates |
| `CHANGELOG.md` exists at repo root | `done` | `grep -c '^## \[' CHANGELOG.md` -> 14 |
| README badges resolve (or are removed) | `done` | Static `img.shields.io` badge URLs return HTTP 200 |
| `cargo tarpaulin --workspace --fail-under 80` still passes | `todo` | None yet |

## Current Recommendation

Start with Task 01 (crate extraction). Every other Phase 12 task is easier to land — and easier to test in isolation — once HTTP routing, MQTT wire decoding, and serve-state live in `roomci-serve` instead of `roomci-cli`. Tasks 02–05 can then proceed in parallel because they touch different concerns inside the new crate. Task 06 is independent of the runtime work and can be picked up at any time.
