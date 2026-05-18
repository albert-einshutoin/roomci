# Task 01 — HTTP Server and Routing

## Objective

Implement serve mode and the HTTP route surface.

## Implementation Scope

- Add `roomci-http` crate.
- Add `roomci serve --config <room.yaml> --http <addr>`.
- Implement:
  - `GET /healthz`
  - `GET /rooms`
  - `GET /rooms/{room_id}`
  - `GET /rooms/{room_id}/devices`
  - `GET /rooms/{room_id}/devices/{device_id}/state`
  - `GET /timeline`
- Add stable response and error envelopes.

## Acceptance Criteria

- Server starts with default `127.0.0.1:8080`.
- Docker/container examples can override bind address.
- Unknown resources return JSON errors.
- Server shutdown is graceful in tests.

## References

- `docs/04_protocol_adapters.md`
- `docs/09_security_and_license_notes.md`
