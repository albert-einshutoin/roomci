# Phase 12 Goal — Serve Runtime Hardening & Release Plumbing

## Goal

Bring the Phase 10 `roomci serve` runtime up to a quality level that matches how it is described in the README and PoC docs. Phase 10 shipped the surface; Phase 12 closes the gap between "callable endpoint" and "endpoint that survives concurrent clients, slow clients, malformed clients, and panicking handlers without taking the process down."

The target claim after this phase is:

```txt
`roomci serve` is a single-binary localhost runtime safe enough to be left running for a multi-step external PoC: concurrent HTTP clients are served independently, slow or malformed clients do not block other requests, a poisoned mutex returns a 5xx instead of killing the process, and `/health` reports the actual run state rather than a constant `"ok"`.
```

This phase is intentionally not about new external protocol depth or new adapter shapes — Phase 11 owns that surface. Phase 12 only fixes the runtime correctness and release plumbing that the strict review of Phase 9 + 10 flagged.

## Why This Phase Exists

The strict review of Phase 9 + 10 found that the implementation has the right shape but several runtime invariants are weaker than the docs imply:

- The HTTP listener loop serves one client at a time and never times out a slow reader, so a single hung TCP connection can stall every other PoC request.
- `/run` holds the serve-state mutex for the duration of a scenario, blocking every other route while a scenario runs.
- Every route panics if any prior request poisoned the serve-state mutex, because `.expect("serve state mutex poisoned")` is used instead of a 500 response.
- `/health` always returns `"status":"ok"` regardless of `latest_report.result`, so an evaluator's controller cannot use it as a real readiness signal.
- MQTT `CONNECT` is blindly accepted: protocol name and protocol level are never validated, which contradicts the published "MQTT 3.1.1 subset" claim.
- `crates/roomci-cli/src/main.rs` has grown past 1000 lines, mixing CLI argument parsing, HTTP routing, JSON rendering, and a hand-rolled MQTT wire decoder. This violates the repo coding-style ceiling of 800 lines per file and makes review and unit testing harder than it should be.
- Release plumbing is incomplete: `Cargo.toml` workspace metadata lacks `description`, `keywords`, `categories`, and `readme`; no `CHANGELOG.md` exists; and the GitHub repository referenced from the README badges still 404s.

None of these are blocking for an internal demo. All of them are visible to an external evaluator who reads the docs and then probes the running service.

## In Scope

- Extract HTTP routing, MQTT wire decoding, and serve-state ownership from `crates/roomci-cli/src/main.rs` into a new `roomci-serve` crate.
- Add per-connection handling and a read/write timeout to the HTTP listener so concurrent and slow clients no longer block each other.
- Replace `.expect("serve state mutex poisoned")` with explicit `PoisonError` recovery that returns HTTP 500 (or equivalent MQTT-side cleanup) instead of panicking the listener thread.
- Execute scenario runs (`/run`) without holding the serve-state mutex across the entire scenario, so other routes remain responsive during execution.
- Make `/health` reflect the latest run result and the serve-state lifecycle, not a hardcoded `"ok"`.
- Validate the MQTT `CONNECT` packet (protocol name `MQTT`, protocol level 4 = MQTT 3.1.1) and reject anything else with the documented `CONNACK` return code.
- Add a workspace `Cargo.toml` metadata block (`description`, `keywords`, `categories`, `readme`) and a `CHANGELOG.md` aligned with the existing phase history.
- Update README badges and `tasks/status.md` once the GitHub repository is public, so badge URLs resolve.

## Out of Scope

- Replacing the hand-rolled HTTP/MQTT stack with `hyper`/`tokio`/`rumqttd`. The hand-rolled stack is acceptable as long as it is hardened. Library adoption is a Phase 13+ candidate if telemetry shows real concurrency need.
- Adding new external protocols (Modbus TCP, BMS webhook). Phase 11 owns that surface.
- Trait-based adapter dispatch. Phase 11 Task 03 (Adapter Contract Kit) owns the adapter abstraction; this phase keeps the existing `mqtt_v3_qos0_subset` validator until Phase 11 lands the trait.
- Coverage tooling changes. Existing `cargo tarpaulin --fail-under 80` gate stays; Phase 12 simply ensures the workspace stays above it after refactor.
- Documentation rewrites beyond reflecting the new crate boundary and the new `/health` semantics.

## Exit Criteria

- `roomci-serve` crate exists, is referenced from `crates/roomci-cli`, and `crates/roomci-cli/src/main.rs` is back under 500 lines.
- `cargo test --workspace` adds at least one regression test per hardening item (concurrent HTTP requests, slow-client timeout, poisoned mutex 500 response, `/run` non-blocking, `/health` reflecting `latest_report.result`, MQTT `CONNECT` rejection for bad protocol name and level).
- `cargo tarpaulin --workspace --fail-under 80` still passes after the refactor.
- `Cargo.toml` exposes `description`, `keywords`, `categories`, and `readme` so the crates are publish-ready.
- `CHANGELOG.md` exists at repo root and references phases 0–12.
- README badge URLs resolve (either by pushing the repo public or by removing/replacing the badges).
- `tasks/status.md` lists Phase 12 with the correct status and links to `phase12/phase_status.md`.

## Dependencies

- None on Phase 11. Phase 11 and Phase 12 can run in parallel — Phase 11 expands the external surface, Phase 12 hardens the existing surface. Where they intersect (adapter dispatch), Phase 12 explicitly defers to Phase 11.
