# Task 01: Adapter Sample CI

## Goal

Make the Go and TypeScript adapter samples mechanically verifiable instead of
being docs-only trust signals.

## Scope

- Add a `make adapter-samples-smoke` target or equivalent.
- Compile the Go sample when Go is available, or run it in a small Docker image.
- Type-check or run the TypeScript sample using a reproducible Node path.
- Prefer a black-box run against `roomci serve` with HTTP, MQTT, and Modbus ports.
- Document toolchain fallback behavior clearly.

## Acceptance Criteria

- Sample verification is part of `make verify` or explicitly documented as a separate release gate.
- The sample check fails on syntax errors.
- The sample check fails if the served HTTP/MQTT/Modbus surfaces regress.

## Out Of Scope

- Publishing official SDK packages.
- Adding production auth, TLS, or vendor-specific clients.
