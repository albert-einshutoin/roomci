# Task 02 — HTTP Concurrency and Read Timeout

## Goal

Make `roomci serve` survive concurrent and slow HTTP clients. Today the listener accepts one connection at a time, runs the full handler synchronously, and never sets a read/write timeout. A single client that opens a TCP connection and never sends a request line stalls every other PoC client until it times out at the OS level.

## Why This Matters

The Phase 10 docs describe `roomci serve` as a runtime an external controller script can probe through `/health`, drive through `/run`, and observe through `/state` and `/timeline`. That contract implicitly requires concurrent requests. A reviewer who runs the controller script in one shell and `curl /state` in another shell expects both to work — today they serialize behind whichever connection arrived first, and a stuck client wedges them all.

## Implementation Scope

- Replace the single-threaded `for stream in listener.incoming()` loop with one of:
  - `thread::spawn` per accepted connection (simplest, acceptable for PoC scale), or
  - a small bounded thread pool (e.g. `threadpool` crate, capped at a low constant) if unbounded spawning is a concern.
- Set `stream.set_read_timeout(Some(Duration::from_secs(N)))` and `stream.set_write_timeout(Some(Duration::from_secs(N)))` on every accepted connection. `N` should be a named constant (e.g. `HTTP_CLIENT_READ_TIMEOUT_SECS = 10`) defined in `roomci-serve`.
- On timeout or read error, close the connection cleanly without panicking the worker.
- Cap the number of in-flight HTTP connections to prevent unbounded thread growth (named constant `HTTP_MAX_INFLIGHT_CONNECTIONS`). If the cap is reached, accept the connection and immediately respond with HTTP 503.
- Document the chosen concurrency model and timeout values in `docs/MQTT_SERVE_SUBSET.md` (or a new `docs/HTTP_SERVE_BEHAVIOR.md`) so external evaluators know what to expect.
- Add regression tests in `crates/roomci-serve/tests/`:
  - `concurrent_health_requests_do_not_serialize`: spawn 3 threads that each call `/health` and assert wall-clock duration is materially less than 3 × per-request budget.
  - `slow_client_does_not_block_fast_client`: open a TCP connection, send no bytes, and concurrently call `/health` from another client and assert the fast client succeeds before the slow client is timed out.
  - `slow_client_is_closed_by_read_timeout`: same as above, but also assert the slow client's TCP read returns an end-of-stream or error within the timeout window.

## Acceptance Criteria

- All three regression tests above pass.
- Existing serve tests still pass.
- `cargo clippy --workspace -- -D warnings` is clean.
- The chosen concurrency cap and timeout values are documented as named constants in `roomci-serve` source.
- A note in `docs/MQTT_SERVE_SUBSET.md` (or new HTTP doc) describes the timeout and concurrency model.

## Out of Scope

- Migrating to `tokio`/`hyper`/`async`. Acceptable as long as the synchronous stack is hardened.
- Per-route concurrency limits beyond the global cap.
- Backpressure or queueing — return 503 immediately when capped.

## Evidence

- `cargo test -p roomci-serve concurrent_health_requests_do_not_serialize` passes.
- `cargo test -p roomci-serve slow_client_does_not_block_fast_client` passes.
- `cargo test -p roomci-serve slow_client_is_closed_by_read_timeout` passes.
- `grep -n HTTP_CLIENT_READ_TIMEOUT_SECS crates/roomci-serve/src/` shows the named constant exists.
