# HTTP Serve Behavior

`roomci serve` exposes a localhost HTTP API for external PoC controllers.

This API is intentionally small and synchronous internally, but each accepted HTTP connection is handled independently so a slow client does not block other clients.

## Connection Model

- One worker thread is spawned per accepted HTTP connection.
- In-flight HTTP connections are capped at 32.
- When the cap is reached, the server returns `HTTP 503` with:

```json
{"error":"too_many_connections"}
```

## Timeouts

Every accepted HTTP connection receives:

- read timeout: 2 seconds
- write timeout: 2 seconds

If a client opens a TCP connection and does not send a complete HTTP request, the worker closes that connection after the read timeout. Other clients continue to be served.

## Scope

This is a local PoC runtime, not a production public HTTP API. It is designed to support short-lived CI and developer evaluation flows where an external controller calls `/health`, injects faults, calls `/finish`, and downloads reports.

Additional external input endpoint:

- `POST /external/bms/contact` accepts BMS/contact event JSON for operations PoCs. See [`EXTERNAL_PROTOCOL_DEPTH.md`](EXTERNAL_PROTOCOL_DEPTH.md).

`POST /external/bms/contact` accepts these hardening fields:

- `severity`: optional, must be `info`, `warning`, `critical`, or `emergency`
- `schema_version`: optional string, copied into evidence
- `replay_id`: optional string, rejected with `HTTP 409` if reused in the same serve process

Production HMAC verification is intentionally outside the local serve runtime;
declare HMAC metadata in an adapter contract instead.

## Run Requests

`POST /run` snapshots the current scenario config, releases the serve-state lock while the scenario executes, and then writes the new report back after execution.

Only one `/run` request can execute at a time. If another `/run` arrives while one is active, `roomci` returns `HTTP 409` with:

```json
{"error":"run_in_progress","message":"a scenario run is already in progress"}
```

If the internal serve-state mutex is poisoned, HTTP routes return `HTTP 500` with `error: "serve_state_poisoned"` instead of panicking the listener.

### External-observation overlay

`POST /run` replaces `latest_report` with a fresh `RunReport`. To
prevent timeline events emitted between runs (via `POST /fault`,
`POST /external/bms/contact`, or an external MQTT publish) from being
silently clobbered by that replacement, the serve runtime keeps a
small external-observation overlay:

- `state.external_observation_timeline` — `TimelineEvent`s recorded by
  external endpoints since the last `/run`. Drained at the next `/run`
  success boundary, appended to `latest_report.timeline`.
- `state.external_observations` — BMS observations keyed by sanitized
  source. Drained at the next `/run` success boundary, merged into
  `latest_report.final_state` under keys prefixed with
  `external.bms.`.

The overlay is also visible while it is queued:

- `GET /state` exposes the overlay under the `external_observations`
  field.
- `GET /timeline` returns the union of `latest_report.timeline` and
  the overlay.
- `GET /reports/latest.json|md|junit.xml` render the same union, so a
  CI consumer sees external events whether they fetch the JSON or the
  human-readable Markdown.

## Health Semantics

`GET /health` returns the serve lifecycle state:

| Status | HTTP | Meaning |
|---|---:|---|
| `idle` | 200 | The service is initialized and waiting for an external controller to finish or run the scenario. |
| `running` | 200 | A `POST /run` request is in flight. |
| `passed` | 200 | The most recent `/run` or `/finish` completed with a passing report. |
| `failed` | 503 | The most recent `/run` or `/finish` completed with a failing report. |

The response also includes `scenario`, `result`, `latest_report_id`, and `serve_version`.
