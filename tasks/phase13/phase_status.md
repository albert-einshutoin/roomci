# Phase 13 Status — Protocol Compliance Track

## Phase Status

`done`

## Task Board

| Task | Status | Owner | Evidence | Notes |
|---|---|---|---|---|
| `01_protocol_conformance_registry_task.md` | `done` | Codex | `docs/PROTOCOL_CONFORMANCE_REGISTRY.md`; docs index; support matrix link | Spec-reference and subset registry is now the source of truth for protocol claims |
| `02_mqtt_3_1_1_conformance_subset_task.md` | `done` | Codex | `standard_mqtt_client_publishes_retained_state_through_serve`; `make protocol-smoke-mqtt`; MQTT subset docs | Current MQTT serve subset is standard-client-tested and documented as a narrow conformance subset |
| `03_mqtt_5_boundary_task.md` | `done` | Codex | `mqtt_connect_with_unsupported_level_is_rejected`; conformance registry MQTT 5 row | MQTT 5 remains an explicit unsupported/rejected boundary |
| `04_modbus_tcp_conformance_subset_task.md` | `done` | Codex | `--modbus-port`; `docs/MODBUS_TCP_SUBSET.md`; `modbus_tcp_*` tests; `protocol_conformance_smoke.yaml` | Narrow Modbus TCP subset supports MBAP, read holding/input, write single, and exception responses |
| `05_docker_protocol_smoke_task.md` | `done` | Codex | `make protocol-smoke`; `examples/controllers/protocol_smoke_controller.py`; Compose `protocol-smoke` service | Docker/Compose smoke uses paho-mqtt and pymodbus clients against roomci serve |
| `06_future_protocol_profile_matrix_task.md` | `done` | Codex | `docs/PROTOCOL_CONFORMANCE_REGISTRY.md`; `docs/HOSPITALITY_STACK_COVERAGE.md` | BACnet, OPC UA, Zigbee, Thread, KNX, and Matter are tracked as future profiles with explicit non-goals |
| `07_protocol_claims_release_gate_task.md` | `done` | Codex | `docs/RELEASE_CHECKLIST.md`; registry release rule | Release checklist now blocks unsupported conformance claims |

## Quality Gate Status

| Gate | Status | Evidence |
|---|---|---|
| Protocol conformance registry exists | `done` | `docs/PROTOCOL_CONFORMANCE_REGISTRY.md` |
| MQTT 3.1.1 subset verified with standard external client | `done` | `rumqttc` CLI black-box test; `make protocol-smoke-mqtt` |
| MQTT 5 boundary documented and tested | `done` | MQTT 5 registry row; protocol-level rejection test |
| Modbus TCP subset selected and tested | `done` | `docs/MODBUS_TCP_SUBSET.md`; `modbus_tcp_*` tests; pymodbus Compose smoke |
| Docker protocol smoke tests exist | `done` | `make protocol-smoke`; Compose `protocol-smoke` service |
| Future protocol profiles are tracked without overclaiming | `done` | Registry future-profile rows; hospitality coverage future-profile tier |
| Release gate blocks unsupported conformance claims | `done` | Release checklist protocol claim rules |

## Current Recommendation

Phase 13 is complete. MQTT 3.1.1 and Modbus TCP now have documented conformance subsets and black-box protocol smoke coverage. Future protocols remain tracked without implementation claims.
