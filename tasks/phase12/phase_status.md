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
| `06_release_metadata_and_changelog_task.md` | `todo` | Unassigned | None yet | Add workspace `description`/`keywords`/`categories`/`readme`, add `CHANGELOG.md`, and fix or remove the broken README badges |

## Blockers

- README badge URL repair depends on the GitHub repository being pushed public under the URL the README references. If that account/repo cannot be created, the badges must be replaced with working ones (or removed) so README claims match reality.
- No other Phase 12 task is blocked.

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
| Workspace metadata is publish-ready | `todo` | None yet |
| `CHANGELOG.md` exists at repo root | `todo` | None yet |
| README badges resolve (or are removed) | `todo` | None yet |
| `cargo tarpaulin --workspace --fail-under 80` still passes | `todo` | None yet |

## Current Recommendation

Start with Task 01 (crate extraction). Every other Phase 12 task is easier to land — and easier to test in isolation — once HTTP routing, MQTT wire decoding, and serve-state live in `roomci-serve` instead of `roomci-cli`. Tasks 02–05 can then proceed in parallel because they touch different concerns inside the new crate. Task 06 is independent of the runtime work and can be picked up at any time.
