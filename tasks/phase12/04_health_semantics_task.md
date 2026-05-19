# Task 04 — `/health` Reflects Actual Run State

## Goal

Make `GET /health` return information that lets an external controller distinguish "process is up but no scenario has been run yet", "scenario is in flight", "last scenario passed", and "last scenario failed". Today it returns a constant `{"status":"ok"}` regardless of `latest_report.result`, which makes it useless as a readiness signal.

## Why This Matters

The Phase 10 PoC contract describes `/health` as the first call an external controller makes to confirm `roomci serve` is ready. The contract also describes `/finish` and `/reports/latest.*` as the final calls that confirm the run outcome. With the current implementation, `/health` cannot be used to distinguish "ready" from "passed" from "failed", so the controller has to call `/finish` or parse a report to know whether the run succeeded. That is the wrong tool for a readiness probe and is misleading when an evaluator's CI uses `/health` to gate downstream steps.

This task also matters for Docker Compose: `compose/docker-compose.yml` declares no `healthcheck:` for `roomci-serve`, and `external-controller` uses `depends_on:` without `condition: service_healthy`. Once `/health` returns meaningful state, the Compose file should use it.

## Implementation Scope

- Extend the `/health` response body to include:
  - `status`: one of `starting`, `idle`, `running`, `passed`, `failed`.
    - `starting`: serve listener is up but `ServeState::initialized == false`.
    - `idle`: serve is initialized but no `/run` has completed yet.
    - `running`: a `/run` is in flight.
    - `passed`: the most recent `/run` (or `/finish`) produced `RunResult::Passed`.
    - `failed`: the most recent `/run` (or `/finish`) produced `RunResult::Failed`.
  - `latest_report_id`: the latest report id if one exists.
  - `serve_version`: the crate version of `roomci-serve`.
- HTTP status code mapping:
  - 200 for `starting`, `idle`, `running`, `passed`.
  - 503 for `failed` (so a CI step that uses `/health` as a gate fails closed).
- Update or add the `/health` integration test in `crates/roomci-serve/tests/` to cover all five states:
  - `health_reports_idle_before_any_run`
  - `health_reports_running_during_run`
  - `health_reports_passed_after_successful_run`
  - `health_reports_failed_after_failing_run`
- Add a `healthcheck:` block to `compose/docker-compose.yml` for the `roomci-serve` service that hits `/health` with the documented interval/timeout/retries, and change `external-controller`'s `depends_on:` to use `condition: service_healthy`.
- Update `examples/controllers/http_poc_controller.sh` to read and assert the new `/health` body shape after `/finish`, and to bail out early if the initial `/health` reports `failed`.
- Update `docs/PRE_ADOPTION_POC_CHECKLIST.md` and `docs/MQTT_SERVE_SUBSET.md` (or the HTTP-specific doc if one was added in Task 02) to document the new `/health` semantics and the 503 mapping.

## Acceptance Criteria

- All four new health-state integration tests pass.
- `make compose-poc` still succeeds end-to-end with the new healthcheck block.
- The existing controller script still passes its assertions against `/finish` and `/reports/latest.*`.
- Docs describe each `status` value, the HTTP status code mapping, and the readiness contract.

## Out of Scope

- Adding a separate `/ready` vs `/live` split. A single `/health` with `status` is sufficient for the PoC contract.
- Persisting health history across process restarts.
- Surfacing per-route latency or error counters. That belongs to a future observability phase.

## Evidence

- `cargo test -p roomci-serve health_reports_passed_after_successful_run` passes.
- `cargo test -p roomci-serve health_reports_failed_after_failing_run` passes.
- `docker compose -f compose/docker-compose.yml up --abort-on-container-exit external-controller` succeeds with `service_healthy` gating.
- `grep -n '"status"' examples/controllers/http_poc_controller.sh` shows the controller asserts the new field.
