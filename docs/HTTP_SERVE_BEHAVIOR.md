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
