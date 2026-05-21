# Phase 15 Status: Evaluator Friction Removal

Status: `done`

## Tasks

| Task | Status | Notes |
|---|---|---|
| 01 MQTT retained subscribe loop | `done` | MQTT serve now supports narrow QoS0 SUBSCRIBE/SUBACK and retained replay for configured state topics; standard client and Docker protocol smoke cover it. |
| 02 Modbus practical subset | `done` | Modbus TCP reads now support contiguous multi-register reads for functions 03/04 with invalid quantity, gap, unit, and read-only exception tests. |
| 03 Docker protocol smoke image | `done` | Compose now builds `compose/protocol-smoke.Dockerfile` with pinned paho-mqtt/pymodbus dependencies instead of installing packages at runtime. |
| 04 Serve protocol module extraction | `done` | Serve runtime is split into HTTP, MQTT, Modbus, protocol tests, and core runtime glue modules; each source file is below the 800-line maintainability target. |
| 05 Protocol evidence automation | `done` | `docs/protocol-evidence.json` plus `make protocol-evidence` now checks verified protocol claims against commands, docs, and non-goals. |
| 06 Core module extraction | `done` | `roomci-core` is split into public report types, runtime, assertions, and tests; each source file is below the 800-line maintainability target. |
| 07 Scenario module extraction | `done` | `roomci-scenario` is split into schema, validation/loading, and tests; each source file is below the 800-line maintainability target. |

## Recommended Order

1. Task 01 + Task 04 together: evaluator MQTT friction and serve maintainability touch the same code path.
2. Task 02 + Task 05 together: Modbus practical depth and public claim evidence should move in lockstep.
3. Task 06 + Task 07 after the protocol surface is stable: reduce long-file debt without mixing it into behavior changes.

## Current Blocker

None. Phase 15 is complete after verification.
