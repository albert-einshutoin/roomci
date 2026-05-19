# Phase 12 Test Plan

## Quality Gates

- `cargo test --workspace` runs every Phase 12 regression test:
  - Two HTTP clients can call `/health` concurrently while a third long-running call is in flight.
  - A client that opens a TCP connection and never sends a request line is closed by the read timeout without blocking other clients.
  - A handler that observes a poisoned `serve_state` mutex returns HTTP 500 and the listener thread keeps accepting new connections.
  - `/run` does not hold the serve-state mutex across its full body — verifiable by issuing `/state` while `/run` is mid-flight and getting a response.
  - `/health` returns the latest `RunResult` (`pending`, `passed`, `failed`) instead of a constant `"ok"`.
  - MQTT CONNECT with protocol name other than `MQTT` is rejected with `CONNACK` return code `0x01` (unacceptable protocol version) and the TCP connection is closed.
  - MQTT CONNECT with protocol name `MQTT` but protocol level other than `4` is rejected with `CONNACK` return code `0x01`.
- `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, and `cargo doc --no-deps --workspace` pass after the refactor.
- `cargo tarpaulin --workspace --fail-under 80 --engine llvm` still passes.
- `wc -l crates/roomci-cli/src/main.rs` reports under 500 lines.
- `cargo metadata --format-version 1 | jq '.workspace_members'` resolves the new `roomci-serve` crate.
- `cargo package -p roomci-cli --list` and `cargo package -p roomci-serve --list` succeed (proves `description`, `license`, `readme` are populated enough to publish).
- `CHANGELOG.md` exists and references every shipped phase from Phase 0 to Phase 12.
- README badge URLs return HTTP 200 (or the badges are explicitly removed if the public repo cannot be created).

## Done Means

Phase 12 is done when an external evaluator can reproduce the following session against `roomci serve` without observing a hang, a panic, or a misleading `/health` response:

1. Start `roomci serve --port 0 --mqtt-port 0`.
2. Probe `/health` — it reports the actual run state.
3. Send a slow HTTP client that hangs after the request line — it is closed by the read timeout.
4. Issue `/run` for a long scenario and concurrently issue `/state`, `/timeline`, and `/health` — every concurrent request returns.
5. Force a panic in one handler (test-only injection) — other clients still get responses, and the listener thread is still alive.
6. Connect with an MQTT client that announces protocol name `MQIsdp` — the server rejects it with the documented `CONNACK` code.

And the published surface matches:

- README badges resolve.
- `Cargo.toml` workspace metadata is publish-ready.
- `CHANGELOG.md` covers Phase 0 through Phase 12.
- `tasks/status.md` lists Phase 12 with status `done` once the gates above pass.
