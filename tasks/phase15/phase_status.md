# Phase 15 Status: Evaluator Friction Removal

Status: `todo`

## Tasks

| Task | Status | Notes |
|---|---|---|
| 01 MQTT retained subscribe loop | `todo` | Add a minimal SUBSCRIBE/SUBACK path and retained replay only for configured state topics, or document why it remains out of scope. |
| 02 Modbus practical subset | `todo` | Add multi-register reads and the next high-value register/coil behavior, with exception semantics and standard-client tests. |
| 03 Docker protocol smoke image | `done` | Compose now builds `compose/protocol-smoke.Dockerfile` with pinned paho-mqtt/pymodbus dependencies instead of installing packages at runtime. |
| 04 Serve protocol module extraction | `todo` | Split HTTP/MQTT/Modbus protocol handlers from the monolithic serve source without changing behavior. |
| 05 Protocol evidence automation | `todo` | Add a machine-checkable mapping from public support claims to tests, smoke targets, or explicit non-goals. |

## Current Blocker

None. This is a release-candidate hardening phase, not a blocker for the completed Phase 11-14 PoC scope.
