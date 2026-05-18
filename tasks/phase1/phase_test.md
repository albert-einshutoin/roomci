# Phase 1 Test Plan

## Quality Gates

- HTTP routes have integration tests using an in-process or local test server.
- API responses are stable JSON and snapshot-tested where useful.
- Commands and faults are verified through the same core engine used by Phase 0.
- No route mutates state outside the core engine.

## Required Test Cases

1. `GET /healthz` returns `status=ok` and version.
2. `GET /rooms` returns configured rooms.
3. `POST /rooms/{room_id}/devices/{device_id}/commands/unlock` updates lock state when online.
4. `POST /faults` with `offline` causes later unlock command to fail or be blocked.
5. `GET /timeline` includes command, fault, and assertion events.
6. Unknown room/device returns a typed 404 error.

## CI Expectations

- Tests bind to random local ports or use in-process handlers.
- HTTP behavior is deterministic.
- No external services are required.

## Done Means

Phase 1 is done when a backend service can use roomci as a local fake smart-room API.
