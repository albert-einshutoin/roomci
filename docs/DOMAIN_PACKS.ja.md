# ドメインパック

## Core

The reusable `roomci` core is domain-neutral:

- MQTT/device contracts: command topics, state topics, retained state, duplicate delivery, reconnect-oriented behavior.
- Edge routing/failover: active edge routing, primary/secondary failover, local-first availability.
- Failure injection: cloud outage, local broker outage, duplicate delivery, command drop, edge power loss, WAN failover.
- Report generation: JSON, Markdown, JUnit, timeline, assertions, final state, guest/field impact messages.
- CI execution: deterministic scenario mode, Docker image, Compose smoke tests, and quality gates.

Domain packs are scenario conventions layered on top of that core. They should not require changing the runner unless the pack introduces a new reusable adapter or assertion type.

## ドメインパック

### Generic MQTT Edge Devices

Generic MQTT examples avoid hospitality naming and show reusable command/state contracts.

Examples:

- `examples/generic_mqtt_retained_state.yaml`
- `examples/generic_mqtt_duplicate_delivery.yaml`

### Hospitality Smart Home

Hospitality scenarios focus on guest-facing local-first behavior, room control, comfort, and field failures.

Examples:

- `examples/local_first_cloud_outage.yaml`
- `examples/edge_server_failover.yaml`
- `examples/comfort_auto_mode.yaml`
- `examples/dali_scene_partial_failure.yaml`

### Building Automation

Building-automation scenarios model practical protocol-adapter behavior rather than full protocol conformance.

Examples:

- `examples/modbus_floor_heating.yaml`
- `examples/dali_scene_partial_failure.yaml`
- `examples/starlink_failover.yaml`

### BMS / Operations

BMS and operations scenarios treat alerts, notification channels, ticket state, and runbooks as first-class test outputs.

Examples:

- `examples/bms_sauna_emergency_alert.yaml`
- `examples/starlink_failover.yaml`

### Commissioning

Commissioning scenarios convert site, room, and device declarations into executable checks.

Examples:

- `examples/commissioning_checklist.yaml`
- `examples/modbus_floor_heating.yaml`

### Access Control

Access-control scenarios check identity/access drift without calling real access systems.

Examples:

- `examples/access_permission_drift.yaml`

## Example Map

| Example | Domain Pack | Reusable Contract |
|---|---|---|
| `generic_mqtt_retained_state.yaml` | Generic MQTT edge devices | Command/state retained MQTT contract |
| `generic_mqtt_duplicate_delivery.yaml` | Generic MQTT edge devices | Duplicate delivery idempotency |
| `local_first_cloud_outage.yaml` | Hospitality smart home | Local-first control during cloud outage |
| `edge_server_failover.yaml` | Hospitality smart home / building automation | Edge failover and local availability |
| `modbus_floor_heating.yaml` | Building automation / commissioning | Register map behavior |
| `bms_sauna_emergency_alert.yaml` | BMS / operations | Alert escalation contract |
| `starlink_failover.yaml` | Building automation / BMS operations | WAN failover and ops notification |
| `comfort_auto_mode.yaml` | Hospitality smart home | Comfort automation contract |
| `access_permission_drift.yaml` | Access control | Identity/access drift detection |
| `commissioning_checklist.yaml` | Commissioning | Field checklist generation |
| `dali_scene_partial_failure.yaml` | Hospitality smart home / building automation | Lighting scene failure detection |
