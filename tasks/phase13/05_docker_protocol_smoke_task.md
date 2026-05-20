# Task 05 — Docker Protocol Smoke Tests

## Goal

Make protocol compatibility verifiable without local Rust tooling by running standard clients against `roomci` in Docker or Docker Compose.

## Implementation Scope

- Add a Compose service or Make target for MQTT standard-client smoke.
- Add a Compose service or Make target for Modbus TCP standard-client smoke once Task 04 lands.
- Ensure smoke tests write or verify report artifacts under `reports/`.
- Keep these tests fast enough for local evaluator use.
- Add the protocol smoke target to release/evidence docs.

## Acceptance Criteria

- A clean checkout can run a protocol smoke command with Docker installed.
- MQTT smoke proves a standard client can publish and the state appears in HTTP reports.
- Modbus smoke proves a standard client/tool can read/write selected registers once the endpoint exists.
- Failures are visible as non-zero command exits.
- `docs/EVALUATION_EVIDENCE_PACK.md` and `docs/RELEASE_CHECKLIST.md` list the smoke command.
