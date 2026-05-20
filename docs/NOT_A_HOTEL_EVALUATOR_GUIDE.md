# NOT A HOTEL Evaluator Guide

This guide keeps NOT A HOTEL as a high-signal hospitality evaluation path without claiming private compatibility.

`roomci` does not know NOT A HOTEL's internal MQTT topics, payload schemas, Modbus maps, BMS contracts, auth model, edge behavior, or acceptance criteria. The current product demonstrates the reliability shape of a local-first hospitality smart-home stack and provides adapter contracts so the real private details can be plugged in for a serious PoC.

## What roomci Demonstrates Today

| Quality Risk | Scenario / Evidence | Why It Matters |
|---|---|---|
| Cloud outage with local control intact | `examples/local_first_cloud_outage.yaml` | Guest-facing control should continue when upstream services are unavailable |
| Edge failover | `examples/edge_server_failover.yaml` | A local control stack needs predictable failover behavior |
| Modbus-style equipment control | `examples/modbus_floor_heating.yaml` | Register maps, scaling, and read/write access need executable checks |
| DALI-like scene failure | `examples/dali_scene_partial_failure.yaml` | Lighting scene failures should produce visible guest-impact reports |
| Contact I/O emergency | `examples/bms_sauna_emergency_alert.yaml` | Safety-critical contacts must trigger operations workflows |
| WAN / backup-link failover | `examples/starlink_failover.yaml` | Site connectivity failure should have explicit behavior |
| Comfort automation | `examples/comfort_auto_mode.yaml` | Quality is whether the room remains comfortable, not only whether commands execute |
| Access drift | `examples/access_permission_drift.yaml` | Access permissions need drift detection as operations scale |
| Commissioning checklist | `examples/commissioning_checklist.yaml` | Field checks should become repeatable evidence |

## What NOT A HOTEL Would Need To Provide

| Unknown | Adapter Contract Field |
|---|---|
| MQTT command/state topics | `mqtt.contracts[].command_topic`, `mqtt.contracts[].state_topic` |
| MQTT payload schemas | `mqtt.contracts[].payload.required_fields` |
| Retained/QoS/session expectations | `mqtt.contracts[].adapter` plus acceptance criteria |
| Device identity and room/site hierarchy | `devices[]`, `site.rooms[]` |
| Edge command/state behavior | `edge.commands[]` |
| Modbus device maps and scaling | `modbus.devices[].registers[]` |
| BMS events, webhooks, routing, ticket lifecycle | `bms.alerts[]` |
| Auth/TLS/network assumptions | `auth` |
| Pass/fail examples and report expectations | `acceptance.criteria`, `acceptance.report_formats` |

Start from [`adapter-contracts/examples/hospitality_local_first_room.yaml`](../adapter-contracts/examples/hospitality_local_first_room.yaml), replace the public-demo placeholders with real internal details, then run:

```bash
cargo run -p roomci-cli -- adapter validate adapter-contracts/examples/hospitality_local_first_room.yaml
```

## Evaluation Rubric

| Area | Score Signal |
|---|---|
| Domain fit | The shipped scenarios map to real local-first, edge, field, and operations risks |
| Contract fit | Real MQTT, Modbus, BMS, and edge specs can be represented without runtime code edits |
| Runtime fit | External tests can drive `roomci serve` over HTTP and MQTT subset locally |
| Evidence fit | JSON, Markdown, and JUnit reports are useful to software, field, and operations teams |
| Boundary honesty | The product clearly separates behavior modeling from private-system compatibility |

## Claims Intentionally Not Made

- Not drop-in compatible with NOT A HOTEL.
- Not a production MQTT broker.
- Not a full Modbus, DALI, KNX, BACnet, Matter, SIP, or BMS implementation.
- Not a replacement for real commissioning.
- Not a claim that NOT A HOTEL uses any unconfirmed internal protocol detail.

## Why It Is Useful Before Private Specs

Before private specs are available, `roomci` lets an evaluator judge whether the product understands the right failure domains, whether the reporting model is useful, and whether the adapter contract format is a plausible bridge from private implementation to repeatable CI. That is the right pre-PoC question.
