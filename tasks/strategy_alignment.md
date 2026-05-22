# Strategy Alignment

This file maps the current product strategy to task status.

## Things We Should Do

| Strategy Item | Task State | Where It Lives |
|---|---|---|
| Make the NOT A HOTEL-like core QA journey the center of the product demo | `done` | Phase 14 defines the core journey; `make poc-core-qa` is the demo path. Control-panel fault depth was promoted in Phase 17. |
| Local MQTT | `done` | Phase 1, Phase 10, Phase 13, and Phase 15 retained subscribe/replay. |
| Edge server | `done` | Phase 2 and Phase 14. |
| Modbus | `done` | Phase 3 scenario model, Phase 13 Modbus TCP subset, and Phase 15 practical multi-register reads. |
| DALI-like | `done` as scenario model | Phase 3 and Phase 14. Full DALI wire behavior is not claimed. |
| Contact I/O | `done` as scenario/BMS input surface | Phase 3, Phase 4, Phase 11, Phase 14. |
| BMS alert | `done` | Phase 4 and Phase 11 plus Phase 17 schema/severity/HMAC/replay boundary hardening. |
| Network failover | `done` | Phase 6 and Phase 14 plus Phase 17 deterministic network/firewall fault profiles. |
| Comfort automation | `done` | Phase 6 and Phase 14 plus Phase 17 sensor-zone/time-series replay. |
| Control panel fault | `done` as scoped QA profile | Phase 14 scoped this safely; Phase 17 added executable control-panel fault profiles. |
| Keep protocols official-spec-backed and subset-scoped | `done` | Phase 13 registry and release checklist; Phase 15 automated evidence; Phase 21 added contract profiles without overclaiming. |
| MQTT 3.1.1 subset | `done` | Phase 13 plus Phase 15 QoS0 SUBSCRIBE/SUBACK retained replay for configured state topics. |
| Modbus TCP subset | `done` | Phase 13 and Phase 15 practical multi-register depth. |
| MQTT 5 unsupported boundary | `done` | Phase 13 and protocol conformance registry. |
| BACnet / OPC UA / KNX / Matter future profiles | `done` as B Tier contract profiles | Phase 21 turns these into contract-profile evidence without certification claims. |
| Move company-specific differences into adapter contracts | `done` | Phase 11 adapter contract kit and onboarding docs; Phase 17 hardened BMS/contact contract boundaries. |
| MQTT topic | `done` | Adapter contracts and MQTT scenario contracts. |
| Payload schema | `done` at required-field level, deeper schema remains future hardening | Adapter contracts; Phase 16 Task 04 if webhook schema versioning is promoted. |
| Register map | `done` | Adapter contracts and Modbus model. |
| Alert severity | `done` | BMS/contact endpoint and Phase 17 severity enum validation. |
| Auth/TLS | `done` as explicit customer input / unsupported runtime boundary | Onboarding, release checklist, MQTT subset docs. |
| Acceptance criteria | `done` | Adapter contracts, PoC packs, evaluator docs. |
| Device identity | `done` | Adapter contracts and MQTT placeholder mapping. |
| Keep NOT A HOTEL-specific language inside the hospitality domain pack/evaluator path | `done` | Phase 8, Phase 9, Phase 11, Phase 14. Core positioning is Smart Home / Building Automation QA Contract Emulator. |
| Ship multiple domain packs | `done` | Generic MQTT edge, hospitality smart home, building automation, and BMS ops PoC packs exist in Phase 11. |
| Verify with standard clients and black-box Docker smoke | `done` | Phase 13 standard MQTT/Modbus smoke; Phase 15 Task 03 replaced runtime package install with a pinned protocol-smoke image build. |

## Things We Should Not Do

| Non-Goal | Task State | Where It Lives |
|---|---|---|
| Recreate the full NOT A HOTEL stack | `documented non-goal` | Phase 14 and backlog inventory. |
| Become AWS/GCP/Cloudflare/UniFi/DoorBird/Twilio vendor emulators | `documented non-goal` | Phase 14 and backlog inventory. |
| Implement BACnet/Matter/KNX/OPC UA as full protocol stacks | `documented non-goal`; B Tier contract profiles `done` | Phase 21 only targets scoped contract profiles and docs/evidence gates. |
| Claim full protocol conformance | `blocked by release rule` | Phase 13 registry and release checklist. |
| Enter physical safety or real lock-control territory | `documented non-goal` | Phase 14, Phase 16 Task 02, Phase 16 Task 03, and backlog inventory. |

## Current Reading

The strategy is taskified.

The implementation-complete base layer is Phase 0-22. Phase 19 completed the S
Tier evidence and observability layer. Phase 20 completed the A Tier Python and
scenario-debugger developer-experience layer. Phase 21 completed the B Tier
protocol-profile layer. Phase 22 completed local VSCode authoring assets without
Marketplace publishing.
Anything beyond those layers is either an explicit customer input or a
documented non-goal.
