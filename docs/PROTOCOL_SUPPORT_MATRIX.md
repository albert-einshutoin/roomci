# Protocol Support Matrix

This matrix is the source of truth for what `roomci` emulates today, what external clients can drive, and what remains a non-goal.

For official specification references, implemented subsets, black-box verification commands, and explicit rejections, see [`PROTOCOL_CONFORMANCE_REGISTRY.md`](PROTOCOL_CONFORMANCE_REGISTRY.md).

Support levels:

- `scenario_model`: executable behavior exists inside YAML scenario runs.
- `serve_endpoint`: the behavior can be driven or observed through `roomci serve`.
- `external_client_tested`: black-box clients exercise the serve surface in tests or Compose.
- `conformance_subset`: `roomci` implements a documented wire-level subset.
- `unsupported`: not implemented, or intentionally outside the product scope.

## Matrix

| Domain / Protocol | Current Support Level | Implementation Evidence | External Surface | Required Customer Inputs | Production Non-goals |
|---|---|---|---|---|---|
| MQTT command/state contracts | `conformance_subset`, `serve_endpoint`, `external_client_tested` | `crates/roomci-mqtt`, `crates/roomci-serve`, `examples/generic_mqtt_retained_state.yaml`, `examples/generic_mqtt_duplicate_delivery.yaml`, `docs/MQTT_SERVE_SUBSET.md`, CLI and serve tests, protocol smoke | MQTT 3.1.1 `CONNECT`; QoS0 `PUBLISH`; HTTP state/report observation | Topic templates, device-id extraction rules, required JSON fields, retained-state expectations, QoS/session expectations | Production broker, ACLs, TLS, persistence, clustering, MQTT 5, QoS1/QoS2 wire semantics |
| Modbus register behavior | `conformance_subset`, `serve_endpoint`, `external_client_tested` | `crates/roomci-device-model`, `crates/roomci-serve`, `examples/modbus_floor_heating.yaml`, `docs/08_modbus_strategy.md`, serve Modbus TCP tests, protocol smoke | Modbus TCP MBAP; read holding/input registers; write single register; HTTP state/report observation | Unit id, register map, type, scale, units, writable/read-only mode, commissioning thresholds | Full Modbus TCP server, RTU electrical behavior, vendor-specific device emulation, electrical commissioning replacement |
| DALI-like lighting scenes | `scenario_model` | `crates/roomci-device-model`, `examples/dali_scene_partial_failure.yaml`, `docs/09_dali_lighting_strategy.md`, fault/assertion tests | No DALI wire endpoint | Fixture IDs, scene targets, expected levels, fault modes, acceptance thresholds | Full DALI bus implementation, gateway certification, photometric validation |
| Contact I/O | `scenario_model` | `crates/roomci-device-model`, `examples/bms_sauna_emergency_alert.yaml`, `docs/06_device_model.md`, `docs/10_bms_operations_emulation.md` | Contact changes can be represented in scenario steps and observed in reports | Contact IDs, normal/open/closed semantics, severity mapping, debounce expectations | Real relay I/O, hardware safety verification |
| BMS / operations alerts | `scenario_model`, `serve_endpoint`, `external_client_tested` | `crates/roomci-ops`, `crates/roomci-serve`, `examples/bms_sauna_emergency_alert.yaml`, `docs/10_bms_operations_emulation.md`, `docs/EXTERNAL_PROTOCOL_DEPTH.md`, ops and serve tests | `POST /external/bms/contact`; state/timeline/report observation | Alert sources, severity, notification routing, ticket lifecycle, acknowledgement contract, runbook URLs | Production BMS, real Slack/phone/ticket integrations, incident-response guarantee |
| Edge controller failover | `scenario_model` | `crates/roomci-edge`, `examples/edge_server_failover.yaml`, `docs/05_edge_server_emulator.md`, edge tests | State/report observation through scenario and serve reports | Edge IDs, failover policy, timeout expectations, command routing assumptions | Replacement for an actual home controller, hardware redundancy validation |
| WAN / network failover | `scenario_model` | `examples/local_first_cloud_outage.yaml`, `examples/starlink_failover.yaml`, `docs/11_network_and_failover.md`, fault model | Faults can be driven through HTTP `POST /fault`; reports expose result | Link names, degraded/offline semantics, fallback timing, guest-impact thresholds | Network simulator, packet-level WAN emulation, ISP validation |
| Access control / intercom | `scenario_model` for access drift; `unsupported` for intercom wire behavior | `examples/access_permission_drift.yaml`, `docs/14_intercom_and_access_control.md`, report recommendation logic | No access/intercom protocol endpoint | Identity source, access-system group, stale-user criteria, intercom event/webhook contracts | Real lock/intercom controller, SIP/DTMF gateway, physical access safety guarantee |
| Comfort / HVAC automation | `scenario_model` | `examples/comfort_auto_mode.yaml`, `docs/13_comfort_automation.md`, core tests | Reports expose comfort assertion results | Comfort target, room model assumptions, sensor names, override behavior, pass/fail threshold | Physical HVAC control, energy optimization, thermal engineering validation |

## Compatibility Position

`roomci` can be used for generic company evaluation when the company maps its private contracts into scenarios and adapter definitions. It should not be described as compatible with a specific company environment until that company supplies its real protocol contracts.

For any organization, the missing inputs are the actual MQTT topics and payloads, Modbus/register maps, BMS or webhook contracts, auth/TLS model, device identity model, and acceptance criteria. The current value is a strong foundation for integrating your own contracts, not a claim of private-system compatibility.

## Roadmap Use

- If a row is `scenario_model` only, the next product step is a serve endpoint or adapter contract.
- If a row is `serve_endpoint` but not `external_client_tested`, add a black-box test or Compose controller.
- If a row is `conformance_subset`, keep the subset documented and avoid broader protocol claims.
- If a real evaluator supplies private specs, encode them as adapter contracts instead of hard-coding customer behavior into the core runtime.
