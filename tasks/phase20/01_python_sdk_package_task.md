# Task 01: Python SDK Package And Smoke

## Goal

Add a lightweight Python SDK sample/package so evaluators can drive `roomci`
from Python without writing raw curl scripts.

## Scope

- Create `examples/adapters/python-http-mqtt-modbus/`.
- Add a small importable package or module:
  - `RoomciClient.health()`
  - `RoomciClient.state()`
  - `RoomciClient.timeline()`
  - `RoomciClient.finish()`
  - `RoomciClient.post_bms_contact(...)`
  - `RoomciClient.latest_report_json()`
- Add MQTT and Modbus helper examples using standard Python libraries already
  present in the protocol smoke image where possible.
- Add Docker smoke target:
  - `make python-sdk-smoke`
- Add docs:
  - `docs/PYTHON_SDK.md`

## Acceptance Criteria

- Python smoke runs against `roomci serve`.
- The sample posts a BMS contact event, publishes MQTT command state, reads
  Modbus registers, fetches report JSON, and exits non-zero on failure.
- No production auth/TLS claim is made.
- The SDK is documented as a reference client, not a supported production SDK.

## Test Commands

```bash
make python-sdk-smoke
cargo test --workspace --all-targets
```

## Out Of Scope

- PyPI publishing.
- Async client support.
- Production retry/auth/TLS policy.
