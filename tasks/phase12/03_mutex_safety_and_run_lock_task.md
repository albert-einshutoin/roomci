# Task 03 — Mutex Poison Safety and `/run` Lock Scope

## Goal

Stop using `.expect("serve state mutex poisoned")` in every route handler, and stop holding the serve-state mutex across the full body of `POST /run`. These two issues turn any handler bug into a process-wide outage, and they make `/run` serialize the entire serve runtime for the duration of a scenario.

## Why This Matters

Today every HTTP handler in the Phase 10 serve runtime calls `state.lock().expect("serve state mutex poisoned")`. If any handler panics while holding the lock (and several can — for example, while rendering a malformed scenario report), every subsequent request panics on lock acquisition, and the listener thread dies with it. The user sees a generic TCP reset instead of an HTTP 500.

Separately, `POST /run` holds the same mutex from the moment it parses the request body until the scenario finishes executing. Every other route blocks behind it. Phase 10's docs describe `/state`, `/timeline`, and `/health` as observable while a scenario runs — they are not, today.

## Implementation Scope

- Introduce a small helper such as `fn lock_serve_state(state: &Arc<Mutex<ServeState>>) -> Result<MutexGuard<'_, ServeState>, ServeError>` that:
  - Returns `Ok(guard)` on a clean lock.
  - Returns `Err(ServeError::StatePoisoned)` if `PoisonError` is observed.
  - Optionally recovers the poisoned state with `into_inner()` if the invariant survives, but only when it has been audited per call site. Default behavior is to surface 500.
- Convert every `.expect("serve state mutex poisoned")` call site to the helper.
- Map `ServeError::StatePoisoned` to HTTP 500 with body `{"error":"serve_state_poisoned"}`.
- Refactor `/run` so the mutex is held only during read/write of `ServeState` fields, never across the scenario execution body itself. Concretely:
  1. Acquire the lock briefly to mark `ServeState::run_in_progress = true` and snapshot any inputs needed.
  2. Release the lock.
  3. Run the scenario.
  4. Re-acquire the lock briefly to write `latest_report`, clear `run_in_progress`, and append timeline events.
- If a second `/run` arrives while `run_in_progress == true`, respond with HTTP 409 Conflict and a documented body, instead of blocking.
- Add regression tests in `crates/roomci-serve/tests/`:
  - `state_route_responds_while_run_in_flight`: start a `/run` against a slow scenario and assert `/state` returns within a small budget.
  - `poisoned_mutex_returns_500_and_listener_survives`: inject a panic in a handler that holds the lock (behind a `#[cfg(test)]` test-only route), assert next request returns 500, assert third request also returns 500 (listener not dead).
  - `second_run_while_first_in_flight_returns_409`: issue two `/run` requests back to back and assert the second one gets 409.

## Acceptance Criteria

- Zero remaining `.expect("serve state mutex poisoned")` occurrences in `roomci-serve`.
- Zero occurrences of holding `ServeState` mutex across scenario execution.
- All three regression tests above pass.
- Existing Phase 10 serve tests still pass.

## Out of Scope

- Replacing `std::sync::Mutex` with `parking_lot`. Either is acceptable as long as poison handling is explicit.
- Queueing concurrent `/run` requests. 409 is sufficient for the PoC contract; queueing belongs to a later phase if real usage demands it.
- Cancellation of an in-flight `/run`.

## Evidence

- `cargo test -p roomci-serve state_route_responds_while_run_in_flight` passes.
- `cargo test -p roomci-serve poisoned_mutex_returns_500_and_listener_survives` passes.
- `cargo test -p roomci-serve second_run_while_first_in_flight_returns_409` passes.
- `grep -rn 'expect("serve state mutex poisoned")' crates/` returns no matches.
