# Phase 13 Status — Protocol Compliance Track

## Phase Status

`todo`

## Task Board

| Task | Status | Owner | Evidence | Notes |
|---|---|---|---|---|
| `01_protocol_conformance_registry_task.md` | `todo` | Codex | None yet | Create the spec-reference and subset registry that becomes the source of truth for protocol claims |
| `02_mqtt_3_1_1_conformance_subset_task.md` | `todo` | Codex | Existing MQTT CONNECT/PUBLISH tests are the starting point | Upgrade current MQTT serve subset into a documented, standard-client-tested conformance subset |
| `03_mqtt_5_boundary_task.md` | `todo` | Codex | Existing MQTT 5 protocol-level rejection test | Keep MQTT 5 as an explicit unsupported/rejected boundary until a real subset is selected |
| `04_modbus_tcp_conformance_subset_task.md` | `todo` | Codex | Existing Modbus scenario model only | Add a narrow Modbus TCP endpoint subset backed by Modbus Application Protocol function-code behavior |
| `05_docker_protocol_smoke_task.md` | `todo` | Codex | Existing `make compose-poc` HTTP controller only | Add Docker/Compose smoke tests using standard MQTT and Modbus clients/tools |
| `06_future_protocol_profile_matrix_task.md` | `todo` | Codex | Existing future integrations docs | Track BACnet, OPC UA, Zigbee, Thread, KNX, and Matter as future profiles with official references and clear non-goals |
| `07_protocol_claims_release_gate_task.md` | `todo` | Codex | Existing release checklist and support matrix | Add release checks that prevent protocol-conformance overclaims |

## Quality Gate Status

| Gate | Status | Evidence |
|---|---|---|
| Protocol conformance registry exists | `todo` | None yet |
| MQTT 3.1.1 subset verified with standard external client | `todo` | Existing hand-written packet and CLI black-box tests are insufficient by themselves |
| MQTT 5 boundary documented and tested | `review` | Existing protocol-level rejection exists; docs need registry linkage |
| Modbus TCP subset selected and tested | `todo` | Existing scenario model does not expose Modbus TCP wire endpoint |
| Docker protocol smoke tests exist | `todo` | `make compose-poc` covers HTTP controller only |
| Future protocol profiles are tracked without overclaiming | `todo` | BACnet/OPC UA/Matter/etc. appear in docs but not as a structured profile matrix |
| Release gate blocks unsupported conformance claims | `todo` | None yet |

## Current Recommendation

Start with Task 01, then Task 02. MQTT is already closest to a real conformance subset and should become the first official-spec-backed proof point. Modbus TCP should be the second implementation target because it is high-value for building automation and more tractable than BACnet, OPC UA, Matter, Zigbee, Thread, or KNX.
