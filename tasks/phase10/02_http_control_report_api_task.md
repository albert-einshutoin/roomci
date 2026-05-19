# Task 02 — HTTP Control and Report API

## Objective

Add a local HTTP API that lets external tests observe `roomci`, inject faults, finish a run, and collect reports.

## Acceptance Criteria

- Expose:
  - `GET /health`
  - `GET /scenario`
  - `GET /state`
  - `GET /timeline`
  - `POST /fault`
  - `POST /finish`
  - `GET /reports/latest.json`
  - `GET /reports/latest.md`
  - `GET /reports/latest.junit.xml`
- Responses are deterministic and suitable for CI assertions.
- `POST /fault` accepts at least the existing fault target/type shape.
- `POST /finish` evaluates pending assertions and finalizes reports.
- Error responses include machine-readable codes and human-readable messages.
- API tests cover success and invalid-request paths.

## Notes

- This API is the main integration surface for CI harnesses even after MQTT/Modbus adapters exist.
- Do not expose this API on public interfaces by default.
